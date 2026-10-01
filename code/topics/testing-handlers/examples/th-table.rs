//! More examples: one table-driven test that checks several URLs in a loop.
//! Run: `cargo test -p testing-handlers --example th-table`
fn main() {
    println!("This example is a test: run `cargo test -p testing-handlers --example th-table`");
}

#[cfg(test)]
mod tests {
    // ANCHOR: test
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use testing_handlers::app;
    use tower::ServiceExt;

    #[tokio::test]
    async fn every_url_gets_the_right_status() {
        let cases = [
            ("/health", StatusCode::OK),
            ("/notes", StatusCode::OK),
            ("/note", StatusCode::NOT_FOUND),
            ("/health/", StatusCode::NOT_FOUND),
        ];
        for (uri, expected) in cases {
            let request = Request::get(uri).body(Body::empty()).unwrap();
            let response = app().oneshot(request).await.unwrap();
            assert_eq!(response.status(), expected, "GET {uri}");
        }
    }
    // ANCHOR_END: test
}
