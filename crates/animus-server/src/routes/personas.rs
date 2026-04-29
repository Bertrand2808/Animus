use animus_core::{
    persona::{
        DEFAULT_INSTRUCTION_TEMPLATE, DEFAULT_REPEAT_PENALTY, DEFAULT_RESPONSE_LENGTH_LIMIT,
        DEFAULT_TEMPERATURE,
    },
    CardImportError, CharacterCardV2, ContentRating, Persona,
};
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
    tracing::debug!(target: "personas", bytes = body.len(), "persona import request received");
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

    tracing::debug!(target: "personas", persona_id = %persona.id, "persona import complete");
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
    #[serde(default)]
    pub model_instructions: String,
    #[serde(default)]
    pub appearance: String,
    #[serde(default)]
    pub speech_style: String,
    #[serde(default)]
    pub character_goals: String,
    #[serde(default)]
    pub post_history_instructions: String,
    #[serde(default = "default_response_length_limit")]
    pub response_length_limit: i64,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: f64,
    #[serde(default = "default_instruction_template")]
    pub instruction_template: String,
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
    #[serde(default)]
    pub model_instructions: String,
    #[serde(default)]
    pub appearance: String,
    #[serde(default)]
    pub speech_style: String,
    #[serde(default)]
    pub character_goals: String,
    #[serde(default)]
    pub post_history_instructions: String,
    #[serde(default = "default_response_length_limit")]
    pub response_length_limit: i64,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: f64,
    #[serde(default = "default_instruction_template")]
    pub instruction_template: String,
}

fn default_response_length_limit() -> i64 {
    DEFAULT_RESPONSE_LENGTH_LIMIT
}

fn default_temperature() -> f64 {
    DEFAULT_TEMPERATURE
}

fn default_repeat_penalty() -> f64 {
    DEFAULT_REPEAT_PENALTY
}

fn default_instruction_template() -> String {
    DEFAULT_INSTRUCTION_TEMPLATE.to_owned()
}

async fn create_persona(
    State(state): State<AppState>,
    Json(req): Json<CreatePersonaRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::debug!(
        target: "personas",
        persona_name = %req.name,
        has_model_instructions = !req.model_instructions.trim().is_empty(),
        has_appearance = !req.appearance.trim().is_empty(),
        has_speech_style = !req.speech_style.trim().is_empty(),
        has_character_goals = !req.character_goals.trim().is_empty(),
        has_post_history_instructions = !req.post_history_instructions.trim().is_empty(),
        response_length_limit = req.response_length_limit,
        temperature = req.temperature,
        repeat_penalty = req.repeat_penalty,
        instruction_template = %req.instruction_template,
        "create persona request received"
    );

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
        model_instructions: req.model_instructions,
        appearance: req.appearance,
        speech_style: req.speech_style,
        character_goals: req.character_goals,
        post_history_instructions: req.post_history_instructions,
        response_length_limit: req.response_length_limit,
        temperature: req.temperature,
        repeat_penalty: req.repeat_penalty,
        instruction_template: req.instruction_template,
    };

    state.personas.insert(&persona).await.map_err(|e| match e {
        RepoError::Duplicate => {
            ApiError::Conflict("a persona with this name already exists".to_owned())
        }
        RepoError::Db(_) => ApiError::Internal,
    })?;

    tracing::debug!(target: "personas", persona_id = %persona.id, "create persona complete");
    Ok((StatusCode::CREATED, Json(PersonaResponse::from(persona))))
}

// --- Update ---

async fn patch_persona(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePersonaRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::debug!(
        target: "personas",
        persona_id = %id,
        persona_name = %req.name,
        has_model_instructions = !req.model_instructions.trim().is_empty(),
        has_appearance = !req.appearance.trim().is_empty(),
        has_speech_style = !req.speech_style.trim().is_empty(),
        has_character_goals = !req.character_goals.trim().is_empty(),
        has_post_history_instructions = !req.post_history_instructions.trim().is_empty(),
        response_length_limit = req.response_length_limit,
        temperature = req.temperature,
        repeat_penalty = req.repeat_penalty,
        instruction_template = %req.instruction_template,
        "patch persona request received"
    );

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
    persona.model_instructions = req.model_instructions;
    persona.appearance = req.appearance;
    persona.speech_style = req.speech_style;
    persona.character_goals = req.character_goals;
    persona.post_history_instructions = req.post_history_instructions;
    persona.response_length_limit = req.response_length_limit;
    persona.temperature = req.temperature;
    persona.repeat_penalty = req.repeat_penalty;
    persona.instruction_template = req.instruction_template;

    state.personas.update(&persona).await.map_err(|e| match e {
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
    pub model_instructions: String,
    pub appearance: String,
    pub speech_style: String,
    pub character_goals: String,
    pub post_history_instructions: String,
    pub response_length_limit: i64,
    pub temperature: f64,
    pub repeat_penalty: f64,
    pub instruction_template: String,
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
            model_instructions: p.model_instructions,
            appearance: p.appearance,
            speech_style: p.speech_style,
            character_goals: p.character_goals,
            post_history_instructions: p.post_history_instructions,
            response_length_limit: p.response_length_limit,
            temperature: p.temperature,
            repeat_penalty: p.repeat_penalty,
            instruction_template: p.instruction_template,
        }
    }
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

    fn make_app(pool: SqlitePool) -> Router {
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
        let body = body_json(res).await;
        assert_eq!(body["model_instructions"], "");
        assert_eq!(body["appearance"], "");
        assert_eq!(body["speech_style"], "");
        assert_eq!(body["character_goals"], "");
        assert_eq!(body["post_history_instructions"], "");
        assert_eq!(body["response_length_limit"], 1200);
        assert_eq!(body["temperature"], 0.65);
        assert_eq!(body["repeat_penalty"], 1.12);
        assert_eq!(body["instruction_template"], "default");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn create_wires_structured_fields(pool: SqlitePool) {
        let app = make_app(pool);
        let body = serde_json::json!({
            "name": "Structured",
            "model_instructions": "Stay in character",
            "appearance": "Silver hair",
            "speech_style": "Concise",
            "character_goals": "Help the user",
            "post_history_instructions": "Use recent context",
            "response_length_limit": 900,
            "temperature": 0.8,
            "repeat_penalty": 1.2,
            "instruction_template": "cinematic"
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
        assert_eq!(body["model_instructions"], "Stay in character");
        assert_eq!(body["appearance"], "Silver hair");
        assert_eq!(body["speech_style"], "Concise");
        assert_eq!(body["character_goals"], "Help the user");
        assert_eq!(body["post_history_instructions"], "Use recent context");
        assert_eq!(body["response_length_limit"], 900);
        assert_eq!(body["temperature"], 0.8);
        assert_eq!(body["repeat_penalty"], 1.2);
        assert_eq!(body["instruction_template"], "cinematic");
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
        repo.insert(&p).await.unwrap();
        let app = make_app(pool);
        let body = serde_json::json!({
            "name": "Updated",
            "model_instructions": "Stay in character",
            "appearance": "Silver hair",
            "speech_style": "Concise",
            "character_goals": "Help the user",
            "post_history_instructions": "Use recent context",
            "response_length_limit": 900,
            "temperature": 0.8,
            "repeat_penalty": 1.2,
            "instruction_template": "cinematic"
        })
        .to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/personas/{}", p.id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert_eq!(body["name"], "Updated");
        assert_eq!(body["model_instructions"], "Stay in character");
        assert_eq!(body["appearance"], "Silver hair");
        assert_eq!(body["speech_style"], "Concise");
        assert_eq!(body["character_goals"], "Help the user");
        assert_eq!(body["post_history_instructions"], "Use recent context");
        assert_eq!(body["response_length_limit"], 900);
        assert_eq!(body["temperature"], 0.8);
        assert_eq!(body["repeat_penalty"], 1.2);
        assert_eq!(body["instruction_template"], "cinematic");
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
            repo.insert(&p).await.unwrap();
        }
        let bob_id = repo
            .find_all(None)
            .await
            .unwrap()
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
