mod health;

use axum::Router;

use crate::{auth, error::AppError, state::AppState, users};

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .fallback(|| async { AppError::NotFound })
}
