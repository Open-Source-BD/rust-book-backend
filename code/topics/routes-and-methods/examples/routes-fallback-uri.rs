//! Your turn (Tweak): the fallback says which path was missing.
//! Run: `cargo run -p routes-and-methods --example routes-fallback-uri`, then
//! `curl -i http://127.0.0.1:3000/magazines`
// ANCHOR: imports
use axum::{
    Router,
    http::{StatusCode, Uri},
    routing::get,
};
// ANCHOR_END: imports

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

// ANCHOR: not_found
async fn not_found(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("nothing lives at {uri}"))
}
// ANCHOR_END: not_found

fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
        .route(
            "/books/{id}",
            get(show_book).put(update_book).delete(delete_book),
        )
        .fallback(not_found)
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
    async fn fallback_names_the_missing_path() {
        assert_eq!(
            send("GET", "/magazines").await,
            (
                StatusCode::NOT_FOUND,
                "nothing lives at /magazines".to_string()
            )
        );
        assert_eq!(
            send("GET", "/books").await,
            (StatusCode::OK, "all books".to_string())
        );
    }
}
