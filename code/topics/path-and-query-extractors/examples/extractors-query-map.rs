//! More examples: a `HashMap<String, String>` query, which accepts any names.
//! Run: `cargo run -p path-and-query-extractors --example extractors-query-map`, then
//! `curl -i "http://127.0.0.1:3000/filters?color=red&size=big"`
// ANCHOR: imports
use std::collections::HashMap;

use axum::{Router, extract::Query, routing::get};
// ANCHOR_END: imports

// ANCHOR: filters
async fn filters(Query(params): Query<HashMap<String, String>>) -> String {
    let color = params.get("color");
    format!("{} option(s); color is {color:?}", params.len())
}
// ANCHOR_END: filters

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/filters", get(filters))
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

    async fn send(uri: &str) -> (StatusCode, String) {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn any_names_are_accepted() {
        assert_eq!(
            send("/filters?color=red&size=big").await,
            (
                StatusCode::OK,
                "2 option(s); color is Some(\"red\")".to_string()
            )
        );
        assert_eq!(
            send("/filters").await,
            (StatusCode::OK, "0 option(s); color is None".to_string())
        );
    }
}
