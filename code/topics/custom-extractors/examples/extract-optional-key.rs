//! More examples: `Option<ApiKey>`. No header is fine (a guest), the right key is a member, and a
//! wrong key is still refused with 403.
//! Run: `cargo run -p custom-extractors --example extract-optional-key`, then GET /greeting.
// ANCHOR: imports
use axum::{
    Router,
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, request::Parts},
    routing::get,
};
// ANCHOR_END: imports

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

// ANCHOR: optional
impl<S> OptionalFromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        if parts.headers.contains_key("x-api-key") {
            let key = <ApiKey as FromRequestParts<S>>::from_request_parts(parts, state).await?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
}
// ANCHOR_END: optional

// ANCHOR: handler
async fn greeting(key: Option<ApiKey>) -> String {
    match key {
        Some(ApiKey(key)) => format!("hello, member {key:?}: here are today's new books"),
        None => "hello, guest: sign up for a key to see more".to_string(),
    }
}
// ANCHOR_END: handler

fn app() -> Router {
    Router::new().route("/greeting", get(greeting))
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
        let mut builder = Request::get("/greeting");
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
    async fn guest_member_and_wrong_key() {
        assert_eq!(
            send(None).await,
            (
                StatusCode::OK,
                "hello, guest: sign up for a key to see more".to_string()
            )
        );
        assert_eq!(
            send(Some("letmein")).await,
            (
                StatusCode::OK,
                "hello, member \"letmein\": here are today's new books".to_string()
            )
        );
        assert_eq!(
            send(Some("guess")).await,
            (StatusCode::FORBIDDEN, "wrong API key".to_string())
        );
    }
}
