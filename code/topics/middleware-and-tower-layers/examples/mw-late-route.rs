//! Common mistakes: a route added after `.layer` is not wrapped by that layer.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-late-route`, then GET / and /late.
use axum::{
    Router,
    extract::Request,
    http::HeaderValue,
    middleware::{self, Next},
    response::Response,
    routing::get,
};

async fn hello() -> &'static str {
    "hello"
}

async fn late() -> &'static str {
    "I was added after the layer"
}

async fn add_powered_by(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-powered-by", HeaderValue::from_static("axum"));
    response
}

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .layer(middleware::from_fn(add_powered_by))
        .route("/late", get(late))
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
    async fn only_routes_above_the_layer_get_the_header() {
        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.headers()["x-powered-by"], "axum");

        let request = Request::get("/late").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert!(response.headers().get("x-powered-by").is_none());
    }
}
