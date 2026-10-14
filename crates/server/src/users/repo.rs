//! Слой доступа к данным. Только SQL, без бизнес-логики.
//!
//! `query_as!` проверяет запрос против реальной схемы во время компиляции:
//! опечатка в имени колонки или несовпадение типов — ошибка `cargo build`.

use sqlx::PgExecutor;
use uuid::Uuid;

use super::User;

pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
}

pub struct Credentials {
    pub id: Uuid,
    pub password_hash: String,
}

pub async fn insert(db: impl PgExecutor<'_>, new: NewUser<'_>) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, username, email, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, display_name, created_at
        "#,
        Uuid::now_v7(),
        new.username,
        new.email,
        new.password_hash,
    )
    .fetch_one(db)
    .await
}

pub async fn find_by_id(db: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        "SELECT id, username, email, display_name, created_at FROM users WHERE id = $1",
        id,
    )
    .fetch_optional(db)
    .await
}

/// Ищет по username или email без учёта регистра.
/// Условие совпадает с выражением в уникальных индексах, поэтому они используются.
pub async fn find_credentials(
    db: impl PgExecutor<'_>,
    login: &str,
) -> sqlx::Result<Option<Credentials>> {
    sqlx::query_as!(
        Credentials,
        r#"
        SELECT id, password_hash FROM users
        WHERE lower(username) = lower($1) OR lower(email) = lower($1)
        "#,
        login,
    )
    .fetch_optional(db)
    .await
}
