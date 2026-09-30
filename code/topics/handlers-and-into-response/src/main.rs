// ANCHOR: imports
use axum::{
    Router,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
// ANCHOR_END: imports

// ANCHOR: simple
async fn text() -> &'static str {
    "plain text"
}

async fn sum() -> String {
    format!("{} + {} = {}", 2, 3, 2 + 3)
}

async fn page() -> Html<&'static str> {
    Html("<p>an HTML page</p>")
}
// ANCHOR_END: simple

// ANCHOR: status
async fn created() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "made it")
}
// ANCHOR_END: status

// ANCHOR: headers
async fn no_cache() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "no-store")],
        "fresh every time",
    )
}
// ANCHOR_END: headers

// ANCHOR: by_hand
async fn by_hand() -> Response {
    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header("x-note", "built by hand")
        .body("queued".into())
        .unwrap()
}
// ANCHOR_END: by_hand

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/text", get(text))
        .route("/sum", get(sum))
        .route("/page", get(page))
        .route("/created", get(created))
        .route("/no-cache", get(no_cache))
        .route("/by-hand", get(by_hand))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
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
    async fn every_return_type_becomes_a_response() {
        assert_eq!(
            send(get("/text")).await,
            (StatusCode::OK, "plain text".to_string())
        );
        assert_eq!(
            send(get("/sum")).await,
            (StatusCode::OK, "2 + 3 = 5".to_string())
        );
        assert_eq!(
            send(get("/page")).await,
            (StatusCode::OK, "<p>an HTML page</p>".to_string())
        );
        assert_eq!(
            send(get("/created")).await,
            (StatusCode::CREATED, "made it".to_string())
        );
    }

    #[tokio::test]
    async fn headers_are_set() {
        let response = app().oneshot(get("/no-cache")).await.unwrap();
        assert_eq!(response.headers()["cache-control"], "no-store");
        let response = app().oneshot(get("/by-hand")).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(response.headers()["x-note"], "built by hand");
    }
}
