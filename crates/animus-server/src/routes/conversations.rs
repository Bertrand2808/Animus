use animus_core::persona::{Conversation, Message, Role};
use animus_llm::{build_prompt, OllamaError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

// DTOs
#[derive(Deserialize)]
pub struct CreateConversationRequest {
    persona_id: Uuid,
}

#[derive(Serialize)]
pub struct ConversationResponse {
    id: String,
    persona_id: String,
    first_message: String,
}

#[derive(Serialize)]
pub struct ConversationDetailResponse {
    id: String,
    persona_id: String,
    created_at: i64,
    messages: Vec<MessageResponse>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    id: String,
    role: String,
    content: String,
    token_count: Option<i64>,
}

#[derive(Serialize)]
pub struct MessageDetailResponse {
    id: String,
    role: String,
    content: String,
}

#[derive(Deserialize)]
pub struct CreateMessageRequest {
    content: String,
}

// Routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/conversations",
            axum::routing::post(create_conversation),
        )
        .route(
            "/api/conversations/:id",
            axum::routing::get(get_conversation),
        )
        .route(
            "/api/conversations/:id/messages",
            axum::routing::post(create_message),
        )
}

// Handlers
async fn create_conversation(
    State(state): State<AppState>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), ApiError> {
    // 1. Fetch persona
    let persona = state
        .personas
        .find_by_id(req.persona_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    // 2. Remplacer placeholders dans first_message
    // TODO : do we really want to replace user by you ? Maybe we should ask user name
    let first_message = persona
        .first_message
        .replace("{{char}}", &persona.name)
        .replace("{{user}}", "You");

    // 3. Créer la conversation
    let conv = Conversation {
        id: Uuid::now_v7(),
        persona_id: persona.id,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };

    // 4. Créer message assistant
    let msg = Message {
        id: Uuid::now_v7(),
        conversation_id: conv.id,
        role: Role::Assistant,
        content: first_message.clone(),
        token_count: None,
    };

    // 5. Persister
    state
        .conversations
        .insert(&conv)
        .await
        .map_err(|_| ApiError::Internal)?;

    state
        .messages
        .insert(&msg)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok((
        StatusCode::CREATED,
        Json(ConversationResponse {
            id: conv.id.to_string(),
            persona_id: conv.persona_id.to_string(),
            first_message,
        }),
    ))
}

async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConversationDetailResponse>, ApiError> {
    // 1. Fetch conversation
    let conv = state
        .conversations
        .find_by_id(id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    // 2. Fetch Messages (derniers 50)
    let messages = state
        .messages
        .find_last_n(id, 50)
        .await
        .map_err(|_| ApiError::Internal)?;

    // 3. Transformer en réponse
    let message_responses = messages
        .into_iter()
        .map(|message| MessageResponse {
            id: message.id.to_string(),
            role: message.role.to_string(),
            content: message.content,
            token_count: message.token_count,
        })
        .collect();

    Ok(Json(ConversationDetailResponse {
        id: conv.id.to_string(),
        persona_id: conv.persona_id.to_string(),
        created_at: conv.created_at,
        messages: message_responses,
    }))
}

async fn create_message(
    State(state): State<AppState>,
    Path(conv_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<MessageDetailResponse>), ApiError> {
    // 1. Fetch conversation
    let conv = state
        .conversations
        .find_by_id(conv_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    // 2. Fetch Persona (doit exister, sinon incohérence DB)
    let persona = state
        .personas
        .find_by_id(conv.persona_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::Internal)?;

    // 3. Persist user message before fetching history
    let user_msg = Message {
        id: Uuid::now_v7(),
        conversation_id: conv_id,
        role: Role::User,
        content: req.content.clone(),
        token_count: None,
    };

    state
        .messages
        .insert(&user_msg)
        .await
        .map_err(|_| ApiError::Internal)?;

    // 4. Fetch history (includes user message just inserted)
    let history = state
        .messages
        .find_last_n(conv_id, 10)
        .await
        .map_err(|_| ApiError::Internal)?;

    // 5. Fetch summary optional
    let summary = state.summaries.find_latest(conv_id).await.ok();

    // 6. Build prompt
    let prompt = build_prompt(&persona, &history, summary.flatten().as_ref());

    // 7. Call Ollama
    // TODO : set an env var for the default model
    let model = persona.model.as_deref().unwrap_or("gemma4");
    let response_text = state
        .ollama
        .complete(model, prompt)
        .await
        .map_err(|e| match e {
            OllamaError::Network(_) => ApiError::ServiceUnavailable,
            OllamaError::Model(_) => ApiError::BadGateway,
            OllamaError::Parse(_) => ApiError::Internal,
        })?;

    // 8. Persist assistant message
    let assistant_msg = Message {
        id: Uuid::now_v7(),
        conversation_id: conv_id,
        role: Role::Assistant,
        content: response_text.clone(),
        token_count: None,
    };

    state
        .messages
        .insert(&assistant_msg)
        .await
        .map_err(|_| ApiError::Internal)?;

    // 9. Retourner la réponse
    Ok((
        StatusCode::CREATED,
        Json(MessageDetailResponse {
            id: assistant_msg.id.to_string(),
            role: "assistant".to_string(),
            content: response_text,
        }),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use animus_core::{ContentRating, Persona};
    use animus_db::{
        persona_repo::PersonaRepo, summary_repo::SummaryRepo, ConversationRepo, MessageRepo,
    };
    use animus_llm::ollama::OllamaClient;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../animus-db/migrations");

    fn make_app(pool: SqlitePool) -> axum::Router {
        let state = crate::state::AppState {
            personas: PersonaRepo::new(pool.clone()),
            conversations: ConversationRepo::new(pool.clone()),
            messages: MessageRepo::new(pool.clone()),
            summaries: SummaryRepo::new(pool),
            ollama: OllamaClient::new("http://localhost:11434"),
        };
        router().with_state(state)
    }

    async fn body_json(r: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(r.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
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
        };
        PersonaRepo::new(pool.clone())
            .insert(&persona)
            .await
            .unwrap();
        persona
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn post_conversation_unknown_persona_returns_404(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"persona_id":"00000000-0000-0000-0000-000000000000"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn post_conversation_returns_201_with_first_message(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hello, I am {{char}}!").await;
        let app = make_app(pool);
        let res = app
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
        let body = body_json(res).await;
        assert!(body["id"].is_string());
        assert_eq!(body["persona_id"].as_str().unwrap(), persona.id.to_string());
        assert!(body["first_message"].as_str().unwrap().contains("Alice"));
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn post_conversation_placeholder_replaced(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Alice", "Hi {{user}}, I am {{char}}.").await;
        let app = make_app(pool);
        let res = app
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
        let body = body_json(res).await;
        let msg = body["first_message"].as_str().unwrap();
        assert!(msg.contains("Alice"), "{{{{char}}}} not replaced: {msg}");
        assert!(msg.contains("You"), "{{{{user}}}} not replaced: {msg}");
        assert!(
            !msg.contains("{{char}}"),
            "raw placeholder still present: {msg}"
        );
        assert!(
            !msg.contains("{{user}}"),
            "raw placeholder still present: {msg}"
        );
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_conversation_returns_messages(pool: SqlitePool) {
        let persona = insert_persona(&pool, "Bob", "Hey!").await;
        // Create conversation via POST
        let app = make_app(pool.clone());
        let res = app
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
        let body = body_json(res).await;
        let conv_id = body["id"].as_str().unwrap().to_owned();

        // GET that conversation
        let app2 = make_app(pool);
        let res2 = app2
            .oneshot(
                Request::builder()
                    .uri(format!("/api/conversations/{conv_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res2.status(), StatusCode::OK);
        let body2 = body_json(res2).await;
        let messages = body2["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"].as_str().unwrap(), "assistant");
        assert_eq!(messages[0]["content"].as_str().unwrap(), "Hey!");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_conversation_not_found_returns_404(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/conversations/00000000-0000-0000-0000-000000000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn post_message_unknown_conv_returns_404(pool: SqlitePool) {
        let app = make_app(pool);

        let request = Request::builder()
            .method("POST")
            .uri("/api/conversations/00000000-0000-0000-0000-000000000000/messages")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"content": "Hello"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Skipped: setting up a conv with orphaned persona_id while FK=ON is unreliable
    // in sqlx test pools (pool resets PRAGMA foreign_keys=ON per connection).
    // The handler code (.ok_or(ApiError::Internal)?) is covered by code review.
    #[ignore]
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn post_message_missing_persona_returns_500(_pool: SqlitePool) {}

    #[tokio::test]
    #[ignore] // Require Ollama running
    async fn post_message_returns_assistant_response() {
        // Setup base
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&pool).await.unwrap();

        let persona = insert_persona(&pool, "Alice", "Hello {{user}}! I'm {{char}}.").await;

        // Fonction helper pour créer un app frais
        let make_app = || make_app(pool.clone());

        // 1. Créer la conversation
        let create_res = make_app()
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

        let create_body = body_json(create_res).await;
        let conv_id = create_body["id"].as_str().unwrap().to_owned();

        // 2. Envoyer le message utilisateur
        let user_content = "Salut, comment ça va aujourd'hui ?";

        let post_res = make_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/conversations/{}/messages", conv_id))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"content":"{}"}}"#, user_content)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(post_res.status(), StatusCode::CREATED);

        let resp_body = body_json(post_res).await;
        let assistant_content = resp_body["content"].as_str().unwrap();

        assert!(!assistant_content.trim().is_empty());
        assert!(assistant_content.len() > 10);

        // 3. Vérifier la persistance via GET (nouveau app)
        let get_res = make_app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/conversations/{}", conv_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_res.status(), StatusCode::OK);

        let conv_detail = body_json(get_res).await;
        let messages = conv_detail["messages"].as_array().unwrap();

        // 3 messages: first_message (assistant) + user + assistant reply
        assert_eq!(
            messages.len(),
            3,
            "Doit contenir first_message + user + assistant"
        );

        let user_msg = &messages[1];
        let assistant_msg = &messages[2];

        assert_eq!(messages[0]["role"].as_str().unwrap(), "assistant");

        assert_eq!(user_msg["role"].as_str().unwrap(), "user");
        assert_eq!(user_msg["content"].as_str().unwrap(), user_content);

        assert_eq!(assistant_msg["role"].as_str().unwrap(), "assistant");
        assert_eq!(
            assistant_msg["content"].as_str().unwrap(),
            assistant_content
        );

        assert_ne!(user_msg["id"].as_str(), assistant_msg["id"].as_str());
    }
}
