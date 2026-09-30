//! Your turn (From scratch): DELETE /books/{id} answers 204 No Content, or a 404 JSON error
//! through AppError when there's no such book.
//! Run: `cargo run -p error-handling-in-axum --example errors-delete`, then DELETE /books/2.
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

// ANCHOR: delete_book
async fn delete_book(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<StatusCode, AppError> {
    let mut books = state.books.lock().map_err(|_| AppError::Poisoned)?;
    let index = books
        .iter()
        .position(|book| book.id == id)
        .ok_or(AppError::NotFound(id))?;
    books.remove(index);
    Ok(StatusCode::NO_CONTENT)
}
// ANCHOR_END: delete_book

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
        .route("/books/{id}", get(show_book).delete(delete_book))
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

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn delete(uri: &str) -> Request<Body> {
        Request::delete(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn delete_then_delete_again() {
        let app = app();
        assert_eq!(
            send_to(app.clone(), delete("/books/2")).await,
            (StatusCode::NO_CONTENT, String::new())
        );
        let get = Request::get("/books/2").body(Body::empty()).unwrap();
        assert_eq!(send_to(app.clone(), get).await.0, StatusCode::NOT_FOUND);
        assert_eq!(
            send_to(app, delete("/books/2")).await,
            (
                StatusCode::NOT_FOUND,
                r#"{"error":"book 2 not found"}"#.to_string()
            )
        );
    }
}
