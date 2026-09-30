// ANCHOR: modules
mod books;
mod health;
// ANCHOR_END: modules

// ANCHOR: imports
use axum::Router;
// ANCHOR_END: imports

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .merge(health::router())
        .nest("/api/books", books::router())
}
// ANCHOR_END: app

// ANCHOR: main
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
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        send_to(app(), request).await
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[allow(dead_code)]
    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    use axum::http::StatusCode;

    #[tokio::test]
    async fn merged_and_nested_routes() {
        assert_eq!(
            send(get("/health")).await,
            (StatusCode::OK, "ok".to_string())
        );
        assert_eq!(
            send(get("/api/books")).await,
            (StatusCode::OK, "all books".to_string())
        );
        assert_eq!(
            send(get("/api/books/7")).await,
            (StatusCode::OK, "book 7".to_string())
        );
        assert_eq!(send(get("/books")).await.0, StatusCode::NOT_FOUND);
        println!("TRAILING {:?}", send(get("/api/books/")).await);
    }
}
