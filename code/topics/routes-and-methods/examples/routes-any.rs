//! More examples: `any(…)` answers every HTTP method on one path.
//! Run: `cargo run -p routes-and-methods --example routes-any`, then
//! `curl -i http://127.0.0.1:3000/ping` (or `-X POST`, `-X DELETE`, `-X PATCH`)
// ANCHOR: imports
use axum::{Router, routing::any};
// ANCHOR_END: imports

// ANCHOR: ping
async fn ping() -> &'static str {
    "pong"
}
// ANCHOR_END: ping

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/ping", any(ping))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(method: &str, uri: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn every_method_gets_pong() {
        for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
            assert_eq!(
                send(method, "/ping").await,
                (StatusCode::OK, "pong".to_string())
            );
        }
    }
}
