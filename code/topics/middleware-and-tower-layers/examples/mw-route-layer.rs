//! More examples: `route_layer` puts the key check on one route only.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-route-layer`, then GET / and /admin.
use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};

async fn hello() -> &'static str {
    "hello"
}

async fn admin() -> &'static str {
    "welcome to the admin page"
}

async fn require_key(request: Request, next: Next) -> Response {
    let key = request.headers().get("x-api-key");
    if key.is_some_and(|key| key == "letmein") {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, "missing or wrong x-api-key\n").into_response()
    }
}

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/", get(hello)).route(
        "/admin",
        get(admin).route_layer(middleware::from_fn(require_key)),
    )
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

    fn get(uri: &str, key: Option<&str>) -> Request {
        let mut builder = Request::get(uri);
        if let Some(key) = key {
            builder = builder.header("x-api-key", key);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn only_admin_needs_the_key() {
        let status = |uri, key| async move { app().oneshot(get(uri, key)).await.unwrap().status() };
        assert_eq!(status("/", None).await, StatusCode::OK);
        assert_eq!(status("/admin", None).await, StatusCode::UNAUTHORIZED);
        assert_eq!(status("/admin", Some("letmein")).await, StatusCode::OK);
        assert_eq!(status("/missing", None).await, StatusCode::NOT_FOUND);
    }
}
