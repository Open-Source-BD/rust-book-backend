//! Common mistakes (the fix): two branches, two types, one `Response`.
//! Run: `cargo run -p handlers-and-into-response --example handlers-if-else`, then
//! `curl -i -X POST http://127.0.0.1:3000/books`
// ANCHOR: imports
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
// ANCHOR_END: imports

// ANCHOR: add_book
fn shelf_is_full() -> bool {
    false
}

async fn add_book() -> Response {
    if shelf_is_full() {
        StatusCode::CONFLICT.into_response()
    } else {
        (StatusCode::CREATED, "book added").into_response()
    }
}
// ANCHOR_END: add_book

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/books", post(add_book))
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
    async fn room_on_the_shelf_means_201() {
        let request = Request::post("/books").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"book added");
    }

    #[test]
    fn both_branches_are_the_same_type() {
        let full: Response = StatusCode::CONFLICT.into_response();
        assert_eq!(full.status(), StatusCode::CONFLICT);
    }
}
