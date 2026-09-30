// ANCHOR: imports
use axum::{Router, response::Html, routing::get};
// ANCHOR_END: imports

// ANCHOR: handlers
async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}

async fn about() -> &'static str {
    "This server is written in Rust."
}
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
}
// ANCHOR_END: app

// ANCHOR: main
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
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn home_is_an_html_page() {
        let response = app().oneshot(get("/")).await.unwrap();
        assert_eq!(
            response.headers()["content-type"],
            "text/html; charset=utf-8"
        );
        let (status, body) = send(get("/")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "<h1>Welcome to my first Axum app</h1>");
    }

    #[tokio::test]
    async fn about_is_plain_text() {
        assert_eq!(
            send(get("/about")).await,
            (
                StatusCode::OK,
                "This server is written in Rust.".to_string()
            )
        );
    }

    #[tokio::test]
    async fn unknown_path_is_404() {
        assert_eq!(send(get("/contact")).await.0, StatusCode::NOT_FOUND);
    }
}
