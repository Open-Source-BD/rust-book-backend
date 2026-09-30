//! More examples: log the real reason for a 500 with `eprintln!`, and send the client a short,
//! safe message instead.
//! Run: `cargo run -p error-handling-in-axum --example errors-logged`, then GET /books/2/cover.
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
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("book {0} not found")]
    NotFound(u32),
    #[error("setting {name} is missing: {reason}")]
    MissingSetting { name: String, reason: String },
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::MissingSetting { .. } | AppError::Poisoned => {
                eprintln!("500 Internal Server Error: {self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong on our side".to_string(),
                )
            }
        };
        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}
// ANCHOR_END: error

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

// ANCHOR: cover
async fn show_cover(Path(id): Path<u32>) -> Result<String, AppError> {
    let name = "COVERS_URL";
    let base = std::env::var(name).map_err(|err| AppError::MissingSetting {
        name: name.to_string(),
        reason: err.to_string(),
    })?;
    Ok(format!("{base}/{id}.jpg"))
}
// ANCHOR_END: cover

// ANCHOR: app
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
        .route("/books/{id}/cover", get(show_cover))
        .with_state(AppState {
            books: Arc::new(Mutex::new(books)),
        })
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

    #[test]
    fn the_client_never_sees_the_details() {
        let error = AppError::MissingSetting {
            name: "COVERS_URL".to_string(),
            reason: "environment variable not found".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "setting COVERS_URL is missing: environment variable not found"
        );
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn not_found_is_still_404() {
        assert_eq!(
            AppError::NotFound(9).into_response().status(),
            StatusCode::NOT_FOUND
        );
    }
}
