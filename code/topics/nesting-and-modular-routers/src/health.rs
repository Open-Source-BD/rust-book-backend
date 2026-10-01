// ANCHOR: health
use axum::{Router, routing::get};

async fn health() -> &'static str {
    "ok"
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}
// ANCHOR_END: health
