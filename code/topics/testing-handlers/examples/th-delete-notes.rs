//! Your turn (Tweak): `DELETE /notes` empties the list, with a test that proves it.
//! This is `src/lib.rs` after the change, with `main` and the test in the same file.
//! Run: `cargo run -p testing-handlers --example th-delete-notes`
//! Test: `cargo test -p testing-handlers --example th-delete-notes`
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Note {
    pub id: u32,
    pub text: String,
}

#[derive(Deserialize)]
pub struct NewNote {
    pub text: String,
}

#[derive(Clone, Default)]
pub struct AppState {
    notes: Arc<Mutex<Vec<Note>>>,
}

async fn list_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    Json(state.notes.lock().unwrap().clone())
}

async fn add_note(
    State(state): State<AppState>,
    Json(input): Json<NewNote>,
) -> (StatusCode, Json<Note>) {
    let mut notes = state.notes.lock().unwrap();
    let note = Note {
        id: notes.len() as u32 + 1,
        text: input.text,
    };
    notes.push(note.clone());
    (StatusCode::CREATED, Json(note))
}

// ANCHOR: handler
async fn clear_notes(State(state): State<AppState>) -> StatusCode {
    state.notes.lock().unwrap().clear();
    StatusCode::NO_CONTENT
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/notes", get(list_notes).post(add_note).delete(clear_notes))
        .with_state(AppState::default())
}
// ANCHOR_END: handler

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

    async fn send(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
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

    // ANCHOR: test
    #[tokio::test]
    async fn delete_empties_the_list() {
        let app = app();
        send(app.clone(), post_json("/notes", r#"{"text":"one"}"#)).await;
        send(app.clone(), post_json("/notes", r#"{"text":"two"}"#)).await;
        let delete = Request::delete("/notes").body(Body::empty()).unwrap();
        let (status, body) = send(app.clone(), delete).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(body, "");
        let list = Request::get("/notes").body(Body::empty()).unwrap();
        let (_, body) = send(app, list).await;
        assert_eq!(body, "[]");
    }
    // ANCHOR_END: test
}
