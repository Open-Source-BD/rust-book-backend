//! More examples: a visit counter in an `AtomicU64`, with no `Mutex`.
//! Run: `cargo run -p shared-state --example state-visit-counter`, then
//! `curl -i http://127.0.0.1:3000/visits` a few times.
// ANCHOR: imports
use axum::{Router, extract::State, routing::get};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
// ANCHOR_END: imports

// ANCHOR: state
#[derive(Clone, Default)]
struct AppState {
    visits: Arc<AtomicU64>,
}
// ANCHOR_END: state

// ANCHOR: handler
async fn visit(State(state): State<AppState>) -> String {
    let number = state.visits.fetch_add(1, Ordering::Relaxed) + 1;
    format!("You are visitor number {number}")
}
// ANCHOR_END: handler

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/visits", get(visit))
        .with_state(AppState::default())
}
// ANCHOR_END: app

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is busy: stop the other program using it, or change the port");
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app())
        .await
        .expect("the server stopped because of an error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn each_visit_gets_the_next_number() {
        let app = app();
        assert_eq!(
            send_to(app.clone(), get("/visits")).await,
            (StatusCode::OK, "You are visitor number 1".to_string())
        );
        send_to(app.clone(), get("/visits")).await;
        assert_eq!(
            send_to(app, get("/visits")).await.1,
            "You are visitor number 3"
        );
    }
}
