use serde::Deserialize;

/// Конфигурация приложения. Источники (по возрастанию приоритета):
/// 1. `config/default.toml`
/// 2. `config/<APP_ENV>.toml` (например, `production`)
/// 3. переменные окружения `APP__SECTION__KEY`
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub log: LogSettings,
    pub auth: AuthSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub cors_origins: Vec<String>,
}

impl ServerSettings {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub run_migrations: bool,
}

#[derive(Clone, Deserialize)]
pub struct AuthSettings {
    pub jwt_secret: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_days: i64,
}

// Debug реализован вручную, чтобы секрет не утёк в логи через `{:?}`.
impl std::fmt::Debug for AuthSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthSettings")
            .field("jwt_secret", &"[redacted]")
            .field("access_token_ttl_secs", &self.access_token_ttl_secs)
            .field("refresh_token_ttl_days", &self.refresh_token_ttl_days)
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogSettings {
    pub level: String,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Pretty,
    Json,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".into());
        let dir = std::env::var("APP_CONFIG_DIR")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/config").to_string());

        config::Config::builder()
            .add_source(config::File::with_name(&format!("{dir}/default")))
            .add_source(config::File::with_name(&format!("{dir}/{env}")).required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("__")
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("server.cors_origins")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
