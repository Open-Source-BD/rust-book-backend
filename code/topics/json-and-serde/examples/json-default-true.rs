//! Your turn, Tweak: `in_stock` defaults to `true`, using `#[serde(default = "yes")]`.
//! Run: `cargo run -p json-and-serde --example json-default-true`, then POST to /books.
// ANCHOR: imports
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
// ANCHOR_END: imports

#[derive(Serialize)]
struct Book {
    id: u32,
    title: String,
    in_stock: bool,
}

// ANCHOR: new_book
fn yes() -> bool {
    true
}

#[derive(Deserialize)]
struct NewBook {
    title: String,
    #[serde(default = "yes")]
    in_stock: bool,
}
// ANCHOR_END: new_book

async fn create(Json(input): Json<NewBook>) -> Json<Book> {
    Json(Book {
        id: 2,
        title: input.title,
        in_stock: input.in_stock,
    })
}

fn app() -> Router {
    Router::new().route("/books", post(create))
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

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn a_missing_in_stock_is_true() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma"}"#)).await.1,
            r#"{"id":2,"title":"Emma","in_stock":true}"#
        );
    }

    #[tokio::test]
    async fn an_explicit_false_is_kept() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma","in_stock":false}"#))
                .await
                .1,
            r#"{"id":2,"title":"Emma","in_stock":false}"#
        );
    }
}
