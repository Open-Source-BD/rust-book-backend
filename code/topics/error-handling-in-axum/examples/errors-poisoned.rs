//! More examples: see a poisoned lock for real. `POST /crash` stands in for any bug that panics
//! while holding the lock; after it, `show_book` answers every request with AppError::Poisoned.
//! Run: `cargo run -p error-handling-in-axum --example errors-poisoned`.
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
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

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("book {0} not found")]
    NotFound(u32),
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}

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

// ANCHOR: crash
async fn crash(State(state): State<AppState>) {
    let _books = state.books.lock().unwrap();
    panic!("a bug in this handler, while it holds the lock");
}
// ANCHOR_END: crash

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
        .route("/crash", post(crash))
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
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn after_a_crash_every_request_is_a_500_json_error() {
        let app = app();
        let crash = Request::post("/crash").body(Body::empty()).unwrap();
        let crashed = tokio::spawn(app.clone().oneshot(crash)).await;
        assert!(crashed.is_err(), "the crash handler should have panicked");
        for uri in ["/books/2", "/books/1"] {
            let request = Request::get(uri).body(Body::empty()).unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(
                &bytes[..],
                br#"{"error":"the book list is unavailable after an earlier crash"}"#
            );
        }
    }
}
