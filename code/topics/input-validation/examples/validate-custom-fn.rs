//! More examples: a rule you write yourself, `no_spaces`, with `custom(function = ...)`.
//! Run: `cargo run -p input-validation --example validate-custom-fn`, then POST /sign-up.
// ANCHOR: imports
use axum::{
    Json, Router,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, de::DeserializeOwned};
use validator::{Validate, ValidationError};
// ANCHOR_END: imports

// ANCHOR: rule
fn no_spaces(username: &str) -> Result<(), ValidationError> {
    if username.contains(' ') {
        Err(ValidationError::new("no_spaces").with_message("must not contain spaces".into()))
    } else {
        Ok(())
    }
}
// ANCHOR_END: rule

// ANCHOR: input
#[derive(Deserialize, Validate)]
struct SignUp {
    #[validate(length(min = 3, max = 20), custom(function = "no_spaces"))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(range(min = 13))]
    age: u32,
}
// ANCHOR_END: input

// ANCHOR: extractor
struct ValidatedJson<T>(T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| rejection.into_response())?;
        value.validate().map_err(|errors| {
            let sorted = serde_json::to_value(errors).expect("errors always convert to JSON");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(sorted)).into_response()
        })?;
        Ok(ValidatedJson(value))
    }
}
// ANCHOR_END: extractor

async fn sign_up(ValidatedJson(input): ValidatedJson<SignUp>) -> (StatusCode, String) {
    (
        StatusCode::CREATED,
        format!("welcome, {} <{}>", input.username, input.email),
    )
}

fn app() -> Router {
    Router::new().route("/sign-up", post(sign_up))
}

// ANCHOR: main
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
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn send(uri: &str, body: &str) -> (StatusCode, String) {
        let request = Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[test]
    fn the_rule_on_its_own() {
        assert!(no_spaces("ada").is_ok());
        assert!(no_spaces("ada lovelace").is_err());
    }

    #[tokio::test]
    async fn a_space_is_refused() {
        let body = r#"{"username":"ada lovelace","email":"ada@example.com","age":36}"#;
        let (status, body) = send("/sign-up", body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("no_spaces"));
    }
}
