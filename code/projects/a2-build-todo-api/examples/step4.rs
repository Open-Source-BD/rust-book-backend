//! Step 4: `PATCH /todos/{id}` changes only the fields you send; `DELETE /todos/{id}` removes.
//! Run: `cargo run -p a2-build-todo-api --example step4`
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

// ANCHOR: update_todo
#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}
// ANCHOR_END: update_todo

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

// ANCHOR: update
async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateTodo>,
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
// ANCHOR_END: update

// ANCHOR: remove
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
// ANCHOR_END: remove

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", get(list).post(create))
        // ANCHOR: route
        .route("/todos/{id}", get(show).patch(update).delete(remove))
        // ANCHOR_END: route
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
    async fn patch_changes_only_what_you_send() {
        let app = app();
        send(
            app.clone(),
            "POST",
            "/todos",
            Some(r#"{"title":"buy milk"}"#),
        )
        .await;
        assert_eq!(
            send(app.clone(), "PATCH", "/todos/1", Some(r#"{"done":true}"#)).await,
            (
                StatusCode::OK,
                r#"{"id":1,"title":"buy milk","done":true}"#.into()
            )
        );
        assert_eq!(
            send(
                app,
                "PATCH",
                "/todos/1",
                Some(r#"{"title":"buy oat milk"}"#)
            )
            .await,
            (
                StatusCode::OK,
                r#"{"id":1,"title":"buy oat milk","done":true}"#.into()
            )
        );
    }

    #[tokio::test]
    async fn delete_then_ids_are_never_reused() {
        let app = app();
        send(app.clone(), "POST", "/todos", Some(r#"{"title":"one"}"#)).await;
        send(app.clone(), "POST", "/todos", Some(r#"{"title":"two"}"#)).await;
        assert_eq!(
            send(app.clone(), "DELETE", "/todos/1", None).await,
            (StatusCode::NO_CONTENT, String::new())
        );
        assert_eq!(
            send(app.clone(), "DELETE", "/todos/1", None).await.0,
            StatusCode::NOT_FOUND
        );
        let (status, body) = send(app, "POST", "/todos", Some(r#"{"title":"three"}"#)).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body, r#"{"id":3,"title":"three","done":false}"#);
    }
}
