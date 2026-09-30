//! Common mistakes: `std::thread::sleep` blocks the thread, so the 1-second timeout cannot fire.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-blocking-sleep`, then GET /slow.
use axum::{Router, http::StatusCode, routing::get};
use std::time::Duration;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

// ANCHOR: handler
async fn slow_blocking() -> &'static str {
    std::thread::sleep(Duration::from_secs(3));
    "finally done"
}
// ANCHOR_END: handler

fn app() -> Router {
    Router::new()
        .route("/slow", get(slow_blocking))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(1),
        ))
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("tower_http=debug")
        .init();
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
    use axum::body::Body;
    use axum::http::Request;
    use std::time::Instant;
    use tower::ServiceExt;

    #[tokio::test]
    async fn the_timeout_cannot_interrupt_blocking_work() {
        let started = Instant::now();
        let request = Request::get("/slow").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(started.elapsed() >= Duration::from_secs(3));
    }
}
