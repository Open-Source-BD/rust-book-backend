//! Common mistake: `.unwrap()` instead of an error value. A missing book panics the handler, the
//! client gets no answer at all, and the panic poisons the lock for every later request.
//! Run: `cargo run -p error-handling-in-axum --example errors-unwrap-panics`, then GET /books/9.
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
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

// ANCHOR: handler
async fn show_book(State(state): State<AppState>, Path(id): Path<u32>) -> Json<Book> {
    let books = state.books.lock().unwrap();
    let book = books.iter().find(|book| book.id == id).cloned().unwrap();
    Json(book)
}
// ANCHOR_END: handler

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
    use axum::{body::Body, http::Request, http::StatusCode};
    use tower::ServiceExt;

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn found_book_still_works() {
        let response = app().oneshot(get("/books/2")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    async fn missing_book_panics_instead_of_answering() {
        let _ = app().oneshot(get("/books/9")).await;
    }

    #[tokio::test]
    async fn the_panic_poisons_the_lock_for_later_requests() {
        let app = app();
        let crashed = tokio::spawn(app.clone().oneshot(get("/books/9"))).await;
        assert!(crashed.is_err(), "the missing book should have panicked");
        let again = tokio::spawn(app.oneshot(get("/books/2"))).await;
        assert!(
            again.is_err(),
            "book 2 should now panic too: the lock is poisoned"
        );
    }
}
