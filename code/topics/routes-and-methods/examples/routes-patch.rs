//! Your turn (Guided): `PATCH /books/{id}` gets its own handler.
//! Run: `cargo run -p routes-and-methods --example routes-patch`, then
//! `curl -i -X PATCH http://127.0.0.1:3000/books/7`
use axum::{Router, http::StatusCode, routing::get};

async fn list_books() -> &'static str {
    "all books"
}

async fn add_book() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "book added")
}

async fn show_book() -> &'static str {
    "one book"
}

async fn update_book() -> &'static str {
    "book updated"
}

async fn delete_book() -> StatusCode {
    StatusCode::NO_CONTENT
}

// ANCHOR: patch_book
async fn patch_book() -> &'static str {
    "book patched"
}
// ANCHOR_END: patch_book

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "nothing lives here")
}

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
        .route(
            "/books/{id}",
            get(show_book)
                .put(update_book)
                .patch(patch_book)
                .delete(delete_book),
        )
        .fallback(not_found)
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
    async fn patch_works_on_one_book_only() {
        assert_eq!(
            send("PATCH", "/books/7").await,
            (StatusCode::OK, "book patched".to_string())
        );
        assert_eq!(
            send("PUT", "/books/7").await,
            (StatusCode::OK, "book updated".to_string())
        );
        assert_eq!(
            send("PATCH", "/books").await.0,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
