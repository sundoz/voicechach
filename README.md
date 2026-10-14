# voicechach

Бэкенд голосового и текстового чата на Rust (axum, tokio, sqlx, Postgres, Redis, WebRTC).

- План по этапам: [docs/ROADMAP.md](docs/ROADMAP.md)

## Быстрый старт

```bash
cp .env.example .env
make deps     # Postgres и Redis в docker compose
make run      # http://localhost:8080/health/ready
make check    # fmt + clippy + тесты
```

Полностью в Docker: `docker compose up --build`.

## Структура

```
crates/server/        HTTP/WS API
  config/             default.toml, production.toml
  src/
    main.rs           точка входа
    app.rs            роутер, middleware, graceful shutdown
    config.rs         загрузка настроек
    error.rs          AppError → JSON-ответ
    state.rs          AppState (пул БД)
    telemetry.rs      логирование
    routes/           HTTP-хендлеры
  tests/              интеграционные тесты
migrations/           SQL-миграции sqlx
```
