//! More examples: a path with only `.post`, so `GET` gets 405.
//! Run: `cargo run -p routes-and-methods --example routes-post-only`, then
//! `curl -i http://127.0.0.1:3000/messages`
// ANCHOR: imports
use axum::{Router, http::StatusCode, routing::post};
// ANCHOR_END: imports

// ANCHOR: send_message
async fn send_message() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "message sent")
}
// ANCHOR_END: send_message

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/messages", post(send_message))
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
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(method: &str, uri: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn post_works_and_get_is_405() {
        assert_eq!(
            send("POST", "/messages").await,
            (StatusCode::CREATED, "message sent".to_string())
        );
        assert_eq!(
            send("GET", "/messages").await.0,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
