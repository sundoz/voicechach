# Roadmap: voicechach

Бэкенд голосового и текстового чата на Rust. Каждый этап — законченный, рабочий
и задеплоенный кусок. Каждый этап учит конкретным вещам в Rust и в бэкенде.

## Целевая архитектура

```
             ┌──────────── клиент (web / mobile) ────────────┐
             │  REST (HTTPS)     WebSocket (WSS)    WebRTC   │
             └──────┬───────────────┬──────────────────┬─────┘
                    │               │                  │ UDP (SRTP)
              ┌─────▼───────────────▼─────┐      ┌─────▼──────┐
              │   api-сервер (axum)       │◄────►│ voice SFU  │
              │  auth, чаты, сообщения,   │ RPC  │ (str0m)    │
              │  WS-шлюз, сигналинг       │      └────────────┘
              └──┬──────────┬──────────┬──┘
                 │          │          │
            ┌────▼───┐ ┌────▼────┐ ┌───▼──────────┐
            │Postgres│ │  Redis  │ │ S3 (вложения)│
            │ данные │ │pub/sub, │ │   MinIO      │
            └────────┘ │presence,│ └──────────────┘
                       │rate-lim │
                       └─────────┘
  Наблюдаемость: tracing → OpenTelemetry → Grafana (Tempo/Loki/Prometheus)
```

Стек: `tokio`, `axum`, `sqlx` (Postgres), `redis`, `jsonwebtoken` + `argon2`,
`tracing` + OpenTelemetry, `str0m` (WebRTC), Docker, GitHub Actions.

## Этапы

### Этап 0 — Каркас ✅
- Cargo workspace, crate `server`
- Конфиг из файлов + переменных окружения (`APP__...`)
- Логирование `tracing` (pretty локально, JSON в проде)
- Единый тип ошибок → HTTP-ответы
- `/health/live`, `/health/ready` (проверка Postgres)
- Graceful shutdown, request-id, таймауты, CORS
- Пул Postgres + миграции `sqlx`
- Dockerfile (multi-stage, cargo-chef), docker-compose (Postgres, Redis)
- CI: fmt, clippy, тесты, сборка образа

**Учим:** модули, `Result`/`?`, `thiserror`, `async`/`tokio`, axum-экстракторы, `Arc`-состояние.

### Этап 1 — Пользователи и аутентификация
- Регистрация / логин, пароли через `argon2`
- Access-JWT (короткий) + refresh-токены в БД с ротацией
- Экстрактор `AuthUser`, middleware
- Валидация входных данных (`validator`)
- Интеграционные тесты с реальным Postgres (`sqlx::test`)

**Учим:** трейты, `FromRequestParts`, работа с БД, тестирование.

### Этап 2 — Текстовый чат (REST)
- Серверы/«гильдии» → каналы (текстовые и голосовые) → сообщения
- Участники и роли, проверка прав
- История сообщений с курсорной пагинацией
- Редактирование / удаление, OpenAPI-документация (`utoipa`)

**Учим:** моделирование домена, транзакции, слои (handler → service → repo).

### Этап 3 — Realtime через WebSocket
- WS-шлюз: аутентификация, heartbeat, подписки на каналы
- Протокол событий (`MESSAGE_CREATE`, `TYPING`, `PRESENCE_UPDATE`...)
- Внутренний fan-out через `tokio::sync::broadcast`
- Presence (онлайн/оффлайн)

**Учим:** каналы tokio, `select!`, конкурентность, backpressure.

### Этап 4 — Горизонтальное масштабирование
- Redis pub/sub между инстансами, presence в Redis
- Rate limiting (token bucket в Redis)
- Нагрузочное тестирование (k6)

**Учим:** распределённые системы, идемпотентность.

### Этап 5 — Голос
- Сигналинг WebRTC через WS (offer/answer/ICE)
- Отдельный сервис `voice` — SFU на `str0m` (пересылка Opus-пакетов)
- Мут/деафен, индикатор речи, TURN (coturn)
- Альтернатива для сравнения: LiveKit как внешний SFU

**Учим:** UDP, WebRTC, работа с бинарными протоколами, sans-IO.

### Этап 6 — Вложения и прочее
- Загрузка файлов через presigned URL в S3/MinIO
- Реакции, упоминания, непрочитанные, поиск

### Этап 7 — Наблюдаемость
- OpenTelemetry трейсы, Prometheus-метрики (`/metrics`)
- Grafana-дашборды, алерты

### Этап 8 — Деплой
- Образы в GHCR, staging + prod
- Вариант A: VPS + docker compose + Caddy (TLS)
- Вариант B: Kubernetes (Helm), managed Postgres
- Миграции при деплое, секреты, бэкапы, CD из GitHub Actions
