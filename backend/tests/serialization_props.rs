//! Property tests for JSON serialization round-trip.
//!
//! **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
//! **Validates: Requirements 8.2, 8.3, 8.4**
//!
//! For any valid content data structure, serializing to JSON and then
//! deserializing should produce an equivalent data structure.

use backend::models::{
    Chapter, Content, ContentProgressResponse, ContentResponse, ContentType, CreateLibraryRequest,
    JwtClaims, Library, LibraryWithStats, LoginRequest, LoginResponse, ProgressResponse,
    ReadingProgress, ScanPath, UpdateLibraryRequest, UpdateProgressRequest, User, UserResponse,
};
use chrono::{DateTime, TimeZone, Utc};
use proptest::prelude::*;

// ============================================================================
// Arbitrary Strategies for Data Types
// ============================================================================

/// Strategy to generate arbitrary ContentType values.
fn arb_content_type() -> impl Strategy<Value = ContentType> {
    prop_oneof![Just(ContentType::Comic), Just(ContentType::Novel),]
}

/// Strategy to generate arbitrary DateTime<Utc> values.
fn arb_datetime() -> impl Strategy<Value = DateTime<Utc>> {
    // Generate timestamps between 2020-01-01 and 2030-01-01
    (1577836800i64..1893456000i64).prop_map(|ts| Utc.timestamp_opt(ts, 0).unwrap())
}

/// Strategy to generate valid non-empty strings for names/titles.
fn arb_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\- ]{1,100}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Name must not be empty", |s| !s.is_empty())
}

/// Strategy to generate valid file paths.
fn arb_path() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_/\\\\\\-\\.]{1,200}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Path must not be empty", |s| !s.is_empty())
}

/// Strategy to generate optional JSON metadata.
fn arb_metadata() -> impl Strategy<Value = Option<serde_json::Value>> {
    prop_oneof![
        Just(None),
        Just(Some(serde_json::json!({}))),
        Just(Some(serde_json::json!({"title": "Test", "rating": 8.5}))),
        Just(Some(
            serde_json::json!({"author": "Author Name", "tags": ["action", "comedy"]})
        )),
    ]
}

/// Strategy to generate arbitrary Library instances.
fn arb_library() -> impl Strategy<Value = Library> {
    (
        1i64..10000,
        arb_name(),
        0i32..1440,
        any::<bool>(),
        arb_datetime(),
        arb_datetime(),
    )
        .prop_map(
            |(id, name, scan_interval, watch_mode, created_at, updated_at)| Library {
                id,
                name,
                scan_interval,
                watch_mode,
                created_at,
                updated_at,
            },
        )
}

/// Strategy to generate arbitrary ScanPath instances.
fn arb_scan_path() -> impl Strategy<Value = ScanPath> {
    (1i64..10000, 1i64..10000, arb_path(), arb_datetime()).prop_map(
        |(id, library_id, path, created_at)| ScanPath {
            id,
            library_id,
            path,
            created_at,
        },
    )
}

/// Strategy to generate arbitrary Content instances.
fn arb_content() -> impl Strategy<Value = Content> {
    (
        1i64..10000,
        1i64..10000,
        1i64..10000,
        arb_content_type(),
        arb_name(),
        arb_path(),
        0i32..1000,
        arb_metadata(),
        arb_datetime(),
        arb_datetime(),
    )
        .prop_map(
            |(
                id,
                library_id,
                scan_path_id,
                content_type,
                title,
                folder_path,
                chapter_count,
                metadata,
                created_at,
                updated_at,
            )| Content {
                id,
                library_id,
                scan_path_id,
                content_type,
                title,
                folder_path,
                chapter_count,
                thumbnail: None, // Skip thumbnail for serialization tests
                metadata: metadata.and_then(|m| serde_json::to_vec(&m).ok()),
                created_at,
                updated_at,
            },
        )
}

/// Strategy to generate arbitrary Chapter instances.
fn arb_chapter() -> impl Strategy<Value = Chapter> {
    (
        1i64..10000,
        1i64..10000,
        arb_name(),
        arb_path(),
        0i32..1000,
        0i64..1000000,
    )
        .prop_map(
            |(id, content_id, title, file_path, sort_order, size)| Chapter {
                id,
                content_id,
                title,
                file_path,
                sort_order,
                size,
                page_count: 0, // Skip page_count for serialization tests
            },
        )
}

/// Strategy to generate arbitrary User instances.
fn arb_user() -> impl Strategy<Value = User> {
    (
        1i64..10000,
        arb_name(),
        "[a-zA-Z0-9$./]{60,100}", // password hash pattern
        prop::option::of(arb_name()),
        arb_datetime(),
        arb_datetime(),
    )
        .prop_map(
            |(id, username, password_hash, bangumi_api_key, created_at, updated_at)| User {
                id,
                username,
                password_hash,
                bangumi_api_key,
                created_at,
                updated_at,
            },
        )
}

/// Strategy to generate arbitrary ReadingProgress instances.
fn arb_reading_progress() -> impl Strategy<Value = ReadingProgress> {
    (
        1i64..10000,
        1i64..10000,
        1i64..10000,
        0i32..10000,
        0.0f32..100.0,
        arb_datetime(),
    )
        .prop_map(
            |(id, user_id, chapter_id, position, percentage, updated_at)| ReadingProgress {
                id,
                user_id,
                chapter_id,
                position,
                percentage,
                updated_at,
            },
        )
}

/// Strategy to generate arbitrary JwtClaims instances.
fn arb_jwt_claims() -> impl Strategy<Value = JwtClaims> {
    (
        1i64..10000,
        arb_name(),
        1577836800i64..1893456000i64,
        1577836800i64..1893456000i64,
    )
        .prop_map(|(sub, username, exp, iat)| JwtClaims {
            sub,
            username,
            exp,
            iat,
        })
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid Library, serializing to JSON and deserializing should
    /// produce an equivalent Library.
    #[test]
    fn library_json_round_trip(library in arb_library()) {
        let json = serde_json::to_string(&library).expect("Should serialize Library to JSON");
        let deserialized: Library = serde_json::from_str(&json).expect("Should deserialize Library from JSON");
        prop_assert_eq!(library, deserialized, "Library round-trip should preserve data");
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid ScanPath, serializing to JSON and deserializing should
    /// produce an equivalent ScanPath.
    #[test]
    fn scan_path_json_round_trip(scan_path in arb_scan_path()) {
        let json = serde_json::to_string(&scan_path).expect("Should serialize ScanPath to JSON");
        let deserialized: ScanPath = serde_json::from_str(&json).expect("Should deserialize ScanPath from JSON");
        prop_assert_eq!(scan_path, deserialized, "ScanPath round-trip should preserve data");
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid Content, serializing to JSON and deserializing should
    /// produce an equivalent Content.
    #[test]
    fn content_json_round_trip(content in arb_content()) {
        let json = serde_json::to_string(&content).expect("Should serialize Content to JSON");
        let deserialized: Content = serde_json::from_str(&json).expect("Should deserialize Content from JSON");
        prop_assert_eq!(content, deserialized, "Content round-trip should preserve data");
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid Chapter, serializing to JSON and deserializing should
    /// produce an equivalent Chapter.
    #[test]
    fn chapter_json_round_trip(chapter in arb_chapter()) {
        let json = serde_json::to_string(&chapter).expect("Should serialize Chapter to JSON");
        let deserialized: Chapter = serde_json::from_str(&json).expect("Should deserialize Chapter from JSON");
        prop_assert_eq!(chapter, deserialized, "Chapter round-trip should preserve data");
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid ReadingProgress, serializing to JSON and deserializing should
    /// produce an equivalent ReadingProgress.
    #[test]
    fn reading_progress_json_round_trip(progress in arb_reading_progress()) {
        let json = serde_json::to_string(&progress).expect("Should serialize ReadingProgress to JSON");
        let deserialized: ReadingProgress = serde_json::from_str(&json).expect("Should deserialize ReadingProgress from JSON");
        prop_assert_eq!(progress, deserialized, "ReadingProgress round-trip should preserve data");
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid JwtClaims, serializing to JSON and deserializing should
    /// produce an equivalent JwtClaims.
    #[test]
    fn jwt_claims_json_round_trip(claims in arb_jwt_claims()) {
        let json = serde_json::to_string(&claims).expect("Should serialize JwtClaims to JSON");
        let deserialized: JwtClaims = serde_json::from_str(&json).expect("Should deserialize JwtClaims from JSON");
        prop_assert_eq!(claims, deserialized, "JwtClaims round-trip should preserve data");
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid ContentType, serializing to JSON and deserializing should
    /// produce an equivalent ContentType.
    #[test]
    fn content_type_json_round_trip(content_type in arb_content_type()) {
        let json = serde_json::to_string(&content_type).expect("Should serialize ContentType to JSON");
        let deserialized: ContentType = serde_json::from_str(&json).expect("Should deserialize ContentType from JSON");
        prop_assert_eq!(content_type, deserialized, "ContentType round-trip should preserve data");
    }
}

// ============================================================================
// Additional Round-Trip Tests for Response Types
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid Content, converting to ContentResponse and round-tripping
    /// through JSON should preserve the response data.
    #[test]
    fn content_response_json_round_trip(content in arb_content()) {
        let response: ContentResponse = content.into();
        let json = serde_json::to_string(&response).expect("Should serialize ContentResponse to JSON");
        let deserialized: ContentResponse = serde_json::from_str(&json).expect("Should deserialize ContentResponse from JSON");

        // Compare fields (ContentResponse doesn't derive PartialEq)
        prop_assert_eq!(response.id, deserialized.id);
        prop_assert_eq!(response.library_id, deserialized.library_id);
        prop_assert_eq!(response.content_type, deserialized.content_type);
        prop_assert_eq!(response.title, deserialized.title);
        prop_assert_eq!(response.chapter_count, deserialized.chapter_count);
        prop_assert_eq!(response.has_thumbnail, deserialized.has_thumbnail);
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid User, converting to UserResponse and round-tripping
    /// through JSON should preserve the response data.
    #[test]
    fn user_response_json_round_trip(user in arb_user()) {
        let response: UserResponse = user.into();
        let json = serde_json::to_string(&response).expect("Should serialize UserResponse to JSON");
        let deserialized: UserResponse = serde_json::from_str(&json).expect("Should deserialize UserResponse from JSON");

        prop_assert_eq!(response.id, deserialized.id);
        prop_assert_eq!(response.username, deserialized.username);
        prop_assert_eq!(response.bangumi_api_key, deserialized.bangumi_api_key);
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid ReadingProgress, converting to ProgressResponse and round-tripping
    /// through JSON should preserve the response data.
    #[test]
    fn progress_response_json_round_trip(progress in arb_reading_progress()) {
        let response: ProgressResponse = progress.into();
        let json = serde_json::to_string(&response).expect("Should serialize ProgressResponse to JSON");
        let deserialized: ProgressResponse = serde_json::from_str(&json).expect("Should deserialize ProgressResponse from JSON");

        prop_assert_eq!(response.chapter_id, deserialized.chapter_id);
        prop_assert_eq!(response.position, deserialized.position);
        // Float comparison with tolerance
        prop_assert!((response.percentage - deserialized.percentage).abs() < 0.001);
    }

    /// **Feature: comic-reader, Property 18: JSON Serialization Round-Trip**
    /// **Validates: Requirements 8.2, 8.3, 8.4**
    ///
    /// For any valid LibraryWithStats, serializing to JSON and deserializing should
    /// produce an equivalent LibraryWithStats.
    #[test]
    fn library_with_stats_json_round_trip(
        library in arb_library(),
        path_count in 0i64..100,
        content_count in 0i64..1000
    ) {
        let stats = LibraryWithStats {
            library,
            path_count,
            content_count,
        };
        let json = serde_json::to_string(&stats).expect("Should serialize LibraryWithStats to JSON");
        let deserialized: LibraryWithStats = serde_json::from_str(&json).expect("Should deserialize LibraryWithStats from JSON");
        prop_assert_eq!(stats, deserialized, "LibraryWithStats round-trip should preserve data");
    }
}

// ============================================================================
// Property Tests for OpenAPI DTO Serialization Round-Trip
// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
// **Validates: Requirements 5.4**
// ============================================================================

/// Strategy to generate arbitrary LoginRequest instances.
fn arb_login_request() -> impl Strategy<Value = LoginRequest> {
    (arb_name(), arb_name()).prop_map(|(username, password)| LoginRequest { username, password })
}

/// Strategy to generate arbitrary CreateLibraryRequest instances.
fn arb_create_library_request() -> impl Strategy<Value = CreateLibraryRequest> {
    (
        arb_name(),
        prop::option::of(0i32..1440),
        prop::option::of(any::<bool>()),
    )
        .prop_map(|(name, scan_interval, watch_mode)| CreateLibraryRequest {
            name,
            scan_interval,
            watch_mode,
        })
}

/// Strategy to generate arbitrary UpdateLibraryRequest instances.
fn arb_update_library_request() -> impl Strategy<Value = UpdateLibraryRequest> {
    (
        prop::option::of(arb_name()),
        prop::option::of(0i32..1440),
        prop::option::of(any::<bool>()),
    )
        .prop_map(|(name, scan_interval, watch_mode)| UpdateLibraryRequest {
            name,
            scan_interval,
            watch_mode,
        })
}

/// Strategy to generate arbitrary UpdateProgressRequest instances.
fn arb_update_progress_request() -> impl Strategy<Value = UpdateProgressRequest> {
    (0i32..10000).prop_map(|position| UpdateProgressRequest { position })
}

/// Strategy to generate arbitrary ContentProgressResponse instances.
fn arb_content_progress_response() -> impl Strategy<Value = ContentProgressResponse> {
    (
        1i64..10000,
        0i32..1000,
        0i32..1000,
        prop::option::of(1i64..10000),
        0.0f32..100.0,
    )
        .prop_map(
            |(
                content_id,
                total_chapters,
                completed_chapters,
                current_chapter_id,
                overall_percentage,
            )| {
                ContentProgressResponse {
                    content_id,
                    total_chapters,
                    completed_chapters,
                    current_chapter_id,
                    overall_percentage,
                }
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
    /// **Validates: Requirements 5.4**
    ///
    /// For any valid LoginRequest, serializing to JSON and deserializing should
    /// produce an equivalent LoginRequest.
    #[test]
    fn login_request_json_round_trip(request in arb_login_request()) {
        let json = serde_json::to_string(&request).expect("Should serialize LoginRequest to JSON");
        let deserialized: LoginRequest = serde_json::from_str(&json).expect("Should deserialize LoginRequest from JSON");
        prop_assert_eq!(request.username, deserialized.username);
        prop_assert_eq!(request.password, deserialized.password);
    }

    /// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
    /// **Validates: Requirements 5.4**
    ///
    /// For any valid CreateLibraryRequest, serializing to JSON and deserializing should
    /// produce an equivalent CreateLibraryRequest.
    #[test]
    fn create_library_request_json_round_trip(request in arb_create_library_request()) {
        let json = serde_json::to_string(&request).expect("Should serialize CreateLibraryRequest to JSON");
        let deserialized: CreateLibraryRequest = serde_json::from_str(&json).expect("Should deserialize CreateLibraryRequest from JSON");
        prop_assert_eq!(request.name, deserialized.name);
        prop_assert_eq!(request.scan_interval, deserialized.scan_interval);
        prop_assert_eq!(request.watch_mode, deserialized.watch_mode);
    }

    /// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
    /// **Validates: Requirements 5.4**
    ///
    /// For any valid UpdateLibraryRequest, serializing to JSON and deserializing should
    /// produce an equivalent UpdateLibraryRequest.
    #[test]
    fn update_library_request_json_round_trip(request in arb_update_library_request()) {
        let json = serde_json::to_string(&request).expect("Should serialize UpdateLibraryRequest to JSON");
        let deserialized: UpdateLibraryRequest = serde_json::from_str(&json).expect("Should deserialize UpdateLibraryRequest from JSON");
        prop_assert_eq!(request.name, deserialized.name);
        prop_assert_eq!(request.scan_interval, deserialized.scan_interval);
        prop_assert_eq!(request.watch_mode, deserialized.watch_mode);
    }

    /// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
    /// **Validates: Requirements 5.4**
    ///
    /// For any valid UpdateProgressRequest, serializing to JSON and deserializing should
    /// produce an equivalent UpdateProgressRequest.
    #[test]
    fn update_progress_request_json_round_trip(request in arb_update_progress_request()) {
        let json = serde_json::to_string(&request).expect("Should serialize UpdateProgressRequest to JSON");
        let deserialized: UpdateProgressRequest = serde_json::from_str(&json).expect("Should deserialize UpdateProgressRequest from JSON");
        prop_assert_eq!(request.position, deserialized.position);
    }

    /// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
    /// **Validates: Requirements 5.4**
    ///
    /// For any valid ContentProgressResponse, serializing to JSON and deserializing should
    /// produce an equivalent ContentProgressResponse.
    #[test]
    fn content_progress_response_json_round_trip(response in arb_content_progress_response()) {
        let json = serde_json::to_string(&response).expect("Should serialize ContentProgressResponse to JSON");
        let deserialized: ContentProgressResponse = serde_json::from_str(&json).expect("Should deserialize ContentProgressResponse from JSON");
        prop_assert_eq!(response.content_id, deserialized.content_id);
        prop_assert_eq!(response.total_chapters, deserialized.total_chapters);
        prop_assert_eq!(response.completed_chapters, deserialized.completed_chapters);
        prop_assert_eq!(response.current_chapter_id, deserialized.current_chapter_id);
        // Float comparison with tolerance
        prop_assert!((response.overall_percentage - deserialized.overall_percentage).abs() < 0.001);
    }

    /// **Feature: openapi-dev-feature, Property 2: DTO Serialization Round-Trip**
    /// **Validates: Requirements 5.4**
    ///
    /// For any valid LoginResponse, serializing to JSON and deserializing should
    /// produce an equivalent LoginResponse.
    #[test]
    fn login_response_json_round_trip(user in arb_user(), token in arb_name()) {
        let response = LoginResponse {
            user: user.into(),
            token,
        };
        let json = serde_json::to_string(&response).expect("Should serialize LoginResponse to JSON");
        let deserialized: LoginResponse = serde_json::from_str(&json).expect("Should deserialize LoginResponse from JSON");
        prop_assert_eq!(response.user.id, deserialized.user.id);
        prop_assert_eq!(response.user.username, deserialized.user.username);
        prop_assert_eq!(response.token, deserialized.token);
    }
}
