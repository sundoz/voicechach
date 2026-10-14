use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

/// Как `Json<T>`, но дополнительно вызывает `T::validate()`, а ошибки
/// разбора и валидации отдаёт в нашем едином JSON-формате.
pub struct ValidJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| AppError::BadRequest(rejection.body_text()))?;
        value.validate()?;
        Ok(Self(value))
    }
}
