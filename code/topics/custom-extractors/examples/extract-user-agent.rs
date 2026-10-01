//! More examples: an extractor that never fails. It reads the `user-agent` header, and uses
//! "unknown" when there is none.
//! Run: `cargo run -p custom-extractors --example extract-user-agent`, then GET /whoami.
// ANCHOR: imports
use axum::{Router, extract::FromRequestParts, http::request::Parts, routing::get};
use std::convert::Infallible;
// ANCHOR_END: imports

// ANCHOR: extractor
struct UserAgent(String);

impl<S> FromRequestParts<S> for UserAgent
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let agent = parts
            .headers
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown");
        Ok(UserAgent(agent.to_owned()))
    }
}
// ANCHOR_END: extractor

// ANCHOR: handler
async fn whoami(UserAgent(agent): UserAgent) -> String {
    format!("your client calls itself {agent:?}")
}
// ANCHOR_END: handler

fn app() -> Router {
    Router::new().route("/whoami", get(whoami))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn reads_the_header_or_says_unknown() {
        let with = Request::get("/whoami")
            .header("user-agent", "book-reader/1.0")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            send(with).await,
            (
                StatusCode::OK,
                "your client calls itself \"book-reader/1.0\"".to_string()
            )
        );
        let without = Request::get("/whoami").body(Body::empty()).unwrap();
        assert_eq!(
            send(without).await,
            (
                StatusCode::OK,
                "your client calls itself \"unknown\"".to_string()
            )
        );
    }
}
