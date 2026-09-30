//! Your turn (Tweak): every error body also carries a short, fixed `code`, such as "not_found".
//! Run: `cargo run -p error-handling-in-axum --example errors-code-field`, then GET /books/9.
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

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("book {0} not found")]
    NotFound(u32),
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}

// ANCHOR: into_response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::Poisoned => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let body = Json(json!({ "code": code, "error": self.to_string() }));
        (status, body).into_response()
    }
}
// ANCHOR_END: into_response

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
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn missing_book_has_a_code() {
        let request = Request::get("/books/9").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &bytes[..],
            br#"{"code":"not_found","error":"book 9 not found"}"#
        );
    }

    #[test]
    fn poisoned_has_the_internal_code() {
        assert_eq!(
            AppError::Poisoned.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
