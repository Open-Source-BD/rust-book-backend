//! Your turn (From scratch): `/authors` with GET and POST, `/authors/{id}` with GET and DELETE.
//! Run: `cargo run -p routes-and-methods --example routes-authors`, then
//! `curl -i http://127.0.0.1:3000/authors`
// ANCHOR: imports
use axum::{Router, http::StatusCode, routing::get};
// ANCHOR_END: imports

// ANCHOR: handlers
async fn list_authors() -> &'static str {
    "all authors"
}

async fn add_author() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "author added")
}

async fn show_author() -> &'static str {
    "one author"
}

async fn delete_author() -> StatusCode {
    StatusCode::NO_CONTENT
}
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/authors", get(list_authors).post(add_author))
        .route("/authors/{id}", get(show_author).delete(delete_author))
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
    async fn authors_routes() {
        assert_eq!(
            send("GET", "/authors").await,
            (StatusCode::OK, "all authors".to_string())
        );
        assert_eq!(
            send("POST", "/authors").await,
            (StatusCode::CREATED, "author added".to_string())
        );
        assert_eq!(
            send("GET", "/authors/3").await,
            (StatusCode::OK, "one author".to_string())
        );
        assert_eq!(
            send("DELETE", "/authors/3").await,
            (StatusCode::NO_CONTENT, String::new())
        );
        assert_eq!(
            send("PUT", "/authors/3").await.0,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
