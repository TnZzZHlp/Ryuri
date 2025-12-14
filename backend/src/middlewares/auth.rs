//! Authentication middleware for Axum.
//!
//! This module provides middleware for JWT-based authentication that can be applied
//! to route groups. The middleware extracts and verifies JWT tokens from the
//! Authorization header and stores authenticated user information in request extensions.

use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};

use crate::error::AppError;
use crate::models::JwtClaims;
use crate::state::AppState;

/// Authenticated user information extracted from JWT token.
///
/// This struct is stored in request extensions by the auth middleware
/// and can be extracted in handlers using the FromRequestParts trait.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
}

impl From<JwtClaims> for AuthUser {
    fn from(claims: JwtClaims) -> Self {
        Self {
            user_id: claims.sub,
            username: claims.username,
        }
    }
}

/// Authentication middleware function.
///
/// This middleware:
/// 1. Extracts the JWT token from the Authorization header
/// 2. Verifies the token using AuthService
/// 3. Stores the authenticated user in request extensions
/// 4. Returns 401 Unauthorized on authentication failures
///
/// # Requirements
/// - 1.1: Extract JWT token from Authorization header
/// - 1.2: Verify token using AuthService
/// - 1.3: Store AuthUser in request extensions on success
/// - 1.4: Return 401 error on token verification failure
/// - 1.5: Return 401 error on missing Authorization header
/// - 5.1: Return JSON error response with appropriate status code
/// - 5.2: Return specific error message for expired tokens
/// - 5.3: Return specific error message for invalid token format
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract the Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Authentication failed: Missing authorization header");
            AppError::Unauthorized("Missing authorization header".to_string())
        })?;

    // Check for Bearer token format
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        tracing::warn!("Authentication failed: Invalid authorization header format");
        AppError::Unauthorized("Invalid authorization header format".to_string())
    })?;

    // Verify the token using AuthService
    let claims = state.auth_service.verify_token(token).map_err(|e| {
        tracing::warn!("Authentication failed: {:?}", e);
        e
    })?;

    // Convert claims to AuthUser and store in request extensions
    let auth_user = AuthUser::from(claims);
    req.extensions_mut().insert(auth_user);

    // Continue to the next middleware or handler
    Ok(next.run(req).await)
}

/// Extractor for authenticated user from request extensions.
///
/// This extractor retrieves the AuthUser that was stored by the auth middleware.
/// It should only be used in handlers that are protected by the auth middleware.
///
/// # Requirements
/// - 3.1: Store AuthUser in request extensions
/// - 3.2: Provide extractor to retrieve AuthUser from extensions
/// - 3.4: Not require access to AppState or AuthService
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract AuthUser from request extensions
        parts.extensions.get::<AuthUser>().cloned().ok_or_else(|| {
            AppError::Unauthorized(
                "Missing authentication. This route requires authentication.".to_string(),
            )
        })
    }
}
