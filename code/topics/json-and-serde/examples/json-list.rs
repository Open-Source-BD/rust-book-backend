//! More examples: a `Vec<Book>` becomes a JSON array.
//! Run: `cargo run -p json-and-serde --example json-list`, then
//! `curl -i http://127.0.0.1:3000/books`
// ANCHOR: imports
use axum::{Json, Router, routing::get};
use serde::Serialize;
// ANCHOR_END: imports

#[derive(Serialize)]
struct Book {
    id: u32,
    title: String,
    in_stock: bool,
}

// ANCHOR: list
async fn list() -> Json<Vec<Book>> {
    Json(vec![
        Book {
            id: 1,
            title: "Dune".to_string(),
            in_stock: true,
        },
        Book {
            id: 2,
            title: "Emma".to_string(),
            in_stock: false,
        },
    ])
}
// ANCHOR_END: list

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/books", get(list))
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
    async fn a_vec_becomes_an_array() {
        assert_eq!(
            send(get("/books")).await.1,
            r#"[{"id":1,"title":"Dune","in_stock":true},{"id":2,"title":"Emma","in_stock":false}]"#
        );
    }
}
