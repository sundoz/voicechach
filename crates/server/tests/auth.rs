//! Каждый `#[sqlx::test]` получает собственную чистую базу с применёнными
//! миграциями, поэтому тесты не мешают друг другу и идут параллельно.
//! Нужен DATABASE_URL (см. `make deps`).

mod common;

use axum::http::StatusCode;
use common::{post, register, send};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn register_returns_user_and_tokens(pool: PgPool) {
    let app = common::app(pool);
    let res = register(&app, "alice").await;

    assert_eq!(res.body["user"]["username"], "alice");
    assert_eq!(res.body["user"]["email"], "alice@example.com");
    assert!(res.body["user"].get("password_hash").is_none());
    assert_eq!(res.body["token_type"], "Bearer");
    assert!(res.body["access_token"].is_string());
    assert!(res.body["refresh_token"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn username_and_email_are_unique_case_insensitive(pool: PgPool) {
    let app = common::app(pool);
    register(&app, "alice").await;

    let res = post(
        &app,
        "/auth/register",
        json!({ "username": "ALICE", "email": "other@example.com", "password": "12345678" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"]["message"], "username is already taken");

    let res = post(
        &app,
        "/auth/register",
        json!({ "username": "bob", "email": "Alice@Example.com", "password": "12345678" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"]["message"], "email is already registered");
}

#[sqlx::test(migrations = "../../migrations")]
async fn register_validates_input(pool: PgPool) {
    let app = common::app(pool);
    let res = post(
        &app,
        "/auth/register",
        json!({ "username": "a b", "email": "not-an-email", "password": "short" }),
    )
    .await;

    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    let details = &res.body["error"]["details"];
    assert!(details["username"].is_array());
    assert!(details["email"].is_array());
    assert!(details["password"].is_array());
}

#[sqlx::test(migrations = "../../migrations")]
async fn malformed_json_is_bad_request(pool: PgPool) {
    let app = common::app(pool);
    let res = post(&app, "/auth/register", json!({ "username": "alice" })).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"]["code"], "bad_request");
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_by_username_or_email(pool: PgPool) {
    let app = common::app(pool);
    register(&app, "alice").await;

    for login in ["alice", "Alice", "ALICE@example.com"] {
        let res = post(
            &app,
            "/auth/login",
            json!({ "login": login, "password": "correct horse battery" }),
        )
        .await;
        assert_eq!(res.status, StatusCode::OK, "login={login}");
        assert_eq!(res.body["user"]["username"], "alice");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_failures_are_indistinguishable(pool: PgPool) {
    let app = common::app(pool);
    register(&app, "alice").await;

    let wrong_password = post(
        &app,
        "/auth/login",
        json!({ "login": "alice", "password": "wrong password" }),
    )
    .await;
    let unknown_user = post(
        &app,
        "/auth/login",
        json!({ "login": "nobody", "password": "whatever" }),
    )
    .await;

    assert_eq!(wrong_password.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_password.status, unknown_user.status);
    assert_eq!(wrong_password.body, unknown_user.body);
}

#[sqlx::test(migrations = "../../migrations")]
async fn me_requires_valid_token(pool: PgPool) {
    let app = common::app(pool);
    let reg = register(&app, "alice").await;
    let token = reg.body["access_token"].as_str().unwrap();

    let res = send(&app, "GET", "/users/@me", None, Some(token)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["username"], "alice");

    let res = send(&app, "GET", "/users/@me", None, None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);

    let res = send(&app, "GET", "/users/@me", None, Some("garbage")).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn refresh_rotates_token(pool: PgPool) {
    let app = common::app(pool);
    let reg = register(&app, "alice").await;
    let old = reg.body["refresh_token"].as_str().unwrap();

    let res = post(&app, "/auth/refresh", json!({ "refresh_token": old })).await;
    assert_eq!(res.status, StatusCode::OK);
    let new = res.body["refresh_token"].as_str().unwrap();
    assert_ne!(old, new);

    // Новый токен работает.
    let res = post(&app, "/auth/refresh", json!({ "refresh_token": new })).await;
    assert_eq!(res.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn refresh_token_reuse_revokes_whole_family(pool: PgPool) {
    let app = common::app(pool);
    let reg = register(&app, "alice").await;
    let stolen = reg.body["refresh_token"].as_str().unwrap();

    // Легитимный клиент обновился и получил новый токен.
    let res = post(&app, "/auth/refresh", json!({ "refresh_token": stolen })).await;
    let current = res.body["refresh_token"].as_str().unwrap().to_owned();

    // Злоумышленник пробует старый — получает 401...
    let res = post(&app, "/auth/refresh", json!({ "refresh_token": stolen })).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);

    // ...и заодно отзывается актуальный токен этого устройства.
    let res = post(&app, "/auth/refresh", json!({ "refresh_token": current })).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn logout_revokes_refresh_token(pool: PgPool) {
    let app = common::app(pool);
    let reg = register(&app, "alice").await;
    let token = reg.body["refresh_token"].as_str().unwrap();

    let res = post(&app, "/auth/logout", json!({ "refresh_token": token })).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);

    let res = post(&app, "/auth/refresh", json!({ "refresh_token": token })).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_on_different_devices_are_independent(pool: PgPool) {
    let app = common::app(pool);
    let phone = register(&app, "alice").await;
    let laptop = post(
        &app,
        "/auth/login",
        json!({ "login": "alice", "password": "correct horse battery" }),
    )
    .await;

    let phone_token = phone.body["refresh_token"].as_str().unwrap();
    let laptop_token = laptop.body["refresh_token"].as_str().unwrap();
    post(
        &app,
        "/auth/logout",
        json!({ "refresh_token": phone_token }),
    )
    .await;

    let res = post(
        &app,
        "/auth/refresh",
        json!({ "refresh_token": laptop_token }),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
}
