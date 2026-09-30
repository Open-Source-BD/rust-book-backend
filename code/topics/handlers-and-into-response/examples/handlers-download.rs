//! Your turn (From scratch): `/download` sends text as a file to save.
//! Run: `cargo run -p handlers-and-into-response --example handlers-download`, then
//! `curl -i http://127.0.0.1:3000/download`
// ANCHOR: imports
use axum::{Router, http::header, response::IntoResponse, routing::get};
// ANCHOR_END: imports

// ANCHOR: download
async fn download() -> impl IntoResponse {
    (
        [(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"notes.txt\"",
        )],
        "Remember: status first, then headers, then the body.",
    )
}
// ANCHOR_END: download

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/download", get(download))
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

    #[tokio::test]
    async fn download_says_save_as_notes_txt() {
        let request = Request::get("/download").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(
            response.headers()["content-disposition"],
            "attachment; filename=\"notes.txt\""
        );
        assert_eq!(
            response.headers()["content-type"],
            "text/plain; charset=utf-8"
        );
    }
}
