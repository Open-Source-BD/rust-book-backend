// ANCHOR: helpers
use a2_build_todo_api::app;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn send(app: Router, method: &str, uri: &str, body: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(json) => {
            builder = builder.header("content-type", "application/json");
            Body::from(json.to_owned())
        }
        None => Body::empty(),
    };
    let response = app.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}
// ANCHOR_END: helpers

// ANCHOR: lifecycle
#[tokio::test]
async fn full_lifecycle() {
    let app = app();
    assert_eq!(
        send(app.clone(), "GET", "/api/todos", None).await,
        (StatusCode::OK, "[]".into())
    );
    let (s, b) = send(
        app.clone(),
        "POST",
        "/api/todos",
        Some(r#"{"title":"buy milk"}"#),
    )
    .await;
    assert_eq!(
        (s, b.as_str()),
        (
            StatusCode::CREATED,
            r#"{"id":1,"title":"buy milk","done":false}"#
        )
    );
    let (s, b) = send(
        app.clone(),
        "PATCH",
        "/api/todos/1",
        Some(r#"{"done":true}"#),
    )
    .await;
    assert_eq!(
        (s, b.as_str()),
        (StatusCode::OK, r#"{"id":1,"title":"buy milk","done":true}"#)
    );
    assert_eq!(
        send(app.clone(), "DELETE", "/api/todos/1", None).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        send(app.clone(), "GET", "/api/todos/1", None).await,
        (
            StatusCode::NOT_FOUND,
            r#"{"error":"todo 1 not found"}"#.into()
        )
    );
    let (s, _) = send(
        app.clone(),
        "POST",
        "/api/todos",
        Some(r#"{"title":"again"}"#),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    assert_eq!(
        send(app, "GET", "/api/todos/2", None).await.0,
        StatusCode::OK
    );
}
// ANCHOR_END: lifecycle

// ANCHOR: validation
#[tokio::test]
async fn validation_and_rejections() {
    let (s, b) = send(app(), "POST", "/api/todos", Some(r#"{"title":""}"#)).await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(b.contains("\"title\""));
    assert_eq!(
        send(app(), "DELETE", "/api/todos/9", None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(app(), "PUT", "/api/todos/1", None).await.0,
        StatusCode::METHOD_NOT_ALLOWED
    );
    assert_eq!(
        send(app(), "GET", "/health", None).await,
        (StatusCode::OK, "ok".into())
    );
}
// ANCHOR_END: validation
