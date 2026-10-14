//! Бизнес-логика аутентификации. Хендлеры только разбирают HTTP,
//! репозитории только ходят в БД, а решения принимаются здесь.

use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use super::{password, sessions};
use crate::{
    error::{AppError, AppResult},
    state::AppState,
    users::{
        User,
        repo::{self, NewUser},
    },
};

#[derive(Debug, Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    /// Время жизни access-токена в секундах.
    pub expires_in: i64,
}

pub struct Register {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register(
    state: &AppState,
    input: Register,
    user_agent: Option<&str>,
) -> AppResult<(User, Tokens)> {
    let email = input.email.to_lowercase();
    let password_hash = password::hash(input.password).await?;

    // Транзакция: пользователь и его первая сессия создаются вместе или никак.
    let mut tx = state.db.begin().await?;
    let user = repo::insert(
        &mut *tx,
        NewUser {
            username: &input.username,
            email: &email,
            password_hash: &password_hash,
        },
    )
    .await
    .map_err(map_unique_violation)?;
    let tokens = issue_tokens(state, &mut tx, user.id, Uuid::now_v7(), user_agent).await?;
    tx.commit().await?;

    tracing::info!(user_id = %user.id, "user registered");
    Ok((user, tokens))
}

pub async fn login(
    state: &AppState,
    login: &str,
    password: String,
    user_agent: Option<&str>,
) -> AppResult<(User, Tokens)> {
    let credentials = repo::find_credentials(&state.db, login).await?;
    let (user_id, hash) = match credentials {
        Some(c) => (Some(c.id), Some(c.password_hash)),
        None => (None, None),
    };

    // Одинаковый ответ на «нет такого пользователя» и «неверный пароль».
    if !password::verify(password, hash).await? {
        return Err(AppError::Unauthorized);
    }
    let user_id = user_id.ok_or(AppError::Unauthorized)?;
    let user = repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Каждый вход — новое «семейство» токенов (отдельное устройство).
    let mut conn = state.db.acquire().await?;
    let tokens = issue_tokens(state, &mut conn, user.id, Uuid::now_v7(), user_agent).await?;
    Ok((user, tokens))
}

/// Ротация: старый refresh-токен гасится, выдаётся новый в том же семействе.
pub async fn refresh(
    state: &AppState,
    refresh_token: &str,
    user_agent: Option<&str>,
) -> AppResult<Tokens> {
    let mut tx = state.db.begin().await?;

    match sessions::consume(&mut tx, refresh_token).await? {
        sessions::Consumed::Valid { user_id, family_id } => {
            let tokens = issue_tokens(state, &mut tx, user_id, family_id, user_agent).await?;
            tx.commit().await?;
            Ok(tokens)
        }
        sessions::Consumed::Reused { family_id } => {
            // Токен предъявили второй раз: либо его украли, либо украденный
            // уже использовали. Не знаем, кто из двоих легитимный, —
            // разлогиниваем это устройство целиком.
            let revoked = sessions::revoke_family(&mut *tx, family_id).await?;
            // Коммитим, хотя возвращаем ошибку: отзыв должен сохраниться.
            tx.commit().await?;
            tracing::warn!(%family_id, revoked, "refresh token reuse detected");
            Err(AppError::Unauthorized)
        }
        // tx откатится сам при выходе из области видимости (Drop).
        sessions::Consumed::Invalid => Err(AppError::Unauthorized),
    }
}

pub async fn logout(state: &AppState, refresh_token: &str) -> AppResult<()> {
    if let Some(family_id) = sessions::family_of(&state.db, refresh_token).await? {
        sessions::revoke_family(&state.db, family_id).await?;
    }
    Ok(())
}

async fn issue_tokens(
    state: &AppState,
    conn: &mut PgConnection,
    user_id: Uuid,
    family_id: Uuid,
    user_agent: Option<&str>,
) -> AppResult<Tokens> {
    let refresh_token =
        sessions::create(conn, user_id, family_id, user_agent, state.refresh_ttl).await?;
    Ok(Tokens {
        access_token: state.jwt.issue(user_id)?,
        refresh_token,
        token_type: "Bearer",
        expires_in: state.jwt.ttl_secs(),
    })
}

/// Нарушение уникального индекса → понятная клиенту ошибка 409.
fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.is_unique_violation()
    {
        return match db_err.constraint() {
            Some("users_username_key") => AppError::Conflict("username is already taken".into()),
            Some("users_email_key") => AppError::Conflict("email is already registered".into()),
            _ => AppError::Database(err),
        };
    }
    AppError::Database(err)
}
