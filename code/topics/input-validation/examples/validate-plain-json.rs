//! Common mistakes: the rules are declared, but the handler takes plain `Json`, so nobody checks them.
//! Run: `cargo run -p input-validation --example validate-plain-json`, then POST /sign-up.
use axum::{Json, Router, http::StatusCode, routing::post};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct SignUp {
    #[validate(length(min = 3, max = 20))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(range(min = 13))]
    age: u32,
}

// ANCHOR: handler
async fn sign_up(Json(input): Json<SignUp>) -> (StatusCode, String) {
    (
        StatusCode::CREATED,
        format!("welcome, {} <{}>", input.username, input.email),
    )
}
// ANCHOR_END: handler

fn app() -> Router {
    Router::new().route("/sign-up", post(sign_up))
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
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn the_rules_are_never_checked() {
        let request = Request::post("/sign-up")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"username":"al","email":"nope","age":9}"#))
            .unwrap();
        let status = app().oneshot(request).await.unwrap().status();
        assert_eq!(status, StatusCode::CREATED);
    }

    #[test]
    fn calling_validate_by_hand_does_find_the_problems() {
        let input = SignUp {
            username: "al".to_owned(),
            email: "nope".to_owned(),
            age: 9,
        };
        assert!(input.validate().is_err());
    }
}
