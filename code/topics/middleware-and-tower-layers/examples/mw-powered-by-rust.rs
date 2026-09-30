//! Your turn (Guided): the lesson's middleware, with a different header value.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-powered-by-rust`, then GET /.
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

// ANCHOR: middleware
async fn add_powered_by(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-powered-by", HeaderValue::from_static("rust and coffee"));
    response
}
// ANCHOR_END: middleware

fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .layer(middleware::from_fn(add_powered_by))
}

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
    async fn the_header_has_the_new_value() {
        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.headers()["x-powered-by"], "rust and coffee");
    }
}
