//! Authentication handlers.
//!
//! This module provides HTTP handlers for authentication endpoints:
//! - POST /api/auth/login - User login
//! - GET /api/auth/me - Get current user
//! - PUT /api/auth/me - Update current user
//! - PUT /api/auth/password - Update password

use axum::{Json, extract::State};

use crate::error::Result;
use crate::middlewares::auth::AuthUser;
use crate::models::{
    LoginRequest, LoginResponse, UpdateUserRequest, UserResponse,
};
use crate::state::AppState;

/// POST /api/auth/login
///
/// Authenticates a user with username and password.
/// Returns the user information and a JWT token on success.
pub async fn login(
    State(state): State<AppState>,
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
pub async fn get_me(
    State(state): State<AppState>,
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
pub async fn update_me(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    let user = state
        .auth_service
        .update_user(auth_user.user_id, req)
        .await?;
    Ok(Json(UserResponse::from(user)))
}
