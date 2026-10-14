use std::sync::LazyLock;

use anyhow::Context;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

/// Хэш-заглушка для входа несуществующего пользователя: проверяем пароль
/// против неё, чтобы ответ занимал столько же времени, сколько для
/// существующего. Иначе по времени ответа можно узнать, какие логины заняты.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    Argon2::default()
        .hash_password(b"dummy-password")
        .expect("hashing dummy password")
        .to_string()
});

/// Argon2 специально медленный (~десятки мс CPU). В async-коде такую работу
/// нельзя делать прямо в обработчике: она заблокирует поток tokio и все
/// остальные запросы на нём. `spawn_blocking` уносит её в отдельный пул потоков.
pub async fn hash(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|e| anyhow::anyhow!("hash password: {e}"))
    })
    .await
    .context("hash task panicked")?
}

/// `None` у `hash` — пользователь не найден; проверяем против заглушки
/// и всё равно возвращаем `false`.
pub async fn verify(password: String, hash: Option<String>) -> anyhow::Result<bool> {
    tokio::task::spawn_blocking(move || {
        let exists = hash.is_some();
        let hash = hash.unwrap_or_else(|| DUMMY_HASH.clone());
        let ok = Argon2::default()
            .verify_password(password.as_bytes(), hash.as_str())
            .is_ok();
        ok && exists
    })
    .await
    .context("verify task panicked")
}
