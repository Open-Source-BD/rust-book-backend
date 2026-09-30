//! More examples: a struct inside a struct becomes a JSON object inside a JSON object.
//! Run: `cargo run -p json-and-serde --example json-nested`, then
//! `curl -i http://127.0.0.1:3000/books/sample`
// ANCHOR: imports
use axum::{Json, Router, routing::get};
use serde::Serialize;
// ANCHOR_END: imports

// ANCHOR: types
#[derive(Serialize)]
struct Author {
    name: String,
    born: u32,
}

#[derive(Serialize)]
struct Book {
    id: u32,
    title: String,
    author: Author,
}
// ANCHOR_END: types

// ANCHOR: sample
async fn sample() -> Json<Book> {
    Json(Book {
        id: 1,
        title: "Dune".to_string(),
        author: Author {
            name: "Frank Herbert".to_string(),
            born: 1920,
        },
    })
}
// ANCHOR_END: sample

fn app() -> Router {
    Router::new().route("/books/sample", get(sample))
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
    async fn the_author_is_an_object_inside_the_book() {
        assert_eq!(
            send(get("/books/sample")).await.1,
            r#"{"id":1,"title":"Dune","author":{"name":"Frank Herbert","born":1920}}"#
        );
    }
}
