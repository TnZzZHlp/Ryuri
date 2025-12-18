//! HTTP request handlers for the API endpoints.
//!
//! This module contains all the Axum handlers that process incoming HTTP requests
//! and return appropriate responses.

pub mod auth;
pub mod apikey;
pub mod content;
pub mod filesystem;
pub mod komga;
pub mod library;
pub mod progress;
pub mod scan_queue;
pub mod static_files;
