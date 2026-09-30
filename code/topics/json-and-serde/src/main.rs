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
}

#[derive(Deserialize)]
struct NewBook {
    title: String,
    #[serde(default)]
    in_stock: bool,
}
// ANCHOR_END: types

// ANCHOR: handlers
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
// ANCHOR_END: handlers

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/books/sample", get(sample))
        .route("/books", post(create))
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

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn struct_becomes_json() {
        assert_eq!(
            send(get("/books/sample")).await.1,
            r#"{"id":1,"title":"Dune","in_stock":true}"#
        );
    }

    #[tokio::test]
    async fn json_becomes_struct_with_default() {
        assert_eq!(
            send(post_json("/books", r#"{"title":"Emma"}"#)).await.1,
            r#"{"id":2,"title":"Emma","in_stock":false}"#
        );
    }

    #[tokio::test]
    async fn bad_json_is_rejected() {
        assert_eq!(
            send(post_json("/books", r#"{"name":"Emma"}"#)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            send(post_json("/books", "{oops")).await.0,
            StatusCode::BAD_REQUEST
        );
        let no_type = Request::post("/books").body(Body::from("{}")).unwrap();
        assert_eq!(send(no_type).await.0, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}
