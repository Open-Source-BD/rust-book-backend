// ANCHOR: books
use axum::{Router, extract::Path, routing::get};

async fn list_books() -> &'static str {
    "all books"
}

async fn show_book(Path(id): Path<u32>) -> String {
    format!("book {id}")
}

async fn show_reviews(Path(id): Path<u32>) -> String {
    format!("reviews of book {id}")
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_books))
        .route("/{id}", get(show_book))
        .route("/{id}/reviews", get(show_reviews))
}
// ANCHOR_END: books
