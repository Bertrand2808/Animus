use animus_core::persona::Role;
use animus_llm::{build_prompt, num_predict_for_char_limits, ollama::StreamChunk, SamplingOptions};
use axum::response::sse::{Event as SseEvent, Sse};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json, Router,
};
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Deserialize)]
pub struct RegenerateMessageRequest {
    instructions: Option<String>,
}

#[derive(Deserialize)]
pub struct EditMessageRequest {
    content: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/messages/:id/regenerate",
            axum::routing::post(regenerate_message),
        )
        .route("/api/messages/:id", axum::routing::patch(edit_message))
}

async fn regenerate_message(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<RegenerateMessageRequest>,
) -> Result<Response, ApiError> {
    let message = state
        .messages
        .find_by_id(message_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    let latest_message = state
        .messages
        .find_latest_by_conversation(message.conversation_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    if message_id != latest_message.id {
        return Err(ApiError::UnprocessableEntity(
            "only the latest message can be regenerated".to_owned(),
        ));
    }

    if message.role != Role::Assistant {
        return Err(ApiError::UnprocessableEntity(
            "only assistant messages can be regenerated".to_owned(),
        ));
    }

    let message_before_target = state
        .messages
        .find_before(latest_message.conversation_id, latest_message.id)
        .await
        .map_err(|_| ApiError::Internal)?;

    let (_, persona) = state
        .conversations
        .find_by_id_with_persona(latest_message.conversation_id)
        .await
        .map_err(|e| {
            tracing::error!(
                conversation_id = %latest_message.conversation_id,
                error = ?e,
                "failed to fetch conversation with persona"
            );
            ApiError::Internal
        })?
        .ok_or(ApiError::NotFound)?;

    let mut persona = persona;
    if let Some(instructions) = request.instructions {
        persona.post_history_instructions = instructions;
    }

    let summary = state
        .summaries
        .find_latest(latest_message.conversation_id)
        .await
        .ok();

    let settings = state.settings.get().await.map_err(|e| {
        tracing::error!(conversation_id = %latest_message.conversation_id, "failed to fetch settings: {:?}", e);
        ApiError::Internal
    })?;

    let prompt = build_prompt(
        &persona,
        &message_before_target,
        summary.flatten().as_ref(),
        &settings.user_name,
    );

    let model = state.model_name.clone();
    let options = SamplingOptions {
        temperature: persona.temperature,
        repeat_penalty: persona.repeat_penalty,
        num_predict: num_predict_for_char_limits(persona.response_length_limit as u32),
    };

    let sse_stream = async_stream::stream! {
        let mut full_text = String::with_capacity(2048);
        let mut ollama_stream = Box::pin(state.ollama.stream(&model, prompt, options));
        while let Some(chunk) = ollama_stream.next().await {
            match chunk {
                Ok(StreamChunk::Token(token)) => {
                    full_text.push_str(&token);
                    let escaped = serde_json::to_string(&token)
                        .expect("string serialization is infallible");
                    let data = format!(r#"{{"text":{escaped}}}"#);
                    yield Ok::<SseEvent, std::convert::Infallible>(
                        SseEvent::default().event("token").data(data),
                    );
                }
                Ok(StreamChunk::Done { eval_count: _ }) => {
                    match state.messages.update_content(message_id, latest_message.conversation_id, &full_text).await {
                        Ok(_) => {
                            let state_for_trigger = state.clone();
                            tokio::spawn(async move {
                                crate::summary_trigger::evaluate_summary_trigger(latest_message.conversation_id, state_for_trigger).await;
                            });
                            let data = serde_json::json!({"message_id": message_id.to_string()}).to_string();
                            yield Ok(SseEvent::default().event("done").data(data));
                        }
                        Err(e) => {
                            tracing::error!("Failed to persist assistant message: {:?}", e);
                            let data = serde_json::json!({"message": "Failed to persist message"}).to_string();
                            yield Ok(SseEvent::default().event("error").data(data));
                        }
                    }
                    return;
                }
                Err(e) => {
                    tracing::error!("Ollama stream error: {:?}", e);
                    let data = r#"{"message":"stream error"}"#.to_owned();
                    yield Ok(SseEvent::default().event("error").data(data));
                    return;
                }
            }
        }
    };

    Ok(Sse::new(sse_stream).into_response())
}

async fn edit_message(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let message = state
        .messages
        .find_by_id(message_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    if request.content.is_empty() {
        return Err(ApiError::UnprocessableEntity(
            "content must not be empty".to_owned(),
        ));
    }

    if message.role != Role::Assistant {
        return Err(ApiError::UnprocessableEntity(
            "only assistant messages can be edited".to_owned(),
        ));
    }

    let latest_message = state
        .messages
        .find_latest_by_conversation(message.conversation_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    if message_id != latest_message.id {
        return Err(ApiError::UnprocessableEntity(
            "only latest message can be edited".to_owned(),
        ));
    }

    let updated_message = state
        .messages
        .update_content(message_id, message.conversation_id, &request.content)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(updated_message))
}

#[cfg(test)]
mod tests {
    use animus_core::{
        persona::{Message, Role},
        ContentRating, Persona,
    };
    use animus_db::{
        persona_repo::PersonaRepo, summary_repo::SummaryRepo, ConversationRepo, MessageRepo,
        SettingsRepo,
    };
    use animus_llm::OllamaClient;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        response::Response,
    };
    use sqlx::SqlitePool;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../animus-db/migrations");

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn make_app(pool: SqlitePool) -> axum::Router {
        make_app_with_ollama_url(pool, "http://localhost:11434")
    }

    fn make_app_with_ollama_url(pool: SqlitePool, ollama_url: &str) -> axum::Router {
        let state = crate::state::AppState {
            personas: PersonaRepo::new(pool.clone()),
            conversations: ConversationRepo::new(pool.clone()),
            messages: MessageRepo::new(pool.clone()),
            summaries: SummaryRepo::new(pool.clone()),
            settings: SettingsRepo::new(pool),
            ollama: OllamaClient::new(ollama_url),
            model_name: "gemma4".to_owned(),
            ollama_url: ollama_url.to_owned(),
            assets_dir: "/tmp/assets".to_owned(),
            backups_dir: "/tmp/backups".to_owned(),
        };

        crate::routes::conversations::router()
            .merge(crate::routes::messages::router())
            .with_state(state)
    }

    async fn spawn_ollama_stub(response_text: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = socket.read(&mut request).await.unwrap();

            let body = format!(
                "{{\"message\":{{\"role\":\"assistant\",\"content\":\"{}\"}},\"done\":false}}\n\
                 {{\"message\":{{\"role\":\"assistant\",\"content\":\"\"}},\"done\":true,\"eval_count\":1}}\n",
                response_text
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/x-ndjson\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );

            socket.write_all(response.as_bytes()).await.unwrap();
        });

        format!("http://{address}")
    }

    async fn insert_persona(pool: &SqlitePool, name: &str, first_message: &str) -> Persona {
        let persona = Persona {
            id: Uuid::now_v7(),
            name: name.to_owned(),
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_message: first_message.to_owned(),
            message_example: String::new(),
            avatar_url: None,
            background_url: None,
            content_rating: ContentRating::Pg,
            model: None,
            raw_card: None,
            model_instructions: String::new(),
            appearance: String::new(),
            speech_style: String::new(),
            character_goals: String::new(),
            post_history_instructions: String::new(),
            response_length_limit: 1200,
            temperature: 0.65,
            repeat_penalty: 1.12,
            instruction_template: "default".to_owned(),
        };
        PersonaRepo::new(pool.clone())
            .insert(&persona)
            .await
            .unwrap();
        persona
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn message_not_found(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/messages/00000000-0000-0000-0000-000000000000/regenerate")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn message_non_latest_on_regenerate(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let first_message_id = messages.first().unwrap().id;
        let newer_message = Message {
            id: Uuid::now_v7(),
            conversation_id,
            role: Role::Assistant,
            content: "newer assistant reply".to_string(),
            token_count: Some(1),
        };

        message_repo.insert(&newer_message).await.unwrap();

        let regenerate = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/messages/{first_message_id}/regenerate"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(regenerate.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_json(regenerate).await;
        assert_eq!(body["error"], "only the latest message can be regenerated");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn regenerate_user_message(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let newer_message = Message {
            id: Uuid::now_v7(),
            conversation_id,
            role: Role::User,
            content: "newer user reply".to_string(),
            token_count: Some(1),
        };

        message_repo.insert(&newer_message).await.unwrap();

        let last_message_id = newer_message.id;

        let regenerate = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/messages/{last_message_id}/regenerate"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(regenerate.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_json(regenerate).await;
        assert_eq!(body["error"], "only assistant messages can be regenerated");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn regenerate_without_instructions_accepts_empty_json(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let first_message_id = messages.first().unwrap().id;

        let regenerate = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/messages/{first_message_id}/regenerate"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(regenerate.status(), StatusCode::OK);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn regenerate_without_body_is_rejected_before_handler(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let first_message_id = messages.first().unwrap().id;
        let regenerate = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/messages/{first_message_id}/regenerate"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(regenerate.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn regenerate_keeps_same_message_id(pool: SqlitePool) {
        let ollama_url = spawn_ollama_stub("regenerated text").await;
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app_with_ollama_url(pool.clone(), &ollama_url);

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let original_message_id = messages.first().unwrap().id;

        let regenerate = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/messages/{original_message_id}/regenerate"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(regenerate.status(), StatusCode::OK);
        let response_body = to_bytes(regenerate.into_body(), usize::MAX).await.unwrap();
        let response_body = String::from_utf8(response_body.to_vec()).unwrap();
        assert!(response_body.contains(&format!(r#""message_id":"{original_message_id}""#)));

        let messages_after_regenerate =
            message_repo.find_last_n(conversation_id, 10).await.unwrap();
        assert_eq!(messages_after_regenerate.len(), 1);
        assert_eq!(messages_after_regenerate[0].id, original_message_id);
        assert_eq!(messages_after_regenerate[0].content, "regenerated text");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn edit_latest_assistant_message_updates_content(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let first_message_id = messages.first().unwrap().id;
        let edit = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/messages/{first_message_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"edited message!"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(edit.status(), StatusCode::OK);

        let edited_message = message_repo
            .find_by_id(first_message_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(edited_message.content, "edited message!");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn empty_content_on_edit_message(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let first_message_id = messages.first().unwrap().id;
        let edit = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/messages/{first_message_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(edit.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn edit_message_not_found(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/messages/00000000-0000-0000-0000-000000000000")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn non_latest_message_on_edit_message(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let messages = message_repo.find_last_n(conversation_id, 10).await.unwrap();
        let first_message_id = messages.first().unwrap().id;
        let newer_message = Message {
            id: Uuid::now_v7(),
            conversation_id,
            role: Role::Assistant,
            content: "newer assistant reply".to_string(),
            token_count: Some(1),
        };

        message_repo.insert(&newer_message).await.unwrap();
        let edit = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/messages/{first_message_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"edited message!"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(edit.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn user_message_on_edit_message(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}").await;
        let message_repo = MessageRepo::new(pool.clone());
        let app = make_app(pool.clone());
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"persona_id":"{}"}}"#, persona.id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let created = body_json(res).await;
        let conversation_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
        let newer_message = Message {
            id: Uuid::now_v7(),
            conversation_id,
            role: Role::User,
            content: "newer user reply".to_string(),
            token_count: Some(1),
        };

        message_repo.insert(&newer_message).await.unwrap();

        let last_message_id = newer_message.id;

        let regenerate = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/messages/{last_message_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"edited message!"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(regenerate.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_json(regenerate).await;
        assert_eq!(body["error"], "only assistant messages can be edited");
    }
}
