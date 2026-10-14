mod health;

use axum::Router;

use crate::{error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .fallback(|| async { AppError::NotFound })
}
