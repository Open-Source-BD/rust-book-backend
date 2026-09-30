//! Your turn, Tweak: an optional `sort` option that defaults to "title".
//! Run: `cargo run -p path-and-query-extractors --example extractors-sort`, then
//! `curl -i "http://127.0.0.1:3000/search?q=rust&sort=year"`
// ANCHOR: imports
use axum::{Router, extract::Query, routing::get};
use serde::Deserialize;
// ANCHOR_END: imports

// ANCHOR: search
#[derive(Deserialize)]
struct Search {
    q: String,
    page: Option<u32>,
    sort: Option<String>,
}

async fn search(Query(params): Query<Search>) -> String {
    let page = params.page.unwrap_or(1);
    let sort = params.sort.unwrap_or("title".to_string());
    format!(
        "searching for {:?}, page {page}, sorted by {sort}",
        params.q
    )
}
// ANCHOR_END: search

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/search", get(search))
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
    async fn sort_defaults_to_title() {
        assert_eq!(
            send("/search?q=rust").await.1,
            "searching for \"rust\", page 1, sorted by title"
        );
        assert_eq!(
            send("/search?q=rust&sort=year").await.1,
            "searching for \"rust\", page 1, sorted by year"
        );
    }
}
