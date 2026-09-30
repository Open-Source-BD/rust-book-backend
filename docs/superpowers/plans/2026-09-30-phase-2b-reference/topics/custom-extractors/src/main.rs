// ANCHOR: imports
use axum::{
    Router,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    routing::get,
};
// ANCHOR_END: imports

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
        if value == "letmein" {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
        }
    }
}
// ANCHOR_END: extractor

// ANCHOR: handlers
async fn public() -> &'static str {
    "anyone can read this"
}

async fn secret(ApiKey(key): ApiKey) -> String {
    format!("welcome, holder of key {key:?}")
}
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/public", get(public))
        .route("/secret", get(secret))
}
// ANCHOR_END: app

// ANCHOR: main
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
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        send_to(app(), request).await
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[allow(dead_code)]
    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    fn with_key(key: &str) -> Request<Body> {
        Request::get("/secret")
            .header("x-api-key", key)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn key_is_checked() {
        assert_eq!(send(get("/public")).await.0, StatusCode::OK);
        assert_eq!(
            send(get("/secret")).await,
            (
                StatusCode::UNAUTHORIZED,
                "missing x-api-key header".to_string()
            )
        );
        assert_eq!(
            send(with_key("guess")).await,
            (StatusCode::FORBIDDEN, "wrong API key".to_string())
        );
        assert_eq!(
            send(with_key("letmein")).await,
            (
                StatusCode::OK,
                "welcome, holder of key \"letmein\"".to_string()
            )
        );
    }
}
