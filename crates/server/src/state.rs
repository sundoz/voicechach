use std::{sync::Arc, time::Duration};

use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{auth::jwt::Jwt, config::Settings};

/// Общее состояние, доступное всем хендлерам через `State<AppState>`.
/// Клонируется на каждый запрос, поэтому внутри только дешёвые
/// для клонирования вещи (`PgPool` и `Arc` — это счётчики ссылок).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt: Arc<Jwt>,
    pub refresh_ttl: chrono::Duration,
}

impl AppState {
    pub async fn new(settings: &Settings) -> anyhow::Result<Self> {
        // connect_lazy: сервер стартует даже если БД ещё не готова,
        // а /health/ready честно покажет, что она недоступна.
        let db = PgPoolOptions::new()
            .max_connections(settings.database.max_connections)
            .acquire_timeout(Duration::from_secs(3))
            .connect_lazy(&settings.database.url)
            .context("invalid database url")?;
        Self::with_pool(db, settings)
    }

    /// Отдельный конструктор нужен тестам: `sqlx::test` сам создаёт пул
    /// к свежей временной базе.
    pub fn with_pool(db: PgPool, settings: &Settings) -> anyhow::Result<Self> {
        let auth = &settings.auth;
        Ok(Self {
            db,
            jwt: Arc::new(Jwt::new(
                &auth.jwt_secret,
                chrono::Duration::seconds(auth.access_token_ttl_secs),
            )?),
            refresh_ttl: chrono::Duration::days(auth.refresh_token_ttl_days),
        })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.db)
            .await
            .context("failed to run migrations")?;
        tracing::info!("migrations applied");
        Ok(())
    }
}
