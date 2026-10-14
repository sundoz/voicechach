use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{
    error::{AppError, AppResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/live", get(live))
        .route("/ready", get(ready))
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
}

/// Liveness: процесс жив и отвечает. Не трогает внешние зависимости,
/// иначе оркестратор будет перезапускать нас из-за упавшей БД.
async fn live() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Readiness: готовы принимать трафик (БД доступна).
async fn ready(State(state): State<AppState>) -> AppResult<Json<Health>> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Unavailable(format!("database: {e}")))?;

    Ok(Json(Health {
        status: "ready",
        version: env!("CARGO_PKG_VERSION"),
    }))
}
