# syntax=docker/dockerfile:1

# --- 1. cargo-chef: кэшируем сборку зависимостей отдельно от нашего кода ---
FROM rust:1.97-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Этот слой пересобирается только при изменении Cargo.toml/Cargo.lock
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin server

# --- 2. Минимальный рантайм-образ ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY crates/server/config ./config

ENV APP_ENV=production \
    APP_CONFIG_DIR=/app/config
USER app
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
    CMD curl -fsS http://localhost:8080/health/live || exit 1
ENTRYPOINT ["server"]
