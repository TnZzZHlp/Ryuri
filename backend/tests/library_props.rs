//! Property tests for library management.
//!
//! This module contains property-based tests for library CRUD operations,
//! scan path management, and cascade deletion behavior.

use backend::db::{DbConfig, init_db};
use backend::models::{CreateLibraryRequest, UpdateLibraryRequest};
use backend::services::library::LibraryService;
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

/// Strategy to generate valid scan intervals (0-1440 minutes).
fn arb_scan_interval() -> impl Strategy<Value = i32> {
    0i32..1440
}

/// Strategy to generate valid file paths.
fn arb_path() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_/\\-]{0,99}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Path must not be empty", |s| !s.is_empty())
}

// ============================================================================
// Property 1: Library CRUD Round-Trip
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 1: Library CRUD Round-Trip**
    /// **Validates: Requirements 1.1, 1.7**
    ///
    /// For any valid library name, scan interval, and watch mode, creating a library
    /// and then retrieving it should return a library with the same values.
    #[test]
    fn library_crud_round_trip(
        name in arb_library_name(),
        scan_interval in arb_scan_interval(),
        watch_mode in any::<bool>()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: name.clone(),
                scan_interval: Some(scan_interval),
                watch_mode: Some(watch_mode),
            };
            let created = service.create(req).await.expect("Should create library");

            // Verify created library has correct values
            prop_assert_eq!(&created.name, &name, "Created library name should match");
            prop_assert_eq!(created.scan_interval, scan_interval, "Created library scan_interval should match");
            prop_assert_eq!(created.watch_mode, watch_mode, "Created library watch_mode should match");

            // Retrieve library
            let retrieved = service.get(created.id).await
                .expect("Should retrieve library")
                .expect("Library should exist");

            // Verify retrieved library matches created
            prop_assert_eq!(retrieved.id, created.id, "Retrieved library id should match");
            prop_assert_eq!(&retrieved.name, &name, "Retrieved library name should match");
            prop_assert_eq!(retrieved.scan_interval, scan_interval, "Retrieved library scan_interval should match");
            prop_assert_eq!(retrieved.watch_mode, watch_mode, "Retrieved library watch_mode should match");

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 1: Library CRUD Round-Trip**
    /// **Validates: Requirements 1.1, 1.7**
    ///
    /// For any valid library, updating it and then retrieving should return
    /// the updated values.
    #[test]
    fn library_update_round_trip(
        initial_name in arb_library_name(),
        updated_name in arb_library_name(),
        initial_interval in arb_scan_interval(),
        updated_interval in arb_scan_interval(),
        initial_watch in any::<bool>(),
        updated_watch in any::<bool>()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: initial_name,
                scan_interval: Some(initial_interval),
                watch_mode: Some(initial_watch),
            };
            let created = service.create(req).await.expect("Should create library");

            // Update library
            let update_req = UpdateLibraryRequest {
                name: Some(updated_name.clone()),
                scan_interval: Some(updated_interval),
                watch_mode: Some(updated_watch),
            };
            let updated = service.update(created.id, update_req).await
                .expect("Should update library");

            // Verify updated values
            prop_assert_eq!(&updated.name, &updated_name, "Updated library name should match");
            prop_assert_eq!(updated.scan_interval, updated_interval, "Updated library scan_interval should match");
            prop_assert_eq!(updated.watch_mode, updated_watch, "Updated library watch_mode should match");

            // Retrieve and verify
            let retrieved = service.get(created.id).await
                .expect("Should retrieve library")
                .expect("Library should exist");

            prop_assert_eq!(&retrieved.name, &updated_name, "Retrieved library name should match updated");
            prop_assert_eq!(retrieved.scan_interval, updated_interval, "Retrieved library scan_interval should match updated");
            prop_assert_eq!(retrieved.watch_mode, updated_watch, "Retrieved library watch_mode should match updated");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 2: Scan Path Association Integrity
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 2: Scan Path Association Integrity**
    /// **Validates: Requirements 1.2**
    ///
    /// For any library and valid scan path, adding the path to the library
    /// and then listing the library's paths should include that path.
    #[test]
    fn scan_path_association_integrity(
        library_name in arb_library_name(),
        path in arb_path()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = service.create(req).await.expect("Should create library");

            // Add scan path
            let scan_path = service.add_scan_path(library.id, path.clone()).await
                .expect("Should add scan path");

            // Verify scan path has correct values
            prop_assert_eq!(scan_path.library_id, library.id, "Scan path library_id should match");
            prop_assert_eq!(&scan_path.path, &path, "Scan path should match");

            // List scan paths
            let paths = service.list_scan_paths(library.id).await
                .expect("Should list scan paths");

            // Verify the path is in the list
            let found = paths.iter().any(|p| p.path == path && p.library_id == library.id);
            prop_assert!(found, "Added scan path should be in the list");

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 2: Scan Path Association Integrity**
    /// **Validates: Requirements 1.2**
    ///
    /// For any library with multiple scan paths, all paths should be retrievable.
    #[test]
    fn multiple_scan_paths_integrity(
        library_name in arb_library_name(),
        paths in prop::collection::vec(arb_path(), 1..5)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = service.create(req).await.expect("Should create library");

            // Add unique scan paths (deduplicate)
            let unique_paths: Vec<String> = paths.into_iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            for path in &unique_paths {
                service.add_scan_path(library.id, path.clone()).await
                    .expect("Should add scan path");
            }

            // List scan paths
            let listed_paths = service.list_scan_paths(library.id).await
                .expect("Should list scan paths");

            // Verify all paths are present
            prop_assert_eq!(
                listed_paths.len(),
                unique_paths.len(),
                "Number of listed paths should match number of added paths"
            );

            for path in &unique_paths {
                let found = listed_paths.iter().any(|p| &p.path == path);
                prop_assert!(found, "Path '{}' should be in the list", path);
            }

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 3: Cascade Deletion - Scan Path Removal
// ============================================================================

/// Helper function to insert content directly into the database for testing.
async fn insert_test_content(
    pool: &Pool<Sqlite>,
    library_id: i64,
    scan_path_id: i64,
    title: &str,
) -> i64 {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        r#"
        INSERT INTO contents (library_id, scan_path_id, content_type, title, folder_path, chapter_count, created_at, updated_at)
        VALUES (?, ?, 'Comic', ?, ?, 0, ?, ?)
        "#,
    )
    .bind(library_id)
    .bind(scan_path_id)
    .bind(title)
    .bind(format!("/path/to/{}", title))
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Should insert test content");

    result.last_insert_rowid()
}

/// Helper function to count contents for a scan path.
async fn count_contents_for_scan_path(pool: &Pool<Sqlite>, scan_path_id: i64) -> i64 {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents WHERE scan_path_id = ?")
        .bind(scan_path_id)
        .fetch_one(pool)
        .await
        .expect("Should count contents");

    result.0
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 3: Cascade Deletion - Scan Path Removal**
    /// **Validates: Requirements 1.3**
    ///
    /// For any library with scan paths and associated contents, removing a scan path
    /// should delete all contents that were imported from that path while preserving
    /// contents from other paths.
    #[test]
    fn cascade_deletion_scan_path_removal(
        library_name in arb_library_name(),
        path1 in arb_path(),
        path2 in arb_path().prop_filter("Paths must be different", |p| p != &"path1")
    ) {
        // Ensure paths are different
        prop_assume!(path1 != path2);

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = service.create(req).await.expect("Should create library");

            // Add two scan paths
            let scan_path1 = service.add_scan_path(library.id, path1.clone()).await
                .expect("Should add scan path 1");
            let scan_path2 = service.add_scan_path(library.id, path2.clone()).await
                .expect("Should add scan path 2");

            // Add content to both paths
            insert_test_content(&pool, library.id, scan_path1.id, "Content1").await;
            insert_test_content(&pool, library.id, scan_path1.id, "Content2").await;
            insert_test_content(&pool, library.id, scan_path2.id, "Content3").await;

            // Verify initial content counts
            let count1_before = count_contents_for_scan_path(&pool, scan_path1.id).await;
            let count2_before = count_contents_for_scan_path(&pool, scan_path2.id).await;
            prop_assert_eq!(count1_before, 2, "Should have 2 contents in path 1");
            prop_assert_eq!(count2_before, 1, "Should have 1 content in path 2");

            // Remove scan path 1
            service.remove_scan_path(library.id, scan_path1.id).await
                .expect("Should remove scan path 1");

            // Verify contents from path 1 are deleted
            let count1_after = count_contents_for_scan_path(&pool, scan_path1.id).await;
            prop_assert_eq!(count1_after, 0, "Contents from path 1 should be deleted");

            // Verify contents from path 2 are preserved
            let count2_after = count_contents_for_scan_path(&pool, scan_path2.id).await;
            prop_assert_eq!(count2_after, 1, "Contents from path 2 should be preserved");

            // Verify scan path 1 is removed from list
            let paths = service.list_scan_paths(library.id).await
                .expect("Should list scan paths");
            let path1_exists = paths.iter().any(|p| p.id == scan_path1.id);
            prop_assert!(!path1_exists, "Scan path 1 should be removed from list");

            // Verify scan path 2 still exists
            let path2_exists = paths.iter().any(|p| p.id == scan_path2.id);
            prop_assert!(path2_exists, "Scan path 2 should still exist");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 4: Library Statistics Accuracy
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 4: Library Statistics Accuracy**
    /// **Validates: Requirements 1.4**
    ///
    /// For any library with scan paths and contents, the library statistics
    /// (path count, content count) should match the actual number of associated
    /// records in the database.
    #[test]
    fn library_statistics_accuracy(
        library_name in arb_library_name(),
        num_paths in 1usize..5,
        contents_per_path in 0usize..4
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = service.create(req).await.expect("Should create library");

            // Add scan paths and contents
            let mut total_contents = 0;
            for i in 0..num_paths {
                let path = format!("/test/path/{}", i);
                let scan_path = service.add_scan_path(library.id, path).await
                    .expect("Should add scan path");

                for j in 0..contents_per_path {
                    let title = format!("Content_{}_{}", i, j);
                    insert_test_content(&pool, library.id, scan_path.id, &title).await;
                    total_contents += 1;
                }
            }

            // Get library with stats
            let stats = service.get_with_stats(library.id).await
                .expect("Should get library with stats")
                .expect("Library should exist");

            // Verify statistics
            prop_assert_eq!(
                stats.path_count as usize,
                num_paths,
                "Path count should match number of added paths"
            );
            prop_assert_eq!(
                stats.content_count as usize,
                total_contents,
                "Content count should match number of added contents"
            );

            // Also verify via list
            let libraries = service.list().await.expect("Should list libraries");
            let lib_stats = libraries.iter().find(|l| l.library.id == library.id)
                .expect("Library should be in list");

            prop_assert_eq!(
                lib_stats.path_count as usize,
                num_paths,
                "Listed path count should match"
            );
            prop_assert_eq!(
                lib_stats.content_count as usize,
                total_contents,
                "Listed content count should match"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 6: Cascade Deletion - Library Removal
// ============================================================================

/// Helper function to count all scan paths for a library.
async fn count_scan_paths_for_library(pool: &Pool<Sqlite>, library_id: i64) -> i64 {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scan_paths WHERE library_id = ?")
        .bind(library_id)
        .fetch_one(pool)
        .await
        .expect("Should count scan paths");

    result.0
}

/// Helper function to count all contents for a library.
async fn count_contents_for_library(pool: &Pool<Sqlite>, library_id: i64) -> i64 {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents WHERE library_id = ?")
        .bind(library_id)
        .fetch_one(pool)
        .await
        .expect("Should count contents");

    result.0
}

/// Helper function to check if a library exists.
async fn library_exists(pool: &Pool<Sqlite>, library_id: i64) -> bool {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM libraries WHERE id = ?")
        .bind(library_id)
        .fetch_one(pool)
        .await
        .expect("Should check library existence");

    result.0 > 0
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 6: Cascade Deletion - Library Removal**
    /// **Validates: Requirements 1.6**
    ///
    /// For any library with scan paths and contents, deleting the library should
    /// remove all associated scan paths and contents from the database.
    #[test]
    fn cascade_deletion_library_removal(
        library_name in arb_library_name(),
        num_paths in 1usize..4,
        contents_per_path in 1usize..3
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let service = LibraryService::new(pool.clone());

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = service.create(req).await.expect("Should create library");
            let library_id = library.id;

            // Add scan paths and contents
            let mut scan_path_ids = Vec::new();
            for i in 0..num_paths {
                let path = format!("/test/path/{}", i);
                let scan_path = service.add_scan_path(library_id, path).await
                    .expect("Should add scan path");
                scan_path_ids.push(scan_path.id);

                for j in 0..contents_per_path {
                    let title = format!("Content_{}_{}", i, j);
                    insert_test_content(&pool, library_id, scan_path.id, &title).await;
                }
            }

            // Verify data exists before deletion
            let paths_before = count_scan_paths_for_library(&pool, library_id).await;
            let contents_before = count_contents_for_library(&pool, library_id).await;
            prop_assert!(paths_before > 0, "Should have scan paths before deletion");
            prop_assert!(contents_before > 0, "Should have contents before deletion");
            prop_assert!(library_exists(&pool, library_id).await, "Library should exist before deletion");

            // Delete library
            service.delete(library_id).await.expect("Should delete library");

            // Verify library is deleted
            prop_assert!(!library_exists(&pool, library_id).await, "Library should not exist after deletion");

            // Verify all scan paths are deleted
            let paths_after = count_scan_paths_for_library(&pool, library_id).await;
            prop_assert_eq!(paths_after, 0, "All scan paths should be deleted");

            // Verify all contents are deleted
            let contents_after = count_contents_for_library(&pool, library_id).await;
            prop_assert_eq!(contents_after, 0, "All contents should be deleted");

            // Verify library is not in list
            let libraries = service.list().await.expect("Should list libraries");
            let lib_exists = libraries.iter().any(|l| l.library.id == library_id);
            prop_assert!(!lib_exists, "Library should not be in list after deletion");

            Ok(())
        })?;
    }
}
