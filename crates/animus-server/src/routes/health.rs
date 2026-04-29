use axum::{extract::State, Json, Router};
use serde::Serialize;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/ollama/status", axum::routing::get(ollama_status))
}

#[derive(Serialize)]
struct OllamaStatusResponse {
    online: bool,
    model: String,
}

async fn ollama_status(State(state): State<AppState>) -> Json<OllamaStatusResponse> {
    let online = state.ollama.ping().await;
    Json(OllamaStatusResponse {
        online,
        model: state.model_name.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use animus_db::{
        persona_repo::PersonaRepo, settings_repo::SettingsRepo, summary_repo::SummaryRepo,
        ConversationRepo, MessageRepo,
    };
    use animus_llm::ollama::OllamaClient;
    use axum::{body::to_bytes, http::Request};
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../animus-db/migrations");

    fn make_app(pool: SqlitePool) -> axum::Router {
        let state = AppState {
            personas: PersonaRepo::new(pool.clone()),
            conversations: ConversationRepo::new(pool.clone()),
            messages: MessageRepo::new(pool.clone()),
            summaries: SummaryRepo::new(pool.clone()),
            settings: SettingsRepo::new(pool),
            ollama: OllamaClient::new("http://localhost:11434"),
            model_name: "gemma4".to_owned(),
            ollama_url: "http://localhost:11434".to_owned(),
            assets_dir: "/tmp/assets".to_owned(),
            backups_dir: "/tmp/backups".to_owned(),
        };
        router().with_state(state)
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn status_returns_json_with_model(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/ollama/status")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(body["online"].is_boolean());
        assert_eq!(body["model"].as_str().unwrap(), "gemma4");
    }
}
