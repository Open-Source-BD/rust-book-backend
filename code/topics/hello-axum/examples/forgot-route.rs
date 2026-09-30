//! Common mistakes: the `about` handler exists, but no `.route` points at it, so `/about` is a 404.
//! Run: `cargo run -p hello-axum --example forgot-route`, then `curl -i http://127.0.0.1:3000/about`
use axum::{Router, response::Html, routing::get};

async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}

// The book's CI treats warnings as errors, so this line silences the
// "function `about` is never used" warning that the lesson shows you.
#[allow(dead_code)]
async fn about() -> &'static str {
    "This server is written in Rust."
}

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/", get(home))
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
    use tower::ServiceExt;

    #[tokio::test]
    async fn about_without_a_route_is_404() {
        let request = Request::get("/about").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
