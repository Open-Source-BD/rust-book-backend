//! More examples: `Redirect::to`, sending the client to another address.
//! Run: `cargo run -p handlers-and-into-response --example handlers-redirect`, then
//! `curl -i http://127.0.0.1:3000/old-text`
// ANCHOR: imports
use axum::{Router, response::Redirect, routing::get};
// ANCHOR_END: imports

// ANCHOR: handlers
async fn text() -> &'static str {
    "plain text"
}

async fn old_text() -> Redirect {
    Redirect::to("/text")
}
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/text", get(text))
        .route("/old-text", get(old_text))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn old_address_redirects_with_303() {
        let request = Request::get("/old-text").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/text");
    }
}
