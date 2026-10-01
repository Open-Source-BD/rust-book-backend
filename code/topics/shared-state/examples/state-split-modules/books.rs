// ANCHOR: imports
use crate::state::AppState;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
// ANCHOR_END: imports

// ANCHOR: types
#[derive(Clone, Serialize)]
pub struct Book {
    id: u32,
    title: String,
}

#[derive(Deserialize)]
struct NewBook {
    title: String,
}
// ANCHOR_END: types

// ANCHOR: handlers
async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {
    let books = state.books.lock().unwrap();
    Json(books.clone())
}

async fn add_book(
    State(state): State<AppState>,
    Json(input): Json<NewBook>,
) -> (StatusCode, Json<Book>) {
    let mut books = state.books.lock().unwrap();
    let book = Book {
        id: books.len() as u32 + 1,
        title: input.title,
    };
    books.push(book.clone());
    (StatusCode::CREATED, Json(book))
}
// ANCHOR_END: handlers

// ANCHOR: router
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_books).post(add_book))
}
// ANCHOR_END: router
