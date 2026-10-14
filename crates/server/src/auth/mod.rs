//! Аутентификация: пароли (argon2), access-токены (JWT),
//! refresh-токены (случайные строки в БД с ротацией).

mod extractor;
pub mod jwt;
mod password;
mod routes;
mod service;
mod sessions;

pub use extractor::AuthUser;
pub use routes::router;
