CREATE TABLE users (
    id            UUID PRIMARY KEY,           -- UUIDv7, генерируется в приложении
    username      TEXT NOT NULL,
    email         TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    display_name  TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Уникальность без учёта регистра: "Bob" и "bob" — один пользователь.
CREATE UNIQUE INDEX users_username_key ON users (lower(username));
CREATE UNIQUE INDEX users_email_key ON users (lower(email));

-- Одна строка = один refresh-токен. При ротации старый помечается revoked_at,
-- создаётся новый с тем же family_id. Повторное использование отозванного
-- токена = утечка, отзываем всё семейство.
CREATE TABLE sessions (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    family_id   UUID NOT NULL,
    token_hash  BYTEA NOT NULL UNIQUE,       -- SHA-256 от токена, сам токен не храним
    user_agent  TEXT,
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_family_id_idx ON sessions (family_id);
