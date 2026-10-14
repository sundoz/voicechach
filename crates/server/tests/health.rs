use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use server::{app, config::Settings, state::AppState};
use tower::ServiceExt;

async fn test_app(db_url: &str) -> axum::Router {
    let mut settings = Settings::load().expect("config");
    settings.database.url = db_url.to_string();
    let state = AppState::new(&settings).await.expect("state");
    app::router(state, &settings.server)
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

// Порт 1 заведомо закрыт — проверяем поведение без БД.
const NO_DB: &str = "postgres://user:pass@127.0.0.1:1/none";

#[tokio::test]
async fn live_is_ok_without_database() {
    let (status, body) = get(test_app(NO_DB).await, "/health/live").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ready_is_unavailable_without_database() {
    let (status, body) = get(test_app(NO_DB).await, "/health/ready").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "unavailable");
}

#[tokio::test]
async fn unknown_route_returns_json_404() {
    let (status, body) = get(test_app(NO_DB).await, "/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

/// Запускается только при наличии DATABASE_URL (локально через docker compose и в CI).
#[tokio::test]
async fn ready_is_ok_with_database() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("DATABASE_URL not set, skipping");
        return;
    };
    let (status, body) = get(test_app(&url).await, "/health/ready").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ready");
}
