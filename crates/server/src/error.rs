use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

/// Единая ошибка приложения. Хендлеры возвращают `Result<T, AppError>`,
/// а `IntoResponse` превращает её в JSON-ответ с нужным статусом.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("validation failed")]
    Validation(Value),
    #[error("{0}")]
    Conflict(String),
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
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::Validation(_) => "validation",
            Self::Conflict(_) => "conflict",
            Self::Unauthorized => "unauthorized",
            Self::Unavailable(_) => "unavailable",
            Self::Database(_) | Self::Internal(_) => "internal",
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        Self::Validation(serde_json::to_value(errors).unwrap_or(Value::Null))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        // Детали внутренних ошибок пишем в лог, но не отдаём клиенту.
        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = ?self, "internal error");
            "internal server error".to_string()
        } else {
            self.to_string()
        };

        let mut error = json!({ "code": self.code(), "message": message });
        if let Self::Validation(details) = self {
            error["details"] = details;
        }
        (status, Json(json!({ "error": error }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
