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
        value
            .validate()
            .map_err(|errors| (StatusCode::UNPROCESSABLE_ENTITY, Json(errors)).into_response())?;
        Ok(ValidatedJson(value))
    }
}
// ANCHOR_END: extractor

// ANCHOR: handler
async fn sign_up(ValidatedJson(input): ValidatedJson<SignUp>) -> (StatusCode, String) {
    (
        StatusCode::CREATED,
        format!("welcome, {} <{}>", input.username, input.email),
    )
}

fn app() -> Router {
    Router::new().route("/sign-up", post(sign_up))
}
// ANCHOR_END: handler

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

    async fn send_to(app: Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        send_to(app(), request).await
    }

    #[allow(dead_code)]
    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    #[allow(dead_code)]
    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn good_input_is_accepted() {
        let body = r#"{"username":"ada","email":"ada@example.com","age":36}"#;
        assert_eq!(
            send(post_json("/sign-up", body)).await,
            (
                StatusCode::CREATED,
                "welcome, ada <ada@example.com>".to_string()
            )
        );
    }

    #[tokio::test]
    async fn bad_values_are_422_with_every_problem() {
        let (status, body) = send(post_json(
            "/sign-up",
            r#"{"username":"al","email":"nope","age":9}"#,
        ))
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        println!("VALIDATION {body}");
        assert!(
            body.contains("\"username\"") && body.contains("\"email\"") && body.contains("\"age\"")
        );
    }

    #[tokio::test]
    async fn broken_json_still_gets_axums_rejection() {
        assert_eq!(
            send(post_json("/sign-up", r#"{"username":"ada"}"#)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            send(post_json("/sign-up", "{oops")).await.0,
            StatusCode::BAD_REQUEST
        );
    }
}
