rust_i18n::i18n!("locales");

pub fn set_lib_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

pub mod db;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod repository;
pub mod router;
pub mod services;
pub mod state;
/// Utils module.
pub mod utils;

