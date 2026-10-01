//! Testing handlers, More examples: a test that proves a slow route gives up with `408` instead of
//! making the caller wait. It lives here because this crate already has tower-http.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-timeout-test`
//! Test: `cargo test -p middleware-and-tower-layers --example mw-timeout-test`
// ANCHOR: app
use axum::{Router, http::StatusCode, routing::get};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

async fn slow() -> &'static str {
    tokio::time::sleep(Duration::from_secs(10)).await;
    "finally done"
}

fn app() -> Router {
    Router::new()
        .route("/fast", get(|| async { "quick" }))
        .route("/slow", get(slow))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_millis(200),
        ))
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
    // ANCHOR: test
    use super::*;
    use axum::{body::Body, http::Request};
    use std::time::Instant;
    use tower::ServiceExt;

    #[tokio::test]
    async fn a_slow_route_gives_up_with_408() {
        let started = Instant::now();
        let request = Request::get("/slow").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn a_fast_route_is_not_affected() {
        let request = Request::get("/fast").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    // ANCHOR_END: test
}
