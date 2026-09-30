//! More examples: a middleware that measures how long the rest of the app took, and says so in
//! an `x-response-time-ms` header. The number changes on every request.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-response-time`, then GET /.
// ANCHOR: imports
use axum::{
    Router,
    extract::Request,
    http::HeaderValue,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use std::time::{Duration, Instant};
// ANCHOR_END: imports

async fn hello() -> &'static str {
    "hello"
}

async fn slowish() -> &'static str {
    tokio::time::sleep(Duration::from_millis(250)).await;
    "that took a moment"
}

// ANCHOR: middleware
async fn response_time(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let mut response = next.run(request).await;
    let millis = started.elapsed().as_millis();
    response
        .headers_mut()
        .insert("x-response-time-ms", HeaderValue::from(millis as u64));
    response
}
// ANCHOR_END: middleware

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/slowish", get(slowish))
        .layer(middleware::from_fn(response_time))
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
    use axum::body::Body;
    use tower::ServiceExt;

    #[tokio::test]
    async fn the_header_holds_the_milliseconds() {
        let request = Request::get("/slowish").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let millis: u64 = response.headers()["x-response-time-ms"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert!(millis >= 250, "expected at least 250 ms, got {millis}");
    }
}
