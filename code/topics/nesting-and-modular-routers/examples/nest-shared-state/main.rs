//! More examples: two routers in two modules, sharing one `AppState`.
//! Run: `cargo run -p nesting-and-modular-routers --example nest-shared-state`.
// ANCHOR: modules
mod books;
mod health;
// ANCHOR_END: modules

// ANCHOR: imports
use axum::Router;
use std::sync::{Arc, atomic::AtomicU64};
// ANCHOR_END: imports

// ANCHOR: state
#[derive(Clone, Default)]
struct AppState {
    lookups: Arc<AtomicU64>,
}
// ANCHOR_END: state

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .merge(health::router())
        .nest("/api/books", books::router())
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
    async fn both_routers_see_the_same_state() {
        let app = app();
        assert_eq!(
            send_to(app.clone(), get("/health")).await,
            (StatusCode::OK, "ok, 0 book lookups so far".to_string())
        );
        assert_eq!(
            send_to(app.clone(), get("/api/books/7")).await,
            (StatusCode::OK, "book 7".to_string())
        );
        send_to(app.clone(), get("/api/books/8")).await;
        assert_eq!(
            send_to(app, get("/health")).await,
            (StatusCode::OK, "ok, 2 book lookups so far".to_string())
        );
    }
}
