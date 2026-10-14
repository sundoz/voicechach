use anyhow::{Context, ensure};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Полезная нагрузка access-токена. Подписана сервером, поэтому
/// проверка не требует похода в БД — в этом смысл JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
}

pub struct Jwt {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    ttl: Duration,
}

impl Jwt {
    pub fn new(secret: &str, ttl: Duration) -> anyhow::Result<Self> {
        ensure!(secret.len() >= 32, "jwt secret must be at least 32 bytes");
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 5; // допуск на рассинхрон часов, секунды
        Ok(Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            validation,
            ttl,
        })
    }

    pub fn ttl_secs(&self) -> i64 {
        self.ttl.num_seconds()
    }

    pub fn issue(&self, user_id: Uuid) -> anyhow::Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            iat: now.timestamp(),
            exp: (now + self.ttl).timestamp(),
        };
        jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .context("encode jwt")
    }

    /// Проверяет подпись и срок действия.
    pub fn verify(&self, token: &str) -> Option<Claims> {
        jsonwebtoken::decode::<Claims>(token, &self.decoding, &self.validation)
            .ok()
            .map(|data| data.claims)
    }
}
