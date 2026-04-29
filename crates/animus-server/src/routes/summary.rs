use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// Response for GET /api/conversations/:id/summary
#[derive(Serialize)]
pub struct SummaryDetailResponse {
    pub id: Option<String>,
    pub conversation_id: String,
    pub content: Option<String>,
    pub message_range_start: Option<String>,
    pub message_range_end: Option<String>,
    pub created_at: Option<i64>,
}

/// GET /api/conversations/:id/summary
///
/// Returns the latest summary for a conversation, or `null` if none exists.
async fn get_summary(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let conv_exists = state
        .conversations
        .find_by_id(conversation_id)
        .await
        .map_err(|e| {
            tracing::error!(%conversation_id, "Failed to fecth conversation: {}", e);
            ApiError::Internal
        })?
        .is_some();

    if !conv_exists {
        return Err(ApiError::NotFound);
    }

    let summary = state
        .summaries
        .find_latest(conversation_id)
        .await
        .map_err(|e| {
            tracing::error!(%conversation_id, "Failed to fetch summary: {}", e);
            ApiError::Internal
        })?;

    let response = match summary {
        None => SummaryDetailResponse {
            id: None,
            conversation_id: conversation_id.to_string(),
            content: None,
            message_range_start: None,
            message_range_end: None,
            created_at: None,
        },
        Some(s) => SummaryDetailResponse {
            id: Some(s.id.to_string()),
            conversation_id: conversation_id.to_string(),
            content: Some(s.content),
            message_range_start: Some(s.message_range_start.to_string()),
            message_range_end: Some(s.message_range_end.to_string()),
            created_at: Some(s.created_at),
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/conversations/:id/summary",
        axum::routing::get(get_summary),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use animus_core::{persona::Summary, ContentRating, Persona};
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
            model_name: "gemma4".to_owned(),
        };
        router().with_state(state)
    }

    async fn body_json(r: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(r.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn seed_conversation(pool: &SqlitePool) -> Uuid {
        let persona = Persona {
            id: Uuid::now_v7(),
            name: "Toto".to_owned(),
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_message: String::new(),
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
        let conv_id = Uuid::now_v7();
        let conv_id_str = conv_id.to_string();
        let persona_id_str = persona.id.to_string();
        sqlx::query!(
            r#"INSERT INTO conversations (id, persona_id, created_at, updated_at) VALUES (?, ?, unixepoch(), unixepoch())"#,
            conv_id_str,
            persona_id_str,
        )
        .execute(pool)
        .await
        .unwrap();
        conv_id
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_summary_unknown_conversation_returns_404(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/conversations/00000000-0000-0000-0000-000000000000/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_summary_no_summary_returns_null_fields(pool: SqlitePool) {
        let conv_id = seed_conversation(&pool).await;
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/conversations/{conv_id}/summary"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert!(body["id"].is_null());
        assert!(body["content"].is_null());
        assert_eq!(
            body["conversation_id"].as_str().unwrap(),
            conv_id.to_string()
        );
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_summary_returns_summary_with_metadata(pool: SqlitePool) {
        let conv_id = seed_conversation(&pool).await;
        let summary = Summary {
            id: Uuid::now_v7(),
            conversation_id: conv_id,
            content: "a quick summary".to_owned(),
            message_range_start: Uuid::now_v7(),
            message_range_end: Uuid::now_v7(),
            created_at: 0,
        };
        SummaryRepo::new(pool.clone())
            .insert(&summary)
            .await
            .unwrap();
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/conversations/{conv_id}/summary"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert_eq!(body["id"].as_str().unwrap(), summary.id.to_string());
        assert_eq!(
            body["conversation_id"].as_str().unwrap(),
            conv_id.to_string()
        );
        assert_eq!(body["content"].as_str().unwrap(), "a quick summary");
        assert_eq!(
            body["message_range_start"].as_str().unwrap(),
            summary.message_range_start.to_string()
        );
        assert_eq!(
            body["message_range_end"].as_str().unwrap(),
            summary.message_range_end.to_string()
        );
        assert!(body["created_at"].is_number());
    }
}
