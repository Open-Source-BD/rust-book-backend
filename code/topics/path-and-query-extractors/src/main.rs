// ANCHOR: imports
use axum::{
    Router,
    extract::{Path, Query},
    routing::get,
};
use serde::Deserialize;
// ANCHOR_END: imports

// ANCHOR: path
async fn show_book(Path(id): Path<u32>) -> String {
    format!("book number {id}")
}

async fn show_chapter(Path((book, chapter)): Path<(u32, u32)>) -> String {
    format!("book {book}, chapter {chapter}")
}
// ANCHOR_END: path

// ANCHOR: query
#[derive(Deserialize)]
struct Search {
    q: String,
    page: Option<u32>,
}

async fn search(Query(params): Query<Search>) -> String {
    let page = params.page.unwrap_or(1);
    format!("searching for {:?}, page {page}", params.q)
}
// ANCHOR_END: query

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/books/{id}", get(show_book))
        .route("/books/{book}/chapters/{chapter}", get(show_chapter))
        .route("/search", get(search))
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
    async fn path_values_are_parsed() {
        assert_eq!(
            send(get("/books/42")).await,
            (StatusCode::OK, "book number 42".to_string())
        );
        assert_eq!(
            send(get("/books/3/chapters/7")).await,
            (StatusCode::OK, "book 3, chapter 7".to_string())
        );
        assert_eq!(send(get("/books/dune")).await.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn query_string_is_parsed_with_an_optional_field() {
        assert_eq!(
            send(get("/search?q=rust")).await.1,
            "searching for \"rust\", page 1"
        );
        assert_eq!(
            send(get("/search?q=rust&page=2")).await.1,
            "searching for \"rust\", page 2"
        );
        assert_eq!(send(get("/search")).await.0, StatusCode::BAD_REQUEST);
    }
}
