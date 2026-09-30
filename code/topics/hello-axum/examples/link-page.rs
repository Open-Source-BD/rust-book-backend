//! More examples: a home page with a link to `/about`.
//! Run: `cargo run -p hello-axum --example link-page`, then `curl -i http://127.0.0.1:3000/`
use axum::{Router, response::Html, routing::get};

// ANCHOR: home
async fn home() -> Html<&'static str> {
    Html(r#"<h1>Welcome to my first Axum app</h1><p><a href="/about">About this server</a></p>"#)
}
// ANCHOR_END: home

async fn about() -> &'static str {
    "This server is written in Rust."
}

fn app() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
}

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

    #[tokio::test]
    async fn home_links_to_about() {
        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["content-type"],
            "text/html; charset=utf-8"
        );
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body.contains(r#"<a href="/about">"#));
    }
}
