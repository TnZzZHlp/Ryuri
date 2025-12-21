//! User-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A user account.
///
/// Users have their own reading progress and settings.
/// Passwords are stored as hashed values using argon2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    /// Unique identifier for the user.
    pub id: i64,
    /// Unique username for login.
    pub username: String,
    /// Hashed password (never serialized to JSON).
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// Optional Bangumi API key for metadata scraping.
    pub bangumi_api_key: Option<String>,
    /// Timestamp when the user was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user was last updated.
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a NewUser instance for database insertion.
    pub fn create(username: String, password_hash: String) -> NewUser {
        NewUser {
            username,
            password_hash,
            bangumi_api_key: None,
        }
    }
}

/// Data for creating a new user.
#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub bangumi_api_key: Option<String>,
}

/// JWT claims for authentication.
///
/// These claims are encoded in the JWT token and used
/// to identify the authenticated user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID).
    pub sub: i64,
    /// Username for display purposes.
    pub username: String,
    /// Expiration timestamp (Unix epoch seconds).
    pub exp: i64,
    /// Issued at timestamp (Unix epoch seconds).
    pub iat: i64,
}

/// Request to update user information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct UpdateUserRequest {
    /// New username (optional).
    pub username: Option<String>,
    /// New password (optional).
    pub password: Option<String>,
    /// Old password (required if changing password).
    pub old_password: Option<String>,
    /// New Bangumi API key (optional).
    pub bangumi_api_key: Option<String>,
}

/// Request for user login.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Request for user registration.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Response for successful login.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
}

/// User data for API responses (without sensitive fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub bangumi_api_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            bangumi_api_key: user.bangumi_api_key,
            created_at: user.created_at,
        }
    }
}
