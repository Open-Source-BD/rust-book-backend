//! Step 3: a `next_id` counter, and `GET /todos/{id}` that answers 404 for a missing todo.
//! Run: `cargo run -p a2-build-todo-api --example step3`
// ANCHOR: all
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize)]
struct Todo {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
}

// ANCHOR: store
#[derive(Default)]
struct Store {
    next_id: u64,
    todos: Vec<Todo>,
}

#[derive(Clone, Default)]
struct AppState {
    store: Arc<Mutex<Store>>,
}
// ANCHOR_END: store

// ANCHOR: error
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("todo {0} not found")]
    NotFound(u64),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
// ANCHOR_END: error

async fn health() -> &'static str {
    "ok"
}

async fn list(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(state.store.lock().unwrap().todos.clone())
}

// ANCHOR: create
async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewTodo>,
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
// ANCHOR_END: create

// ANCHOR: show
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
// ANCHOR_END: show

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", get(list).post(create))
        .route("/todos/{id}", get(show))
        .with_state(AppState::default())
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is busy: stop the other program using it, or change the port");
    println!("Todo API listening on http://127.0.0.1:3000");
    axum::serve(listener, app())
        .await
        .expect("the server stopped because of an error");
}
// ANCHOR_END: all

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(
        app: Router,
        method: &str,
        uri: &str,
        body: Option<&str>,
    ) -> (StatusCode, String) {
        let mut builder = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(json) => {
                builder = builder.header("content-type", "application/json");
                Body::from(json.to_owned())
            }
            None => Body::empty(),
        };
        let response = app.oneshot(builder.body(body).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn show_finds_a_todo_or_answers_404() {
        let app = app();
        send(
            app.clone(),
            "POST",
            "/todos",
            Some(r#"{"title":"buy milk"}"#),
        )
        .await;
        assert_eq!(
            send(app.clone(), "GET", "/todos/1", None).await,
            (
                StatusCode::OK,
                r#"{"id":1,"title":"buy milk","done":false}"#.into()
            )
        );
        assert_eq!(
            send(app, "GET", "/todos/99", None).await,
            (
                StatusCode::NOT_FOUND,
                r#"{"error":"todo 99 not found"}"#.into()
            )
        );
    }
}
