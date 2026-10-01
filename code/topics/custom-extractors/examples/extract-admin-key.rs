//! Your turn (From scratch): an `AdminKey` extractor. It needs a valid API key AND the header
//! `x-role: admin`; otherwise 403 "admins only".
//! Run: `cargo run -p custom-extractors --example extract-admin-key`, then GET /admin.
use axum::{
    Router,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    routing::get,
};

struct ApiKey(String);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
        if value == "letmein" {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
        }
    }
}

// ANCHOR: admin
struct AdminKey(String);

impl<S> FromRequestParts<S> for AdminKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ApiKey(key) = ApiKey::from_request_parts(parts, state).await?;
        let role = parts
            .headers
            .get("x-role")
            .and_then(|value| value.to_str().ok());
        if role == Some("admin") {
            Ok(AdminKey(key))
        } else {
            Err((StatusCode::FORBIDDEN, "admins only"))
        }
    }
}
// ANCHOR_END: admin

// ANCHOR: handlers
async fn secret(ApiKey(key): ApiKey) -> String {
    format!("welcome, holder of key {key:?}")
}

async fn admin(AdminKey(key): AdminKey) -> String {
    format!("welcome to the admin room, holder of key {key:?}")
}

fn app() -> Router {
    Router::new()
        .route("/secret", get(secret))
        .route("/admin", get(admin))
}
// ANCHOR_END: handlers

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
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn admin_with(headers: &[(&str, &str)]) -> (StatusCode, String) {
        let mut builder = Request::get("/admin");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let response = app()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn needs_key_and_role() {
        assert_eq!(admin_with(&[]).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(
            admin_with(&[("x-api-key", "letmein")]).await,
            (StatusCode::FORBIDDEN, "admins only".to_string())
        );
        assert_eq!(
            admin_with(&[("x-api-key", "letmein"), ("x-role", "reader")]).await,
            (StatusCode::FORBIDDEN, "admins only".to_string())
        );
        assert_eq!(
            admin_with(&[("x-api-key", "guess"), ("x-role", "admin")]).await,
            (StatusCode::FORBIDDEN, "wrong API key".to_string())
        );
        assert_eq!(
            admin_with(&[("x-api-key", "letmein"), ("x-role", "admin")]).await,
            (
                StatusCode::OK,
                "welcome to the admin room, holder of key \"letmein\"".to_string()
            )
        );
    }
}
