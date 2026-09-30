//! Common mistakes, fixed: `Path` first, `Json` last, because `Json` reads the body.
//! Run: `cargo run -p json-and-serde --example json-path-then-body`, then PUT to /books/7.
// ANCHOR: imports
use axum::{Json, Router, extract::Path, routing::put};
use serde::{Deserialize, Serialize};
// ANCHOR_END: imports

#[derive(Serialize)]
struct Book {
    id: u32,
    title: String,
    in_stock: bool,
}

#[derive(Deserialize)]
struct NewBook {
    title: String,
    #[serde(default)]
    in_stock: bool,
}

// ANCHOR: update
async fn update(Path(id): Path<u32>, Json(input): Json<NewBook>) -> Json<Book> {
    Json(Book {
        id,
        title: input.title,
        in_stock: input.in_stock,
    })
}
// ANCHOR_END: update

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/books/{id}", put(update))
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

    #[tokio::test]
    async fn the_id_comes_from_the_path_and_the_rest_from_the_body() {
        let request = Request::put("/books/7")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"title":"Emma","in_stock":true}"#))
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(bytes, r#"{"id":7,"title":"Emma","in_stock":true}"#);
    }
}
