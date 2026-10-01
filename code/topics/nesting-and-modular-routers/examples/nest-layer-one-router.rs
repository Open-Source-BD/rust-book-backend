//! More examples: a middleware layer on one nested router only.
//! Run: `cargo run -p nesting-and-modular-routers --example nest-layer-one-router`.
// ANCHOR: imports
use axum::{
    Router,
    extract::{Path, Request},
    http::HeaderValue,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
// ANCHOR_END: imports

async fn health() -> &'static str {
    "ok"
}

async fn show_book(Path(id): Path<u32>) -> String {
    format!("book {id}")
}

// ANCHOR: middleware
async fn add_area(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-area", HeaderValue::from_static("books"));
    response
}
// ANCHOR_END: middleware

// ANCHOR: app
fn books_router() -> Router {
    Router::new()
        .route("/{id}", get(show_book))
        .layer(middleware::from_fn(add_area))
}

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api/books", books_router())
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
    async fn only_the_books_routes_get_the_header() {
        let books = app().oneshot(get("/api/books/7")).await.unwrap();
        assert_eq!(books.headers()["x-area"], "books");
        let health = app().oneshot(get("/health")).await.unwrap();
        assert!(health.headers().get("x-area").is_none());
        assert_eq!(send(get("/health")).await.1, "ok");
    }
}
