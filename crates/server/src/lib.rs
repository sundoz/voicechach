//! Библиотечная часть сервера. `main.rs` только собирает всё вместе,
//! а тесты используют те же функции, что и продакшен.

pub mod app;
pub mod auth;
pub mod config;
pub mod error;
pub mod extract;
pub mod routes;
pub mod state;
pub mod telemetry;
pub mod users;
