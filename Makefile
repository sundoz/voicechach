.PHONY: dev deps down run test lint fmt check docker

deps:            ## Поднять Postgres и Redis
	docker compose up -d postgres redis

down:            ## Остановить окружение
	docker compose down

run:             ## Запустить сервер локально
	cargo run --bin server

test:            ## Тесты (DATABASE_URL включает тесты с БД)
	cargo test --all

lint:            ## Проверки как в CI
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

check: lint test

docker:          ## Собрать образ
	docker build -t voicechach-server .
