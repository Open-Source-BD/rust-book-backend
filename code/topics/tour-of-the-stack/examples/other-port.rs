//! More examples: serve on port 8080 instead of 3000.
//! Run: `cargo run -p tour-of-the-stack --example other-port`, then `curl -i http://127.0.0.1:8080/`
use axum::{Router, routing::get};

async fn hello() -> &'static str {
    "Hello from Axum!"
}

fn app() -> Router {
    Router::new().route("/", get(hello))
}

#[tokio::main]
async fn main() {
    // ANCHOR: bind
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("port 8080 is busy: stop the other program using it, or change the port");
    println!("Listening on http://127.0.0.1:8080");
    // ANCHOR_END: bind
    axum::serve(listener, app())
        .await
        .expect("the server stopped because of an error");
}
