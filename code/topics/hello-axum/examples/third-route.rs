//! More examples: a third route, `GET /hello`.
//! Run: `cargo run -p hello-axum --example third-route`, then `curl -i http://127.0.0.1:3000/hello`
use axum::{Router, response::Html, routing::get};

async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}

async fn about() -> &'static str {
    "This server is written in Rust."
}

// ANCHOR: hello
async fn hello() -> &'static str {
    "Hello, visitor!"
}
// ANCHOR_END: hello

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .route("/hello", get(hello))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn hello_says_hello() {
        let request = Request::get("/hello").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"Hello, visitor!");
    }
}
