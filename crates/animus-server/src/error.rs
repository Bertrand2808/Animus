use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum ApiError {
    BadRequest(String),
    UnprocessableEntity(String),
    Conflict(String),
    NotFound,
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_owned()),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
            ),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn bad_request_returns_400() {
        let r = ApiError::BadRequest("oops".to_owned()).into_response();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let body = body_json(r).await;
        assert_eq!(body["error"], "oops");
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let r = ApiError::NotFound.into_response();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn conflict_returns_409() {
        let r = ApiError::Conflict("dupe".to_owned()).into_response();
        assert_eq!(r.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn unprocessable_returns_422() {
        let r = ApiError::UnprocessableEntity("missing name".to_owned()).into_response();
        assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
