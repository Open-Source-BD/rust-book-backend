//! More examples: choose the status code by returning `(StatusCode, body)`.
//! Run: `cargo run -p tour-of-the-stack --example status-code`, then
//! `curl -i -X POST http://127.0.0.1:3000/things`
// ANCHOR: imports
use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};
// ANCHOR_END: imports

async fn hello() -> &'static str {
    "Hello from Axum!"
}

// ANCHOR: handler
async fn make_thing() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "made")
}
// ANCHOR_END: handler

fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/things", post(make_thing))
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
