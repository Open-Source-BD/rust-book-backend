//! Your turn (Guided): `created` answers 202 Accepted instead of 201 Created.
//! Run: `cargo run -p handlers-and-into-response --example handlers-accepted`, then
//! `curl -i http://127.0.0.1:3000/created`
use axum::{Router, http::StatusCode, routing::get};

// ANCHOR: created
async fn created() -> (StatusCode, &'static str) {
    (StatusCode::ACCEPTED, "made it")
}
// ANCHOR_END: created

fn app() -> Router {
    Router::new().route("/created", get(created))
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
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn created_now_answers_202() {
        assert_eq!(
            send("/created").await,
            (StatusCode::ACCEPTED, "made it".to_string())
        );
    }
}
