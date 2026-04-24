use animus_core::persona::{Conversation, Message, Role};
use axum::{Json, Router, extract::{Path, State}, http::StatusCode};
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

// Routes
pub fn router() -> Router<AppState> {
  Router::new()
    .route("/api/conversations", axum::routing::post(create_conversation))
    .route("/api/conversations/:id", axum::routing::get(get_conversation))
}

// Handlers
async fn create_conversation(
  State(state): State<AppState>,
  Json(req): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), ApiError> {
  // 1. Fetch persona
  let persona = state.personas
    .find_by_id(req.persona_id)
    .await
    .map_err(|_| ApiError::Internal)?
    .ok_or(ApiError::NotFound)?;

  // 2. Remplacer placeholders dans first_message
  // TODO : do we really want to replace user by you ? Maybe we should ask user name
  let first_message = persona.first_message
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
  state.conversations
    .insert(&conv)
    .await
    .map_err(|_| ApiError::Internal)?;

  state.messages
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
  let conv = state.conversations
    .find_by_id(id)
    .await
    .map_err(|_| ApiError::Internal)?
    .ok_or(ApiError::NotFound)?;

  // 2. Fetch Messages (derniers 50)
  let messages = state.messages
    .find_last_n(id, 50)
    .await
    .map_err(|_| ApiError::Internal)?;

  // 3. Transformer en réponse
  let message_responses = messages.into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;
    use animus_core::{ContentRating, Persona};
    use animus_db::{persona_repo::PersonaRepo, ConversationRepo, MessageRepo};
    use axum::{body::to_bytes, body::Body, http::Request};
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../animus-db/migrations");

    fn make_app(pool: SqlitePool) -> axum::Router {
        let state = crate::state::AppState {
            personas: PersonaRepo::new(pool.clone()),
            conversations: ConversationRepo::new(pool.clone()),
            messages: MessageRepo::new(pool),
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
        PersonaRepo::new(pool.clone()).insert(&persona).await.unwrap();
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
                    .body(Body::from(r#"{"persona_id":"00000000-0000-0000-0000-000000000000"}"#))
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
        assert!(!msg.contains("{{char}}"), "raw placeholder still present: {msg}");
        assert!(!msg.contains("{{user}}"), "raw placeholder still present: {msg}");
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
}
