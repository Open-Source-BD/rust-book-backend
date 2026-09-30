//! More examples: an HTML page built while the program runs, returned as `Html<String>`.
//! Run: `cargo run -p hello-axum --example html-string`, then `curl -i http://127.0.0.1:3000/menu`
use axum::{Router, response::Html, routing::get};

async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}

// ANCHOR: menu
async fn menu() -> Html<String> {
    let dishes = ["Soup", "Bread", "Tea"];
    let mut items = String::new();
    for dish in dishes {
        items.push_str(&format!("<li>{dish}</li>"));
    }
    Html(format!("<h1>Today's menu</h1><ul>{items}</ul>"))
}
// ANCHOR_END: menu

fn app() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/menu", get(menu))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn menu_is_an_html_list() {
        let request = Request::get("/menu").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["content-type"],
            "text/html; charset=utf-8"
        );
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &bytes[..],
            b"<h1>Today's menu</h1><ul><li>Soup</li><li>Bread</li><li>Tea</li></ul>"
        );
    }
}
