//! Your turn, Guided: the server greets you by name (here, Ada).
//! Run: `cargo run -p tour-of-the-stack --example hello-name`, then `curl -i http://127.0.0.1:3000/`
use axum::{Router, routing::get};

// ANCHOR: hello
async fn hello() -> &'static str {
    "Hello from Ada!"
}
// ANCHOR_END: hello

fn app() -> Router {
    Router::new().route("/", get(hello))
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
