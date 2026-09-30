//! More examples: `Result<String, StatusCode>`, a preview of error handling.
//! Run: `cargo run -p handlers-and-into-response --example handlers-result`, then
//! `curl -i http://127.0.0.1:3000/books/1`
// ANCHOR: imports
use axum::{Router, http::StatusCode, routing::get};
// ANCHOR_END: imports

// ANCHOR: find_book
fn find_book(id: u32) -> Result<String, StatusCode> {
    if id == 1 {
        Ok(format!("book {id}: The Rust Programming Language"))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn book_one() -> Result<String, StatusCode> {
    find_book(1)
}

async fn book_nine() -> Result<String, StatusCode> {
    find_book(9)
}
// ANCHOR_END: find_book

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/books/1", get(book_one))
        .route("/books/9", get(book_nine))
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

    async fn send(uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn ok_is_the_text_and_err_is_the_status() {
        assert_eq!(
            send("/books/1").await,
            (
                StatusCode::OK,
                "book 1: The Rust Programming Language".to_string()
            )
        );
        assert_eq!(
            send("/books/9").await,
            (StatusCode::NOT_FOUND, String::new())
        );
    }
}
