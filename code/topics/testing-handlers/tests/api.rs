// ANCHOR: helpers
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use testing_handlers::{Note, app};
use tower::ServiceExt;

async fn send(app: Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post_json(uri: &str, body: &str) -> Request<Body> {
    Request::post(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}
// ANCHOR_END: helpers

// ANCHOR: first_test
#[tokio::test]
async fn health_says_ok() {
    let (status, body) = send(app(), get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}
// ANCHOR_END: first_test

// ANCHOR: json_test
#[tokio::test]
async fn a_new_note_comes_back_as_json() {
    let (status, body) = send(app(), post_json("/notes", r#"{"text":"buy milk"}"#)).await;
    assert_eq!(status, StatusCode::CREATED);
    let note: Note = serde_json::from_str(&body).unwrap();
    assert_eq!(
        note,
        Note {
            id: 1,
            text: "buy milk".to_string()
        }
    );
}
// ANCHOR_END: json_test

// ANCHOR: state_test
#[tokio::test]
async fn notes_are_remembered_between_requests() {
    let app = app();
    send(app.clone(), post_json("/notes", r#"{"text":"one"}"#)).await;
    send(app.clone(), post_json("/notes", r#"{"text":"two"}"#)).await;
    let (_, body) = send(app, get("/notes")).await;
    let notes: Vec<Note> = serde_json::from_str(&body).unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[1].text, "two");
}
// ANCHOR_END: state_test

// ANCHOR: error_test
#[tokio::test]
async fn bad_json_is_rejected() {
    let (status, _) = send(app(), post_json("/notes", r#"{"words":"oops"}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
// ANCHOR_END: error_test
