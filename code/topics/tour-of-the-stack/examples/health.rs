//! Your turn, Tweak: `GET /health` answers "ok", with a test.
//! Run: `cargo run -p tour-of-the-stack --example health`, then `curl -i http://127.0.0.1:3000/health`
//! Test: `cargo test -p tour-of-the-stack --all-targets` (examples' tests need `--all-targets`
//! or `--examples`).
use axum::{Router, routing::get};

async fn hello() -> &'static str {
    "Hello from Axum!"
}

// ANCHOR: health
async fn health() -> &'static str {
    "ok"
}

fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
}
// ANCHOR_END: health

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

    #[tokio::test]
    async fn root_says_hello() {
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello from Axum!");
    }

    // Kept on one line to match the lesson and the `root_says_hello` pattern in src/main.rs.
    #[rustfmt::skip]
    // ANCHOR: health_test
    #[tokio::test]
    async fn health_says_ok() {
        let request = Request::builder().uri("/health").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"ok");
    }
    // ANCHOR_END: health_test
}
