//! More examples: state that holds read-only settings, such as the app's name.
//! Run: `cargo run -p shared-state --example state-config`, then `curl -i http://127.0.0.1:3000/`
// ANCHOR: imports
use axum::{Router, extract::State, routing::get};
// ANCHOR_END: imports

// ANCHOR: state
#[derive(Clone)]
struct AppState {
    app_name: &'static str,
}
// ANCHOR_END: state

// ANCHOR: handler
async fn welcome(State(state): State<AppState>) -> String {
    format!("Welcome to {}!", state.app_name)
}
// ANCHOR_END: handler

// ANCHOR: app
fn app() -> Router {
    let state = AppState {
        app_name: "The Book Nook",
    };
    Router::new().route("/", get(welcome)).with_state(state)
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
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn the_name_comes_from_state() {
        assert_eq!(
            send_to(app(), get("/")).await,
            (StatusCode::OK, "Welcome to The Book Nook!".to_string())
        );
    }
}
