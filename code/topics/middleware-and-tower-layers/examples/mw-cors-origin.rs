//! More examples: CORS for one known website only, instead of `CorsLayer::permissive()`.
//! Run: `cargo run -p middleware-and-tower-layers --example mw-cors-origin`, then GET / with an
//! `origin` header.
use axum::{Router, http::HeaderValue, routing::get};
use tower_http::cors::CorsLayer;

async fn hello() -> &'static str {
    "hello"
}

// ANCHOR: app
fn app() -> Router {
    let cors = CorsLayer::new().allow_origin("https://myapp.com".parse::<HeaderValue>().unwrap());
    Router::new().route("/", get(hello)).layer(cors)
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
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    fn from_origin(origin: &str) -> Request<Body> {
        Request::get("/")
            .header("origin", origin)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn the_answer_always_names_the_one_allowed_website() {
        for origin in ["https://myapp.com", "https://example.com"] {
            let response = app().oneshot(from_origin(origin)).await.unwrap();
            assert_eq!(
                response.headers()["access-control-allow-origin"],
                "https://myapp.com"
            );
        }
    }
}
