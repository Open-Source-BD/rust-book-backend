//! Your turn, From scratch: `GET /time` returns an HTML page with the seconds since the Unix epoch.
//! Run: `cargo run -p hello-axum --example time-page`, then `curl -i http://127.0.0.1:3000/time`
use axum::{Router, response::Html, routing::get};
// ANCHOR: use_time
use std::time::{SystemTime, UNIX_EPOCH};
// ANCHOR_END: use_time

async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}

async fn about() -> &'static str {
    "This server is written in Rust."
}

// ANCHOR: time
async fn time() -> Html<String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the computer's clock is set before 1970")
        .as_secs();
    Html(format!("<p>Seconds since 1970: {seconds}</p>"))
}
// ANCHOR_END: time

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .route("/time", get(time))
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

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn time_is_an_html_page() {
        let request = Request::get("/time").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["content-type"],
            "text/html; charset=utf-8"
        );
    }
}
// ANCHOR_END: test
