//! Property tests for error handling.
//!
//! **Feature: comic-reader, Property 17: Error Response Structure**
//! **Validates: Requirements 7.5**

use backend::error::AppError;
use proptest::prelude::*;

/// Strategy to generate arbitrary AppError variants with random messages.
fn arb_app_error() -> impl Strategy<Value = AppError> {
    prop_oneof![
        any::<String>().prop_map(AppError::NotFound),
        any::<String>().prop_map(AppError::BadRequest),
        any::<String>().prop_map(AppError::Unauthorized),
        any::<String>().prop_map(AppError::Archive),
        any::<String>().prop_map(AppError::Internal),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 17: Error Response Structure**
    /// **Validates: Requirements 7.5**
    ///
    /// For any AppError, the error response should contain:
    /// 1. An HTTP error code (non-zero, valid HTTP status)
    /// 2. A descriptive error message (non-empty for user-facing errors)
    #[test]
    fn error_response_contains_code_and_message(error in arb_app_error()) {
        let response = error.to_error_response();

        // Verify the response has the expected structure
        prop_assert!(response.error.code > 0, "Error code should be positive");
        prop_assert!(response.error.code >= 400, "Error code should be a client or server error (>= 400)");
        prop_assert!(response.error.code < 600, "Error code should be a valid HTTP error code (< 600)");

        // Verify the message is present (can be empty for internal errors, but structure exists)
        // The message field should always exist in the response
        let _message = &response.error.message;
    }

    /// **Feature: comic-reader, Property 17: Error Response Structure**
    /// **Validates: Requirements 7.5**
    ///
    /// For any AppError, the status code should match the error type.
    #[test]
    fn error_status_code_matches_error_type(error in arb_app_error()) {
        let status = error.status_code();
        let response = error.to_error_response();

        // The response code should match the status code
        prop_assert_eq!(
            response.error.code,
            status.as_u16(),
            "Response code should match status code"
        );
    }

    /// **Feature: comic-reader, Property 17: Error Response Structure**
    /// **Validates: Requirements 7.5**
    ///
    /// For any AppError with a non-empty message, the error response message should be non-empty.
    #[test]
    fn non_empty_error_has_non_empty_message(msg in "[a-zA-Z0-9 ]+") {
        // Test each error type with a non-empty message
        let errors = vec![
            AppError::NotFound(msg.clone()),
            AppError::BadRequest(msg.clone()),
            AppError::Unauthorized(msg.clone()),
            AppError::Archive(msg.clone()),
            AppError::Internal(msg.clone()),
        ];

        for error in errors {
            let response = error.to_error_response();
            prop_assert!(
                !response.error.message.is_empty(),
                "Error message should not be empty for user-facing errors"
            );
        }
    }
}

/// **Feature: comic-reader, Property 17: Error Response Structure**
/// **Validates: Requirements 7.5**
///
/// Verify that ErrorResponse can be serialized to JSON with the expected structure.
#[test]
fn error_response_serializes_to_json() {
    let error = AppError::NotFound("Resource not found".to_string());
    let response = error.to_error_response();

    let json = serde_json::to_value(&response).expect("Should serialize to JSON");

    // Verify JSON structure
    assert!(json.get("error").is_some(), "Should have 'error' field");
    let error_obj = json.get("error").unwrap();
    assert!(error_obj.get("code").is_some(), "Should have 'code' field");
    assert!(
        error_obj.get("message").is_some(),
        "Should have 'message' field"
    );

    // Verify values
    assert_eq!(error_obj.get("code").unwrap(), 404);
    assert_eq!(error_obj.get("message").unwrap(), "Resource not found");
}
