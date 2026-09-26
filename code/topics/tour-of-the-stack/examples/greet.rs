//! More examples: text built at run time, returned as a `String`.
//! Run: `cargo run -p tour-of-the-stack --example greet`, then `curl -i http://127.0.0.1:3000/greet`
use axum::{Router, routing::get};

async fn hello() -> &'static str {
    "Hello from Axum!"
}

// ANCHOR: greet
async fn greet() -> String {
    let name = "Ada";
    let items_in_cart = 3;
    format!("Hello, {name}! You have {items_in_cart} items in your cart.")
}
// ANCHOR_END: greet

fn app() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/greet", get(greet))
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
