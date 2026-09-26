//! Roughly what `#[tokio::main]` turns `async fn main` into: an ordinary `main` that builds a
//! Tokio runtime and runs the async body on it. Run: `cargo run -p tour-of-the-stack --example tokio-main-expanded`

// ANCHOR: expanded
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async {
            // the body of your async main goes here
        })
}
// ANCHOR_END: expanded
