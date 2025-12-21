//! Authentication service.
//!
//! This module provides authentication functionality including password hashing,
//! JWT token generation/verification, and user management.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sqlx::{Pool, Sqlite};
use tracing::instrument;

use crate::error::{AppError, Result};
use crate::models::{JwtClaims, NewUser, UpdateUserRequest, User};
use crate::repository::user::UserRepository;

/// Configuration for the authentication service.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Secret key for JWT signing.
    pub jwt_secret: String,
    /// JWT token expiration time in hours.
    pub jwt_expiration_hours: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-change-in-production".to_string(),
            jwt_expiration_hours: 24,
        }
    }
}

/// Password hashing utilities using Argon2.
pub struct PasswordHashService;

impl PasswordHashService {
    /// Hash a password using Argon2id.
    ///
    /// Returns the hashed password as a PHC string format.
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))
    }

    /// Verify a password against a stored hash.
    ///
    /// Returns true if the password matches, false otherwise.
    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| AppError::Internal(format!("Invalid password hash format: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

/// JWT token utilities.
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: i64,
}

impl JwtService {
    /// Create a new JWT service with the given secret.
    pub fn new(secret: &str, expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiration_hours,
        }
    }

    /// Generate a JWT token for a user.
    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);

        let claims = JwtClaims {
            sub: user_id,
            username: username.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))
    }

    /// Verify and decode a JWT token.
    ///
    /// Returns the claims if the token is valid, or an error if invalid/expired.
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims> {
        let validation = Validation::default();

        decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    }
}

/// Authentication service for user management.
pub struct AuthService {
    pool: Pool<Sqlite>,
    jwt_service: JwtService,
}

impl AuthService {
    /// Create a new authentication service.
    pub fn new(pool: Pool<Sqlite>, config: AuthConfig) -> Self {
        Self {
            pool,
            jwt_service: JwtService::new(&config.jwt_secret, config.jwt_expiration_hours),
        }
    }

    /// Register a new user.
    ///
    /// Returns the created user on success.
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn register(&self, username: String, password: String) -> Result<User> {
        // Validate input
        if username.trim().is_empty() {
            return Err(AppError::BadRequest("Username cannot be empty".to_string()));
        }
        if password.len() < 6 {
            return Err(AppError::BadRequest(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        // Hash the password
        let password_hash = PasswordHashService::hash_password(&password)?;

        // Create the user
        let new_user = NewUser {
            username,
            password_hash,
            bangumi_api_key: None,
        };

        UserRepository::create(&self.pool, new_user).await
    }

    /// Login a user with username and password.
    ///
    /// Returns the user and a JWT token on success.
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn login(&self, username: String, password: String) -> Result<(User, String)> {
        // Find the user
        let user = UserRepository::find_by_username(&self.pool, &username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

        // Verify the password
        let is_valid = PasswordHashService::verify_password(&password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        // Generate JWT token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        Ok((user, token))
    }

    /// Verify a JWT token and return the claims.
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims> {
        self.jwt_service.verify_token(token)
    }

    /// Update user information.
    pub async fn update_user(&self, user_id: i64, req: UpdateUserRequest) -> Result<User> {
        // Get the current user
        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

        // Validate Password if changing (requires old_password)
        let password_hash = if let Some(new_password) = &req.password {
            // Check if old_password is provided
            let old_password = req.old_password.as_ref().ok_or_else(|| {
                AppError::BadRequest("Current password is required to change password".to_string())
            })?;

            // Verify old password
            let is_valid = PasswordHashService::verify_password(old_password, &user.password_hash)?;
            if !is_valid {
                return Err(AppError::Unauthorized(
                    "Current password is incorrect".to_string(),
                ));
            }

            // Hash new password
            Some(PasswordHashService::hash_password(new_password)?)
        } else {
            None
        };

        // Validate Username if changing
        let username = if let Some(new_username) = &req.username {
            if new_username.trim().is_empty() {
                return Err(AppError::BadRequest("Username cannot be empty".to_string()));
            }
            // Check if different from current
            if new_username != &user.username {
                // Uniqueness check is handled by DB constraint and repository error mapping
                Some(new_username.clone())
            } else {
                None
            }
        } else {
            None
        };

        let bangumi_api_key_update = if let Some(key) = &req.bangumi_api_key {
            if key.is_empty() {
                Some(None) // Clear
            } else {
                Some(Some(key.clone())) // Set
            }
        } else {
            None // No change
        };

        UserRepository::update(
            &self.pool,
            user_id,
            username,
            password_hash,
            bangumi_api_key_update,
        )
        .await
    }

    /// Get a user by ID.
    pub async fn get_user(&self, user_id: i64) -> Result<Option<User>> {
        UserRepository::find_by_id(&self.pool, user_id).await
    }
}

// Re-export for convenience
pub use crate::models::{LoginRequest, LoginResponse, RegisterRequest, UserResponse};
