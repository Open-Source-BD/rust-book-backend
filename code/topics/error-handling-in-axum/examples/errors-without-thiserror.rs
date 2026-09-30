//! More examples: the same AppError with no thiserror, writing by hand the two trait
//! implementations that `#[derive(thiserror::Error)]` writes for you.
//! Run: `cargo run -p error-handling-in-axum --example errors-without-thiserror`, then GET /books/9.
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize)]
struct Book {
    id: u32,
    title: String,
}

#[derive(Clone)]
struct AppState {
    books: Arc<Mutex<Vec<Book>>>,
}

// ANCHOR: error
#[derive(Debug)]
enum AppError {
    NotFound(u32),
    Poisoned,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(id) => write!(f, "book {id} not found"),
            AppError::Poisoned => write!(f, "the book list is unavailable after an earlier crash"),
        }
    }
}

impl std::error::Error for AppError {}
// ANCHOR_END: error

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Poisoned => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

async fn show_book(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Book>, AppError> {
    let books = state.books.lock().map_err(|_| AppError::Poisoned)?;
    let book = books
        .iter()
        .find(|book| book.id == id)
        .cloned()
        .ok_or(AppError::NotFound(id))?;
    Ok(Json(book))
}

fn app() -> Router {
    let books = vec![
        Book {
            id: 1,
            title: "Dune".to_string(),
        },
        Book {
            id: 2,
            title: "Emma".to_string(),
        },
    ];
    Router::new()
        .route("/books/{id}", get(show_book))
        .with_state(AppState {
            books: Arc::new(Mutex::new(books)),
        })
}

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

    #[test]
    fn same_messages_as_thiserror() {
        assert_eq!(AppError::NotFound(9).to_string(), "book 9 not found");
        assert_eq!(
            AppError::Poisoned.to_string(),
            "the book list is unavailable after an earlier crash"
        );
    }

    #[test]
    fn it_is_a_real_std_error() {
        let error: Box<dyn std::error::Error> = Box::new(AppError::NotFound(9));
        assert_eq!(error.to_string(), "book 9 not found");
    }
}
