//! Property tests for reading progress management.
//!
//! This module contains property-based tests for progress tracking,
//! percentage calculation, and user isolation.

use backend::db::{DbConfig, init_db};
use backend::models::CreateLibraryRequest;
use backend::repository::content::{ChapterRepository, ContentRepository};
use backend::services::library::LibraryService;
use backend::services::progress::ProgressService;
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

/// Create a test user in the database.
async fn create_test_user(pool: &Pool<Sqlite>, username: &str) -> i64 {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        r#"
        INSERT INTO users (username, password_hash, created_at, updated_at)
        VALUES (?, 'test_hash', ?, ?)
        "#,
    )
    .bind(username)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Should create test user");

    result.last_insert_rowid()
}

/// Create a test library with a scan path.
async fn create_test_library_with_path(pool: &Pool<Sqlite>) -> (i64, i64) {
    let service = LibraryService::new(pool.clone());
    let library = service
        .create(CreateLibraryRequest {
            name: "Test Library".to_string(),
            scan_interval: None,
            watch_mode: None,
        })
        .await
        .expect("Should create library");

    let scan_path = service
        .add_scan_path(library.id, "/test/path".to_string())
        .await
        .expect("Should add scan path");

    (library.id, scan_path.id)
}

/// Create test content with chapters.
async fn create_test_content_with_chapters(
    pool: &Pool<Sqlite>,
    library_id: i64,
    scan_path_id: i64,
    num_chapters: i32,
) -> (i64, Vec<i64>) {
    use backend::models::{ContentType, NewChapter, NewContent};

    let content = ContentRepository::create(
        pool,
        NewContent {
            library_id,
            scan_path_id,
            content_type: ContentType::Comic,
            title: "Test Content".to_string(),
            folder_path: format!(
                "/test/content_{}",
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
            chapter_count: num_chapters,
            thumbnail: None,
            metadata: None,
        },
    )
    .await
    .expect("Should create content");

    let mut chapter_ids = Vec::new();
    for i in 0..num_chapters {
        let chapter = ChapterRepository::create(
            pool,
            NewChapter {
                content_id: content.id,
                title: format!("Chapter {}", i + 1),
                file_path: format!("/test/chapter_{}.cbz", i),
                sort_order: i,
                page_count: 10,
                size: 1024,
            },
        )
        .await
        .expect("Should create chapter");
        chapter_ids.push(chapter.id);
    }

    (content.id, chapter_ids)
}

/// Strategy to generate valid positions (0-1000).
fn arb_position() -> impl Strategy<Value = i32> {
    0i32..1000
}

/// Strategy to generate valid percentages (0.0-100.0).
fn arb_percentage() -> impl Strategy<Value = f32> {
    (0u32..=10000).prop_map(|v| v as f32 / 100.0)
}



// ============================================================================
// Property 13: Progress Persistence Round-Trip
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 13: Progress Persistence Round-Trip**
    /// **Validates: Requirements 3.5, 4.5, 5.1**
    ///
    /// For any content and valid progress data, saving the progress and then
    /// retrieving it should return equivalent progress values.
    #[test]
    fn progress_persistence_round_trip(
        position in arb_position(),
        percentage in arb_percentage()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create test data
            let user_id = create_test_user(&pool, "test_user").await;
            let (library_id, scan_path_id) = create_test_library_with_path(&pool).await;
            let (_, chapter_ids) = create_test_content_with_chapters(&pool, library_id, scan_path_id, 1).await;
            let chapter_id = chapter_ids[0];

            // Save progress
            let saved = service
                .update_progress_with_percentage(user_id, chapter_id, position, percentage)
                .await
                .expect("Should save progress");

            // Verify saved values
            prop_assert_eq!(saved.user_id, user_id, "Saved user_id should match");
            prop_assert_eq!(saved.chapter_id, chapter_id, "Saved chapter_id should match");
            prop_assert_eq!(saved.position, position, "Saved position should match");
            prop_assert!((saved.percentage - percentage).abs() < 0.01, "Saved percentage should match");

            // Retrieve progress
            let retrieved = service
                .get_chapter_progress(user_id, chapter_id)
                .await
                .expect("Should retrieve progress")
                .expect("Progress should exist");

            // Verify retrieved values match saved
            prop_assert_eq!(retrieved.user_id, user_id, "Retrieved user_id should match");
            prop_assert_eq!(retrieved.chapter_id, chapter_id, "Retrieved chapter_id should match");
            prop_assert_eq!(retrieved.position, position, "Retrieved position should match");
            prop_assert!((retrieved.percentage - percentage).abs() < 0.01, "Retrieved percentage should match");

            Ok(())
        })?;
    }


    /// **Feature: comic-reader, Property 13: Progress Persistence Round-Trip**
    /// **Validates: Requirements 3.5, 4.5, 5.1**
    ///
    /// For any content, updating progress multiple times should always reflect
    /// the latest values.
    #[test]
    fn progress_update_overwrites_previous(
        position1 in arb_position(),
        position2 in arb_position(),
        percentage1 in arb_percentage(),
        percentage2 in arb_percentage()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create test data
            let user_id = create_test_user(&pool, "test_user").await;
            let (library_id, scan_path_id) = create_test_library_with_path(&pool).await;
            let (_, chapter_ids) = create_test_content_with_chapters(&pool, library_id, scan_path_id, 1).await;
            let chapter_id = chapter_ids[0];

            // Save first progress
            service
                .update_progress_with_percentage(user_id, chapter_id, position1, percentage1)
                .await
                .expect("Should save first progress");

            // Save second progress (should overwrite)
            let updated = service
                .update_progress_with_percentage(user_id, chapter_id, position2, percentage2)
                .await
                .expect("Should save second progress");

            // Verify updated values are the second set
            prop_assert_eq!(updated.position, position2, "Updated position should be second value");
            prop_assert!((updated.percentage - percentage2).abs() < 0.01, "Updated percentage should be second value");

            // Retrieve and verify
            let retrieved = service
                .get_chapter_progress(user_id, chapter_id)
                .await
                .expect("Should retrieve progress")
                .expect("Progress should exist");

            prop_assert_eq!(retrieved.position, position2, "Retrieved position should be second value");
            prop_assert!((retrieved.percentage - percentage2).abs() < 0.01, "Retrieved percentage should be second value");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 14: Progress Percentage Accuracy
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 14: Progress Percentage Accuracy**
    /// **Validates: Requirements 5.4**
    ///
    /// For any content with reading progress, the percentage should accurately
    /// reflect the position relative to the total content.
    #[test]
    fn progress_percentage_calculation_accuracy(
        position in 0i32..100,
        total in 1i32..100
    ) {
        // Calculate expected percentage
        let expected_percentage = (position as f32 / total as f32) * 100.0;
        let expected_clamped = expected_percentage.clamp(0.0, 100.0);

        // Use the service's calculation function
        let calculated = ProgressService::calculate_percentage_from_total(position, total);

        // Verify calculation is accurate
        prop_assert!(
            (calculated - expected_clamped).abs() < 0.01,
            "Calculated percentage {} should match expected {}",
            calculated,
            expected_clamped
        );

        // Verify percentage is within valid range
        prop_assert!(calculated >= 0.0, "Percentage should be >= 0");
        prop_assert!(calculated <= 100.0, "Percentage should be <= 100");
    }

    /// **Feature: comic-reader, Property 14: Progress Percentage Accuracy**
    /// **Validates: Requirements 5.4**
    ///
    /// For any content with multiple chapters, overall percentage should
    /// accurately reflect completed chapters plus partial progress.
    #[test]
    fn overall_percentage_calculation_accuracy(
        completed in 0i32..10,
        total in 1i32..10,
        current_percentage in arb_percentage()
    ) {
        // Ensure completed <= total
        let completed = completed.min(total);

        // Calculate expected overall percentage
        let base = (completed as f32 / total as f32) * 100.0;
        let partial = current_percentage / total as f32;
        let expected = (base + partial).clamp(0.0, 100.0);

        // Use the service's calculation function
        let calculated = ProgressService::calculate_overall_percentage(
            completed,
            total,
            current_percentage,
        );

        // Verify calculation is accurate
        prop_assert!(
            (calculated - expected).abs() < 0.01,
            "Calculated overall percentage {} should match expected {}",
            calculated,
            expected
        );

        // Verify percentage is within valid range
        prop_assert!(calculated >= 0.0, "Overall percentage should be >= 0");
        prop_assert!(calculated <= 100.0, "Overall percentage should be <= 100");
    }



}

// ============================================================================
// Property 25: Progress User Isolation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 25: Progress User Isolation**
    /// **Validates: Requirements 9.7**
    ///
    /// For any two different users reading the same content, their reading
    /// progress should be stored and retrieved independently.
    #[test]
    fn progress_user_isolation(
        position1 in arb_position(),
        position2 in arb_position(),
        percentage1 in arb_percentage(),
        percentage2 in arb_percentage()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create two test users
            let user1_id = create_test_user(&pool, "user1").await;
            let user2_id = create_test_user(&pool, "user2").await;

            // Create shared content
            let (library_id, scan_path_id) = create_test_library_with_path(&pool).await;
            let (_, chapter_ids) = create_test_content_with_chapters(&pool, library_id, scan_path_id, 1).await;
            let chapter_id = chapter_ids[0];

            // Save progress for user 1
            service
                .update_progress_with_percentage(user1_id, chapter_id, position1, percentage1)
                .await
                .expect("Should save user1 progress");

            // Save progress for user 2
            service
                .update_progress_with_percentage(user2_id, chapter_id, position2, percentage2)
                .await
                .expect("Should save user2 progress");

            // Retrieve user 1's progress
            let user1_progress = service
                .get_chapter_progress(user1_id, chapter_id)
                .await
                .expect("Should retrieve user1 progress")
                .expect("User1 progress should exist");

            // Retrieve user 2's progress
            let user2_progress = service
                .get_chapter_progress(user2_id, chapter_id)
                .await
                .expect("Should retrieve user2 progress")
                .expect("User2 progress should exist");

            // Verify user 1's progress is independent
            prop_assert_eq!(user1_progress.user_id, user1_id, "User1 progress should belong to user1");
            prop_assert_eq!(user1_progress.position, position1, "User1 position should match");
            prop_assert!((user1_progress.percentage - percentage1).abs() < 0.01, "User1 percentage should match");

            // Verify user 2's progress is independent
            prop_assert_eq!(user2_progress.user_id, user2_id, "User2 progress should belong to user2");
            prop_assert_eq!(user2_progress.position, position2, "User2 position should match");
            prop_assert!((user2_progress.percentage - percentage2).abs() < 0.01, "User2 percentage should match");

            // Verify they are different records
            prop_assert_ne!(user1_progress.id, user2_progress.id, "Progress records should be different");

            Ok(())
        })?;
    }


    /// **Feature: comic-reader, Property 25: Progress User Isolation**
    /// **Validates: Requirements 9.7**
    ///
    /// For any user, updating their progress should not affect other users' progress.
    #[test]
    fn progress_update_does_not_affect_other_users(
        initial_position in arb_position(),
        initial_percentage in arb_percentage(),
        updated_position in arb_position(),
        updated_percentage in arb_percentage()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create two test users
            let user1_id = create_test_user(&pool, "user1").await;
            let user2_id = create_test_user(&pool, "user2").await;

            // Create shared content
            let (library_id, scan_path_id) = create_test_library_with_path(&pool).await;
            let (_, chapter_ids) = create_test_content_with_chapters(&pool, library_id, scan_path_id, 1).await;
            let chapter_id = chapter_ids[0];

            // Save initial progress for both users
            service
                .update_progress_with_percentage(user1_id, chapter_id, initial_position, initial_percentage)
                .await
                .expect("Should save user1 initial progress");

            service
                .update_progress_with_percentage(user2_id, chapter_id, initial_position, initial_percentage)
                .await
                .expect("Should save user2 initial progress");

            // Update user 1's progress
            service
                .update_progress_with_percentage(user1_id, chapter_id, updated_position, updated_percentage)
                .await
                .expect("Should update user1 progress");

            // Verify user 2's progress is unchanged
            let user2_progress = service
                .get_chapter_progress(user2_id, chapter_id)
                .await
                .expect("Should retrieve user2 progress")
                .expect("User2 progress should exist");

            prop_assert_eq!(
                user2_progress.position,
                initial_position,
                "User2 position should be unchanged"
            );
            prop_assert!(
                (user2_progress.percentage - initial_percentage).abs() < 0.01,
                "User2 percentage should be unchanged"
            );

            // Verify user 1's progress is updated
            let user1_progress = service
                .get_chapter_progress(user1_id, chapter_id)
                .await
                .expect("Should retrieve user1 progress")
                .expect("User1 progress should exist");

            prop_assert_eq!(
                user1_progress.position,
                updated_position,
                "User1 position should be updated"
            );
            prop_assert!(
                (user1_progress.percentage - updated_percentage).abs() < 0.01,
                "User1 percentage should be updated"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 16: Progress Validation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 16: Progress Validation**
    /// **Validates: Requirements 7.4**
    ///
    /// For any progress update request, invalid data (negative position,
    /// percentage > 100, non-existent content_id) should be rejected with
    /// an appropriate error response.
    #[test]
    fn progress_validation_rejects_negative_position(
        negative_position in -1000i32..-1,
        percentage in arb_percentage()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create test data
            let user_id = create_test_user(&pool, "test_user").await;
            let (library_id, scan_path_id) = create_test_library_with_path(&pool).await;
            let (_, chapter_ids) = create_test_content_with_chapters(&pool, library_id, scan_path_id, 1).await;
            let chapter_id = chapter_ids[0];

            // Try to save progress with negative position
            let result = service
                .update_progress_with_percentage(user_id, chapter_id, negative_position, percentage)
                .await;

            // Should fail with BadRequest error
            prop_assert!(result.is_err(), "Should reject negative position");
            let err = result.unwrap_err();
            prop_assert!(
                matches!(err, backend::error::AppError::BadRequest(_)),
                "Error should be BadRequest"
            );

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 16: Progress Validation**
    /// **Validates: Requirements 7.4**
    ///
    /// Progress update should reject percentage values outside 0-100 range.
    #[test]
    fn progress_validation_rejects_invalid_percentage(
        position in arb_position(),
        invalid_percentage in prop::sample::select(vec![-10.0f32, -1.0, 100.1, 150.0, 200.0])
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create test data
            let user_id = create_test_user(&pool, "test_user").await;
            let (library_id, scan_path_id) = create_test_library_with_path(&pool).await;
            let (_, chapter_ids) = create_test_content_with_chapters(&pool, library_id, scan_path_id, 1).await;
            let chapter_id = chapter_ids[0];

            // Try to save progress with invalid percentage
            let result = service
                .update_progress_with_percentage(user_id, chapter_id, position, invalid_percentage)
                .await;

            // Should fail with BadRequest error
            prop_assert!(result.is_err(), "Should reject invalid percentage {}", invalid_percentage);
            let err = result.unwrap_err();
            prop_assert!(
                matches!(err, backend::error::AppError::BadRequest(_)),
                "Error should be BadRequest"
            );

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 16: Progress Validation**
    /// **Validates: Requirements 7.4**
    ///
    /// Progress update should reject non-existent chapter IDs.
    #[test]
    fn progress_validation_rejects_nonexistent_chapter(
        position in arb_position(),
        percentage in arb_percentage(),
        fake_chapter_id in 10000i64..20000
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = ProgressService::new(pool.clone());

            // Create test user only (no content)
            let user_id = create_test_user(&pool, "test_user").await;

            // Try to save progress for non-existent chapter
            let result = service
                .update_progress_with_percentage(user_id, fake_chapter_id, position, percentage)
                .await;

            // Should fail with NotFound error
            prop_assert!(result.is_err(), "Should reject non-existent chapter");
            let err = result.unwrap_err();
            prop_assert!(
                matches!(err, backend::error::AppError::NotFound(_)),
                "Error should be NotFound"
            );

            Ok(())
        })?;
    }
}
