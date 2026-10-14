use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Единая ошибка приложения. Хендлеры возвращают `Result<T, AppError>`,
/// а `IntoResponse` превращает её в JSON-ответ с нужным статусом.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("service unavailable: {0}")]
    Unavailable(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized => "unauthorized",
            Self::Unavailable(_) => "unavailable",
            Self::Database(_) | Self::Internal(_) => "internal",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        // Детали внутренних ошибок пишем в лог, но не отдаём клиенту.
        let message = if status.is_server_error() && status != StatusCode::SERVICE_UNAVAILABLE {
            tracing::error!(error = ?self, "internal error");
            "internal server error".to_string()
        } else {
            self.to_string()
        };

        let body = json!({ "error": { "code": self.code(), "message": message } });
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
