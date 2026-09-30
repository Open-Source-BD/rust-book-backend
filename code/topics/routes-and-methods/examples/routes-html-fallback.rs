//! More examples: a fallback that answers with an HTML page.
//! Run: `cargo run -p routes-and-methods --example routes-html-fallback`, then
//! `curl -i http://127.0.0.1:3000/nope`
// ANCHOR: imports
use axum::{Router, http::StatusCode, response::Html, routing::get};
// ANCHOR_END: imports

async fn home() -> Html<&'static str> {
    Html("<h1>The library</h1>")
}

// ANCHOR: not_found
async fn not_found() -> (StatusCode, Html<&'static str>) {
    (
        StatusCode::NOT_FOUND,
        Html("<h1>Page not found</h1><p><a href=\"/\">Back to the library</a></p>"),
    )
}
// ANCHOR_END: not_found

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/", get(home)).fallback(not_found)
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
    async fn unknown_path_gets_the_html_page() {
        let (status, body) = send("GET", "/nope").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.starts_with("<h1>Page not found</h1>"));
    }
}
