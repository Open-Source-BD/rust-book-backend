// ANCHOR: authors
use axum::{Router, extract::Path, routing::get};

async fn list_authors() -> &'static str {
    "all authors"
}

async fn show_author(Path(id): Path<u32>) -> String {
    format!("author {id}")
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_authors))
        .route("/{id}", get(show_author))
}
// ANCHOR_END: authors
