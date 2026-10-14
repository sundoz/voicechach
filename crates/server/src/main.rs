use anyhow::Context;
use server::{app, config::Settings, state::AppState, telemetry};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load().context("failed to load configuration")?;
    telemetry::init(&settings.log);

    let state = AppState::new(&settings).await?;
    if settings.database.run_migrations {
        state.run_migrations().await?;
    }

    let addr = settings.server.addr();
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!(%addr, "server started");

    axum::serve(listener, app::router(state, &settings.server))
        .with_graceful_shutdown(app::shutdown_signal())
        .await
        .context("server error")?;

    tracing::info!("server stopped");
    Ok(())
}
