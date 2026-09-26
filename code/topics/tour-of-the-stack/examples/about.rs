//! More examples: a second page, `GET /about`.
//! Run: `cargo run -p tour-of-the-stack --example about`, then `curl -i http://127.0.0.1:3000/about`
use axum::{Router, routing::get};

async fn hello() -> &'static str {
    "Hello from Axum!"
}

// ANCHOR: about
async fn about() -> &'static str {
    "ShopRS: a small online shop, built while learning Rust."
}

fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/about", get(about))
}
// ANCHOR_END: about

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
