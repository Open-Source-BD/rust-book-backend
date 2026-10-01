//! Your turn (Tweak): accept the key from the `x-api-key` header OR from a `?key=` query parameter.
//! Run: `cargo run -p custom-extractors --example extract-key-or-query`, then GET /secret.
// ANCHOR: imports
use axum::{
    Router,
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
    routing::get,
};
use std::collections::HashMap;
// ANCHOR_END: imports

// ANCHOR: extractor
struct ApiKey(String);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let from_header = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_owned());
        let from_query = Query::<HashMap<String, String>>::try_from_uri(&parts.uri)
            .ok()
            .and_then(|Query(mut params)| params.remove("key"));
        let value = from_header
            .or(from_query)
            .ok_or((StatusCode::UNAUTHORIZED, "missing API key"))?;
        if value == "letmein" {
            Ok(ApiKey(value))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
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
    use tower::ServiceExt;

    async fn status_of(request: Request<Body>) -> StatusCode {
        app().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn header_or_query() {
        let header = Request::get("/secret")
            .header("x-api-key", "letmein")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status_of(header).await, StatusCode::OK);
        let query = Request::get("/secret?key=letmein")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status_of(query).await, StatusCode::OK);
        let wrong = Request::get("/secret?key=guess")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status_of(wrong).await, StatusCode::FORBIDDEN);
        let none = Request::get("/secret").body(Body::empty()).unwrap();
        assert_eq!(status_of(none).await, StatusCode::UNAUTHORIZED);
    }
}
