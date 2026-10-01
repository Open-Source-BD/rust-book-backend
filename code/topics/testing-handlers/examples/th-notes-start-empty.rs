//! Your turn (Guided): a test that GET /notes starts with an empty list.
//! Run: `cargo test -p testing-handlers --example th-notes-start-empty`
fn main() {
    println!(
        "This example is a test: run `cargo test -p testing-handlers --example th-notes-start-empty`"
    );
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use testing_handlers::{Note, app};
    use tower::ServiceExt;

    async fn send(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    // ANCHOR: test
    #[tokio::test]
    async fn notes_start_empty() {
        let (status, body) = send(app(), get("/notes")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "[]");
        let notes: Vec<Note> = serde_json::from_str(&body).unwrap();
        assert!(notes.is_empty());
    }
    // ANCHOR_END: test
}
