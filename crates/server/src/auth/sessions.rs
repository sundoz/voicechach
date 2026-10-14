//! Refresh-токены. Это не JWT, а просто 32 случайных байта: вся информация
//! о сессии лежит в БД, а в базе хранится только SHA-256 от токена.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgExecutor};
use uuid::Uuid;

pub fn hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn generate() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Создаёт сессию и возвращает сам токен (единственный раз, когда он
/// существует в открытом виде — дальше только у клиента).
pub async fn create(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    family_id: Uuid,
    user_agent: Option<&str>,
    ttl: Duration,
) -> sqlx::Result<String> {
    let token = generate();
    sqlx::query!(
        r#"
        INSERT INTO sessions (id, user_id, family_id, token_hash, user_agent, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        Uuid::now_v7(),
        user_id,
        family_id,
        hash(&token),
        user_agent,
        Utc::now() + ttl,
    )
    .execute(db)
    .await?;
    Ok(token)
}

pub enum Consumed {
    /// Токен валиден и теперь погашен.
    Valid { user_id: Uuid, family_id: Uuid },
    /// Токен уже был использован — признак кражи.
    Reused { family_id: Uuid },
    /// Не существует или истёк.
    Invalid,
}

/// Гасит токен. Один `UPDATE ... RETURNING` атомарен: два одновременных
/// запроса с одним токеном не смогут оба получить `Valid`.
pub async fn consume(conn: &mut PgConnection, token: &str) -> sqlx::Result<Consumed> {
    let token_hash = hash(token);

    let row = sqlx::query!(
        r#"
        UPDATE sessions SET revoked_at = now()
        WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()
        RETURNING user_id, family_id
        "#,
        token_hash,
    )
    .fetch_optional(&mut *conn)
    .await?;

    if let Some(row) = row {
        return Ok(Consumed::Valid {
            user_id: row.user_id,
            family_id: row.family_id,
        });
    }

    let reused = sqlx::query_scalar!(
        "SELECT family_id FROM sessions WHERE token_hash = $1 AND revoked_at IS NOT NULL",
        token_hash,
    )
    .fetch_optional(&mut *conn)
    .await?;

    Ok(match reused {
        Some(family_id) => Consumed::Reused { family_id },
        None => Consumed::Invalid,
    })
}

pub async fn revoke_family(db: impl PgExecutor<'_>, family_id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        "UPDATE sessions SET revoked_at = now() WHERE family_id = $1 AND revoked_at IS NULL",
        family_id,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

pub async fn family_of(db: impl PgExecutor<'_>, token: &str) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "SELECT family_id FROM sessions WHERE token_hash = $1",
        hash(token),
    )
    .fetch_optional(db)
    .await
}
