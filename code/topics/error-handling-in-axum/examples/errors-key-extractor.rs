//! Custom extractors, More examples: an API-key extractor whose rejection is the app's own
//! `AppError`, so a refused request gets the same JSON error body as every other error.
//! It lives in this crate because it needs serde_json and thiserror.
//! Run: `cargo run -p error-handling-in-axum --example errors-key-extractor`, then GET /secret.
// ANCHOR: imports
use axum::{
    Json, Router,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
// ANCHOR_END: imports

// ANCHOR: error
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("missing x-api-key header")]
    MissingKey,
    #[error("wrong API key")]
    WrongKey,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::MissingKey => StatusCode::UNAUTHORIZED,
            AppError::WrongKey => StatusCode::FORBIDDEN,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
// ANCHOR_END: error

// ANCHOR: extractor
struct ApiKey(String);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::MissingKey)?;
        if value == "letmein" {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err(AppError::WrongKey)
        }
    }
}
// ANCHOR_END: extractor

async fn secret(ApiKey(key): ApiKey) -> String {
    format!("welcome, holder of key {key:?}")
}

fn app() -> Router {
    Router::new().route("/secret", get(secret))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is busy: stop the other program using it, or change the port");
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app())
        .await
        .expect("the server stopped because of an error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(key: Option<&str>) -> (StatusCode, String) {
        let mut builder = Request::get("/secret");
        if let Some(key) = key {
            builder = builder.header("x-api-key", key);
        }
        let response = app()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn rejections_are_json() {
        assert_eq!(
            send(None).await,
            (
                StatusCode::UNAUTHORIZED,
                r#"{"error":"missing x-api-key header"}"#.to_string()
            )
        );
        assert_eq!(
            send(Some("guess")).await,
            (
                StatusCode::FORBIDDEN,
                r#"{"error":"wrong API key"}"#.to_string()
            )
        );
        assert_eq!(send(Some("letmein")).await.0, StatusCode::OK);
    }
}
