//! Property tests for authentication service.
//!
//! This module contains property-based tests for password hashing,
//! JWT token generation/verification, and user registration.

use backend::services::auth::{JwtService, PasswordHashService};
use proptest::prelude::*;

// ============================================================================
// Arbitrary Strategies for Auth Data
// ============================================================================

/// Strategy to generate valid passwords (at least 6 characters).
fn arb_password() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9!@#$%^&*()_+\\-=\\[\\]{};':\",./<>?]{6,50}"
        .prop_filter("Password must be at least 6 chars", |s| s.len() >= 6)
}

/// Strategy to generate valid usernames.
fn arb_username() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{2,30}".prop_filter("Username must not be empty", |s| !s.trim().is_empty())
}

/// Strategy to generate valid JWT secrets.
fn arb_jwt_secret() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{32,64}"
}

// ============================================================================
// Property Tests for Password Hashing
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 23: Password Hashing Security**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// For any user password, the stored password_hash should not equal
    /// the original password.
    #[test]
    fn password_hash_differs_from_original(password in arb_password()) {
        let hash = PasswordHashService::hash_password(&password)
            .expect("Password hashing should succeed");

        prop_assert_ne!(
            password, hash,
            "Password hash should not equal the original password"
        );
    }

    /// **Feature: comic-reader, Property 23: Password Hashing Security**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// For any user password, verifying the correct password against
    /// its hash should succeed.
    #[test]
    fn correct_password_verifies_successfully(password in arb_password()) {
        let hash = PasswordHashService::hash_password(&password)
            .expect("Password hashing should succeed");

        let is_valid = PasswordHashService::verify_password(&password, &hash)
            .expect("Password verification should not error");

        prop_assert!(
            is_valid,
            "Correct password should verify successfully"
        );
    }

    /// **Feature: comic-reader, Property 23: Password Hashing Security**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// For any two different passwords, verifying the wrong password
    /// against a hash should fail.
    #[test]
    fn wrong_password_fails_verification(
        password1 in arb_password(),
        password2 in arb_password()
    ) {
        // Skip if passwords happen to be the same
        prop_assume!(password1 != password2);

        let hash = PasswordHashService::hash_password(&password1)
            .expect("Password hashing should succeed");

        let is_valid = PasswordHashService::verify_password(&password2, &hash)
            .expect("Password verification should not error");

        prop_assert!(
            !is_valid,
            "Wrong password should not verify successfully"
        );
    }

    /// **Feature: comic-reader, Property 23: Password Hashing Security**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// Hashing the same password twice should produce different hashes
    /// (due to random salt).
    #[test]
    fn same_password_produces_different_hashes(password in arb_password()) {
        let hash1 = PasswordHashService::hash_password(&password)
            .expect("First password hashing should succeed");
        let hash2 = PasswordHashService::hash_password(&password)
            .expect("Second password hashing should succeed");

        prop_assert_ne!(
            hash1, hash2,
            "Same password should produce different hashes due to random salt"
        );
    }
}

// ============================================================================
// Property Tests for JWT Token Validity
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 24: JWT Token Validity**
    /// **Validates: Requirements 9.2, 9.4**
    ///
    /// For any valid login, the returned JWT token should be verifiable
    /// and contain the correct user information until expiration.
    #[test]
    fn jwt_token_round_trip(
        user_id in 1i64..10000,
        username in arb_username(),
        secret in arb_jwt_secret()
    ) {
        let jwt_service = JwtService::new(&secret, 24);

        // Generate a token
        let token = jwt_service.generate_token(user_id, &username)
            .expect("Token generation should succeed");

        // Verify the token
        let claims = jwt_service.verify_token(&token)
            .expect("Token verification should succeed");

        // Check that claims match
        prop_assert_eq!(
            claims.sub, user_id,
            "Token should contain correct user ID"
        );
        prop_assert_eq!(
            claims.username, username,
            "Token should contain correct username"
        );
    }

    /// **Feature: comic-reader, Property 24: JWT Token Validity**
    /// **Validates: Requirements 9.2, 9.4**
    ///
    /// A token generated with one secret should not verify with a different secret.
    #[test]
    fn jwt_token_wrong_secret_fails(
        user_id in 1i64..10000,
        username in arb_username(),
        secret1 in arb_jwt_secret(),
        secret2 in arb_jwt_secret()
    ) {
        // Skip if secrets happen to be the same
        prop_assume!(secret1 != secret2);

        let jwt_service1 = JwtService::new(&secret1, 24);
        let jwt_service2 = JwtService::new(&secret2, 24);

        // Generate a token with secret1
        let token = jwt_service1.generate_token(user_id, &username)
            .expect("Token generation should succeed");

        // Try to verify with secret2 - should fail
        let result = jwt_service2.verify_token(&token);

        prop_assert!(
            result.is_err(),
            "Token should not verify with wrong secret"
        );
    }

    /// **Feature: comic-reader, Property 24: JWT Token Validity**
    /// **Validates: Requirements 9.2, 9.4**
    ///
    /// Token expiration time should be set correctly based on configuration.
    #[test]
    fn jwt_token_expiration_is_set(
        user_id in 1i64..10000,
        username in arb_username(),
        secret in arb_jwt_secret(),
        expiration_hours in 1i64..168  // 1 hour to 1 week
    ) {
        let jwt_service = JwtService::new(&secret, expiration_hours);

        let token = jwt_service.generate_token(user_id, &username)
            .expect("Token generation should succeed");

        let claims = jwt_service.verify_token(&token)
            .expect("Token verification should succeed");

        // Check that expiration is in the future
        let now = chrono::Utc::now().timestamp();
        prop_assert!(
            claims.exp > now,
            "Token expiration should be in the future"
        );

        // Check that expiration is approximately correct (within 1 minute tolerance)
        let expected_exp = now + (expiration_hours * 3600);
        let diff = (claims.exp - expected_exp).abs();
        prop_assert!(
            diff < 60,
            "Token expiration should be approximately {} hours from now (diff: {} seconds)",
            expiration_hours,
            diff
        );
    }

    /// **Feature: comic-reader, Property 24: JWT Token Validity**
    /// **Validates: Requirements 9.2, 9.4**
    ///
    /// Token issued-at time should be set to approximately now.
    #[test]
    fn jwt_token_issued_at_is_set(
        user_id in 1i64..10000,
        username in arb_username(),
        secret in arb_jwt_secret()
    ) {
        let jwt_service = JwtService::new(&secret, 24);

        let before = chrono::Utc::now().timestamp();
        let token = jwt_service.generate_token(user_id, &username)
            .expect("Token generation should succeed");
        let after = chrono::Utc::now().timestamp();

        let claims = jwt_service.verify_token(&token)
            .expect("Token verification should succeed");

        // Check that issued-at is between before and after
        prop_assert!(
            claims.iat >= before && claims.iat <= after,
            "Token issued-at should be approximately now"
        );
    }
}

/// **Feature: comic-reader, Property 24: JWT Token Validity**
/// **Validates: Requirements 9.2, 9.4**
///
/// Malformed tokens should fail verification.
#[test]
fn malformed_token_fails_verification() {
    let jwt_service = JwtService::new("test-secret-key-for-testing", 24);

    // Test various malformed tokens
    let malformed_tokens = vec![
        "",
        "not-a-token",
        "header.payload",
        "header.payload.signature.extra",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
    ];

    for token in malformed_tokens {
        let result = jwt_service.verify_token(token);
        assert!(
            result.is_err(),
            "Malformed token '{}' should fail verification",
            token
        );
    }
}

// ============================================================================
// Property Tests for User Registration Uniqueness
// ============================================================================

/// **Feature: comic-reader, Property 22: User Registration Uniqueness**
/// **Validates: Requirements 9.1**
///
/// These tests require database access and are run as async tests.
#[cfg(test)]
mod registration_tests {
    use backend::db::{DbConfig, init_db};
    use backend::services::auth::{AuthConfig, AuthService};
    use proptest::prelude::*;
    use tokio::runtime::Runtime;

    /// Strategy to generate valid usernames.
    fn arb_username() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_]{2,20}"
            .prop_filter("Username must not be empty", |s| !s.trim().is_empty())
    }

    /// Strategy to generate valid passwords.
    fn arb_password() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9!@#$%^&*]{6,30}"
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Feature: comic-reader, Property 22: User Registration Uniqueness**
        /// **Validates: Requirements 9.1**
        ///
        /// For any username, attempting to register with an already existing
        /// username should fail with an appropriate error.
        #[test]
        fn duplicate_username_registration_fails(
            username in arb_username(),
            password1 in arb_password(),
            password2 in arb_password()
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Create an in-memory database for testing
                let config = DbConfig {
                    database_url: "sqlite::memory:".to_string(),
                    max_connections: 1,
                };
                let pool = init_db(&config).await.expect("Failed to init db");

                let auth_config = AuthConfig::default();
                let auth_service = AuthService::new(pool, auth_config);

                // First registration should succeed
                let result1 = auth_service.register(username.clone(), password1).await;
                prop_assert!(
                    result1.is_ok(),
                    "First registration should succeed: {:?}",
                    result1.err()
                );

                // Second registration with same username should fail
                let result2 = auth_service.register(username.clone(), password2).await;
                prop_assert!(
                    result2.is_err(),
                    "Second registration with same username should fail"
                );

                // Verify the error message mentions the username
                if let Err(e) = result2 {
                    let error_msg = e.to_string();
                    prop_assert!(
                        error_msg.contains("already exists") || error_msg.contains(&username),
                        "Error should mention username conflict: {}",
                        error_msg
                    );
                }

                Ok(())
            })?;
        }

        /// **Feature: comic-reader, Property 22: User Registration Uniqueness**
        /// **Validates: Requirements 9.1**
        ///
        /// Different usernames should be able to register independently.
        #[test]
        fn different_usernames_can_register(
            username1 in arb_username(),
            username2 in arb_username(),
            password in arb_password()
        ) {
            // Skip if usernames happen to be the same
            prop_assume!(username1 != username2);

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let config = DbConfig {
                    database_url: "sqlite::memory:".to_string(),
                    max_connections: 1,
                };
                let pool = init_db(&config).await.expect("Failed to init db");

                let auth_config = AuthConfig::default();
                let auth_service = AuthService::new(pool, auth_config);

                // Both registrations should succeed
                let result1 = auth_service.register(username1.clone(), password.clone()).await;
                prop_assert!(
                    result1.is_ok(),
                    "First user registration should succeed: {:?}",
                    result1.err()
                );

                let result2 = auth_service.register(username2.clone(), password.clone()).await;
                prop_assert!(
                    result2.is_ok(),
                    "Second user registration should succeed: {:?}",
                    result2.err()
                );

                // Verify both users have different IDs
                let user1 = result1.unwrap();
                let user2 = result2.unwrap();
                prop_assert_ne!(
                    user1.id, user2.id,
                    "Different users should have different IDs"
                );

                Ok(())
            })?;
        }
    }
}
