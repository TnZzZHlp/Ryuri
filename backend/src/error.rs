//! Error handling infrastructure for the comic reader application.
//!
//! This module provides a unified error type that can be converted into HTTP responses.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Unified error type for the application.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Error response body structure.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Error detail containing code and message.
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: u16,
    pub message: String,
}

impl AppError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::FileSystem(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Archive(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the error message for the response.
    pub fn error_message(&self) -> String {
        match self {
            AppError::NotFound(msg) => msg.clone(),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Unauthorized(msg) => msg.clone(),
            AppError::Database(_) => "Database error".to_string(),
            AppError::FileSystem(_) => "File system error".to_string(),
            AppError::Archive(msg) => msg.clone(),
            AppError::Internal(msg) => msg.clone(),
        }
    }

    /// Converts the error into an ErrorResponse structure.
    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.status_code().as_u16(),
                message: self.error_message(),
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.to_error_response());
        (status, body).into_response()
    }
}

/// Result type alias using AppError.
pub type Result<T> = std::result::Result<T, AppError>;
