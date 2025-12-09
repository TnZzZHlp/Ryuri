//! User-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A user account.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub bangumi_api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// JWT claims for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: i64,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

/// Request to update user information.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub bangumi_api_key: Option<String>,
}
