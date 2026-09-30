//! More examples: `Path` and `Query` in the same handler.
//! Run: `cargo run -p path-and-query-extractors --example extractors-both`, then
//! `curl -i "http://127.0.0.1:3000/books/7/reviews?page=2"`
// ANCHOR: imports
use axum::{
    Router,
    extract::{Path, Query},
    routing::get,
};
use serde::Deserialize;
// ANCHOR_END: imports

// ANCHOR: reviews
#[derive(Deserialize)]
struct ReviewOptions {
    page: Option<u32>,
}

async fn reviews(Path(id): Path<u32>, Query(options): Query<ReviewOptions>) -> String {
    let page = options.page.unwrap_or(1);
    format!("reviews of book {id}, page {page}")
}
// ANCHOR_END: reviews

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/books/{id}/reviews", get(reviews))
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

    async fn send(uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn path_and_query_are_both_read() {
        assert_eq!(
            send("/books/7/reviews?page=2").await,
            (StatusCode::OK, "reviews of book 7, page 2".to_string())
        );
        assert_eq!(
            send("/books/7/reviews").await,
            (StatusCode::OK, "reviews of book 7, page 1".to_string())
        );
    }
}
