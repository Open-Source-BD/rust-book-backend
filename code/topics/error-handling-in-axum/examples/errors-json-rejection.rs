//! More examples: `From<JsonRejection>` and `From<PathRejection>` for AppError, so Axum's own
//! rejections become JSON errors too.
//! Run: `cargo run -p error-handling-in-axum --example errors-json-rejection`.
// ANCHOR: imports
use axum::{
    Json, Router,
    extract::{
        Path, State,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
// ANCHOR_END: imports

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
    #[error("{message}")]
    Rejected { status: StatusCode, message: String },
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Rejected { status, .. } => status,
            AppError::Poisoned => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}
// ANCHOR_END: error

// ANCHOR: from
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}
// ANCHOR_END: from

// ANCHOR: handlers
async fn show_book(
    State(state): State<AppState>,
    path: Result<Path<u32>, PathRejection>,
) -> Result<Json<Book>, AppError> {
    let Path(id) = path?;
    let books = state.books.lock().map_err(|_| AppError::Poisoned)?;
    let book = books
        .iter()
        .find(|book| book.id == id)
        .cloned()
        .ok_or(AppError::NotFound(id))?;
    Ok(Json(book))
}

async fn add_book(
    State(state): State<AppState>,
    body: Result<Json<NewBook>, JsonRejection>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    let Json(input) = body?;
    let mut books = state.books.lock().map_err(|_| AppError::Poisoned)?;
    let book = Book {
        id: books.len() as u32 + 1,
        title: input.title,
    };
    books.push(book.clone());
    Ok((StatusCode::CREATED, Json(book)))
}
// ANCHOR_END: handlers

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
    async fn bad_path_is_400_json() {
        let request = Request::get("/books/abc").body(Body::empty()).unwrap();
        assert_eq!(
            send(request).await,
            (
                StatusCode::BAD_REQUEST,
                r#"{"error":"Invalid URL: Cannot parse `abc` to a `u32`"}"#.to_string()
            )
        );
    }

    #[tokio::test]
    async fn broken_json_is_400_json() {
        let (status, body) = send(post_json("/books", r#"{"title":"Emma""#)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.starts_with(r#"{"error":"Failed to parse the request body as JSON"#));
    }

    #[tokio::test]
    async fn missing_field_is_422_json() {
        let (status, body) = send(post_json("/books", "{}")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("missing field `title`"));
    }
}
