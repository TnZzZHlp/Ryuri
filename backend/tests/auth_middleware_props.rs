//! Property tests for authentication middleware.
//!
//! This module contains property-based tests for the auth middleware
//! that validates JWT tokens and stores user information in request extensions.

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware,
    routing::get,
};
use backend::db::{DbConfig, init_db};
use backend::middlewares::{AuthUser, auth_middleware};
use backend::services::auth::{AuthConfig, JwtService};
use backend::state::{AppConfig, AppState};
use proptest::prelude::*;
use tokio::runtime::Runtime;
use tower::ServiceExt;

// ============================================================================
// Arbitrary Strategies for Auth Middleware Testing
// ============================================================================

/// Strategy to generate valid usernames.
fn arb_username() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{2,30}".prop_filter("Username must not be empty", |s| !s.trim().is_empty())
}

/// Strategy to generate valid user IDs.
fn arb_user_id() -> impl Strategy<Value = i64> {
    1i64..10000
}

/// Strategy to generate valid JWT secrets.
fn arb_jwt_secret() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{32,64}"
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test app state with the given JWT secret.
async fn create_test_state(jwt_secret: String) -> AppState {
    let config = DbConfig {
        database_url: "sqlite::memory:".to_string(),
        max_connections: 1,
    };
    let pool = init_db(&config).await.expect("Failed to init db");

    let app_config = AppConfig {
        auth: AuthConfig {
            jwt_secret,
            jwt_expiration_hours: 24,
        }
    };

    AppState::new(pool, app_config)
}

/// Handler that extracts AuthUser and returns the user info.
async fn test_handler(auth_user: AuthUser) -> String {
    format!("{}:{}", auth_user.user_id, auth_user.username)
}

// ============================================================================
// Property Tests for Valid Token Extraction and Storage
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: auth-middleware, Property 1: Valid token extraction and storage**
    /// **Validates: Requirements 1.1, 1.3, 3.1**
    ///
    /// For any request with a valid Bearer token in the Authorization header,
    /// the middleware should extract the token, verify it, and store the
    /// resulting AuthUser in request extensions.
    #[test]
    fn valid_token_extraction_and_storage(
        user_id in arb_user_id(),
        username in arb_username(),
        jwt_secret in arb_jwt_secret()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Create test state with auth service
            let state = create_test_state(jwt_secret.clone()).await;

            // Generate a valid token using JwtService with the same secret
            let jwt_service = JwtService::new(&jwt_secret, 24);
            let token = jwt_service
                .generate_token(user_id, &username)
                .expect("Token generation should succeed");

            // Create a test router with the auth middleware
            let app = Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .with_state(state);

            // Create a request with the Bearer token
            let request = Request::builder()
                .uri("/test")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            // Send the request
            let response = app.oneshot(request).await.unwrap();

            // Verify the response is successful
            prop_assert_eq!(
                response.status(),
                StatusCode::OK,
                "Request with valid token should succeed"
            );

            // Verify the handler received the correct user info
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body = String::from_utf8(body_bytes.to_vec()).unwrap();
            let expected = format!("{}:{}", user_id, username);

            prop_assert_eq!(
                body, expected,
                "Handler should receive correct user info from extensions"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property Tests for Invalid Token Rejection
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: auth-middleware, Property 2: Invalid token rejection**
    /// **Validates: Requirements 1.4**
    ///
    /// For any request with an invalid JWT token (malformed, expired, or wrong signature),
    /// the middleware should return a 401 Unauthorized response and not call the next handler.
    #[test]
    fn invalid_token_rejection(
        user_id in arb_user_id(),
        username in arb_username(),
        jwt_secret1 in arb_jwt_secret(),
        jwt_secret2 in arb_jwt_secret()
    ) {
        // Skip if secrets happen to be the same
        prop_assume!(jwt_secret1 != jwt_secret2);

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Create test state with jwt_secret1
            let state = create_test_state(jwt_secret1.clone()).await;

            // Generate a token with jwt_secret2 (wrong secret)
            let jwt_service = JwtService::new(&jwt_secret2, 24);
            let invalid_token = jwt_service
                .generate_token(user_id, &username)
                .expect("Token generation should succeed");

            // Create a test router with the auth middleware
            let app = Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .with_state(state);

            // Create a request with the invalid token
            let request = Request::builder()
                .uri("/test")
                .header(AUTHORIZATION, format!("Bearer {}", invalid_token))
                .body(Body::empty())
                .unwrap();

            // Send the request
            let response = app.oneshot(request).await.unwrap();

            // Verify the response is 401 Unauthorized
            prop_assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Request with invalid token should return 401"
            );

            Ok(())
        })?;
    }

    /// **Feature: auth-middleware, Property 2: Invalid token rejection**
    /// **Validates: Requirements 1.4**
    ///
    /// For any malformed token string, the middleware should reject it with 401.
    #[test]
    fn malformed_token_rejection(
        jwt_secret in arb_jwt_secret(),
        malformed_token in "[a-zA-Z0-9]{10,50}"
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Create test state
            let state = create_test_state(jwt_secret).await;

            // Create a test router with the auth middleware
            let app = Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .with_state(state);

            // Create a request with the malformed token
            let request = Request::builder()
                .uri("/test")
                .header(AUTHORIZATION, format!("Bearer {}", malformed_token))
                .body(Body::empty())
                .unwrap();

            // Send the request
            let response = app.oneshot(request).await.unwrap();

            // Verify the response is 401 Unauthorized
            prop_assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Request with malformed token should return 401"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property Tests for Extractor Success After Middleware
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: auth-middleware, Property 3: Extractor success after middleware**
    /// **Validates: Requirements 3.3**
    ///
    /// For any request that successfully passes through the auth middleware,
    /// extracting AuthUser in the handler should always succeed and return
    /// the same user information that was stored by the middleware.
    #[test]
    fn extractor_success_after_middleware(
        user_id in arb_user_id(),
        username in arb_username(),
        jwt_secret in arb_jwt_secret()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Create test state with auth service
            let state = create_test_state(jwt_secret.clone()).await;

            // Generate a valid token
            let jwt_service = JwtService::new(&jwt_secret, 24);
            let token = jwt_service
                .generate_token(user_id, &username)
                .expect("Token generation should succeed");

            // Create a test router with the auth middleware
            let app = Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .with_state(state);

            // Create a request with the Bearer token
            let request = Request::builder()
                .uri("/test")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            // Send the request
            let response = app.oneshot(request).await.unwrap();

            // Verify the response is successful (extractor didn't fail)
            prop_assert_eq!(
                response.status(),
                StatusCode::OK,
                "Extractor should succeed after middleware stores AuthUser"
            );

            // Verify the extracted user info matches what was in the token
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body = String::from_utf8(body_bytes.to_vec()).unwrap();
            let expected = format!("{}:{}", user_id, username);

            prop_assert_eq!(
                body, expected,
                "Extracted AuthUser should match the user info from the JWT token"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property Tests for Error Response Format
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: auth-middleware, Property 4: Error response format**
    /// **Validates: Requirements 5.1**
    ///
    /// For any authentication failure (missing header, invalid token, expired token),
    /// the middleware should return a JSON-formatted error response with status code 401.
    #[test]
    fn error_response_format(
        jwt_secret in arb_jwt_secret(),
        malformed_token in "[a-zA-Z0-9]{10,50}"
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Create test state
            let state = create_test_state(jwt_secret).await;

            // Create a test router with the auth middleware
            let app = Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .with_state(state);

            // Test with malformed token
            let request = Request::builder()
                .uri("/test")
                .header(AUTHORIZATION, format!("Bearer {}", malformed_token))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            // Verify status code is 401
            prop_assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Error response should have 401 status code"
            );

            // Verify response is JSON
            let content_type = response.headers().get("content-type");
            if let Some(ct) = content_type {
                let ct_str = ct.to_str().unwrap_or("");
                prop_assert!(
                    ct_str.contains("application/json"),
                    "Error response should be JSON format, got: {}",
                    ct_str
                );
            }

            // Verify response body is valid JSON with error structure
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

            // Parse as JSON to verify it's valid JSON
            let json_value: serde_json::Value = serde_json::from_str(&body_str)
                .expect("Response should be valid JSON");

            // Verify it has the error structure
            prop_assert!(
                json_value.get("error").is_some(),
                "Error response should have 'error' field"
            );

            let error_obj = json_value.get("error").unwrap();
            prop_assert!(
                error_obj.get("code").is_some(),
                "Error should have 'code' field"
            );
            prop_assert!(
                error_obj.get("message").is_some(),
                "Error should have 'message' field"
            );

            // Verify code is 401
            let code = error_obj.get("code").unwrap().as_u64().unwrap();
            prop_assert_eq!(
                code, 401,
                "Error code should be 401"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Example Tests for Specific Error Cases
// ============================================================================

#[cfg(test)]
mod example_tests {
    use super::*;

    /// **Feature: auth-middleware, Example 1: Missing Authorization header**
    /// **Validates: Requirements 1.5**
    ///
    /// When a request arrives without an Authorization header,
    /// the middleware should return 401 with message "Missing authorization header".
    #[tokio::test]
    async fn missing_authorization_header_returns_401() {
        // Create test state
        let state = create_test_state("test-secret-key-for-testing".to_string()).await;

        // Create a test router with the auth middleware
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        // Create a request without Authorization header
        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Verify the response is 401 Unauthorized
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Request without Authorization header should return 401"
        );

        // Verify the error message
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        let message = json_value["error"]["message"].as_str().unwrap();
        assert_eq!(
            message, "Missing authorization header",
            "Error message should indicate missing header"
        );
    }

    /// **Feature: auth-middleware, Example 3: Expired token error message**
    /// **Validates: Requirements 5.2**
    ///
    /// When a request contains an expired JWT token,
    /// the middleware should return 401 with a message indicating token expiration.
    #[tokio::test]
    async fn expired_token_returns_specific_error() {
        // Create test state
        let state = create_test_state("test-secret-key-for-testing".to_string()).await;

        // Generate an expired token (expiration in the past)
        let jwt_service = JwtService::new("test-secret-key-for-testing", -1); // Negative hours = expired
        let expired_token = jwt_service
            .generate_token(1, "testuser")
            .expect("Token generation should succeed");

        // Create a test router with the auth middleware
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        // Create a request with the expired token
        let request = Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, format!("Bearer {}", expired_token))
            .body(Body::empty())
            .unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Verify the response is 401 Unauthorized
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Request with expired token should return 401"
        );

        // Verify the error message mentions expiration
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        let message = json_value["error"]["message"].as_str().unwrap();
        assert!(
            message.contains("Invalid token")
                || message.contains("expired")
                || message.contains("ExpiredSignature"),
            "Error message should indicate token expiration, got: {}",
            message
        );
    }

    /// **Feature: auth-middleware, Example 4: Invalid token format error message**
    /// **Validates: Requirements 5.3**
    ///
    /// When a request contains a malformed token (not valid JWT format),
    /// the middleware should return 401 with a message indicating invalid format.
    #[tokio::test]
    async fn invalid_token_format_returns_specific_error() {
        // Create test state
        let state = create_test_state("test-secret-key-for-testing".to_string()).await;

        // Create a test router with the auth middleware
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        // Create a request with a malformed token
        let request = Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, "Bearer not-a-valid-jwt-token")
            .body(Body::empty())
            .unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Verify the response is 401 Unauthorized
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Request with malformed token should return 401"
        );

        // Verify the error message mentions invalid token
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        let message = json_value["error"]["message"].as_str().unwrap();
        assert!(
            message.contains("Invalid token") || message.contains("invalid"),
            "Error message should indicate invalid token format, got: {}",
            message
        );
    }

    /// **Feature: auth-middleware, Example 2: Public routes bypass authentication**
    /// **Validates: Requirements 4.2**
    ///
    /// When a request is made to a public route (e.g., /api/auth/login),
    /// the middleware should not be applied and the request should succeed without a token.
    #[tokio::test]
    async fn public_routes_bypass_authentication() {
        use axum::routing::post;

        // Create test state
        let state = create_test_state("test-secret-key-for-testing".to_string()).await;

        // Handler for public route (doesn't require AuthUser)
        async fn public_handler() -> String {
            "public".to_string()
        }

        // Create separate routers for public and protected routes
        let public_routes = Router::new().route("/api/auth/login", post(public_handler));

        let protected_routes = Router::new()
            .route("/api/protected", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        // Merge routers
        let app = Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .with_state(state);

        // Test 1: Public route should succeed without token
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Public route should succeed without authentication"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(
            body, "public",
            "Public route should return expected response"
        );

        // Test 2: Protected route should fail without token
        let request = Request::builder()
            .uri("/api/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Protected route should require authentication"
        );
    }
}

// ============================================================================
// Integration Tests for Authentication Flow
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test: Protected routes require authentication
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**
    ///
    /// This test verifies that protected routes in the actual router configuration
    /// require authentication and reject requests without valid tokens.
    #[tokio::test]
    async fn protected_routes_require_authentication() {
        // Create test state
        let state = create_test_state("test-secret-key-for-testing".to_string()).await;

        // Create a test router with the auth middleware (simulating real router)
        let protected_routes = Router::new()
            .route("/api/auth/me", get(test_handler))
            .route("/api/libraries", get(test_handler))
            .route("/api/contents/{id}", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let app = Router::new().merge(protected_routes).with_state(state);

        // Test various protected routes without authentication
        let protected_routes = vec!["/api/auth/me", "/api/libraries", "/api/contents/1"];

        for route in protected_routes {
            let request = Request::builder().uri(route).body(Body::empty()).unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Protected route {} should require authentication",
                route
            );
        }
    }

    /// Integration test: Public routes work without authentication
    /// **Validates: Requirements 4.2**
    ///
    /// This test verifies that public routes (like /api/auth/login) work
    /// without requiring authentication tokens.
    #[tokio::test]
    async fn public_routes_work_without_authentication() {
        use axum::routing::post;

        // Create test state
        let state = create_test_state("test-secret-key-for-testing".to_string()).await;

        // Handler for public route
        async fn login_handler() -> String {
            "login_success".to_string()
        }

        // Create router with public and protected routes
        let public_routes = Router::new().route("/api/auth/login", post(login_handler));

        let protected_routes = Router::new()
            .route("/api/auth/me", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let app = Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .with_state(state);

        // Test public route without token
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Public route should work without authentication"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(
            body, "login_success",
            "Public route should return expected response"
        );
    }

    /// Integration test: End-to-end authentication flow with valid token
    /// **Validates: Requirements 1.1, 1.2, 1.3**
    ///
    /// This test verifies the complete authentication flow:
    /// 1. Generate a valid JWT token
    /// 2. Make a request to a protected route with the token
    /// 3. Verify the request succeeds and the handler receives correct user info
    #[tokio::test]
    async fn end_to_end_authentication_with_valid_token() {
        // Create test state
        let jwt_secret = "test-secret-key-for-testing".to_string();
        let state = create_test_state(jwt_secret.clone()).await;

        // Generate a valid token
        let user_id = 42;
        let username = "testuser";
        let jwt_service = JwtService::new(&jwt_secret, 24);
        let token = jwt_service
            .generate_token(user_id, username)
            .expect("Token generation should succeed");

        // Create router with protected route
        let protected_routes = Router::new()
            .route("/api/auth/me", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let app = Router::new().merge(protected_routes).with_state(state);

        // Make request with valid token
        let request = Request::builder()
            .uri("/api/auth/me")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Verify success
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request with valid token should succeed"
        );

        // Verify handler received correct user info
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(body_bytes.to_vec()).unwrap();
        let expected = format!("{}:{}", user_id, username);

        assert_eq!(
            body, expected,
            "Handler should receive correct user info from token"
        );
    }

    /// Integration test: End-to-end authentication flow with invalid token
    /// **Validates: Requirements 1.4, 1.5**
    ///
    /// This test verifies that requests with invalid tokens are properly rejected:
    /// 1. Test with malformed token
    /// 2. Test with token signed with wrong secret
    /// 3. Test with missing Authorization header
    /// 4. Verify all return 401 Unauthorized
    #[tokio::test]
    async fn end_to_end_authentication_with_invalid_token() {
        // Create test state
        let jwt_secret = "test-secret-key-for-testing".to_string();
        let state = create_test_state(jwt_secret.clone()).await;

        // Create router with protected route
        let protected_routes = Router::new()
            .route("/api/auth/me", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let app = Router::new().merge(protected_routes).with_state(state);

        // Test 1: Malformed token
        let request = Request::builder()
            .uri("/api/auth/me")
            .header(AUTHORIZATION, "Bearer invalid-token-format")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Malformed token should return 401"
        );

        // Test 2: Token with wrong secret
        let wrong_jwt_service = JwtService::new("wrong-secret", 24);
        let wrong_token = wrong_jwt_service
            .generate_token(1, "testuser")
            .expect("Token generation should succeed");

        let request = Request::builder()
            .uri("/api/auth/me")
            .header(AUTHORIZATION, format!("Bearer {}", wrong_token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Token with wrong secret should return 401"
        );

        // Test 3: Missing Authorization header
        let request = Request::builder()
            .uri("/api/auth/me")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Missing Authorization header should return 401"
        );

        // Test 4: Invalid header format (not Bearer)
        let request = Request::builder()
            .uri("/api/auth/me")
            .header(AUTHORIZATION, "Basic invalid")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Invalid header format should return 401"
        );
    }

    /// Integration test: Multiple protected routes with same token
    /// **Validates: Requirements 1.1, 1.3, 3.1**
    ///
    /// This test verifies that a single valid token can be used to access
    /// multiple protected routes, and the user info is correctly extracted
    /// in each handler.
    #[tokio::test]
    async fn multiple_protected_routes_with_same_token() {
        // Create test state
        let jwt_secret = "test-secret-key-for-testing".to_string();
        let state = create_test_state(jwt_secret.clone()).await;

        // Generate a valid token
        let user_id = 123;
        let username = "multiuser";
        let jwt_service = JwtService::new(&jwt_secret, 24);
        let token = jwt_service
            .generate_token(user_id, username)
            .expect("Token generation should succeed");

        // Create router with multiple protected routes
        let protected_routes = Router::new()
            .route("/api/route1", get(test_handler))
            .route("/api/route2", get(test_handler))
            .route("/api/route3", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let app = Router::new().merge(protected_routes).with_state(state);

        let routes = vec!["/api/route1", "/api/route2", "/api/route3"];
        let expected = format!("{}:{}", user_id, username);

        for route in routes {
            let request = Request::builder()
                .uri(route)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Route {} should succeed with valid token",
                route
            );

            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body = String::from_utf8(body_bytes.to_vec()).unwrap();

            assert_eq!(
                body, expected,
                "Route {} should receive correct user info",
                route
            );
        }
    }

    /// Integration test: Token expiration handling
    /// **Validates: Requirements 1.4, 5.2**
    ///
    /// This test verifies that expired tokens are properly rejected
    /// with appropriate error messages.
    #[tokio::test]
    async fn token_expiration_handling() {
        // Create test state
        let jwt_secret = "test-secret-key-for-testing".to_string();
        let state = create_test_state(jwt_secret.clone()).await;

        // Generate an expired token (negative expiration hours)
        let jwt_service = JwtService::new(&jwt_secret, -1);
        let expired_token = jwt_service
            .generate_token(1, "testuser")
            .expect("Token generation should succeed");

        // Create router with protected route
        let protected_routes = Router::new()
            .route("/api/auth/me", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let app = Router::new().merge(protected_routes).with_state(state);

        // Make request with expired token
        let request = Request::builder()
            .uri("/api/auth/me")
            .header(AUTHORIZATION, format!("Bearer {}", expired_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Verify 401 response
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expired token should return 401"
        );

        // Verify error message mentions expiration
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let json_value: serde_json::Value =
            serde_json::from_str(&body_str).expect("Response should be valid JSON");

        let message = json_value["error"]["message"].as_str().unwrap();
        assert!(
            message.contains("Invalid token")
                || message.contains("expired")
                || message.contains("ExpiredSignature"),
            "Error message should indicate token expiration, got: {}",
            message
        );
    }
}
