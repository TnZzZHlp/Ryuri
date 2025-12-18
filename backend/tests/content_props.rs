//! Property tests for content management.
//!
//! This module contains property-based tests for content operations including
//! retrieval, deletion, search, and image ordering.

use backend::db::{DbConfig, init_db};
use backend::models::ContentType;
use backend::services::content::ContentService;
use chrono::Utc;
use proptest::prelude::*;
use sqlx::{Pool, Sqlite};
use tokio::runtime::Runtime;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create an in-memory database for testing.
async fn create_test_db() -> Pool<Sqlite> {
    let config = DbConfig {
        database_url: "sqlite::memory:".to_string(),
        max_connections: 1,
    };
    init_db(&config)
        .await
        .expect("Failed to initialize test database")
}

/// Strategy to generate valid library names.
fn arb_library_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_ ]{0,49}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Name must not be empty", |s| !s.is_empty())
}

/// Strategy to generate valid content titles.
fn arb_content_title() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_ ]{0,49}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Title must not be empty", |s| !s.is_empty())
}


/// Helper function to create a test library.
async fn create_test_library(pool: &Pool<Sqlite>, name: &str) -> i64 {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        r#"
        INSERT INTO libraries (name, scan_interval, watch_mode, created_at, updated_at)
        VALUES (?, 0, 0, ?, ?)
        "#,
    )
    .bind(name)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Should create test library");

    result.last_insert_rowid()
}

/// Helper function to create a test scan path.
async fn create_test_scan_path(pool: &Pool<Sqlite>, library_id: i64, path: &str) -> i64 {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        r#"
        INSERT INTO scan_paths (library_id, path, created_at)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(library_id)
    .bind(path)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Should create test scan path");

    result.last_insert_rowid()
}

/// Helper function to insert content directly into the database for testing.
async fn insert_test_content(
    pool: &Pool<Sqlite>,
    library_id: i64,
    scan_path_id: i64,
    title: &str,
    content_type: ContentType,
) -> i64 {
    let now = Utc::now().to_rfc3339();
    let content_type_str = match content_type {
        ContentType::Comic => "Comic",
        ContentType::Novel => "Novel",
    };
    let result = sqlx::query(
        r#"
        INSERT INTO contents (library_id, scan_path_id, content_type, title, folder_path, chapter_count, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 0, ?, ?)
        "#,
    )
    .bind(library_id)
    .bind(scan_path_id)
    .bind(content_type_str)
    .bind(title)
    .bind(format!("/path/to/{}", title))
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Should insert test content");

    result.last_insert_rowid()
}

/// Helper function to insert a chapter for testing.
async fn insert_test_chapter(
    pool: &Pool<Sqlite>,
    content_id: i64,
    title: &str,
    file_path: &str,
    sort_order: i32,
) -> i64 {
    let result = sqlx::query(
        r#"
        INSERT INTO chapters (content_id, title, file_path, sort_order)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(content_id)
    .bind(title)
    .bind(file_path)
    .bind(sort_order)
    .execute(pool)
    .await
    .expect("Should insert test chapter");

    result.last_insert_rowid()
}

/// Helper function to count chapters for a content.
async fn count_chapters_for_content(pool: &Pool<Sqlite>, content_id: i64) -> i64 {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapters WHERE content_id = ?")
        .bind(content_id)
        .fetch_one(pool)
        .await
        .expect("Should count chapters");

    result.0
}

/// Helper function to check if content exists.
async fn content_exists(pool: &Pool<Sqlite>, content_id: i64) -> bool {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents WHERE id = ?")
        .bind(content_id)
        .fetch_one(pool)
        .await
        .expect("Should check content existence");

    result.0 > 0
}

// ============================================================================
// Property 5: Content Retrieval Completeness
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 5: Content Retrieval Completeness**
    /// **Validates: Requirements 1.5**
    ///
    /// For any library with contents across multiple scan paths, querying the
    /// library's contents should return all contents from all associated scan paths.
    #[test]
    fn content_retrieval_completeness(
        library_name in arb_library_name(),
        num_paths in 1usize..4,
        contents_per_path in 1usize..4
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library
            let library_id = create_test_library(&pool, &library_name).await;

            // Track all content IDs we create
            let mut all_content_ids: Vec<i64> = Vec::new();

            // Add scan paths and contents
            for i in 0..num_paths {
                let path = format!("/test/path/{}", i);
                let scan_path_id = create_test_scan_path(&pool, library_id, &path).await;

                for j in 0..contents_per_path {
                    let title = format!("Content_{}_{}", i, j);
                    let content_id = insert_test_content(
                        &pool,
                        library_id,
                        scan_path_id,
                        &title,
                        ContentType::Comic,
                    ).await;
                    all_content_ids.push(content_id);
                }
            }

            // Retrieve all contents for the library
            let contents = ContentService::list_contents(&pool, library_id).await
                .expect("Should list contents");

            // Verify we got all contents
            prop_assert_eq!(
                contents.len(),
                all_content_ids.len(),
                "Should retrieve all contents from all scan paths"
            );

            // Verify each content ID is present
            for content_id in &all_content_ids {
                let found = contents.iter().any(|c| c.id == *content_id);
                prop_assert!(found, "Content {} should be in the list", content_id);
            }

            // Verify all returned contents belong to the library
            for content in &contents {
                prop_assert_eq!(
                    content.library_id,
                    library_id,
                    "All contents should belong to the library"
                );
            }

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 10: Content Deletion Cascade
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 10: Content Deletion Cascade**
    /// **Validates: Requirements 2.9**
    ///
    /// For any content with chapters, deleting the content should remove all
    /// associated chapter records from the database.
    #[test]
    fn content_deletion_cascade(
        library_name in arb_library_name(),
        content_title in arb_content_title(),
        num_chapters in 1usize..6
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create content
            let content_id = insert_test_content(
                &pool,
                library_id,
                scan_path_id,
                &content_title,
                ContentType::Comic,
            ).await;

            // Add chapters
            for i in 0..num_chapters {
                let chapter_title = format!("Chapter {}", i + 1);
                let file_path = format!("/path/to/{}/chapter_{}.cbz", content_title, i + 1);
                insert_test_chapter(&pool, content_id, &chapter_title, &file_path, i as i32).await;
            }

            // Verify chapters exist before deletion
            let chapters_before = count_chapters_for_content(&pool, content_id).await;
            prop_assert_eq!(
                chapters_before as usize,
                num_chapters,
                "Should have {} chapters before deletion",
                num_chapters
            );

            // Delete content
            ContentService::delete_content(&pool, content_id).await
                .expect("Should delete content");

            // Verify content is deleted
            prop_assert!(
                !content_exists(&pool, content_id).await,
                "Content should not exist after deletion"
            );

            // Verify all chapters are deleted (cascade)
            let chapters_after = count_chapters_for_content(&pool, content_id).await;
            prop_assert_eq!(
                chapters_after,
                0,
                "All chapters should be deleted after content deletion"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 11: Search Result Relevance
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 11: Search Result Relevance**
    /// **Validates: Requirements 2.10**
    ///
    /// For any search query and library, all returned contents should have titles
    /// that contain the search keyword (case-insensitive).
    #[test]
    fn search_result_relevance(
        library_name in arb_library_name(),
        search_keyword in "[a-zA-Z]{2,10}",
        num_matching in 1usize..4,
        num_non_matching in 0usize..4
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create matching contents (titles contain the keyword)
            let mut matching_ids: Vec<i64> = Vec::new();
            for i in 0..num_matching {
                let title = format!("Series {} with {} in title", i, search_keyword);
                let content_id = insert_test_content(
                    &pool,
                    library_id,
                    scan_path_id,
                    &title,
                    ContentType::Comic,
                ).await;
                matching_ids.push(content_id);
            }

            // Create non-matching contents (titles don't contain the keyword)
            for i in 0..num_non_matching {
                let title = format!("Other Series {}", i);
                // Make sure the title doesn't accidentally contain the keyword
                if !title.to_lowercase().contains(&search_keyword.to_lowercase()) {
                    insert_test_content(
                        &pool,
                        library_id,
                        scan_path_id,
                        &title,
                        ContentType::Novel,
                    ).await;
                }
            }

            // Search for the keyword
            let results = ContentService::search_contents(&pool, library_id, &search_keyword).await
                .expect("Should search contents");

            // Verify all results contain the keyword (case-insensitive)
            for content in &results {
                let title_lower = content.title.to_lowercase();
                let keyword_lower = search_keyword.to_lowercase();
                prop_assert!(
                    title_lower.contains(&keyword_lower),
                    "Search result '{}' should contain keyword '{}'",
                    content.title,
                    search_keyword
                );
            }

            // Verify all matching contents are in results
            for matching_id in &matching_ids {
                let found = results.iter().any(|c| c.id == *matching_id);
                prop_assert!(
                    found,
                    "Matching content {} should be in search results",
                    matching_id
                );
            }

            // Verify result count is at least the number of matching contents
            prop_assert!(
                results.len() >= matching_ids.len(),
                "Should have at least {} results, got {}",
                matching_ids.len(),
                results.len()
            );

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 11: Search Result Relevance**
    /// **Validates: Requirements 2.10**
    ///
    /// Empty search should return no results or all results depending on implementation.
    /// Non-matching search should return empty results.
    #[test]
    fn search_no_match_returns_empty(
        library_name in arb_library_name(),
        content_title in arb_content_title()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create content with a specific title
            insert_test_content(
                &pool,
                library_id,
                scan_path_id,
                &content_title,
                ContentType::Comic,
            ).await;

            // Search for something that definitely won't match
            let non_matching_query = "ZZZZNONEXISTENT12345";
            let results = ContentService::search_contents(&pool, library_id, non_matching_query).await
                .expect("Should search contents");

            // Verify no results for non-matching query
            prop_assert!(
                results.is_empty(),
                "Non-matching search should return empty results"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 12: Image Ordering Consistency
// ============================================================================

use backend::extractors::natural_sort_key;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 12: Image Ordering Consistency**
    /// **Validates: Requirements 3.2**
    ///
    /// For any comic chapter, the images should be returned in a consistent order
    /// based on their filenames within the archive. This tests that the natural
    /// sort algorithm produces consistent, deterministic ordering.
    #[test]
    fn image_ordering_consistency(
        filenames in prop::collection::vec("[a-zA-Z0-9_]{1,20}\\.(jpg|png|gif)", 1..20)
    ) {
        // Sort the filenames using natural sort
        let mut sorted1 = filenames.clone();
        sorted1.sort_by_key(|s| natural_sort_key(s));

        // Sort again to verify consistency
        let mut sorted2 = filenames.clone();
        sorted2.sort_by_key(|s| natural_sort_key(s));

        // Verify both sorts produce the same order
        prop_assert_eq!(
            &sorted1,
            &sorted2,
            "Natural sort should produce consistent ordering"
        );

        // Verify the sort is stable (same input always produces same output)
        let mut sorted3 = sorted2.clone();
        sorted3.sort_by_key(|s| natural_sort_key(s));
        prop_assert_eq!(
            &sorted2,
            &sorted3,
            "Sorting already sorted list should not change order"
        );
    }

    /// **Feature: comic-reader, Property 12: Image Ordering Consistency**
    /// **Validates: Requirements 3.2**
    ///
    /// Natural sort should correctly order numeric sequences.
    #[test]
    fn natural_sort_numeric_ordering(
        prefix in "[a-zA-Z]{1,5}",
        numbers in prop::collection::vec(1u32..1000, 2..10)
    ) {
        // Create filenames with numeric suffixes
        let filenames: Vec<String> = numbers.iter()
            .map(|n| format!("{}{}.jpg", prefix, n))
            .collect();

        // Sort using natural sort
        let mut sorted = filenames.clone();
        sorted.sort_by_key(|s| natural_sort_key(s));

        // Extract numbers from sorted filenames and verify they're in order
        let mut prev_num: Option<u32> = None;
        for filename in &sorted {
            // Extract the number from the filename
            let num_str: String = filename.chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            if let Ok(num) = num_str.parse::<u32>() {
                if let Some(prev) = prev_num {
                    prop_assert!(
                        num >= prev,
                        "Numbers should be in ascending order: {} should come after {}",
                        num,
                        prev
                    );
                }
                prev_num = Some(num);
            }
        }
    }

    /// **Feature: comic-reader, Property 12: Image Ordering Consistency**
    /// **Validates: Requirements 3.2**
    ///
    /// Natural sort should handle page1, page2, ..., page10 correctly
    /// (page2 < page10, not page10 < page2 as lexicographic sort would do).
    #[test]
    fn natural_sort_page_ordering(
        page_count in 2usize..50
    ) {
        // Create page filenames
        let filenames: Vec<String> = (1..=page_count)
            .map(|n| format!("page{}.jpg", n))
            .collect();

        // Shuffle and sort
        let mut shuffled = filenames.clone();
        // Simple deterministic shuffle based on index
        shuffled.reverse();

        let mut sorted = shuffled.clone();
        sorted.sort_by_key(|s| natural_sort_key(s));

        // Verify order matches original sequential order
        prop_assert_eq!(
            sorted,
            filenames,
            "Natural sort should order pages correctly"
        );
    }
}

// ============================================================================
// Property 15: API Response Completeness
// ============================================================================

use backend::models::ContentResponse;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 15: API Response Completeness**
    /// **Validates: Requirements 7.1**
    ///
    /// For any content list request, the response should contain all required fields
    /// (id, title, chapterCount, has_thumbnail) for each content item.
    #[test]
    fn api_response_completeness(
        library_name in arb_library_name(),
        num_contents in 1usize..6,
        content_types in prop::collection::vec(prop::bool::ANY, 1..6)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create contents with varying types
            let mut created_ids: Vec<i64> = Vec::new();
            for i in 0..num_contents {
                let content_type = if content_types.get(i).copied().unwrap_or(true) {
                    ContentType::Comic
                } else {
                    ContentType::Novel
                };
                let title = format!("Content_{}", i);
                let content_id = insert_test_content(
                    &pool,
                    library_id,
                    scan_path_id,
                    &title,
                    content_type,
                ).await;
                created_ids.push(content_id);
            }

            // Retrieve contents
            let contents = ContentService::list_contents(&pool, library_id).await
                .expect("Should list contents");

            // Convert to ContentResponse (simulating API response)
            let responses: Vec<ContentResponse> = contents
                .into_iter()
                .map(ContentResponse::from)
                .collect();

            // Verify all required fields are present for each response
            for response in &responses {
                // Verify id is valid (positive)
                prop_assert!(
                    response.id > 0,
                    "Response should have a valid positive id"
                );

                // Verify library_id matches
                prop_assert_eq!(
                    response.library_id,
                    library_id,
                    "Response should have correct library_id"
                );

                // Verify title is not empty
                prop_assert!(
                    !response.title.is_empty(),
                    "Response should have a non-empty title"
                );

                // Verify chapter_count is non-negative
                prop_assert!(
                    response.chapter_count >= 0,
                    "Response should have non-negative chapter_count"
                );

                // Verify content_type is valid (Comic or Novel)
                prop_assert!(
                    matches!(response.content_type, ContentType::Comic | ContentType::Novel),
                    "Response should have a valid content_type"
                );

                // Verify has_thumbnail field exists (boolean)
                // The field exists by virtue of being in the struct, but we verify it's accessible
                let _has_thumb: bool = response.has_thumbnail;

                // Verify created_at is present and valid
                // DateTime<Utc> is always valid if it exists
                let _created: chrono::DateTime<Utc> = response.created_at;
            }

            // Verify we got responses for all created contents
            prop_assert_eq!(
                responses.len(),
                created_ids.len(),
                "Should have response for each created content"
            );

            // Verify each created content has a corresponding response
            for content_id in &created_ids {
                let found = responses.iter().any(|r| r.id == *content_id);
                prop_assert!(
                    found,
                    "Content {} should have a response",
                    content_id
                );
            }

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 15: API Response Completeness**
    /// **Validates: Requirements 7.1**
    ///
    /// ContentResponse conversion should preserve all essential data from Content.
    #[test]
    fn content_response_preserves_data(
        library_name in arb_library_name(),
        content_title in arb_content_title()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create content
            let content_id = insert_test_content(
                &pool,
                library_id,
                scan_path_id,
                &content_title,
                ContentType::Comic,
            ).await;

            // Retrieve the content
            let content = ContentService::get_content(&pool, content_id).await
                .expect("Should get content");

            // Convert to response
            let response = ContentResponse::from(content.clone());

            // Verify all fields are preserved correctly
            prop_assert_eq!(response.id, content.id, "id should be preserved");
            prop_assert_eq!(response.library_id, content.library_id, "library_id should be preserved");
            prop_assert_eq!(response.content_type, content.content_type, "content_type should be preserved");
            prop_assert_eq!(response.title, content.title, "title should be preserved");
            prop_assert_eq!(response.chapter_count, content.chapter_count, "chapter_count should be preserved");
            prop_assert_eq!(response.has_thumbnail, content.thumbnail.is_some(), "has_thumbnail should reflect thumbnail presence");
            prop_assert_eq!(response.metadata, content.metadata, "metadata should be preserved");
            prop_assert_eq!(response.created_at, content.created_at, "created_at should be preserved");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 21: Metadata JSON Blob Storage
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 21: Metadata JSON Blob Storage**
    /// **Validates: Requirements 8.4, 8.6**
    ///
    /// For any valid JSON blob from Bangumi API, storing it directly in the content
    /// record and then retrieving it should return an equivalent JSON structure.
    #[test]
    fn metadata_json_blob_storage_round_trip(
        library_name in arb_library_name(),
        content_title in arb_content_title(),
        metadata_variant in 0usize..6
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create content without metadata
            let content_id = insert_test_content(
                &pool,
                library_id,
                scan_path_id,
                &content_title,
                ContentType::Comic,
            ).await;

            // Generate various metadata JSON blobs that simulate Bangumi API responses
            let metadata: Option<serde_json::Value> = match metadata_variant {
                0 => None, // No metadata
                1 => Some(serde_json::json!({})), // Empty object
                2 => Some(serde_json::json!({
                    "id": 12345,
                    "name": "Test Manga",
                    "name_cn": "测试漫画"
                })), // Basic metadata
                3 => Some(serde_json::json!({
                    "id": 67890,
                    "name": "Complex Series",
                    "name_cn": "复杂系列",
                    "summary": "A long description of the series...",
                    "rating": {
                        "score": 8.5,
                        "total": 1234
                    },
                    "tags": [
                        {"name": "action", "count": 100},
                        {"name": "comedy", "count": 50}
                    ]
                })), // Complex nested metadata
                4 => Some(serde_json::json!({
                    "id": 11111,
                    "name": "Unicode Test 日本語",
                    "name_cn": "中文测试",
                    "summary": "Contains special chars: <>&\"'",
                    "eps": 24,
                    "air_date": "2024-01-15"
                })), // Unicode and special characters
                _ => Some(serde_json::json!({
                    "id": 99999,
                    "name": "Deeply Nested",
                    "info": {
                        "level1": {
                            "level2": {
                                "level3": {
                                    "value": "deep"
                                }
                            }
                        }
                    },
                    "array_of_objects": [
                        {"a": 1, "b": 2},
                        {"a": 3, "b": 4}
                    ],
                    "mixed_array": [1, "two", true, null, {"nested": "object"}]
                })), // Deeply nested structure
            };

            // Update content with metadata
            let updated = ContentService::update_content(&pool, content_id, None, metadata.clone()).await
                .expect("Should update metadata");

            // Verify the updated content has the correct metadata
            prop_assert_eq!(
                updated.metadata,
                metadata.clone(),
                "Updated content should have the metadata we set"
            );

            // Retrieve the content again to verify persistence
            let retrieved = ContentService::get_content(&pool, content_id).await
                .expect("Should retrieve content");

            // Verify the retrieved metadata matches what we stored
            prop_assert_eq!(
                retrieved.metadata,
                metadata,
                "Retrieved metadata should match stored metadata"
            );

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 21: Metadata JSON Blob Storage**
    /// **Validates: Requirements 8.4, 8.6**
    ///
    /// Metadata can be updated multiple times and each update should be persisted correctly.
    #[test]
    fn metadata_multiple_updates(
        library_name in arb_library_name(),
        content_title in arb_content_title()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create content
            let content_id = insert_test_content(
                &pool,
                library_id,
                scan_path_id,
                &content_title,
                ContentType::Comic,
            ).await;

            // First update: set initial metadata
            let metadata1 = Some(serde_json::json!({
                "id": 1,
                "name": "First Version"
            }));
            let updated1 = ContentService::update_content(&pool, content_id, None, metadata1.clone()).await
                .expect("Should update metadata first time");
            prop_assert_eq!(updated1.metadata, metadata1, "First update should persist");

            // Second update: change metadata
            let metadata2 = Some(serde_json::json!({
                "id": 2,
                "name": "Second Version",
                "extra_field": "added"
            }));
            let updated2 = ContentService::update_content(&pool, content_id, None, metadata2.clone()).await
                .expect("Should update metadata second time");
            prop_assert_eq!(updated2.metadata, metadata2, "Second update should persist");

            // Fourth update: set new metadata (skipping clear test as API uses partial update)
            let metadata4 = Some(serde_json::json!({
                "id": 4,
                "name": "Fourth Version"
            }));
            let updated4 = ContentService::update_content(&pool, content_id, None, metadata4.clone()).await
                .expect("Should update metadata");
            prop_assert_eq!(updated4.metadata, metadata4.clone(), "Update should persist");

            // Final verification: retrieve and check
            let final_content = ContentService::get_content(&pool, content_id).await
                .expect("Should retrieve content");
            prop_assert_eq!(final_content.metadata, metadata4, "Final retrieval should match last update");

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 21: Metadata JSON Blob Storage**
    /// **Validates: Requirements 8.4, 8.6**
    ///
    /// Metadata JSON serialization should preserve all JSON types correctly.
    #[test]
    fn metadata_preserves_json_types(
        library_name in arb_library_name(),
        content_title in arb_content_title(),
        int_val in -1000000i64..1000000,
        // Use a reasonable float range to avoid precision issues with JSON serialization
        float_val in -1e10f64..1e10f64,
        bool_val in any::<bool>(),
        string_val in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;

            // Create library and scan path
            let library_id = create_test_library(&pool, &library_name).await;
            let scan_path_id = create_test_scan_path(&pool, library_id, "/test/path").await;

            // Create content
            let content_id = insert_test_content(
                &pool,
                library_id,
                scan_path_id,
                &content_title,
                ContentType::Comic,
            ).await;

            // Create metadata with various JSON types
            let metadata = Some(serde_json::json!({
                "integer": int_val,
                "float": float_val,
                "boolean": bool_val,
                "string": string_val,
                "null_value": null,
                "array": [1, 2, 3],
                "object": {"nested": "value"}
            }));

            // Update and retrieve
            ContentService::update_content(&pool, content_id, None, metadata.clone()).await
                .expect("Should update metadata");

            let retrieved = ContentService::get_content(&pool, content_id).await
                .expect("Should retrieve content");

            // Verify each type is preserved
            let retrieved_meta = retrieved.metadata.expect("Should have metadata");

            prop_assert_eq!(
                retrieved_meta["integer"].as_i64(),
                Some(int_val),
                "Integer should be preserved"
            );

            // Float comparison with relative tolerance for floating point precision
            // JSON serialization can introduce small precision errors
            let retrieved_float = retrieved_meta["float"].as_f64().expect("Should have float");
            let tolerance = if float_val.abs() > 1.0 {
                float_val.abs() * 1e-9  // Relative tolerance for larger numbers
            } else {
                1e-9  // Absolute tolerance for small numbers
            };
            prop_assert!(
                (retrieved_float - float_val).abs() < tolerance ||
                (retrieved_float == float_val),
                "Float should be preserved (got {} expected {})",
                retrieved_float,
                float_val
            );

            prop_assert_eq!(
                retrieved_meta["boolean"].as_bool(),
                Some(bool_val),
                "Boolean should be preserved"
            );

            prop_assert_eq!(
                retrieved_meta["string"].as_str(),
                Some(string_val.as_str()),
                "String should be preserved"
            );

            prop_assert!(
                retrieved_meta["null_value"].is_null(),
                "Null should be preserved"
            );

            prop_assert!(
                retrieved_meta["array"].is_array(),
                "Array should be preserved"
            );

            prop_assert!(
                retrieved_meta["object"].is_object(),
                "Object should be preserved"
            );

            Ok(())
        })?;
    }
}
