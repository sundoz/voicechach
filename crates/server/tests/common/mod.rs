#![allow(dead_code)] // каждый тестовый файл использует свою часть хелперов

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::{app, config::Settings, state::AppState};
use sqlx::PgPool;
use tower::ServiceExt;

pub fn app(pool: PgPool) -> Router {
    let settings = Settings::load().expect("config");
    let state = AppState::with_pool(pool, &settings).expect("state");
    app::router(state, &settings.server)
}

pub struct Response {
    pub status: StatusCode,
    pub body: Value,
}

pub async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> Response {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let req = match body {
        Some(body) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();

    // Router — это Service; clone дешёвый, oneshot выполняет один запрос.
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    Response { status, body }
}

pub async fn post(app: &Router, uri: &str, body: Value) -> Response {
    send(app, "POST", uri, Some(body), None).await
}

pub async fn register(app: &Router, username: &str) -> Response {
    let res = post(
        app,
        "/auth/register",
        json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": "correct horse battery",
        }),
    )
    .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.body);
    res
}
