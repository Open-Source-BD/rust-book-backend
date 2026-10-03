// ANCHOR: modules
mod authors;
// ANCHOR_END: modules

// ANCHOR: imports
use axum::{
    Json, Router,
    extract::{
        FromRequest, FromRequestParts, Path, Query, Request, State, rejection::JsonRejection,
    },
    http::{HeaderValue, StatusCode, header, request::Parts},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use validator::Validate;
// ANCHOR_END: imports

// ANCHOR: start
async fn hello() -> &'static str {
    "hello, Axum"
}

fn hello_routes() -> Router {
    Router::new().route("/", get(hello))
}
// ANCHOR_END: start

// ANCHOR: methods
async fn list_notes() -> &'static str {
    "all notes"
}

async fn add_note() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "note added")
}

async fn update_note() -> &'static str {
    "note updated"
}

async fn delete_note() -> StatusCode {
    StatusCode::NO_CONTENT
}

fn note_routes() -> Router {
    Router::new()
        .route("/notes", get(list_notes).post(add_note))
        .route("/notes/{id}", put(update_note).delete(delete_note))
}
// ANCHOR_END: methods

// ANCHOR: status_headers
async fn create_draft() -> impl IntoResponse {
    (
        StatusCode::CREATED,
        [(header::LOCATION, "/drafts/1")],
        "draft 1 saved",
    )
}

fn draft_routes() -> Router {
    Router::new().route("/drafts", post(create_draft))
}
// ANCHOR_END: status_headers

// ANCHOR: path
async fn show_book(Path(id): Path<u32>) -> String {
    format!("book number {id}")
}

async fn show_chapter(Path((book, chapter)): Path<(u32, u32)>) -> String {
    format!("book {book}, chapter {chapter}")
}

fn book_routes() -> Router {
    Router::new()
        .route("/books/{id}", get(show_book))
        .route("/books/{book}/chapters/{chapter}", get(show_chapter))
}
// ANCHOR_END: path

// ANCHOR: query
#[derive(Deserialize)]
struct Search {
    q: String,
    page: Option<u32>,
}

async fn search(Query(params): Query<Search>) -> String {
    let page = params.page.unwrap_or(1);
    format!("searching for {:?}, page {page}", params.q)
}

fn search_routes() -> Router {
    Router::new().route("/search", get(search))
}
// ANCHOR_END: query

// ANCHOR: json
#[derive(Deserialize)]
struct NewBook {
    title: String,
}

#[derive(Serialize)]
struct Book {
    id: u32,
    title: String,
}

async fn create_book(Json(input): Json<NewBook>) -> (StatusCode, Json<Book>) {
    let book = Book {
        id: 1,
        title: input.title,
    };
    (StatusCode::CREATED, Json(book))
}

fn json_routes() -> Router {
    Router::new().route("/books", post(create_book))
}
// ANCHOR_END: json

// ANCHOR: state
#[derive(Clone, Default)]
struct AppState {
    visits: Arc<Mutex<u32>>,
}

async fn count_visit(State(state): State<AppState>) -> String {
    let mut visits = state.visits.lock().unwrap();
    *visits += 1;
    format!("visit number {visits}")
}

fn visit_routes() -> Router {
    Router::new()
        .route("/visits", get(count_visit))
        .with_state(AppState::default())
}
// ANCHOR_END: state

// ANCHOR: errors
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("book {0} not found")]
    NotFound(u32),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        };
        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

fn find_title(id: u32) -> Result<&'static str, AppError> {
    match id {
        1 => Ok("Dune"),
        2 => Ok("Emma"),
        _ => Err(AppError::NotFound(id)),
    }
}

async fn show_title(Path(id): Path<u32>) -> Result<String, AppError> {
    let title = find_title(id)?;
    Ok(format!("book {id} is {title}"))
}

fn title_routes() -> Router {
    Router::new().route("/titles/{id}", get(show_title))
}
// ANCHOR_END: errors

// ANCHOR: middleware
async fn add_powered_by(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-powered-by", HeaderValue::from_static("axum"));
    response
}
// ANCHOR_END: middleware

// ANCHOR: extractor
struct ApiKey(String);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
        if value == "letmein" {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
        }
    }
}

async fn secret(ApiKey(key): ApiKey) -> String {
    format!("welcome, holder of key {key:?}")
}

fn secret_routes() -> Router {
    Router::new().route("/secret", get(secret))
}
// ANCHOR_END: extractor

// ANCHOR: validation
#[derive(Deserialize, Validate)]
struct SignUp {
    #[validate(length(min = 3, max = 20))]
    username: String,
    #[validate(email)]
    email: String,
}

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

async fn sign_up(ValidatedJson(input): ValidatedJson<SignUp>) -> (StatusCode, String) {
    (
        StatusCode::CREATED,
        format!("welcome, {} <{}>", input.username, input.email),
    )
}

fn sign_up_routes() -> Router {
    Router::new().route("/sign-up", post(sign_up))
}
// ANCHOR_END: validation

// ANCHOR: slow
async fn slow() -> &'static str {
    tokio::time::sleep(Duration::from_secs(3)).await;
    "finally done"
}
// ANCHOR_END: slow

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .merge(hello_routes())
        .merge(note_routes())
        .merge(draft_routes())
        .merge(book_routes())
        .merge(search_routes())
        .merge(json_routes())
        .merge(visit_routes())
        .merge(title_routes())
        .merge(secret_routes())
        .merge(sign_up_routes())
        .route("/slow", get(slow))
        .nest("/api/authors", authors::router())
        .layer(middleware::from_fn(add_powered_by))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(1),
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
// ANCHOR_END: app

// ANCHOR: main
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("tower_http=debug")
        .init();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is busy: stop the other program using it, or change the port");
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app())
        .await
        .expect("the server stopped because of an error");
}
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    // ANCHOR: test_helpers
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

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        send_to(app(), request).await
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
    // ANCHOR_END: test_helpers

    fn with(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn start_a_server() {
        assert_eq!(
            send(get("/")).await,
            (StatusCode::OK, "hello, Axum".to_string())
        );
    }

    #[tokio::test]
    async fn route_by_method() {
        assert_eq!(
            send(get("/notes")).await,
            (StatusCode::OK, "all notes".to_string())
        );
        assert_eq!(
            send(with("POST", "/notes")).await,
            (StatusCode::CREATED, "note added".to_string())
        );
        assert_eq!(
            send(with("PUT", "/notes/1")).await,
            (StatusCode::OK, "note updated".to_string())
        );
        assert_eq!(
            send(with("DELETE", "/notes/1")).await.0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            send(with("PATCH", "/notes")).await.0,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn return_status_and_headers() {
        let response = app().oneshot(with("POST", "/drafts")).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers()["location"], "/drafts/1");
    }

    #[tokio::test]
    async fn read_a_path_value() {
        assert_eq!(
            send(get("/books/42")).await,
            (StatusCode::OK, "book number 42".to_string())
        );
        assert_eq!(
            send(get("/books/3/chapters/7")).await,
            (StatusCode::OK, "book 3, chapter 7".to_string())
        );
        assert_eq!(send(get("/books/abc")).await.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn read_the_query_string() {
        assert_eq!(
            send(get("/search?q=rust&page=2")).await,
            (StatusCode::OK, "searching for \"rust\", page 2".to_string())
        );
        assert_eq!(
            send(get("/search?q=rust")).await.1,
            "searching for \"rust\", page 1"
        );
        assert_eq!(send(get("/search")).await.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn receive_and_send_json() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Dune"}"#)).await,
            (
                StatusCode::CREATED,
                r#"{"id":1,"title":"Dune"}"#.to_string()
            )
        );
        assert_eq!(
            send(post_json("/books", r#"{"name":"Dune"}"#)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[tokio::test]
    async fn share_state() {
        let app = app();
        assert_eq!(
            send_to(app.clone(), get("/visits")).await.1,
            "visit number 1"
        );
        assert_eq!(send_to(app, get("/visits")).await.1, "visit number 2");
    }

    #[tokio::test]
    async fn return_errors_with_question_mark() {
        assert_eq!(
            send(get("/titles/2")).await,
            (StatusCode::OK, "book 2 is Emma".to_string())
        );
        assert_eq!(
            send(get("/titles/9")).await,
            (
                StatusCode::NOT_FOUND,
                r#"{"error":"book 9 not found"}"#.to_string()
            )
        );
    }

    #[tokio::test]
    async fn add_middleware() {
        let response = app().oneshot(get("/")).await.unwrap();
        assert_eq!(response.headers()["x-powered-by"], "axum");
        let response = app().oneshot(get("/missing")).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()["x-powered-by"], "axum");
    }

    #[tokio::test]
    async fn add_cors_timeout_and_logging_layers() {
        let request = Request::get("/")
            .header("origin", "https://example.com")
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.headers()["access-control-allow-origin"], "*");
        assert_eq!(send(get("/slow")).await.0, StatusCode::REQUEST_TIMEOUT);
    }

    #[tokio::test]
    async fn split_into_modules_and_nest() {
        assert_eq!(
            send(get("/api/authors")).await,
            (StatusCode::OK, "all authors".to_string())
        );
        assert_eq!(
            send(get("/api/authors/7")).await,
            (StatusCode::OK, "author 7".to_string())
        );
        assert_eq!(send(get("/authors")).await.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn write_a_custom_extractor() {
        assert_eq!(
            send(get("/secret")).await,
            (
                StatusCode::UNAUTHORIZED,
                "missing x-api-key header".to_string()
            )
        );
        let request = Request::get("/secret")
            .header("x-api-key", "letmein")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            send(request).await,
            (
                StatusCode::OK,
                "welcome, holder of key \"letmein\"".to_string()
            )
        );
    }

    #[tokio::test]
    async fn validate_input() {
        assert_eq!(
            send(post_json(
                "/sign-up",
                r#"{"username":"ada","email":"ada@example.com"}"#
            ))
            .await,
            (
                StatusCode::CREATED,
                "welcome, ada <ada@example.com>".to_string()
            )
        );
        let (status, body) =
            send(post_json("/sign-up", r#"{"username":"al","email":"nope"}"#)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("\"username\"") && body.contains("\"email\""));
    }

    // ANCHOR: test_example
    #[tokio::test]
    async fn a_new_book_comes_back_as_json() {
        let (status, body) = send(post_json("/books", r#"{"title":"Emma"}"#)).await;
        assert_eq!(status, StatusCode::CREATED);
        let book: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(book["title"], "Emma");
    }
    // ANCHOR_END: test_example
}
