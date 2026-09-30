//! Common mistakes, fixed: let go of the lock before the `.await`.
//! Run: `cargo run -p shared-state --example state-short-lock`, then POST and GET /books.
// ANCHOR: imports
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
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

#[derive(Clone, Default)]
struct AppState {
    books: Arc<Mutex<Vec<Book>>>,
}

async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {
    let books = state.books.lock().unwrap();
    Json(books.clone())
}

// ANCHOR: add_book
async fn add_book(
    State(state): State<AppState>,
    Json(input): Json<NewBook>,
) -> (StatusCode, Json<Book>) {
    let book = {
        let mut books = state.books.lock().unwrap();
        let book = Book {
            id: books.len() as u32 + 1,
            title: input.title,
        };
        books.push(book.clone());
        book
    }; // the lock is released here, before the .await below

    // Pretend to do something slow, such as sending an email.
    tokio::time::sleep(Duration::from_millis(100)).await;
    (StatusCode::CREATED, Json(book))
}
// ANCHOR_END: add_book

fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
        .with_state(AppState::default())
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

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn the_book_is_saved_and_returned() {
        let app = app();
        let (status, body) = send_to(app.clone(), post_json("/books", r#"{"title":"Dune"}"#)).await;
        assert_eq!(
            (status, body.as_str()),
            (StatusCode::CREATED, r#"{"id":1,"title":"Dune"}"#)
        );
        assert_eq!(
            send_to(app, get("/books")).await.1,
            r#"[{"id":1,"title":"Dune"}]"#
        );
    }
}
