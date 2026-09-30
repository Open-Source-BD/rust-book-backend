//! Your turn (Tweak): a second middleware numbers every request in an `x-request-count` header,
//! with a counter kept in a `static AtomicU64`.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-request-count`, then GET / a few
//! times.
// ANCHOR: imports
use axum::{
    Router,
    extract::Request,
    http::HeaderValue,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use std::sync::atomic::{AtomicU64, Ordering};
// ANCHOR_END: imports

async fn hello() -> &'static str {
    "hello"
}

async fn add_powered_by(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-powered-by", HeaderValue::from_static("axum"));
    response
}

// ANCHOR: middleware
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

async fn add_request_count(request: Request, next: Next) -> Response {
    let number = REQUEST_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-request-count", HeaderValue::from(number));
    response
}
// ANCHOR_END: middleware

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .layer(middleware::from_fn(add_powered_by))
        .layer(middleware::from_fn(add_request_count))
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

    fn count_of(response: &Response) -> u64 {
        response.headers()["x-request-count"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    #[tokio::test]
    async fn each_request_gets_the_next_number() {
        let request = Request::get("/").body(Body::empty()).unwrap();
        let first = app().oneshot(request).await.unwrap();
        let request = Request::get("/").body(Body::empty()).unwrap();
        let second = app().oneshot(request).await.unwrap();
        assert_eq!(count_of(&second), count_of(&first) + 1);
        assert_eq!(second.headers()["x-powered-by"], "axum");
    }
}
