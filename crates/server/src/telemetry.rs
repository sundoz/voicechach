use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{LogFormat, LogSettings};

/// Инициализирует глобальный subscriber для `tracing`.
/// `RUST_LOG`, если задан, имеет приоритет над конфигом.
pub fn init(settings: &LogSettings) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&settings.level));

    let registry = tracing_subscriber::registry().with(filter);
    match settings.format {
        LogFormat::Json => registry
            .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
            .init(),
        LogFormat::Pretty => registry.with(tracing_subscriber::fmt::layer()).init(),
    }
}
