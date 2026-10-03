// ANCHOR: imports
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
// ANCHOR_END: imports

// ANCHOR: error
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("todo {0} not found")]
    NotFound(u64),
}
// ANCHOR_END: error

// ANCHOR: into_response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
// ANCHOR_END: into_response
