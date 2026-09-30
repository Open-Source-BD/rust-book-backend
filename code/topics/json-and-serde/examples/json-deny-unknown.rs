//! More examples: `#[serde(deny_unknown_fields)]` turns an extra JSON field into a 422.
//! Run: `cargo run -p json-and-serde --example json-deny-unknown`, then POST to /books.
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewBook {
    title: String,
    #[serde(default)]
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
    async fn known_fields_still_work() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma"}"#)).await,
            (
                StatusCode::OK,
                r#"{"id":2,"title":"Emma","in_stock":false}"#.to_string()
            )
        );
    }

    #[tokio::test]
    async fn an_unknown_field_is_422() {
        let (status, body) = send(post_json(
            "/books",
            r#"{"title":"Emma","author":"Jane Austen"}"#,
        ))
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("unknown field `author`"));
    }
}
