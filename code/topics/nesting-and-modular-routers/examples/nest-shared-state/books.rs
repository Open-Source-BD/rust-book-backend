// ANCHOR: books
use crate::AppState;
use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use std::sync::atomic::Ordering;

async fn list_books() -> &'static str {
    "all books"
}

async fn show_book(State(state): State<AppState>, Path(id): Path<u32>) -> String {
    state.lookups.fetch_add(1, Ordering::Relaxed);
    format!("book {id}")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_books))
        .route("/{id}", get(show_book))
}
// ANCHOR_END: books
