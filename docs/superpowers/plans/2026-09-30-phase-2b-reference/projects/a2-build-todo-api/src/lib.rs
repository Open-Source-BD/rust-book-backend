mod error;
mod todos;
mod validated;

use axum::Router;

pub fn app() -> Router {
    Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest("/api/todos", todos::router())
}
