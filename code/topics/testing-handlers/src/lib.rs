// ANCHOR: lib
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

pub fn app() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/notes", get(list_notes).post(add_note))
        .with_state(AppState::default())
}
// ANCHOR_END: lib
