use animus_core::AppSettings;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/settings", get(get_settings).patch(patch_settings))
}

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub user_name: String,
    pub default_model: String,
    pub ollama_url: String,
    pub assets_dir: String,
    pub backups_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchSettingsRequest {
    pub user_name: Option<String>,
    pub default_model: Option<String>,
}

async fn get_settings(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    tracing::debug!(target: "settings", "GET /api/settings");
    let s = state.settings.get().await.map_err(|_| ApiError::Internal)?;
    Ok(Json(SettingsResponse {
        user_name: s.user_name,
        default_model: s.default_model,
        ollama_url: state.ollama_url,
        assets_dir: state.assets_dir,
        backups_dir: state.backups_dir,
    }))
}

async fn patch_settings(
    State(state): State<AppState>,
    Json(body): Json<PatchSettingsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::debug!(target: "settings", "PATCH /api/settings");

    if let Some(ref name) = body.user_name {
        if name.trim().is_empty() {
            return Err(ApiError::BadRequest("user_name cannot be empty".to_owned()));
        }
    }

    let current = state.settings.get().await.map_err(|_| ApiError::Internal)?;

    let updated = AppSettings {
        user_name: body.user_name.unwrap_or(current.user_name),
        default_model: body.default_model.unwrap_or(current.default_model),
    };

    state
        .settings
        .upsert(&updated)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(SettingsResponse {
        user_name: updated.user_name,
        default_model: updated.default_model,
        ollama_url: state.ollama_url,
        assets_dir: state.assets_dir,
        backups_dir: state.backups_dir,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use animus_db::{persona_repo::PersonaRepo, summary_repo::SummaryRepo};
    use animus_db::{settings_repo::SettingsRepo, ConversationRepo, MessageRepo};
    use animus_llm::ollama::OllamaClient;
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request, StatusCode},
    };
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../crates/animus-db/migrations");

    fn make_state(pool: SqlitePool) -> AppState {
        AppState {
            personas: PersonaRepo::new(pool.clone()),
            conversations: ConversationRepo::new(pool.clone()),
            messages: MessageRepo::new(pool.clone()),
            summaries: SummaryRepo::new(pool.clone()),
            settings: SettingsRepo::new(pool),
            ollama: OllamaClient::new("http://localhost:11434".to_owned()),
            model_name: "gemma4".to_owned(),
            ollama_url: "http://localhost:11434".to_owned(),
            assets_dir: "/tmp/assets".to_owned(),
            backups_dir: "/tmp/backups".to_owned(),
        }
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_returns_defaults_on_fresh_db(pool: SqlitePool) {
        let state = make_state(pool);
        state.settings.init_defaults("gemma4").await.unwrap();

        let app = router().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/settings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["user_name"], "User");
        assert_eq!(body["default_model"], "gemma4");
        assert_eq!(body["ollama_url"], "http://localhost:11434");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_updates_user_name_and_model(pool: SqlitePool) {
        let state = make_state(pool);
        state.settings.init_defaults("gemma4").await.unwrap();

        let app = router().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PATCH)
                    .uri("/api/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"user_name":"Bertrand","default_model":"mistral"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["user_name"], "Bertrand");
        assert_eq!(body["default_model"], "mistral");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_empty_user_name_returns_400(pool: SqlitePool) {
        let state = make_state(pool);
        state.settings.init_defaults("gemma4").await.unwrap();

        let app = router().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PATCH)
                    .uri("/api/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"user_name":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_partial_updates_preserve_other_fields(pool: SqlitePool) {
        let state = make_state(pool);
        state.settings.init_defaults("gemma4").await.unwrap();

        let app = router().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PATCH)
                    .uri("/api/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"user_name":"Alice"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["user_name"], "Alice");
        assert_eq!(body["default_model"], "gemma4", "model must be preserved");
    }
}
