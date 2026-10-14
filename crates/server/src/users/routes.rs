use axum::{Json, Router, extract::State, routing::get};

use super::{User, repo};
use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/@me", get(me))
}

/// Наличие `AuthUser` в аргументах = эндпоинт требует авторизации.
async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<User>> {
    let user = repo::find_by_id(&state.db, user.id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(user))
}
