//! Nesting and modular routers, Your turn (From scratch): the shared-state app,
//! split into a `state` module and a `books` module.
//! Run: `cargo run -p shared-state --example state-split-modules`.
// ANCHOR: modules
mod books;
mod state;
// ANCHOR_END: modules

// ANCHOR: imports
use axum::Router;
use state::AppState;
// ANCHOR_END: imports

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .nest("/books", books::router())
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

// The two tests below are the shared-state lesson's tests, unchanged.
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

    #[tokio::test]
    async fn added_books_are_remembered_by_the_same_app() {
        let app = app();
        assert_eq!(send_to(app.clone(), get("/books")).await.1, "[]");
        let (status, body) = send_to(app.clone(), post_json("/books", r#"{"title":"Dune"}"#)).await;
        assert_eq!(
            (status, body.as_str()),
            (StatusCode::CREATED, r#"{"id":1,"title":"Dune"}"#)
        );
        send_to(app.clone(), post_json("/books", r#"{"title":"Emma"}"#)).await;
        assert_eq!(
            send_to(app, get("/books")).await.1,
            r#"[{"id":1,"title":"Dune"},{"id":2,"title":"Emma"}]"#
        );
    }

    #[tokio::test]
    async fn a_new_app_starts_empty() {
        assert_eq!(send(get("/books")).await.1, "[]");
    }
}
