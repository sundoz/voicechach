use axum::{extract::FromRequestParts, http::header::AUTHORIZATION, http::request::Parts};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

/// Авторизованный пользователь. Достаточно добавить `user: AuthUser`
/// в аргументы хендлера — без валидного `Authorization: Bearer <jwt>`
/// хендлер даже не будет вызван, клиент получит 401.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = state.jwt.verify(token).ok_or(AppError::Unauthorized)?;
        Ok(Self { id: claims.sub })
    }
}
