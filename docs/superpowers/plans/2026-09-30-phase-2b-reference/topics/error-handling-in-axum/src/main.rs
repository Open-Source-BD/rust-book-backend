// ANCHOR: imports
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
// ANCHOR_END: imports

// ANCHOR: types
#[derive(Clone, Serialize)]
struct Book {
    id: u32,
    title: String,
}

#[derive(Clone)]
struct AppState {
    books: Arc<Mutex<Vec<Book>>>,
}
// ANCHOR_END: types

// ANCHOR: error
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("book {0} not found")]
    NotFound(u32),
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}
// ANCHOR_END: error

// ANCHOR: into_response
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
// ANCHOR_END: into_response

// ANCHOR: handler
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
// ANCHOR_END: handler

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
        .with_state(AppState {
            books: Arc::new(Mutex::new(books)),
        })
}
// ANCHOR_END: app

// ANCHOR: main
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
// ANCHOR_END: main

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

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        send_to(app(), request).await
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[allow(dead_code)]
    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn found_book_is_200_json() {
        assert_eq!(
            send(get("/books/2")).await,
            (StatusCode::OK, r#"{"id":2,"title":"Emma"}"#.to_string())
        );
    }

    #[tokio::test]
    async fn missing_book_is_404_json_error() {
        assert_eq!(
            send(get("/books/9")).await,
            (
                StatusCode::NOT_FOUND,
                r#"{"error":"book 9 not found"}"#.to_string()
            )
        );
    }

    #[test]
    fn poisoned_maps_to_500() {
        assert_eq!(
            AppError::Poisoned.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
