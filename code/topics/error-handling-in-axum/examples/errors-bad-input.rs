//! More examples: an `AppError::BadInput` variant, answered with 422 Unprocessable Entity.
//! Run: `cargo run -p error-handling-in-axum --example errors-bad-input`, then POST /books.
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize)]
struct Book {
    id: u32,
    title: String,
}

#[derive(Deserialize)]
struct NewBook {
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
    #[error("{0}")]
    BadInput(String),
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadInput(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Poisoned => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({ "error": self.to_string() }));
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

// ANCHOR: add_book
async fn add_book(
    State(state): State<AppState>,
    Json(input): Json<NewBook>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::BadInput("title must not be empty".to_string()));
    }
    let mut books = state.books.lock().map_err(|_| AppError::Poisoned)?;
    let book = Book {
        id: books.len() as u32 + 1,
        title: title.to_string(),
    };
    books.push(book.clone());
    Ok((StatusCode::CREATED, Json(book)))
}
// ANCHOR_END: add_book

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
        .route("/books", post(add_book))
        .route("/books/{id}", get(show_book))
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

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn blank_title_is_422_json() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"   "}"#)).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                r#"{"error":"title must not be empty"}"#.to_string()
            )
        );
    }

    #[tokio::test]
    async fn good_title_is_201() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Hamlet"}"#)).await,
            (
                StatusCode::CREATED,
                r#"{"id":3,"title":"Hamlet"}"#.to_string()
            )
        );
    }
}
