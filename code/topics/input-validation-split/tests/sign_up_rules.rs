use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use input_validation_split::app;
use serde_json::Value;
use tower::ServiceExt;

async fn send(app: Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn sign_up(body: &str) -> (StatusCode, String) {
    let request = Request::post("/sign-up")
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap();
    send(app(), request).await
}

/// The names of the fields in a validation error body, in order.
fn bad_fields(body: &str) -> Vec<String> {
    let errors: Value = serde_json::from_str(body).unwrap();
    errors.as_object().unwrap().keys().cloned().collect()
}

#[tokio::test]
async fn a_good_sign_up_is_welcomed() {
    let (status, body) = sign_up(r#"{"username":"ada","email":"ada@example.com","age":36}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body, "welcome, ada <ada@example.com>");
}

#[tokio::test]
async fn the_edges_of_each_rule_are_allowed() {
    let cases = [
        r#"{"username":"ada","email":"ada@example.com","age":13}"#,
        r#"{"username":"abcdefghijabcdefghij","email":"ada@example.com","age":36}"#,
    ];
    for body in cases {
        assert_eq!(sign_up(body).await.0, StatusCode::CREATED, "{body}");
    }
}

#[tokio::test]
async fn each_rule_refuses_its_bad_value() {
    let cases = [
        (
            r#"{"username":"al","email":"ada@example.com","age":36}"#,
            "username",
        ),
        (
            r#"{"username":"abcdefghijabcdefghijk","email":"ada@example.com","age":36}"#,
            "username",
        ),
        (r#"{"username":"ada","email":"nope","age":36}"#, "email"),
        (
            r#"{"username":"ada","email":"ada@example.com","age":12}"#,
            "age",
        ),
    ];
    for (body, field) in cases {
        let (status, errors) = sign_up(body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(bad_fields(&errors), [field], "{body}");
    }
}

#[tokio::test]
async fn every_problem_comes_back_at_once() {
    let (status, errors) = sign_up(r#"{"username":"al","email":"nope","age":9}"#).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_fields(&errors), ["age", "email", "username"]);
}

#[tokio::test]
async fn broken_json_never_reaches_the_rules() {
    assert_eq!(
        sign_up(r#"{"username":"ada"}"#).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(sign_up("{oops").await.0, StatusCode::BAD_REQUEST);
}
