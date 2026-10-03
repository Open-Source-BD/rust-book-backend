//! Step 5: `ValidatedJson` refuses a title that is empty or longer than 100 characters.
//! Run: `cargo run -p a2-build-todo-api --example step5`
// ANCHOR: all
// ANCHOR: imports
use axum::{
    Json, Router,
    extract::{FromRequest, Path, Request, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use std::sync::{Arc, Mutex};
use validator::Validate;
// ANCHOR_END: imports

#[derive(Clone, Serialize)]
struct Todo {
    id: u64,
    title: String,
    done: bool,
}

// ANCHOR: inputs
#[derive(Deserialize, Validate)]
struct NewTodo {
    #[validate(length(min = 1, max = 100))]
    title: String,
}

#[derive(Deserialize, Validate)]
struct UpdateTodo {
    #[validate(length(min = 1, max = 100))]
    title: Option<String>,
    done: Option<bool>,
}
// ANCHOR_END: inputs

// ANCHOR: extractor
struct ValidatedJson<T>(T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| rejection.into_response())?;
        value.validate().map_err(|errors| {
            let sorted = serde_json::to_value(errors).expect("errors always convert to JSON");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(sorted)).into_response()
        })?;
        Ok(ValidatedJson(value))
    }
}
// ANCHOR_END: extractor

#[derive(Default)]
struct Store {
    next_id: u64,
    todos: Vec<Todo>,
}

#[derive(Clone, Default)]
struct AppState {
    store: Arc<Mutex<Store>>,
}

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

async fn health() -> &'static str {
    "ok"
}

async fn list(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(state.store.lock().unwrap().todos.clone())
}

// ANCHOR: create
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
// ANCHOR_END: create

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

// ANCHOR: update
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
// ANCHOR_END: update

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

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/todos", get(list).post(create))
        .route("/todos/{id}", get(show).patch(update).delete(remove))
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
    async fn an_empty_title_is_refused() {
        assert_eq!(
            send(app(), "POST", "/todos", Some(r#"{"title":""}"#)).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                r#"{"title":[{"code":"length","message":null,"params":{"max":100,"min":1,"value":""}}]}"#
                    .into()
            )
        );
    }

    #[tokio::test]
    async fn patch_is_validated_too_but_done_alone_is_fine() {
        let app = app();
        send(
            app.clone(),
            "POST",
            "/todos",
            Some(r#"{"title":"buy milk"}"#),
        )
        .await;
        assert_eq!(
            send(app.clone(), "PATCH", "/todos/1", Some(r#"{"title":""}"#))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            send(app, "PATCH", "/todos/1", Some(r#"{"done":true}"#)).await,
            (
                StatusCode::OK,
                r#"{"id":1,"title":"buy milk","done":true}"#.into()
            )
        );
    }
}
