// ANCHOR: imports
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use std::time::Duration;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
// ANCHOR_END: imports

// ANCHOR: handlers
async fn hello() -> &'static str {
    "hello"
}

async fn slow() -> &'static str {
    tokio::time::sleep(Duration::from_secs(3)).await;
    "finally done"
}
// ANCHOR_END: handlers

// ANCHOR: middleware
async fn add_powered_by(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-powered-by", HeaderValue::from_static("axum"));
    response
}
// ANCHOR_END: middleware

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/slow", get(slow))
        .layer(middleware::from_fn(add_powered_by))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(1),
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
// ANCHOR_END: app

// ANCHOR: main
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

    #[tokio::test]
    async fn every_response_gets_the_header() {
        let response = app().oneshot(get("/")).await.unwrap();
        assert_eq!(response.headers()["x-powered-by"], "axum");
        let response = app().oneshot(get("/missing")).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()["x-powered-by"], "axum");
    }

    #[tokio::test]
    async fn slow_handler_times_out() {
        assert_eq!(send(get("/slow")).await.0, StatusCode::REQUEST_TIMEOUT);
    }

    #[tokio::test]
    async fn cors_header_is_added() {
        let request = Request::get("/")
            .header("origin", "https://example.com")
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.headers()["access-control-allow-origin"], "*");
    }
}
