//! Your turn, From scratch: `/range?from=1&to=5` answers `1,2,3,4,5`, and 400 if from > to.
//! Run: `cargo run -p path-and-query-extractors --example extractors-range`, then
//! `curl -i "http://127.0.0.1:3000/range?from=1&to=5"`
// ANCHOR: imports
use axum::{Router, extract::Query, http::StatusCode, routing::get};
use serde::Deserialize;
// ANCHOR_END: imports

// ANCHOR: range
#[derive(Deserialize)]
struct Range {
    from: u32,
    to: u32,
}

async fn range(Query(range): Query<Range>) -> (StatusCode, String) {
    if range.from > range.to {
        return (
            StatusCode::BAD_REQUEST,
            format!("from ({}) is bigger than to ({})", range.from, range.to),
        );
    }
    let mut numbers = Vec::new();
    for n in range.from..=range.to {
        numbers.push(n.to_string());
    }
    (StatusCode::OK, numbers.join(","))
}
// ANCHOR_END: range

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/range", get(range))
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

    async fn send(uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn numbers_are_joined_by_commas() {
        assert_eq!(
            send("/range?from=1&to=5").await,
            (StatusCode::OK, "1,2,3,4,5".to_string())
        );
        assert_eq!(
            send("/range?from=4&to=4").await,
            (StatusCode::OK, "4".to_string())
        );
    }

    #[tokio::test]
    async fn from_bigger_than_to_is_400() {
        assert_eq!(
            send("/range?from=5&to=1").await,
            (
                StatusCode::BAD_REQUEST,
                "from (5) is bigger than to (1)".to_string()
            )
        );
    }
}
