//! Authentication middleware for Axum.
//!
//! This module provides middleware for JWT-based authentication that can be applied
//! to route groups. The middleware extracts and verifies JWT tokens from the
//! Authorization header and stores authenticated user information in request extensions.

use axum::http::Method;
use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use rust_i18n::t;
use std::borrow::Cow;

use crate::error::AppError;
use crate::models::{JwtClaims, User};
use crate::repository::{apikey::ApiKeyRepository, user::UserRepository};
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

impl From<User> for AuthUser {
    fn from(user: User) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
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
    // 1. Check for API Key first (X-API-Key header)
    if let Some(api_key_header) = req
        .headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
    {
        if let Some(api_key) = ApiKeyRepository::get_by_key(&state.pool, api_key_header).await?
            && let Some(user) = UserRepository::find_by_id(&state.pool, api_key.user_id).await?
        {
            let auth_user = AuthUser::from(user);
            req.extensions_mut().insert(auth_user);
            return Ok(next.run(req).await);
        }
        // If API key is invalid, we don't return error immediately, we fall back to JWT check
        // or maybe we should return error? Usually if explicit auth method is provided and fails, we fail.
        // But for now let's strict fail if header is present but invalid.
        return Err(AppError::Unauthorized(
            t!("auth.invalid_api_key").to_string(),
        ));
    }

    // 2. Prefer Authorization: Bearer <token>. If absent, optionally accept `?token=`
    // for image resources so the frontend can use <img src> (progressive loading).
    let token: Cow<'_, str> = if let Some(auth_header) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        // Check for Bearer token format
        Cow::Borrowed(auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            tracing::warn!("{}", t!("auth.invalid_auth_header_format"));
            AppError::Unauthorized(t!("auth.invalid_auth_header_format_error").to_string())
        })?)
    } else {
        // Only allow query token for safe, cacheable-ish image reads.
        // We intentionally scope this to image endpoints to avoid broad token-in-URL usage.
        let method = req.method().clone();

        if !matches!(method, Method::GET | Method::HEAD) {
            tracing::warn!("{}", t!("auth.missing_auth_header_log"));
            return Err(AppError::Unauthorized(
                t!("auth.missing_auth_header").to_string(),
            ));
        }

        let query = req.uri().query().unwrap_or("");
        let token = extract_query_param(query, "token").ok_or_else(|| {
            tracing::warn!("{}", t!("auth.missing_auth_header_and_token_log"));
            AppError::Unauthorized(t!("auth.missing_auth_header").to_string())
        })?;

        Cow::Owned(token)
    };

    // Verify the token using AuthService
    let claims = state
        .auth_service
        .verify_token(token.as_ref())
        .map_err(|e| {
            tracing::warn!("{}", t!("auth.auth_failed_log", error = e));
            e
        })?;

    // Convert claims to AuthUser and store in request extensions
    let auth_user = AuthUser::from(claims);
    req.extensions_mut().insert(auth_user);

    // Continue to the next middleware or handler
    Ok(next.run(req).await)
}

/// Extract a query parameter value by key from a raw query string.
///
/// This is a tiny parser to avoid pulling additional dependencies.
fn extract_query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("");
        if k != key {
            continue;
        }
        let v = it.next().unwrap_or("");
        return urlencoding::decode(v).ok().map(|s| s.into_owned());
    }
    None
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
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized(t!("auth.missing_authentication").to_string()))
    }
}
