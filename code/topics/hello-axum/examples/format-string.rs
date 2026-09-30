//! More examples: text built with `format!`, returned as a `String`.
//! Run: `cargo run -p hello-axum --example format-string`, then `curl -i http://127.0.0.1:3000/about`
use axum::{Router, response::Html, routing::get};

async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}

// ANCHOR: about
async fn about() -> String {
    let language = "Rust";
    let pages = 2;
    format!("This server is written in {language} and has {pages} pages.")
}
// ANCHOR_END: about

fn app() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn about_is_built_with_format() {
        let request = Request::get("/about").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["content-type"],
            "text/plain; charset=utf-8"
        );
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &bytes[..],
            b"This server is written in Rust and has 2 pages."
        );
    }
}
