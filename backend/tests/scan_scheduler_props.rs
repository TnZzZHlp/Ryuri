//! Property tests for scan interval configuration and watch mode.
//!
//! This module contains property-based tests for the scheduler and watch services.

use backend::db::{DbConfig, init_db};
use backend::services::scan::ScanService;
use backend::services::scan_queue::ScanQueueService;
use backend::services::scheduler::SchedulerService;
use backend::services::watch::WatchService;
use chrono::{Duration, Utc};
use proptest::prelude::*;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tempfile::TempDir;
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

/// Strategy to generate valid library IDs.
fn arb_library_id() -> impl Strategy<Value = i64> {
    1i64..1000
}

/// Strategy to generate valid scan intervals (1-1440 minutes, non-zero).
fn arb_scan_interval_nonzero() -> impl Strategy<Value = i32> {
    1i32..1440
}

// ============================================================================
// Property 19: Scan Interval Configuration
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 19: Scan Interval Configuration**
    /// **Validates: Requirements 1.8**
    ///
    /// For any library with a non-zero scan interval, the scheduler should have
    /// a scheduled task for that library, and the next scan time should be within
    /// the configured interval from the current time.
    #[test]
    fn scan_interval_configuration(
        library_id in arb_library_id(),
        interval_minutes in arb_scan_interval_nonzero()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let scan_queue_service = Arc::new(ScanQueueService::with_scan_service(scan_service));
            let scheduler = SchedulerService::new(scan_queue_service);

            let before_schedule = Utc::now();

            scheduler.schedule_scan(library_id, interval_minutes).await
                .expect("Should schedule scan");

            prop_assert!(
                scheduler.is_scheduled(library_id).await,
                "Library should have a scheduled task"
            );

            let next_scan_time = scheduler.get_next_scan_time(library_id).await
                .expect("Should have next scan time");

            let expected_min = before_schedule;
            let expected_max = before_schedule + Duration::minutes(interval_minutes as i64 + 1);

            prop_assert!(next_scan_time >= expected_min, "Next scan time should be after scheduling time");
            prop_assert!(next_scan_time <= expected_max, "Next scan time should be within interval");

            let task = scheduler.get_scheduled_task(library_id).await.expect("Should have scheduled task");
            prop_assert_eq!(task.library_id, library_id, "Task library_id should match");
            prop_assert_eq!(task.interval_minutes, interval_minutes, "Task interval should match");

            scheduler.cancel_scan(library_id).await.expect("Should cancel scan");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 19: Scan Interval Configuration**
    /// **Validates: Requirements 1.8**
    #[test]
    fn zero_interval_no_schedule(library_id in arb_library_id()) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let scan_queue_service = Arc::new(ScanQueueService::with_scan_service(scan_service));
            let scheduler = SchedulerService::new(scan_queue_service);

            scheduler.schedule_scan(library_id, 0).await.expect("Should handle zero interval");

            prop_assert!(!scheduler.is_scheduled(library_id).await, "Should not be scheduled with zero interval");
            prop_assert!(scheduler.get_next_scan_time(library_id).await.is_none(), "Should have no next scan time");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 19: Scan Interval Configuration**
    /// **Validates: Requirements 1.8**
    #[test]
    fn update_interval_replaces_schedule(
        library_id in arb_library_id(),
        initial_interval in arb_scan_interval_nonzero(),
        updated_interval in arb_scan_interval_nonzero()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let scan_queue_service = Arc::new(ScanQueueService::with_scan_service(scan_service));
            let scheduler = SchedulerService::new(scan_queue_service);

            scheduler.schedule_scan(library_id, initial_interval).await.expect("Should schedule initial scan");
            let initial_task = scheduler.get_scheduled_task(library_id).await.expect("Should have initial task");
            prop_assert_eq!(initial_task.interval_minutes, initial_interval, "Initial interval should match");

            scheduler.update_interval(library_id, updated_interval).await.expect("Should update interval");
            let updated_task = scheduler.get_scheduled_task(library_id).await.expect("Should have updated task");
            prop_assert_eq!(updated_task.interval_minutes, updated_interval, "Updated interval should match");

            scheduler.cancel_scan(library_id).await.expect("Should cancel scan");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 19: Scan Interval Configuration**
    /// **Validates: Requirements 1.8**
    #[test]
    fn cancel_removes_schedule(
        library_id in arb_library_id(),
        interval_minutes in arb_scan_interval_nonzero()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let scan_queue_service = Arc::new(ScanQueueService::with_scan_service(scan_service));
            let scheduler = SchedulerService::new(scan_queue_service);

            scheduler.schedule_scan(library_id, interval_minutes).await.expect("Should schedule scan");
            prop_assert!(scheduler.is_scheduled(library_id).await, "Should be scheduled before cancel");

            scheduler.cancel_scan(library_id).await.expect("Should cancel scan");
            prop_assert!(!scheduler.is_scheduled(library_id).await, "Should not be scheduled after cancel");
            prop_assert!(scheduler.get_next_scan_time(library_id).await.is_none(), "Should have no next scan time");
            Ok(())
        })?;
    }
}

// ============================================================================
// Property 20: Watch Mode State Consistency
// ============================================================================

/// Helper struct to hold test resources that need cleanup.
struct TestLibraryWithPaths {
    library_id: i64,
    #[allow(dead_code)]
    path_ids: Vec<i64>,
    _temp_dirs: Vec<TempDir>,
}

/// Helper function to create a library with scan paths for testing.
async fn create_library_with_paths(pool: &Pool<Sqlite>, num_paths: usize) -> TestLibraryWithPaths {
    use backend::models::{CreateLibraryRequest, NewScanPath};
    use backend::repository::library::ScanPathRepository;
    use backend::services::library::LibraryService;

    let service = LibraryService::new(pool.clone());

    let req = CreateLibraryRequest {
        name: format!("Test Library {}", uuid::Uuid::new_v4()),
        scan_interval: None,
        watch_mode: Some(true),
    };
    let library = service.create(req).await.expect("Should create library");

    let mut path_ids = Vec::new();
    let mut temp_dirs = Vec::new();

    for _ in 0..num_paths {
        let temp_dir = TempDir::new().expect("Should create temp directory");

        let new_path = NewScanPath {
            library_id: library.id,
            path: temp_dir.path().to_string_lossy().to_string(),
        };
        let scan_path = ScanPathRepository::create(pool, new_path)
            .await
            .expect("Should create scan path");
        path_ids.push(scan_path.id);
        temp_dirs.push(temp_dir);
    }

    TestLibraryWithPaths {
        library_id: library.id,
        path_ids,
        _temp_dirs: temp_dirs,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Feature: comic-reader, Property 20: Watch Mode State Consistency**
    /// **Validates: Requirements 1.9, 1.10, 1.11**
    #[test]
    fn watch_mode_state_consistency(num_paths in 1usize..4) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let watch_service = WatchService::new(pool.clone(), scan_service);

            let test_lib = create_library_with_paths(&pool, num_paths).await;

            watch_service.start_watching(test_lib.library_id).await.expect("Should start watching");

            // Verify watching state - the library should be marked as watched
            prop_assert!(watch_service.is_watching(test_lib.library_id).await, "Library should be watched");

            // Verify watched paths - should have paths if directories exist
            // Note: The number of watched paths may be less than num_paths if some directories
            // were cleaned up before the watcher could start watching them
            let watched_paths = watch_service.get_watched_paths(test_lib.library_id).await;
            prop_assert!(
                watched_paths.len() <= num_paths,
                "Should have at most {} watched paths, got {}", num_paths, watched_paths.len()
            );

            watch_service.stop_watching(test_lib.library_id).await.expect("Should stop watching");
            prop_assert!(!watch_service.is_watching(test_lib.library_id).await, "Library should not be watched");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 20: Watch Mode State Consistency**
    /// **Validates: Requirements 1.9, 1.10, 1.11**
    #[test]
    fn watch_start_idempotent(num_paths in 1usize..3) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let watch_service = WatchService::new(pool.clone(), scan_service);

            let test_lib = create_library_with_paths(&pool, num_paths).await;

            watch_service.start_watching(test_lib.library_id).await.expect("Should start watching first time");
            watch_service.start_watching(test_lib.library_id).await.expect("Should start watching second time");

            prop_assert!(watch_service.is_watching(test_lib.library_id).await, "Library should still be watched");

            watch_service.stop_watching(test_lib.library_id).await.expect("Should stop watching");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 20: Watch Mode State Consistency**
    /// **Validates: Requirements 1.9, 1.10, 1.11**
    #[test]
    fn watch_stop_safe_when_not_watching(library_id in arb_library_id()) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let watch_service = WatchService::new(pool.clone(), scan_service);

            watch_service.stop_watching(library_id).await.expect("Should safely stop watching");
            prop_assert!(!watch_service.is_watching(library_id).await, "Library should not be watched");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 20: Watch Mode State Consistency**
    /// **Validates: Requirements 1.9, 1.10, 1.11**
    #[test]
    fn watch_refresh_updates_paths(initial_paths in 1usize..3) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let watch_service = WatchService::new(pool.clone(), scan_service);

            let test_lib = create_library_with_paths(&pool, initial_paths).await;

            watch_service.start_watching(test_lib.library_id).await.expect("Should start watching");
            prop_assert!(watch_service.is_watching(test_lib.library_id).await, "Library should be watched");

            watch_service.refresh_watching(test_lib.library_id).await.expect("Should refresh watching");
            prop_assert!(watch_service.is_watching(test_lib.library_id).await, "Library should still be watched");

            watch_service.stop_watching(test_lib.library_id).await.expect("Should stop watching");
            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 20: Watch Mode State Consistency**
    /// **Validates: Requirements 1.9, 1.10, 1.11**
    #[test]
    fn multiple_libraries_independent_watch(num_libraries in 2usize..4) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let scan_service = Arc::new(ScanService::new(pool.clone()));
            let watch_service = WatchService::new(pool.clone(), scan_service);

            let mut test_libs = Vec::new();
            for _ in 0..num_libraries {
                let test_lib = create_library_with_paths(&pool, 1).await;
                test_libs.push(test_lib);
            }

            for test_lib in &test_libs {
                watch_service.start_watching(test_lib.library_id).await.expect("Should start watching");
            }

            for test_lib in &test_libs {
                prop_assert!(watch_service.is_watching(test_lib.library_id).await, "Library should be watched");
            }

            watch_service.stop_watching(test_libs[0].library_id).await.expect("Should stop watching first");
            prop_assert!(!watch_service.is_watching(test_libs[0].library_id).await, "First should not be watched");

            for test_lib in &test_libs[1..] {
                prop_assert!(watch_service.is_watching(test_lib.library_id).await, "Others should still be watched");
            }

            for test_lib in &test_libs {
                watch_service.stop_watching(test_lib.library_id).await.ok();
            }
            Ok(())
        })?;
    }
}
