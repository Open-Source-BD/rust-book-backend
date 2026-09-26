//! Your turn, From scratch: `GET /time` returns whole seconds since the Unix epoch.
//! Run: `cargo run -p tour-of-the-stack --example time`, then `curl -i http://127.0.0.1:3000/time`
use axum::{Router, routing::get};
// ANCHOR: use_time
use std::time::{SystemTime, UNIX_EPOCH};
// ANCHOR_END: use_time

async fn hello() -> &'static str {
    "Hello from Axum!"
}

// ANCHOR: time
async fn time() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the computer's clock is set before 1970")
        .as_secs();
    seconds.to_string()
}
// ANCHOR_END: time

fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/time", get(time))
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
