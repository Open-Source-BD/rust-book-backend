//! More examples: a `Vec<u8>` body, raw bytes.
//! Run: `cargo run -p handlers-and-into-response --example handlers-bytes`, then
//! `curl -i http://127.0.0.1:3000/bytes`
// ANCHOR: imports
use axum::{Router, routing::get};
// ANCHOR_END: imports

// ANCHOR: bytes
async fn bytes() -> Vec<u8> {
    vec![104, 105, 33]
}
// ANCHOR_END: bytes

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/bytes", get(bytes))
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
    async fn bytes_go_out_as_octet_stream() {
        let request = Request::get("/bytes").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(
            response.headers()["content-type"],
            "application/octet-stream"
        );
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"hi!");
    }
}
