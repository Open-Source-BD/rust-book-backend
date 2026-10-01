//! More examples: send `/api/books/` (with a trailing slash) on to `/api/books`.
//! Run: `cargo run -p nesting-and-modular-routers --example nest-trailing-redirect`.
// ANCHOR: imports
use axum::{Router, response::Redirect, routing::get};
// ANCHOR_END: imports

async fn list_books() -> &'static str {
    "all books"
}

fn books_router() -> Router {
    Router::new().route("/", get(list_books))
}

// ANCHOR: redirect
async fn drop_trailing_slash() -> Redirect {
    Redirect::permanent("/api/books")
}
// ANCHOR_END: redirect

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .nest("/api/books", books_router())
        .route("/api/books/", get(drop_trailing_slash))
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
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn trailing_slash_redirects_to_the_list() {
        let response = app().oneshot(get("/api/books/")).await.unwrap();
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(response.headers()["location"], "/api/books");
        assert_eq!(
            send(get("/api/books")).await,
            (StatusCode::OK, "all books".to_string())
        );
    }
}
