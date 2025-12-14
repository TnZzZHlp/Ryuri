//! Middleware modules for the application.

pub mod auth;

// Re-export commonly used items
pub use auth::{AuthUser, auth_middleware};
