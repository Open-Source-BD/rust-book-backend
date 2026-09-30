//! More examples: a `StatusCode` on its own is a whole response.
//! Run: `cargo run -p handlers-and-into-response --example handlers-no-content`, then
//! `curl -i -X DELETE http://127.0.0.1:3000/books/7`
// ANCHOR: imports
use axum::{Router, http::StatusCode, routing::delete};
// ANCHOR_END: imports

// ANCHOR: delete_book
async fn delete_book() -> StatusCode {
    StatusCode::NO_CONTENT
}
// ANCHOR_END: delete_book

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/books/7", delete(delete_book))
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
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(method: &str, uri: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn delete_answers_204_with_no_body() {
        assert_eq!(
            send("DELETE", "/books/7").await,
            (StatusCode::NO_CONTENT, String::new())
        );
    }
}
