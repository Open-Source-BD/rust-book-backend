//! More examples: `Path<String>` for a slug, a word-like name in the address.
//! Run: `cargo run -p path-and-query-extractors --example extractors-slug`, then
//! `curl -i http://127.0.0.1:3000/articles/my-first-post`
// ANCHOR: imports
use axum::{Router, extract::Path, routing::get};
// ANCHOR_END: imports

// ANCHOR: article
async fn article(Path(slug): Path<String>) -> String {
    format!("the article called {slug:?}")
}
// ANCHOR_END: article

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/articles/{slug}", get(article))
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
    async fn any_segment_is_a_string() {
        assert_eq!(
            send("/articles/my-first-post").await,
            (
                StatusCode::OK,
                "the article called \"my-first-post\"".to_string()
            )
        );
        assert_eq!(
            send("/articles/42").await,
            (StatusCode::OK, "the article called \"42\"".to_string())
        );
    }
}
