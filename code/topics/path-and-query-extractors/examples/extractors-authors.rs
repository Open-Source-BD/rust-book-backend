//! Your turn, Guided: `/authors/{name}` answers with a greeting.
//! Run: `cargo run -p path-and-query-extractors --example extractors-authors`, then
//! `curl -i http://127.0.0.1:3000/authors/ada`
// ANCHOR: imports
use axum::{Router, extract::Path, routing::get};
// ANCHOR_END: imports

// ANCHOR: greet_author
async fn greet_author(Path(name): Path<String>) -> String {
    format!("hello, {name}!")
}
// ANCHOR_END: greet_author

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/authors/{name}", get(greet_author))
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

    #[tokio::test]
    async fn greets_the_author_by_name() {
        let request = Request::get("/authors/ada").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"hello, ada!");
    }
}
