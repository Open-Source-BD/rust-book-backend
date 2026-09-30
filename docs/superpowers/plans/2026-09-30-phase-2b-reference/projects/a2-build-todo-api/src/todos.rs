use crate::{error::AppError, validated::ValidatedJson};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

#[derive(Deserialize, Validate)]
pub struct NewTodo {
    #[validate(length(min = 1, max = 100))]
    pub title: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateTodo {
    #[validate(length(min = 1, max = 100))]
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[derive(Default)]
struct Store {
    next_id: u64,
    todos: Vec<Todo>,
}

#[derive(Clone, Default)]
struct AppState {
    store: Arc<Mutex<Store>>,
}

async fn list(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(state.store.lock().unwrap().todos.clone())
}

async fn create(
    State(state): State<AppState>,
    ValidatedJson(input): ValidatedJson<NewTodo>,
) -> (StatusCode, Json<Todo>) {
    let mut store = state.store.lock().unwrap();
    store.next_id += 1;
    let todo = Todo {
        id: store.next_id,
        title: input.title,
        done: false,
    };
    store.todos.push(todo.clone());
    (StatusCode::CREATED, Json(todo))
}

async fn show(State(state): State<AppState>, Path(id): Path<u64>) -> Result<Json<Todo>, AppError> {
    let store = state.store.lock().unwrap();
    let todo = store
        .todos
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .ok_or(AppError::NotFound(id))?;
    Ok(Json(todo))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    ValidatedJson(input): ValidatedJson<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    let mut store = state.store.lock().unwrap();
    let todo = store
        .todos
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or(AppError::NotFound(id))?;
    if let Some(title) = input.title {
        todo.title = title;
    }
    if let Some(done) = input.done {
        todo.done = done;
    }
    Ok(Json(todo.clone()))
}

async fn remove(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    let mut store = state.store.lock().unwrap();
    let before = store.todos.len();
    store.todos.retain(|t| t.id != id);
    if store.todos.len() == before {
        return Err(AppError::NotFound(id));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(show).patch(update).delete(remove))
        .with_state(AppState::default())
}
