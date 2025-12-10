pub mod db;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod router;
pub mod services;
pub mod state;

/// OpenAPI documentation module (only available with `dev` feature).
///
/// Requirements: 1.1
#[cfg(feature = "dev")]
pub mod openapi;
