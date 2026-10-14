use std::time::Duration;

use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Settings;

/// Общее состояние, доступное всем хендлерам через `State<AppState>`.
/// Клонируется на каждый запрос, поэтому внутри только дешёвые
/// для клонирования вещи (`PgPool` — это `Arc` внутри).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
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
        Ok(Self { db })
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
