
//! Property tests for OpenAPI Schema Completeness.
//!
//! **Feature: openapi-dev-feature, Property 1: Schema Completeness**
//! **Validates: Requirements 1.3, 5.1, 5.2**
//!
//! For any type annotated with `ToSchema`, the generated OpenAPI specification
//! SHALL contain a corresponding schema definition in the `components/schemas` section.

#![cfg(feature = "dev")]

use backend::openapi::ApiDoc;
use utoipa::OpenApi;

/// List of all schema names that should be present in the OpenAPI spec.
/// These correspond to all types annotated with `ToSchema` in the codebase.
const EXPECTED_SCHEMAS: &[&str] = &[
    // Auth schemas
    "LoginRequest",
    "LoginResponse",
    "UserResponse",
    "UpdateUserRequest",
    // Library schemas
    "Library",
    "LibraryWithStats",
    "ScanPath",
    "CreateLibraryRequest",
    "UpdateLibraryRequest",
    "AddScanPathRequest",
    "ScanPathParams",
    // Content schemas
    "ContentType",
    "ContentResponse",
    "Chapter",
    "SearchQuery",
    "ChapterTextResponse",
    "ChapterTextParams",
    "PageParams",
    // Progress schemas
    "ProgressResponse",
    "UpdateProgressRequest",
    "UpdateProgressWithPercentageRequest",
];

/// **Feature: openapi-dev-feature, Property 1: Schema Completeness**
/// **Validates: Requirements 1.3, 5.1, 5.2**
///
/// For any type annotated with `ToSchema`, the generated OpenAPI specification
/// SHALL contain a corresponding schema definition in the `components/schemas` section.
#[test]
fn schema_completeness_all_expected_schemas_present() {
    let openapi = ApiDoc::openapi();
    let components = openapi
        .components
        .as_ref()
        .expect("OpenAPI spec should have components section");

    let schema_names: Vec<&str> = components.schemas.keys().map(|s| s.as_str()).collect();

    let mut missing_schemas = Vec::new();

    for expected in EXPECTED_SCHEMAS {
        if !schema_names.contains(expected) {
            missing_schemas.push(*expected);
        }
    }

    assert!(
        missing_schemas.is_empty(),
        "Missing schemas in OpenAPI spec: {:?}\nAvailable schemas: {:?}",
        missing_schemas,
        schema_names
    );
}

/// **Feature: openapi-dev-feature, Property 1: Schema Completeness**
/// **Validates: Requirements 1.3, 5.1, 5.2**
///
/// Verify that the OpenAPI spec contains all expected API paths.
#[test]
fn schema_completeness_all_expected_paths_present() {
    let openapi = ApiDoc::openapi();

    let expected_paths = [
        // Auth endpoints
        "/api/auth/login",
        "/api/auth/me",
        // Library endpoints
        "/api/libraries",
        "/api/libraries/{id}",
        "/api/libraries/{id}/paths",
        "/api/libraries/{id}/paths/{path_id}",
        // Content endpoints
        "/api/libraries/{id}/contents",
        "/api/libraries/{id}/scan",
        "/api/libraries/{id}/search",
        "/api/contents/{id}",
        "/api/contents/{id}/chapters",
        "/api/contents/{id}/chapters/{chapter}/pages/{page}",
        "/api/contents/{id}/chapters/{chapter}/text",
        // Progress endpoints
        "/api/contents/{id}/progress",
        "/api/chapters/{id}/progress",
    ];

    let path_keys: Vec<&str> = openapi.paths.paths.keys().map(|s| s.as_str()).collect();

    let mut missing_paths = Vec::new();

    for expected in expected_paths {
        if !path_keys.contains(&expected) {
            missing_paths.push(expected);
        }
    }

    assert!(
        missing_paths.is_empty(),
        "Missing paths in OpenAPI spec: {:?}\nAvailable paths: {:?}",
        missing_paths,
        path_keys
    );
}

/// **Feature: openapi-dev-feature, Property 1: Schema Completeness**
/// **Validates: Requirements 1.3, 5.1, 5.2**
///
/// Verify that the OpenAPI spec can be serialized to valid JSON.
#[test]
fn schema_completeness_valid_json_output() {
    let openapi = ApiDoc::openapi();

    let json_result = serde_json::to_string_pretty(&openapi);
    assert!(
        json_result.is_ok(),
        "OpenAPI spec should serialize to valid JSON: {:?}",
        json_result.err()
    );

    let json = json_result.unwrap();

    // Verify it can be parsed back
    let parsed: serde_json::Value = serde_json::from_str(&json)
        .expect("Serialized OpenAPI JSON should be parseable");

    // Verify basic structure
    assert!(
        parsed.get("openapi").is_some(),
        "OpenAPI spec should have 'openapi' version field"
    );
    assert!(
        parsed.get("info").is_some(),
        "OpenAPI spec should have 'info' section"
    );
    assert!(
        parsed.get("paths").is_some(),
        "OpenAPI spec should have 'paths' section"
    );
    assert!(
        parsed.get("components").is_some(),
        "OpenAPI spec should have 'components' section"
    );
}

/// **Feature: openapi-dev-feature, Property 1: Schema Completeness**
/// **Validates: Requirements 1.3, 5.1, 5.2**
///
/// Verify that all expected tags are present in the OpenAPI spec.
#[test]
fn schema_completeness_all_expected_tags_present() {
    let openapi = ApiDoc::openapi();

    let expected_tags = ["auth", "libraries", "contents", "chapters", "progress"];

    let tag_names: Vec<&str> = openapi
        .tags
        .as_ref()
        .map(|tags| tags.iter().map(|t| t.name.as_str()).collect())
        .unwrap_or_default();

    let mut missing_tags = Vec::new();

    for expected in expected_tags {
        if !tag_names.contains(&expected) {
            missing_tags.push(expected);
        }
    }

    assert!(
        missing_tags.is_empty(),
        "Missing tags in OpenAPI spec: {:?}\nAvailable tags: {:?}",
        missing_tags,
        tag_names
    );
}

/// **Feature: openapi-dev-feature, Property 1: Schema Completeness**
/// **Validates: Requirements 1.3, 5.1, 5.2**
///
/// Verify that the OpenAPI spec has correct metadata.
#[test]
fn schema_completeness_correct_metadata() {
    let openapi = ApiDoc::openapi();

    assert_eq!(
        openapi.info.title, "Comic Reader API",
        "OpenAPI spec should have correct title"
    );
    assert_eq!(
        openapi.info.version, "1.0.0",
        "OpenAPI spec should have correct version"
    );
}
