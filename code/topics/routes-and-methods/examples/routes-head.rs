//! More examples: a `GET` route answers `HEAD` requests too.
//! Run: `cargo run -p routes-and-methods --example routes-head`, then
//! `curl -I http://127.0.0.1:3000/`
use axum::{Router, routing::get};

// ANCHOR: home
async fn home() -> &'static str {
    "Welcome to the library"
}
// ANCHOR_END: home

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/", get(home))
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
    async fn head_gets_the_status_but_no_body() {
        assert_eq!(
            send("GET", "/").await,
            (StatusCode::OK, "Welcome to the library".to_string())
        );
        assert_eq!(send("HEAD", "/").await, (StatusCode::OK, String::new()));
    }
}
