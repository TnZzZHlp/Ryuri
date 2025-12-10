//! Authentication handlers.
//!
//! This module provides HTTP handlers for authentication endpoints:
//! - POST /api/auth/login - User login
//! - GET /api/auth/me - Get current user
//! - PUT /api/auth/me - Update current user
//! - PUT /api/auth/password - Update password

use axum::{Json, extract::State};
use std::sync::Arc;

use crate::error::Result;
use crate::models::{
    LoginRequest, LoginResponse, UpdatePasswordRequest, UpdateUserRequest, UserResponse,
};
use crate::services::auth::{AuthService, middleware::AuthUser};

/// Application state containing the auth service.
/// This will be replaced with the actual AppState when the router is wired up.
#[derive(Clone)]
pub struct AuthState {
    pub auth_service: Arc<AuthService>,
}

impl crate::services::auth::middleware::HasAuthService for AuthState {
    fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }
}

/// POST /api/auth/login
///
/// Authenticates a user with username and password.
/// Returns the user information and a JWT token on success.
///
/// # Request Body
/// ```json
/// {
///     "username": "string",
///     "password": "string"
/// }
/// ```
///
/// # Response
/// ```json
/// {
///     "user": {
///         "id": 1,
///         "username": "string",
///         "bangumi_api_key": null,
///         "created_at": "2024-01-01T00:00:00Z"
///     },
///     "token": "jwt_token_string"
/// }
/// ```
pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let (user, token) = state.auth_service.login(req.username, req.password).await?;

    Ok(Json(LoginResponse {
        user: UserResponse::from(user),
        token,
    }))
}

/// GET /api/auth/me
///
/// Returns the currently authenticated user's information.
/// Requires a valid JWT token in the Authorization header.
///
/// # Response
/// ```json
/// {
///     "id": 1,
///     "username": "string",
///     "bangumi_api_key": null,
///     "created_at": "2024-01-01T00:00:00Z"
/// }
/// ```
pub async fn get_current_user(
    State(state): State<AuthState>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>> {
    let user = state
        .auth_service
        .get_user(auth_user.user_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

/// PUT /api/auth/me
///
/// Updates the currently authenticated user's information.
/// Currently supports updating the Bangumi API key.
///
/// # Request Body
/// ```json
/// {
///     "bangumi_api_key": "optional_string"
/// }
/// ```
///
/// # Response
/// Returns the updated user information.
pub async fn update_current_user(
    State(state): State<AuthState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    let user = state
        .auth_service
        .update_user(auth_user.user_id, req)
        .await?;

    Ok(Json(UserResponse::from(user)))
}

/// PUT /api/auth/password
///
/// Updates the currently authenticated user's password.
/// Requires the old password for verification.
///
/// # Request Body
/// ```json
/// {
///     "old_password": "string",
///     "new_password": "string"
/// }
/// ```
///
/// # Response
/// Returns 200 OK with empty body on success.
pub async fn update_password(
    State(state): State<AuthState>,
    auth_user: AuthUser,
    Json(req): Json<UpdatePasswordRequest>,
) -> Result<Json<()>> {
    state
        .auth_service
        .update_password(auth_user.user_id, req.old_password, req.new_password)
        .await?;

    Ok(Json(()))
}
