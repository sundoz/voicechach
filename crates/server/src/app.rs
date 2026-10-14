use std::time::Duration;

use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::{config::ServerSettings, routes, state::AppState};

const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Собирает роутер со всеми middleware.
pub fn router(state: AppState, settings: &ServerSettings) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            settings
                .cors_origins
                .iter()
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // Порядок: запрос проходит слои сверху вниз, ответ — снизу вверх.
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid))
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &axum::extract::Request| {
                let request_id = req
                    .headers()
                    .get(&REQUEST_ID)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("-");
                tracing::info_span!(
                    "http",
                    method = %req.method(),
                    uri = %req.uri(),
                    request_id,
                )
            }),
        )
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(settings.request_timeout_secs),
        ))
        .layer(cors);

    routes::router().layer(middleware).with_state(state)
}

/// Ждёт Ctrl+C или SIGTERM (его шлют Docker/Kubernetes при остановке).
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
