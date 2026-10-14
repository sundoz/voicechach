pub mod repo;
mod routes;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

pub use routes::router;

/// Пользователь в том виде, в каком его можно отдавать наружу.
/// Хэша пароля здесь нет намеренно: его невозможно случайно сериализовать.
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
