//! Property tests for scan service.
//!
//! This module contains property-based tests for content scanning,
//! title derivation, and scan path associations.

use backend::db::{DbConfig, init_db};
use backend::models::CreateLibraryRequest;
use backend::services::library::LibraryService;
use backend::services::scan_queue::ScanService;
use proptest::prelude::*;
use sqlx::{Pool, Sqlite};
use std::fs;
use std::path::{Path, PathBuf};
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

/// Strategy to generate valid folder names (content titles).
/// Folder names should be valid filesystem names.
fn arb_folder_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_ -]{0,30}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Name must not be empty", |s| !s.is_empty())
}

/// Strategy to generate valid library names.
fn arb_library_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_ ]{0,49}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("Name must not be empty", |s| !s.is_empty())
}

/// Create a test directory structure with a content folder containing a dummy archive.
fn create_test_content_folder(base_dir: &Path, folder_name: &str) -> PathBuf {
    let content_folder = base_dir.join(folder_name);
    fs::create_dir_all(&content_folder).expect("Should create content folder");

    // Create a minimal ZIP file as a chapter
    let chapter_path = content_folder.join("chapter01.zip");
    create_minimal_zip(&chapter_path);

    content_folder
}

/// Create a minimal valid ZIP file with a dummy image.
fn create_minimal_zip(path: &Path) {
    use std::io::Write;

    let file = fs::File::create(path).expect("Should create ZIP file");
    let mut zip = zip::ZipWriter::new(file);

    // Add a minimal PNG image (1x1 pixel)
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("page001.png", options)
        .expect("Should start file in ZIP");

    // Minimal PNG: 1x1 transparent pixel
    let png_data: [u8; 69] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77,
        0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC,
        0x59, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    zip.write_all(&png_data).expect("Should write PNG data");
    zip.finish().expect("Should finish ZIP");
}

// ============================================================================
// Property 8: Content Title Derivation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 8: Content Title Derivation**
    /// **Validates: Requirements 2.4**
    ///
    /// For any imported content, the content title should equal the folder name
    /// from which it was imported.
    #[test]
    fn content_title_derivation(
        library_name in arb_library_name(),
        folder_name in arb_folder_name()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let library_service = LibraryService::new(pool.clone());
            let scan_service = ScanService::new(pool.clone());

            // Create a temporary directory for the test
            let temp_dir = TempDir::new().expect("Should create temp dir");
            let base_path = temp_dir.path().to_path_buf();

            // Create a content folder with the generated name
            create_test_content_folder(&base_path, &folder_name);

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = library_service.create(req).await.expect("Should create library");

            // Add scan path
            library_service
                .add_scan_path(library.id, base_path.to_string_lossy().to_string())
                .await
                .expect("Should add scan path");

            // Scan the library
            let result = scan_service.scan_library(library.id).await
                .expect("Should scan library");

            // Verify content was added
            prop_assert_eq!(result.added.len(), 1, "Should have added exactly one content");

            // Verify the content title matches the folder name
            let content = &result.added[0];
            prop_assert_eq!(
                &content.title,
                &folder_name,
                "Content title should equal folder name"
            );



            Ok(())
        })?;
    }
}

// ============================================================================
// Property 9: Content-ScanPath Association
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 9: Content-ScanPath Association**
    /// **Validates: Requirements 2.7**
    ///
    /// For any imported content, the content should have a valid scan_path_id
    /// that references the scan path from which it was imported.
    #[test]
    fn content_scanpath_association(
        library_name in arb_library_name(),
        folder_name in arb_folder_name()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let library_service = LibraryService::new(pool.clone());
            let scan_service = ScanService::new(pool.clone());

            // Create a temporary directory for the test
            let temp_dir = TempDir::new().expect("Should create temp dir");
            let base_path = temp_dir.path().to_path_buf();

            // Create a content folder
            create_test_content_folder(&base_path, &folder_name);

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = library_service.create(req).await.expect("Should create library");

            // Add scan path
            let scan_path = library_service
                .add_scan_path(library.id, base_path.to_string_lossy().to_string())
                .await
                .expect("Should add scan path");

            // Scan the library
            let result = scan_service.scan_library(library.id).await
                .expect("Should scan library");

            // Verify content was added
            prop_assert_eq!(result.added.len(), 1, "Should have added exactly one content");

            // Verify the content has the correct scan_path_id
            let content = &result.added[0];
            prop_assert_eq!(
                content.scan_path_id,
                scan_path.id,
                "Content scan_path_id should match the scan path it was imported from"
            );

            // Verify the content has the correct library_id
            prop_assert_eq!(
                content.library_id,
                library.id,
                "Content library_id should match the library"
            );

            // Verify the folder_path is set correctly
            let expected_folder_path = base_path.join(&folder_name);
            prop_assert_eq!(
                &content.folder_path,
                &expected_folder_path.to_string_lossy().to_string(),
                "Content folder_path should match the actual folder path"
            );

            Ok(())
        })?;
    }

    /// **Feature: comic-reader, Property 9: Content-ScanPath Association**
    /// **Validates: Requirements 2.7**
    ///
    /// For multiple scan paths, content should be associated with the correct scan path.
    #[test]
    fn content_scanpath_association_multiple_paths(
        library_name in arb_library_name(),
        folder_name1 in arb_folder_name(),
        folder_name2 in arb_folder_name()
    ) {
        // Ensure folder names are different
        prop_assume!(folder_name1 != folder_name2);

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_test_db().await;
            let library_service = LibraryService::new(pool.clone());
            let scan_service = ScanService::new(pool.clone());

            // Create two temporary directories for the test
            let temp_dir1 = TempDir::new().expect("Should create temp dir 1");
            let temp_dir2 = TempDir::new().expect("Should create temp dir 2");
            let base_path1 = temp_dir1.path().to_path_buf();
            let base_path2 = temp_dir2.path().to_path_buf();

            // Create content folders in each directory
            create_test_content_folder(&base_path1, &folder_name1);
            create_test_content_folder(&base_path2, &folder_name2);

            // Create library
            let req = CreateLibraryRequest {
                name: library_name,
                scan_interval: None,
                watch_mode: None,
            };
            let library = library_service.create(req).await.expect("Should create library");

            // Add both scan paths
            let scan_path1 = library_service
                .add_scan_path(library.id, base_path1.to_string_lossy().to_string())
                .await
                .expect("Should add scan path 1");
            let scan_path2 = library_service
                .add_scan_path(library.id, base_path2.to_string_lossy().to_string())
                .await
                .expect("Should add scan path 2");

            // Scan the library
            let result = scan_service.scan_library(library.id).await
                .expect("Should scan library");

            // Verify both contents were added
            prop_assert_eq!(result.added.len(), 2, "Should have added exactly two contents");

            // Find content from each scan path
            let content1 = result.added.iter()
                .find(|c| c.title == folder_name1)
                .expect("Should find content 1");
            let content2 = result.added.iter()
                .find(|c| c.title == folder_name2)
                .expect("Should find content 2");

            // Verify each content is associated with the correct scan path
            prop_assert_eq!(
                content1.scan_path_id,
                scan_path1.id,
                "Content 1 should be associated with scan path 1"
            );
            prop_assert_eq!(
                content2.scan_path_id,
                scan_path2.id,
                "Content 2 should be associated with scan path 2"
            );

            Ok(())
        })?;
    }
}
