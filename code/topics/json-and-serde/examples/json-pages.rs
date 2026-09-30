//! Your turn, Guided: add `pages: u32` to `Book` and `NewBook`.
//! Run: `cargo run -p json-and-serde --example json-pages`, then
//! `curl -i http://127.0.0.1:3000/books/sample`
// ANCHOR: imports
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
// ANCHOR_END: imports

// ANCHOR: types
#[derive(Serialize)]
struct Book {
    id: u32,
    title: String,
    in_stock: bool,
    pages: u32,
}

#[derive(Deserialize)]
struct NewBook {
    title: String,
    #[serde(default)]
    in_stock: bool,
    pages: u32,
}
// ANCHOR_END: types

// ANCHOR: handlers
async fn sample() -> Json<Book> {
    Json(Book {
        id: 1,
        title: "Dune".to_string(),
        in_stock: true,
        pages: 412,
    })
}

async fn create(Json(input): Json<NewBook>) -> Json<Book> {
    Json(Book {
        id: 2,
        title: input.title,
        in_stock: input.in_stock,
        pages: input.pages,
    })
}
// ANCHOR_END: handlers

fn app() -> Router {
    Router::new()
        .route("/books/sample", get(sample))
        .route("/books", post(create))
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

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn pages_go_out_and_come_in() {
        assert_eq!(
            send(get("/books/sample")).await.1,
            r#"{"id":1,"title":"Dune","in_stock":true,"pages":412}"#
        );
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma","pages":474}"#))
                .await
                .1,
            r#"{"id":2,"title":"Emma","in_stock":false,"pages":474}"#
        );
    }

    #[tokio::test]
    async fn pages_are_required() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma"}"#)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
