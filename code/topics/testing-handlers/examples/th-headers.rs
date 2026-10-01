//! More examples: a test that checks a response header, not only the status and the body.
//! Run: `cargo test -p testing-handlers --example th-headers`
fn main() {
    println!("This example is a test: run `cargo test -p testing-handlers --example th-headers`");
}

#[cfg(test)]
mod tests {
    // ANCHOR: test
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use testing_handlers::app;
    use tower::ServiceExt;

    #[tokio::test]
    async fn notes_are_labelled_as_json() {
        let request = Request::get("/notes").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    }

    #[tokio::test]
    async fn health_is_plain_text() {
        let request = Request::get("/health").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/plain; charset=utf-8"
        );
    }
    // ANCHOR_END: test
}
