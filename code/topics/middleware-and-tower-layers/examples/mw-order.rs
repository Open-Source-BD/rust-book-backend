//! The idea, slowly: see the onion order for real. Two middlewares each write their name on the
//! way in (into the request) and on the way out (into the response).
//! Run: `cargo run -p middleware-and-tower-layers --example mw-order`, then GET /.
// ANCHOR: imports
use axum::{
    Router,
    extract::Request,
    http::HeaderValue,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
// ANCHOR_END: imports

// ANCHOR: middleware
async fn one(mut request: Request, next: Next) -> Response {
    request
        .headers_mut()
        .append("x-trail", HeaderValue::from_static("one"));
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .append("x-trail", HeaderValue::from_static("one"));
    response
}

async fn two(mut request: Request, next: Next) -> Response {
    request
        .headers_mut()
        .append("x-trail", HeaderValue::from_static("two"));
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .append("x-trail", HeaderValue::from_static("two"));
    response
}
// ANCHOR_END: middleware

// ANCHOR: handler
async fn show_trail(request: Request) -> String {
    let names: Vec<&str> = request
        .headers()
        .get_all("x-trail")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect();
    format!("on the way in: {}\n", names.join(" → "))
}
// ANCHOR_END: handler

// ANCHOR: app
fn app() -> Router {
    Router::new()
        .route("/", get(show_trail))
        .layer(middleware::from_fn(one))
        .layer(middleware::from_fn(two))
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
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn the_last_layer_is_the_outermost() {
        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let way_out: Vec<&str> = response
            .headers()
            .get_all("x-trail")
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect();
        assert_eq!(way_out, ["one", "two"]);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(bytes, "on the way in: two → one\n");
    }
}
