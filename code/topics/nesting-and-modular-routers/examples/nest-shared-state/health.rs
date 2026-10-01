// ANCHOR: health
use crate::AppState;
use axum::{Router, extract::State, routing::get};
use std::sync::atomic::Ordering;

async fn health(State(state): State<AppState>) -> String {
    let lookups = state.lookups.load(Ordering::Relaxed);
    format!("ok, {lookups} book lookups so far")
}

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
// ANCHOR_END: health
