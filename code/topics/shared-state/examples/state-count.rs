//! Your turn (Guided): GET /books/count answers with how many books there are.
//! Run: `cargo run -p shared-state --example state-count`, add books, then GET /books/count.
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Default)]
struct AppState {
    books: Arc<Mutex<Vec<Book>>>,
}

async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {
    let books = state.books.lock().unwrap();
    Json(books.clone())
}

async fn add_book(
    State(state): State<AppState>,
    Json(input): Json<NewBook>,
) -> (StatusCode, Json<Book>) {
    let mut books = state.books.lock().unwrap();
    let book = Book {
        id: books.len() as u32 + 1,
        title: input.title,
    };
    books.push(book.clone());
    (StatusCode::CREATED, Json(book))
}

// ANCHOR: count
async fn count_books(State(state): State<AppState>) -> Json<usize> {
    let books = state.books.lock().unwrap();
    Json(books.len())
}
// ANCHOR_END: count

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
        .route("/books/count", get(count_books))
        .with_state(AppState::default())
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
    async fn counts_the_books() {
        let app = app();
        assert_eq!(send_to(app.clone(), get("/books/count")).await.1, "0");
        send_to(app.clone(), post_json("/books", r#"{"title":"Dune"}"#)).await;
        send_to(app.clone(), post_json("/books", r#"{"title":"Emma"}"#)).await;
        assert_eq!(
            send_to(app, get("/books/count")).await,
            (StatusCode::OK, "2".to_string())
        );
    }
}
