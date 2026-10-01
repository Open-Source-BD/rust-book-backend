//! Your turn (Tweak): an authors.rs module, mounted at /api/authors.
//! Run: `cargo run -p nesting-and-modular-routers --example nest-authors`.
// ANCHOR: modules
mod authors;
mod books;
mod health;
// ANCHOR_END: modules

use axum::Router;

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .merge(health::router())
        .nest("/api/books", books::router())
        .nest("/api/authors", authors::router())
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
    async fn authors_and_books_side_by_side() {
        assert_eq!(
            send(get("/api/authors")).await,
            (StatusCode::OK, "all authors".to_string())
        );
        assert_eq!(
            send(get("/api/authors/3")).await,
            (StatusCode::OK, "author 3".to_string())
        );
        assert_eq!(
            send(get("/api/books/7")).await,
            (StatusCode::OK, "book 7".to_string())
        );
    }
}
