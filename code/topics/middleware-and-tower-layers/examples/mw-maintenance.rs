//! Your turn (From scratch): when the environment variable `MAINTENANCE` is `1`, answer every
//! path except /health with `503 maintenance`.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-maintenance` (normal), or
//! `MAINTENANCE=1 cargo run -p middleware-and-tower-layers --example mw-maintenance`.
// ANCHOR: imports
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
// ANCHOR_END: imports

async fn hello() -> &'static str {
    "hello"
}

async fn health() -> &'static str {
    "ok"
}

// ANCHOR: middleware
async fn maintenance_mode(State(on): State<bool>, request: Request, next: Next) -> Response {
    if on && request.uri().path() != "/health" {
        (StatusCode::SERVICE_UNAVAILABLE, "maintenance\n").into_response()
    } else {
        next.run(request).await
    }
}
// ANCHOR_END: middleware

// ANCHOR: app
fn app(maintenance: bool) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
        .layer(middleware::from_fn_with_state(
            maintenance,
            maintenance_mode,
        ))
}
// ANCHOR_END: app

// ANCHOR: main
#[tokio::main]
async fn main() {
    let maintenance = std::env::var("MAINTENANCE").is_ok_and(|value| value == "1");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is busy: stop the other program using it, or change the port");
    println!("Listening on http://127.0.0.1:3000 (maintenance: {maintenance})");
    axum::serve(listener, app(maintenance))
        .await
        .expect("the server stopped because of an error");
}
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(app: Router, uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    // ANCHOR: test
    #[tokio::test]
    async fn in_maintenance_only_health_answers() {
        assert_eq!(
            send(app(true), "/").await,
            (StatusCode::SERVICE_UNAVAILABLE, "maintenance\n".to_string())
        );
        assert_eq!(
            send(app(true), "/missing").await.0,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            send(app(true), "/health").await,
            (StatusCode::OK, "ok".to_string())
        );
    }
    // ANCHOR_END: test

    #[tokio::test]
    async fn normally_everything_answers() {
        assert_eq!(
            send(app(false), "/").await,
            (StatusCode::OK, "hello".to_string())
        );
        assert_eq!(send(app(false), "/health").await.0, StatusCode::OK);
    }
}
