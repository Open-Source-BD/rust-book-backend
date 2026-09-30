// ANCHOR: imports
use axum::{Router, http::StatusCode, routing::get};
// ANCHOR_END: imports

// ANCHOR: handlers
async fn list_books() -> &'static str {
    "all books"
}

async fn add_book() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "book added")
}

async fn show_book() -> &'static str {
    "one book"
}

async fn update_book() -> &'static str {
    "book updated"
}

async fn delete_book() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "nothing lives here")
}
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
        .route(
            "/books/{id}",
            get(show_book).put(update_book).delete(delete_book),
        )
        .fallback(not_found)
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

    fn with(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn one_path_many_methods() {
        assert_eq!(
            send(get("/books")).await,
            (StatusCode::OK, "all books".to_string())
        );
        assert_eq!(
            send(with("POST", "/books")).await,
            (StatusCode::CREATED, "book added".to_string())
        );
        assert_eq!(
            send(get("/books/7")).await,
            (StatusCode::OK, "one book".to_string())
        );
        assert_eq!(
            send(with("PUT", "/books/7")).await,
            (StatusCode::OK, "book updated".to_string())
        );
        assert_eq!(
            send(with("DELETE", "/books/7")).await,
            (StatusCode::NO_CONTENT, String::new())
        );
    }

    #[tokio::test]
    async fn wrong_method_is_405_and_unknown_path_hits_the_fallback() {
        assert_eq!(
            send(with("PATCH", "/books")).await.0,
            StatusCode::METHOD_NOT_ALLOWED
        );
        assert_eq!(
            send(get("/magazines")).await,
            (StatusCode::NOT_FOUND, "nothing lives here".to_string())
        );
    }
}
