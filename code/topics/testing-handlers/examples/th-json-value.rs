//! More examples: a helper that gives back the body as parsed JSON (`serde_json::Value`).
//! Run: `cargo test -p testing-handlers --example th-json-value`
fn main() {
    println!(
        "This example is a test: run `cargo test -p testing-handlers --example th-json-value`"
    );
}

#[cfg(test)]
mod tests {
    // ANCHOR: helper
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use testing_handlers::app;
    use tower::ServiceExt;

    async fn send_json(app: Router, request: Request<Body>) -> (StatusCode, Value) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }
    // ANCHOR_END: helper

    // ANCHOR: test
    #[tokio::test]
    async fn a_new_note_as_a_json_value() {
        let request = Request::post("/notes")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"text":"buy milk"}"#))
            .unwrap();
        let (status, body) = send_json(app(), request).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["text"], "buy milk");
        assert_eq!(body, json!({ "id": 1, "text": "buy milk" }));
    }
    // ANCHOR_END: test
}
