//! More examples: `#[serde(rename_all = "camelCase")]` writes and reads `inStock`, not `in_stock`.
//! Run: `cargo run -p json-and-serde --example json-camel-case`, then
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
#[serde(rename_all = "camelCase")]
struct Book {
    id: u32,
    title: String,
    in_stock: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewBook {
    title: String,
    #[serde(default)]
    in_stock: bool,
}
// ANCHOR_END: types

async fn sample() -> Json<Book> {
    Json(Book {
        id: 1,
        title: "Dune".to_string(),
        in_stock: true,
    })
}

async fn create(Json(input): Json<NewBook>) -> Json<Book> {
    Json(Book {
        id: 2,
        title: input.title,
        in_stock: input.in_stock,
    })
}

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
    async fn fields_are_camel_case_both_ways() {
        assert_eq!(
            send(get("/books/sample")).await.1,
            r#"{"id":1,"title":"Dune","inStock":true}"#
        );
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma","inStock":true}"#))
                .await
                .1,
            r#"{"id":2,"title":"Emma","inStock":true}"#
        );
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma","in_stock":true}"#))
                .await
                .1,
            r#"{"id":2,"title":"Emma","inStock":false}"#
        );
    }
}
