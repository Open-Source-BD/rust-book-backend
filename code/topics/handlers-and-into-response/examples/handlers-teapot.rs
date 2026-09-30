//! Your turn (Tweak): `/teapot` answers 418 with a body.
//! Run: `cargo run -p handlers-and-into-response --example handlers-teapot`, then
//! `curl -i http://127.0.0.1:3000/teapot`
// ANCHOR: imports
use axum::{Router, http::StatusCode, routing::get};
// ANCHOR_END: imports

// ANCHOR: teapot
async fn teapot() -> (StatusCode, &'static str) {
    (StatusCode::IM_A_TEAPOT, "I'm a teapot: I can't brew coffee")
}
// ANCHOR_END: teapot

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/teapot", get(teapot))
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

    async fn send(uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn teapot_answers_418() {
        assert_eq!(
            send("/teapot").await,
            (
                StatusCode::IM_A_TEAPOT,
                "I'm a teapot: I can't brew coffee".to_string()
            )
        );
    }
}
