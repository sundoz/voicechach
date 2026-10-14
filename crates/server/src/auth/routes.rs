use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::USER_AGENT},
    routing::post,
};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::service::{self, Register, Tokens};
use crate::{error::AppResult, extract::ValidJson, state::AppState, users::User};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

#[derive(Deserialize, Validate)]
struct RegisterRequest {
    #[validate(length(min = 3, max = 32), custom(function = "validate_username"))]
    username: String,
    #[validate(email, length(max = 254))]
    email: String,
    // Верхняя граница не для красоты: argon2 от мегабайтного пароля — DoS.
    #[validate(length(min = 8, max = 128))]
    password: String,
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        Ok(())
    } else {
        Err(ValidationError::new("username_chars")
            .with_message("only latin letters, digits, '_' and '.' are allowed".into()))
    }
}

#[derive(Deserialize, Validate)]
struct LoginRequest {
    /// Username или email.
    #[validate(length(min = 1, max = 254))]
    login: String,
    #[validate(length(min = 1, max = 128))]
    password: String,
}

#[derive(Deserialize, Validate)]
struct RefreshRequest {
    #[validate(length(min = 1, max = 128))]
    refresh_token: String,
}

#[derive(Serialize)]
struct AuthResponse {
    user: User,
    #[serde(flatten)]
    tokens: Tokens,
}

fn user_agent(headers: &HeaderMap) -> Option<&str> {
    headers.get(USER_AGENT).and_then(|v| v.to_str().ok())
}

// Экстрактор тела (ValidJson) всегда последний аргумент: тело читается один раз.
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidJson(req): ValidJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    let input = Register {
        username: req.username,
        email: req.email,
        password: req.password,
    };
    let (user, tokens) = service::register(&state, input, user_agent(&headers)).await?;
    Ok((StatusCode::CREATED, Json(AuthResponse { user, tokens })))
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidJson(req): ValidJson<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let (user, tokens) =
        service::login(&state, &req.login, req.password, user_agent(&headers)).await?;
    Ok(Json(AuthResponse { user, tokens }))
}

async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidJson(req): ValidJson<RefreshRequest>,
) -> AppResult<Json<Tokens>> {
    let tokens = service::refresh(&state, &req.refresh_token, user_agent(&headers)).await?;
    Ok(Json(tokens))
}

async fn logout(
    State(state): State<AppState>,
    ValidJson(req): ValidJson<RefreshRequest>,
) -> AppResult<StatusCode> {
    service::logout(&state, &req.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}
