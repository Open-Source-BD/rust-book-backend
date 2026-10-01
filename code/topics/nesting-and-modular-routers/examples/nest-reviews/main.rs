//! Your turn (Guided): a reviews route, added inside books.rs only.
//! Run: `cargo run -p nesting-and-modular-routers --example nest-reviews`.
mod books;
mod health;

use axum::Router;

fn app() -> Router {
    Router::new()
        .merge(health::router())
        .nest("/api/books", books::router())
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
    async fn reviews_live_under_each_book() {
        assert_eq!(
            send(get("/api/books/7/reviews")).await,
            (StatusCode::OK, "reviews of book 7".to_string())
        );
        assert_eq!(
            send(get("/api/books/7")).await,
            (StatusCode::OK, "book 7".to_string())
        );
    }
}
