//! Common mistakes: a rule on an `Option` field does not make the field required.
//! Run: `cargo run -p input-validation --example validate-option-mistake`, then POST /sign-up.
// ANCHOR: imports
use axum::{
    Json, Router,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, de::DeserializeOwned};
use validator::Validate;
// ANCHOR_END: imports

// ANCHOR: input
#[derive(Deserialize, Validate)]
struct SignUp {
    #[validate(length(min = 3, max = 20))]
    username: String,
    #[validate(email)]
    email: Option<String>,
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

// ANCHOR: handler
async fn sign_up(ValidatedJson(input): ValidatedJson<SignUp>) -> (StatusCode, String) {
    let email = input.email.as_deref().unwrap_or("no email");
    (
        StatusCode::CREATED,
        format!("welcome, {} <{email}>", input.username),
    )
}
// ANCHOR_END: handler

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

    #[tokio::test]
    async fn a_missing_email_slips_through() {
        assert_eq!(
            send("/sign-up", r#"{"username":"ada"}"#).await,
            (StatusCode::CREATED, "welcome, ada <no email>".to_string())
        );
    }

    #[tokio::test]
    async fn a_present_email_is_still_checked() {
        let body = r#"{"username":"ada","email":"nope"}"#;
        assert_eq!(
            send("/sign-up", body).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
