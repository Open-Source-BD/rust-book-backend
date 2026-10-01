//! Common mistakes: nesting at "/api/books/", with a trailing slash.
//! Run: `cargo run -p nesting-and-modular-routers --example nest-slash-mistake`.
use axum::{Router, extract::Path, routing::get};

async fn list_books() -> &'static str {
    "all books"
}

async fn show_book(Path(id): Path<u32>) -> String {
    format!("book {id}")
}

fn books_router() -> Router {
    Router::new()
        .route("/", get(list_books))
        .route("/{id}", get(show_book))
}

// ANCHOR: app
fn app() -> Router {
    Router::new().nest("/api/books/", books_router())
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
    async fn the_list_moves_to_the_slash_path() {
        assert_eq!(send(get("/api/books")).await.0, StatusCode::NOT_FOUND);
        assert_eq!(
            send(get("/api/books/")).await,
            (StatusCode::OK, "all books".to_string())
        );
        assert_eq!(
            send(get("/api/books/7")).await,
            (StatusCode::OK, "book 7".to_string())
        );
    }
}
