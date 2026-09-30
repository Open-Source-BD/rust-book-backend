//! Your turn, From scratch: POST /orders adds up the quantities of every item.
//! Run: `cargo run -p json-and-serde --example json-orders`, then POST to /orders.
// ANCHOR: imports
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
// ANCHOR_END: imports

// ANCHOR: types
#[derive(Deserialize)]
struct OrderItem {
    #[allow(dead_code)]
    book_id: u32,
    qty: u32,
}

#[derive(Deserialize)]
struct NewOrder {
    items: Vec<OrderItem>,
}

#[derive(Serialize)]
struct OrderSummary {
    total_items: u32,
}
// ANCHOR_END: types

// ANCHOR: handler
async fn create_order(Json(order): Json<NewOrder>) -> Json<OrderSummary> {
    let mut total_items = 0;
    for item in &order.items {
        total_items += item.qty;
    }
    Json(OrderSummary { total_items })
}
// ANCHOR_END: handler

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/orders", post(create_order))
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn quantities_are_added_up() {
        assert_eq!(
            send(post_json("/orders", r#"{"items":[{"book_id":1,"qty":2}]}"#))
                .await
                .1,
            r#"{"total_items":2}"#
        );
        assert_eq!(
            send(post_json(
                "/orders",
                r#"{"items":[{"book_id":1,"qty":2},{"book_id":7,"qty":3}]}"#
            ))
            .await
            .1,
            r#"{"total_items":5}"#
        );
    }

    #[tokio::test]
    async fn an_empty_order_is_zero() {
        assert_eq!(
            send(post_json("/orders", r#"{"items":[]}"#)).await.1,
            r#"{"total_items":0}"#
        );
    }
}
