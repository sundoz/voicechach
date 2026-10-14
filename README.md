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

### Работа с БД

Нужен `sqlx-cli`: `cargo install sqlx-cli --no-default-features --features postgres,rustls`.

```bash
make migration name=add_guilds   # создать файл миграции
make migrate                     # применить
make prepare                     # после изменения SQL — обновить .sqlx/ и закоммитить
```

SQL в `query!`/`query_as!` проверяется при компиляции. Без живой БД
(CI, Docker) используется сохранённая схема из `.sqlx/` (`SQLX_OFFLINE=true`).

## API

| Метод | Путь | Описание |
|---|---|---|
| POST | `/auth/register` | `{username, email, password}` → пользователь + токены |
| POST | `/auth/login` | `{login, password}` (login = username или email) |
| POST | `/auth/refresh` | `{refresh_token}` → новая пара токенов (старый гасится) |
| POST | `/auth/logout` | `{refresh_token}` → отзыв сессии устройства |
| GET | `/users/@me` | текущий пользователь, `Authorization: Bearer <access_token>` |
| GET | `/health/live`, `/health/ready` | проверки для оркестратора |

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
    extract.rs        ValidJson: JSON + валидация
    auth/             пароли, JWT, refresh-сессии, экстрактор AuthUser
    users/            модель пользователя, репозиторий, /users/@me
    routes/           сборка роутера, health
  tests/              интеграционные тесты
migrations/           SQL-миграции sqlx
.sqlx/                офлайн-схема для проверки запросов без БД
```
