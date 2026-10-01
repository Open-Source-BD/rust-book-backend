// ANCHOR: books
use axum::{Router, extract::Path, routing::get};

async fn list_books() -> &'static str {
    "all books"
}

async fn show_book(Path(id): Path<u32>) -> String {
    format!("book {id}")
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_books))
        .route("/{id}", get(show_book))
}
// ANCHOR_END: books
