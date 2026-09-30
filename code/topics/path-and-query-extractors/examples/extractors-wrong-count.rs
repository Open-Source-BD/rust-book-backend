//! Common mistakes: a handler that asks for two path values on a route with one placeholder.
//! Run: `cargo run -p path-and-query-extractors --example extractors-wrong-count`, then
//! `curl -i http://127.0.0.1:3000/books/3`
// ANCHOR: imports
use axum::{Router, extract::Path, routing::get};
// ANCHOR_END: imports

// ANCHOR: mistake
async fn show_chapter(Path((book, chapter)): Path<(u32, u32)>) -> String {
    format!("book {book}, chapter {chapter}")
}

fn app() -> Router {
    Router::new().route("/books/{id}", get(show_chapter))
}
// ANCHOR_END: mistake

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
    async fn two_values_from_one_placeholder_is_a_server_error() {
        let request = Request::get("/books/3").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
