//! Your turn (From scratch): `POST /books` with a title, a page count and a 13-digit ISBN.
//! Run: `cargo run -p input-validation --example validate-create-book`, then POST /books.
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
fn thirteen_digits(isbn: &str) -> Result<(), ValidationError> {
    if isbn.len() == 13 && isbn.bytes().all(|byte| byte.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new("isbn").with_message("must be exactly 13 digits".into()))
    }
}
// ANCHOR_END: rule

// ANCHOR: input
#[derive(Deserialize, Validate)]
struct CreateBook {
    #[validate(length(min = 1, max = 200))]
    title: String,
    #[validate(range(min = 1, max = 5000))]
    pages: u32,
    #[validate(custom(function = "thirteen_digits"))]
    isbn: String,
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
async fn create_book(ValidatedJson(book): ValidatedJson<CreateBook>) -> (StatusCode, String) {
    (
        StatusCode::CREATED,
        format!(
            "added {:?}, {} pages, ISBN {}",
            book.title, book.pages, book.isbn
        ),
    )
}

fn app() -> Router {
    Router::new().route("/books", post(create_book))
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
    fn the_isbn_rule_on_its_own() {
        assert!(thirteen_digits("9780441013593").is_ok());
        assert!(thirteen_digits("978044101359").is_err());
        assert!(thirteen_digits("978-0441013593").is_err());
    }

    #[tokio::test]
    async fn a_good_book_is_added() {
        let body = r#"{"title":"Dune","pages":412,"isbn":"9780441013593"}"#;
        assert_eq!(
            send("/books", body).await,
            (
                StatusCode::CREATED,
                r#"added "Dune", 412 pages, ISBN 9780441013593"#.to_string()
            )
        );
    }

    #[tokio::test]
    async fn every_bad_field_is_reported() {
        let body = r#"{"title":"","pages":0,"isbn":"978-0441013593"}"#;
        let (status, body) = send("/books", body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            body.contains("\"isbn\"") && body.contains("\"pages\"") && body.contains("\"title\"")
        );
    }
}
