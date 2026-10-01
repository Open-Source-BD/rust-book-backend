//! More examples: the allowed keys live in the app's state, so the extractor implements
//! `FromRequestParts<AppState>` and reads them from `state`.
//! Run: `cargo run -p custom-extractors --example extract-keys-in-state`, then GET /secret.
// ANCHOR: imports
use axum::{
    Router,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    routing::get,
};
use std::sync::Arc;
// ANCHOR_END: imports

// ANCHOR: state
#[derive(Clone)]
struct AppState {
    keys: Arc<Vec<String>>,
}
// ANCHOR_END: state

// ANCHOR: extractor
struct ApiKey(String);

impl FromRequestParts<AppState> for ApiKey {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
        if state.keys.iter().any(|key| key == value) {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
        }
    }
}
// ANCHOR_END: extractor

async fn secret(ApiKey(key): ApiKey) -> String {
    format!("welcome, holder of key {key:?}")
}

// ANCHOR: app
fn app() -> Router {
    let keys = vec!["letmein".to_string(), "opensesame".to_string()];
    Router::new()
        .route("/secret", get(secret))
        .with_state(AppState {
            keys: Arc::new(keys),
        })
}
// ANCHOR_END: app

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
    async fn any_listed_key_works() {
        assert_eq!(send(Some("letmein")).await.0, StatusCode::OK);
        assert_eq!(
            send(Some("opensesame")).await,
            (
                StatusCode::OK,
                "welcome, holder of key \"opensesame\"".to_string()
            )
        );
        assert_eq!(send(Some("guess")).await.0, StatusCode::FORBIDDEN);
        assert_eq!(send(None).await.0, StatusCode::UNAUTHORIZED);
    }
}
