//! Your turn (From scratch): POST /likes/{id} adds a like to a book; GET /likes/{id} reads them.
//! Run: `cargo run -p shared-state --example state-likes`, then POST and GET /likes/1.
// ANCHOR: imports
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// ANCHOR_END: imports

// ANCHOR: types
#[derive(Serialize)]
struct Likes {
    book_id: u32,
    likes: u32,
}

#[derive(Clone, Default)]
struct AppState {
    likes: Arc<Mutex<HashMap<u32, u32>>>,
}
// ANCHOR_END: types

// ANCHOR: handlers
async fn add_like(State(state): State<AppState>, Path(book_id): Path<u32>) -> Json<Likes> {
    let mut likes = state.likes.lock().unwrap();
    let count = likes.entry(book_id).or_insert(0);
    *count += 1;
    Json(Likes {
        book_id,
        likes: *count,
    })
}

async fn get_likes(State(state): State<AppState>, Path(book_id): Path<u32>) -> Json<Likes> {
    let likes = state.likes.lock().unwrap();
    let count = likes.get(&book_id).copied().unwrap_or(0);
    Json(Likes {
        book_id,
        likes: count,
    })
}
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/likes/{id}", get(get_likes).post(add_like))
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
    use axum::http::StatusCode;
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

    fn post(uri: &str) -> Request<Body> {
        Request::post(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn likes_are_counted_per_book() {
        let app = app();
        send_to(app.clone(), post("/likes/1")).await;
        assert_eq!(
            send_to(app.clone(), post("/likes/1")).await,
            (StatusCode::OK, r#"{"book_id":1,"likes":2}"#.to_string())
        );
        send_to(app.clone(), post("/likes/2")).await;
        assert_eq!(
            send_to(app.clone(), get("/likes/1")).await.1,
            r#"{"book_id":1,"likes":2}"#
        );
        assert_eq!(
            send_to(app, get("/likes/3")).await.1,
            r#"{"book_id":3,"likes":0}"#
        );
    }
}
