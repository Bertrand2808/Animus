use animus_core::{CardImportError, CharacterCardV2, ContentRating, Persona};
use animus_db::persona_repo::RepoError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/personas/import", post(import_persona))
        .route("/api/personas", post(create_persona).get(list_personas))
        .route(
            "/api/personas/:id",
            get(get_persona).delete(remove_persona).patch(patch_persona),
        )
}

// --- Import ---

async fn import_persona(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let card: CharacterCardV2 =
        serde_json::from_slice(&body).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let persona = Persona::try_from(card).map_err(|e| match e {
        CardImportError::InvalidSpec(s) => {
            ApiError::UnprocessableEntity(format!("invalid spec: {s}"))
        }
        CardImportError::Serialization(_) => ApiError::Internal,
    })?;

    state.personas.insert(&persona).await.map_err(|e| match e {
        RepoError::Duplicate => {
            ApiError::Conflict("a persona with this name already exists".to_owned())
        }
        RepoError::Db(_) => ApiError::Internal,
    })?;

    Ok((StatusCode::CREATED, Json(PersonaResponse::from(persona))))
}

// --- Create ---

#[derive(Debug, Deserialize)]
pub struct CreatePersonaRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub scenario: String,
    #[serde(default)]
    pub first_message: String,
    #[serde(default)]
    pub message_example: String,
    pub content_rating: Option<ContentRating>,
    pub model: Option<String>,
    pub avatar_url: Option<String>,
    pub background_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonaRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub scenario: String,
    #[serde(default)]
    pub first_message: String,
    #[serde(default)]
    pub message_example: String,
    pub content_rating: Option<ContentRating>,
    pub model: Option<String>,
    pub avatar_url: Option<String>,
    pub background_url: Option<String>,
}

async fn create_persona(
    State(state): State<AppState>,
    Json(req): Json<CreatePersonaRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::UnprocessableEntity("name is required".to_owned()));
    }

    let persona = Persona {
        id: Uuid::now_v7(),
        name: req.name,
        description: req.description,
        personality: req.personality,
        scenario: req.scenario,
        first_message: req.first_message,
        message_example: req.message_example,
        avatar_url: req.avatar_url,
        background_url: req.background_url,
        content_rating: req.content_rating.unwrap_or(ContentRating::Pg),
        model: req.model,
        raw_card: None,
    };

    state.personas.insert(&persona).await.map_err(|e| match e {
        RepoError::Duplicate => {
            ApiError::Conflict("a persona with this name already exists".to_owned())
        }
        RepoError::Db(_) => ApiError::Internal,
    })?;

    Ok((StatusCode::CREATED, Json(PersonaResponse::from(persona))))
}

// --- Update ---

async fn patch_persona(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePersonaRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::debug!(target: "personas", persona_id = %id, "patch request received");

    if req.name.trim().is_empty() {
        return Err(ApiError::UnprocessableEntity("name is required".to_owned()));
    }

    let mut persona = state
        .personas
        .find_by_id(id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    persona.name = req.name;
    persona.description = req.description;
    persona.personality = req.personality;
    persona.scenario = req.scenario;
    persona.first_message = req.first_message;
    persona.message_example = req.message_example;
    if let Some(cr) = req.content_rating {
        persona.content_rating = cr;
    }
    persona.model = req.model;
    persona.avatar_url = req.avatar_url;
    persona.background_url = req.background_url;

    state
        .personas
        .update(&persona)
        .await
        .map_err(|e| match e {
            RepoError::Duplicate => {
                ApiError::Conflict("a persona with this name already exists".to_owned())
            }
            RepoError::Db(_) => ApiError::Internal,
        })?;

    tracing::debug!(target: "personas", persona_id = %id, "patch complete");
    Ok(Json(PersonaResponse::from(persona)))
}

// --- List ---

#[derive(Debug, Deserialize)]
struct ListQuery {
    content_rating: Option<ContentRating>,
}

async fn list_personas(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let personas = state
        .personas
        .find_all(q.content_rating)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(
        personas
            .into_iter()
            .map(PersonaResponse::from)
            .collect::<Vec<_>>(),
    ))
}

// --- Get by ID ---

async fn get_persona(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let persona = state
        .personas
        .find_by_id(id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(PersonaResponse::from(persona)))
}

// --- Delete ---

async fn remove_persona(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let found = state
        .personas
        .delete(id)
        .await
        .map_err(|_| ApiError::Internal)?;

    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

// --- Response type ---

#[derive(Debug, Serialize)]
pub struct PersonaResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_message: String,
    pub message_example: String,
    pub avatar_url: Option<String>,
    pub background_url: Option<String>,
    pub content_rating: ContentRating,
    pub model: Option<String>,
}

impl From<Persona> for PersonaResponse {
    fn from(p: Persona) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name,
            description: p.description,
            personality: p.personality,
            scenario: p.scenario,
            first_message: p.first_message,
            message_example: p.message_example,
            avatar_url: p.avatar_url,
            background_url: p.background_url,
            content_rating: p.content_rating,
            model: p.model,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use animus_db::{
        persona_repo::PersonaRepo, summary_repo::SummaryRepo, ConversationRepo, MessageRepo,
    };
    use animus_llm::ollama::OllamaClient;
    use axum::{body::to_bytes, http::Request};
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../animus-db/migrations");

    fn make_app(pool: SqlitePool) -> Router {
        let state = AppState {
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

    const VALID_CARD: &str = r#"{
        "spec": "chara_card_v2",
        "spec_version": "2.0",
        "data": { "name": "Aria", "first_mes": "Hello!" }
    }"#;

    // --- Import ---

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn import_valid_card_returns_201(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/personas/import")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(VALID_CARD))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body = body_json(res).await;
        assert_eq!(body["name"], "Aria");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn import_malformed_json_returns_400(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/personas/import")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("not json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn import_duplicate_returns_409(pool: SqlitePool) {
        let app = make_app(pool.clone());
        let req = || {
            Request::builder()
                .method("POST")
                .uri("/api/personas/import")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(VALID_CARD))
                .unwrap()
        };
        let app2 = make_app(pool);
        app.oneshot(req()).await.unwrap();
        let res = app2.oneshot(req()).await.unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn import_invalid_spec_returns_422(pool: SqlitePool) {
        let app = make_app(pool);
        let body = r#"{"spec":"chara_card_v1","spec_version":"1.0","data":{"name":"Old"}}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/personas/import")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // --- Create ---

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn create_valid_returns_201(pool: SqlitePool) {
        let app = make_app(pool);
        let body = r#"{"name":"Bob"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/personas")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn create_empty_name_returns_422(pool: SqlitePool) {
        let app = make_app(pool);
        let body = r#"{"name":"  "}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/personas")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn create_duplicate_name_returns_409(pool: SqlitePool) {
        let app = make_app(pool.clone());
        let app2 = make_app(pool);
        let body = r#"{"name":"Bob"}"#;
        let req = || {
            Request::builder()
                .method("POST")
                .uri("/api/personas")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(body))
                .unwrap()
        };
        app.oneshot(req()).await.unwrap();
        let res = app2.oneshot(req()).await.unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);
    }

    // --- List ---

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn list_empty_returns_empty_array(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/personas")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert_eq!(body, serde_json::json!([]));
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn list_returns_all_personas(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool.clone());
        for name in ["A", "B"] {
            let p = Persona {
                id: Uuid::now_v7(),
                name: name.to_owned(),
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
            };
            repo.insert(&p).await.unwrap();
        }
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/personas")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_json(res).await;
        assert_eq!(body.as_array().unwrap().len(), 2);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn list_filtered_by_content_rating(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool.clone());
        let pg = Persona {
            id: Uuid::now_v7(),
            name: "PG".into(),
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
        };
        let nsfw = Persona {
            id: Uuid::now_v7(),
            name: "NSFW".into(),
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_message: String::new(),
            message_example: String::new(),
            avatar_url: None,
            background_url: None,
            content_rating: ContentRating::Nsfw,
            model: None,
            raw_card: None,
        };
        repo.insert(&pg).await.unwrap();
        repo.insert(&nsfw).await.unwrap();
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/personas?content_rating=nsfw")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_json(res).await;
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "NSFW");
    }

    // --- Get by ID ---

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_existing_persona(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool.clone());
        let p = Persona {
            id: Uuid::now_v7(),
            name: "Aria".into(),
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
        };
        repo.insert(&p).await.unwrap();
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/personas/{}", p.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert_eq!(body["name"], "Aria");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_not_found_returns_404(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/personas/{}", Uuid::now_v7()))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    // --- Delete ---

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn delete_existing_returns_204(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool.clone());
        let p = Persona {
            id: Uuid::now_v7(),
            name: "ToDelete".into(),
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
        };
        repo.insert(&p).await.unwrap();
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/personas/{}", p.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn delete_not_found_returns_404(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/personas/{}", Uuid::now_v7()))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    // --- Patch ---

    fn make_patch_body(name: &str) -> String {
        serde_json::json!({ "name": name }).to_string()
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_existing_returns_200(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool.clone());
        let p = Persona {
            id: Uuid::now_v7(),
            name: "Original".into(),
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
        };
        repo.insert(&p).await.unwrap();
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/personas/{}", p.id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(make_patch_body("Updated")))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert_eq!(body["name"], "Updated");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_not_found_returns_404(pool: SqlitePool) {
        let app = make_app(pool);
        let res = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/personas/{}", Uuid::now_v7()))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(make_patch_body("X")))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_empty_name_returns_422(pool: SqlitePool) {
        let app = make_app(pool);
        let body = serde_json::json!({ "name": "  " }).to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/personas/{}", Uuid::now_v7()))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn patch_duplicate_name_returns_409(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool.clone());
        for name in ["Aria", "Bob"] {
            let p = Persona {
                id: Uuid::now_v7(),
                name: name.to_owned(),
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
            };
            repo.insert(&p).await.unwrap();
        }
        let bob_id = repo.find_all(None).await.unwrap()
            .into_iter()
            .find(|p| p.name == "Bob")
            .unwrap()
            .id;
        let app = make_app(pool);
        let body = serde_json::json!({ "name": "Aria" }).to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/personas/{}", bob_id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn create_wires_avatar_url(pool: SqlitePool) {
        let app = make_app(pool);
        let body = serde_json::json!({
            "name": "WithAvatar",
            "avatar_url": "data:image/png;base64,abc123"
        })
        .to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/personas")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body = body_json(res).await;
        assert_eq!(body["avatar_url"], "data:image/png;base64,abc123");
    }
}
