//! Step 2: `GET /todos` lists, `POST /todos` adds. The list lives in memory.
//! Run: `cargo run -p a2-build-todo-api --example step2`
// ANCHOR: all
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Default)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

async fn health() -> &'static str {
    "ok"
}

async fn list(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(state.todos.lock().unwrap().clone())
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewTodo>,
) -> (StatusCode, Json<Todo>) {
    let mut todos = state.todos.lock().unwrap();
    let todo = Todo {
        id: todos.len() as u64 + 1,
        title: input.title,
        done: false,
    };
    todos.push(todo.clone());
    (StatusCode::CREATED, Json(todo))
}

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", get(list).post(create))
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
    async fn create_then_list() {
        let app = app();
        assert_eq!(
            send(app.clone(), "GET", "/todos", None).await,
            (StatusCode::OK, "[]".into())
        );
        assert_eq!(
            send(
                app.clone(),
                "POST",
                "/todos",
                Some(r#"{"title":"buy milk"}"#)
            )
            .await,
            (
                StatusCode::CREATED,
                r#"{"id":1,"title":"buy milk","done":false}"#.into()
            )
        );
        assert_eq!(
            send(app, "GET", "/todos", None).await,
            (
                StatusCode::OK,
                r#"[{"id":1,"title":"buy milk","done":false}]"#.into()
            )
        );
    }
}
