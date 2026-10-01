//! Your turn (Guided): the same app, with the key changed to "open-sesame".
//! Run: `cargo run -p custom-extractors --example extract-new-key`, then GET /secret.
use axum::{
    Router,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    routing::get,
};

// ANCHOR: extractor
struct ApiKey(String);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
        if value == "open-sesame" {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
        }
    }
}
// ANCHOR_END: extractor

async fn public() -> &'static str {
    "anyone can read this"
}

async fn secret(ApiKey(key): ApiKey) -> String {
    format!("welcome, holder of key {key:?}")
}

fn app() -> Router {
    Router::new()
        .route("/public", get(public))
        .route("/secret", get(secret))
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
    use tower::ServiceExt;

    async fn status_with(key: &str) -> StatusCode {
        let request = Request::get("/secret")
            .header("x-api-key", key)
            .body(Body::empty())
            .unwrap();
        app().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn only_the_new_key_works() {
        assert_eq!(status_with("open-sesame").await, StatusCode::OK);
        assert_eq!(status_with("letmein").await, StatusCode::FORBIDDEN);
    }
}
