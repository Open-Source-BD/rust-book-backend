//! More examples: a nest inside a nest, for versioned APIs (/api/v1, /api/v2).
//! Run: `cargo run -p nesting-and-modular-routers --example nest-two-levels`.
// ANCHOR: imports
use axum::{Router, extract::Path, routing::get};
// ANCHOR_END: imports

// ANCHOR: handlers
async fn show_book_v1(Path(id): Path<u32>) -> String {
    format!("book {id}")
}

async fn show_book_v2(Path(id): Path<u32>) -> String {
    format!("book number {id}, from version 2")
}
// ANCHOR_END: handlers

// ANCHOR: app
fn v1_router() -> Router {
    Router::new().route("/books/{id}", get(show_book_v1))
}

fn v2_router() -> Router {
    Router::new().route("/books/{id}", get(show_book_v2))
}

fn api_router() -> Router {
    Router::new()
        .nest("/v1", v1_router())
        .nest("/v2", v2_router())
}

fn app() -> Router {
    Router::new().nest("/api", api_router())
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
    async fn both_versions_answer_under_api() {
        assert_eq!(
            send(get("/api/v1/books/7")).await,
            (StatusCode::OK, "book 7".to_string())
        );
        assert_eq!(
            send(get("/api/v2/books/7")).await,
            (StatusCode::OK, "book number 7, from version 2".to_string())
        );
        assert_eq!(send(get("/v1/books/7")).await.0, StatusCode::NOT_FOUND);
    }
}
