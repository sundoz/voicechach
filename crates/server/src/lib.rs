//! Библиотечная часть сервера. `main.rs` только собирает всё вместе,
//! а тесты используют те же функции, что и продакшен.

pub mod app;
pub mod config;
pub mod error;
pub mod routes;
pub mod state;
pub mod telemetry;
