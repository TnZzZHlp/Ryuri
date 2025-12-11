//! OpenAPI documentation module.
//!
//! This module provides automatic OpenAPI documentation generation using utoipa.
//! It is only compiled when the `dev` feature is enabled.
//!
//! Requirements: 1.1, 1.3

use utoipa::OpenApi;

use crate::handlers::bangumi::BangumiSearchQuery;
use crate::handlers::content::{
    ChapterTextParams, ChapterTextResponse, PageParams, ScanResponse, SearchQuery,
    UpdateMetadataRequest,
};
use crate::handlers::library::{AddScanPathRequest, ScanPathParams};
use crate::handlers::progress::UpdateProgressWithPercentageRequest;
use crate::handlers::scan_queue::{ListTasksResponse, SubmitScanResponse};
use crate::models::{
    Chapter, ContentProgressResponse, ContentResponse, ContentType, CreateLibraryRequest, Library,
    LibraryWithStats, LoginRequest, LoginResponse, ProgressResponse, ScanPath, ScanTask,
    UpdateLibraryRequest, UpdatePasswordRequest, UpdateProgressRequest, UpdateUserRequest,
    UserResponse,
};
use crate::services::bangumi::BangumiSearchResult;

/// OpenAPI documentation for the Comic Reader API.
///
/// This struct generates the complete OpenAPI specification including
/// all paths and schemas when the `dev` feature is enabled.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Comic Reader API",
        version = "1.0.0",
        description = "API for managing comic and novel libraries, content, and reading progress."
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "libraries", description = "Library management endpoints"),
        (name = "contents", description = "Content management endpoints"),
        (name = "chapters", description = "Chapter and reading endpoints"),
        (name = "progress", description = "Reading progress endpoints"),
        (name = "scan-tasks", description = "Scan queue management endpoints"),
        (name = "bangumi", description = "Bangumi metadata endpoints")
    ),
    paths(
        // Auth endpoints
        crate::openapi::paths::auth_login,
        crate::openapi::paths::auth_get_me,
        crate::openapi::paths::auth_update_me,
        crate::openapi::paths::auth_update_password,
        // Library endpoints
        crate::openapi::paths::list_libraries,
        crate::openapi::paths::create_library,
        crate::openapi::paths::get_library,
        crate::openapi::paths::update_library,
        crate::openapi::paths::delete_library,
        crate::openapi::paths::list_scan_paths,
        crate::openapi::paths::add_scan_path,
        crate::openapi::paths::remove_scan_path,
        // Content endpoints
        crate::openapi::paths::list_contents,
        crate::openapi::paths::scan_library,
        crate::openapi::paths::search_contents,
        crate::openapi::paths::get_content,
        crate::openapi::paths::delete_content,
        crate::openapi::paths::update_metadata,
        crate::openapi::paths::list_chapters,
        crate::openapi::paths::get_page,
        crate::openapi::paths::get_chapter_text,
        // Progress endpoints
        crate::openapi::paths::get_content_progress,
        crate::openapi::paths::get_chapter_progress,
        crate::openapi::paths::update_chapter_progress,
        // Scan queue endpoints
        crate::openapi::paths::submit_scan,
        crate::openapi::paths::get_task,
        crate::openapi::paths::list_tasks,
        crate::openapi::paths::cancel_task,
        // Bangumi endpoints
        crate::openapi::paths::search_bangumi,
    ),
    components(
        schemas(
            // Auth schemas
            LoginRequest,
            LoginResponse,
            UserResponse,
            UpdateUserRequest,
            UpdatePasswordRequest,
            // Library schemas
            Library,
            LibraryWithStats,
            ScanPath,
            CreateLibraryRequest,
            UpdateLibraryRequest,
            AddScanPathRequest,
            ScanPathParams,
            // Content schemas
            ContentType,
            ContentResponse,
            Chapter,
            ScanResponse,
            SearchQuery,
            ChapterTextResponse,
            ChapterTextParams,
            PageParams,
            UpdateMetadataRequest,
            // Progress schemas
            ProgressResponse,
            ContentProgressResponse,
            UpdateProgressRequest,
            UpdateProgressWithPercentageRequest,
            // Scan queue schemas
            ScanTask,
            SubmitScanResponse,
            ListTasksResponse,
            // Bangumi schemas
            BangumiSearchQuery,
            BangumiSearchResult,
        )
    )
)]
pub struct ApiDoc;

/// Path operation definitions for OpenAPI documentation.
///
/// These are stub functions that define the OpenAPI metadata for each endpoint.
/// The actual implementation is in the router module.
pub mod paths {
    #![allow(unused_imports)]

    use crate::handlers::bangumi::BangumiSearchQuery;
    use crate::handlers::content::{
        ChapterTextParams, ChapterTextResponse, PageParams, ScanResponse, SearchQuery,
        UpdateMetadataRequest,
    };
    use crate::handlers::library::AddScanPathRequest;
    use crate::handlers::progress::UpdateProgressWithPercentageRequest;
    use crate::handlers::scan_queue::{ListTasksResponse, SubmitScanResponse};
    use crate::models::{
        Chapter, ContentProgressResponse, ContentResponse, CreateLibraryRequest, Library,
        LibraryWithStats, LoginRequest, LoginResponse, ProgressResponse, ScanPath, ScanTask,
        UpdateLibraryRequest, UpdatePasswordRequest, UpdateUserRequest, UserResponse,
    };
    use crate::services::bangumi::BangumiSearchResult;

    // ========================================================================
    // Auth endpoints
    // ========================================================================

    /// Login with username and password
    #[utoipa::path(
        post,
        path = "/api/auth/login",
        tag = "auth",
        request_body = LoginRequest,
        responses(
            (status = 200, description = "Login successful", body = LoginResponse),
            (status = 401, description = "Invalid credentials")
        )
    )]
    pub async fn auth_login() {}

    /// Get current authenticated user
    #[utoipa::path(
        get,
        path = "/api/auth/me",
        tag = "auth",
        security(("bearer_auth" = [])),
        responses(
            (status = 200, description = "Current user info", body = UserResponse),
            (status = 401, description = "Not authenticated")
        )
    )]
    pub async fn auth_get_me() {}

    /// Update current user information
    #[utoipa::path(
        put,
        path = "/api/auth/me",
        tag = "auth",
        security(("bearer_auth" = [])),
        request_body = UpdateUserRequest,
        responses(
            (status = 200, description = "User updated", body = UserResponse),
            (status = 401, description = "Not authenticated")
        )
    )]
    pub async fn auth_update_me() {}

    /// Update current user password
    #[utoipa::path(
        put,
        path = "/api/auth/password",
        tag = "auth",
        security(("bearer_auth" = [])),
        request_body = UpdatePasswordRequest,
        responses(
            (status = 200, description = "Password updated"),
            (status = 400, description = "Invalid old password"),
            (status = 401, description = "Not authenticated")
        )
    )]
    pub async fn auth_update_password() {}

    // ========================================================================
    // Library endpoints
    // ========================================================================

    /// List all libraries
    #[utoipa::path(
        get,
        path = "/api/libraries",
        tag = "libraries",
        responses(
            (status = 200, description = "List of libraries", body = Vec<LibraryWithStats>)
        )
    )]
    pub async fn list_libraries() {}

    /// Create a new library
    #[utoipa::path(
        post,
        path = "/api/libraries",
        tag = "libraries",
        request_body = CreateLibraryRequest,
        responses(
            (status = 200, description = "Library created", body = Library)
        )
    )]
    pub async fn create_library() {}

    /// Get a library by ID
    #[utoipa::path(
        get,
        path = "/api/libraries/{id}",
        tag = "libraries",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        responses(
            (status = 200, description = "Library details", body = LibraryWithStats),
            (status = 404, description = "Library not found")
        )
    )]
    pub async fn get_library() {}

    /// Update a library
    #[utoipa::path(
        put,
        path = "/api/libraries/{id}",
        tag = "libraries",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        request_body = UpdateLibraryRequest,
        responses(
            (status = 200, description = "Library updated", body = Library),
            (status = 404, description = "Library not found")
        )
    )]
    pub async fn update_library() {}

    /// Delete a library
    #[utoipa::path(
        delete,
        path = "/api/libraries/{id}",
        tag = "libraries",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        responses(
            (status = 200, description = "Library deleted"),
            (status = 404, description = "Library not found")
        )
    )]
    pub async fn delete_library() {}

    /// List scan paths for a library
    #[utoipa::path(
        get,
        path = "/api/libraries/{id}/paths",
        tag = "libraries",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        responses(
            (status = 200, description = "List of scan paths", body = Vec<ScanPath>)
        )
    )]
    pub async fn list_scan_paths() {}

    /// Add a scan path to a library
    #[utoipa::path(
        post,
        path = "/api/libraries/{id}/paths",
        tag = "libraries",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        request_body = AddScanPathRequest,
        responses(
            (status = 200, description = "Scan path added", body = ScanPath)
        )
    )]
    pub async fn add_scan_path() {}

    /// Remove a scan path from a library
    #[utoipa::path(
        delete,
        path = "/api/libraries/{id}/paths/{path_id}",
        tag = "libraries",
        params(
            ("id" = i64, Path, description = "Library ID"),
            ("path_id" = i64, Path, description = "Scan path ID")
        ),
        responses(
            (status = 200, description = "Scan path removed"),
            (status = 404, description = "Scan path not found")
        )
    )]
    pub async fn remove_scan_path() {}

    // ========================================================================
    // Content endpoints
    // ========================================================================

    /// List all contents in a library
    #[utoipa::path(
        get,
        path = "/api/libraries/{id}/contents",
        tag = "contents",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        responses(
            (status = 200, description = "List of contents", body = Vec<ContentResponse>)
        )
    )]
    pub async fn list_contents() {}

    /// Scan a library for new content
    #[utoipa::path(
        post,
        path = "/api/libraries/{id}/scan",
        tag = "contents",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        responses(
            (status = 200, description = "Scan results", body = ScanResponse)
        )
    )]
    pub async fn scan_library() {}

    /// Search contents by title
    #[utoipa::path(
        get,
        path = "/api/libraries/{id}/search",
        tag = "contents",
        params(
            ("id" = i64, Path, description = "Library ID"),
            ("q" = String, Query, description = "Search query")
        ),
        responses(
            (status = 200, description = "Search results", body = Vec<ContentResponse>)
        )
    )]
    pub async fn search_contents() {}

    /// Get a content by ID
    #[utoipa::path(
        get,
        path = "/api/contents/{id}",
        tag = "contents",
        params(
            ("id" = i64, Path, description = "Content ID")
        ),
        responses(
            (status = 200, description = "Content details", body = ContentResponse),
            (status = 404, description = "Content not found")
        )
    )]
    pub async fn get_content() {}

    /// Delete a content
    #[utoipa::path(
        delete,
        path = "/api/contents/{id}",
        tag = "contents",
        params(
            ("id" = i64, Path, description = "Content ID")
        ),
        responses(
            (status = 200, description = "Content deleted"),
            (status = 404, description = "Content not found")
        )
    )]
    pub async fn delete_content() {}

    /// Update content metadata
    #[utoipa::path(
        put,
        path = "/api/contents/{id}/metadata",
        tag = "contents",
        params(
            ("id" = i64, Path, description = "Content ID")
        ),
        request_body = UpdateMetadataRequest,
        responses(
            (status = 200, description = "Metadata updated", body = ContentResponse),
            (status = 404, description = "Content not found")
        )
    )]
    pub async fn update_metadata() {}

    /// List chapters for a content
    #[utoipa::path(
        get,
        path = "/api/contents/{id}/chapters",
        tag = "chapters",
        params(
            ("id" = i64, Path, description = "Content ID")
        ),
        responses(
            (status = 200, description = "List of chapters", body = Vec<Chapter>)
        )
    )]
    pub async fn list_chapters() {}

    /// Get a page image from a comic chapter
    #[utoipa::path(
        get,
        path = "/api/contents/{id}/chapters/{chapter}/pages/{page}",
        tag = "chapters",
        params(
            ("id" = i64, Path, description = "Content ID"),
            ("chapter" = i32, Path, description = "Chapter index (0-based)"),
            ("page" = i32, Path, description = "Page index (0-based)")
        ),
        responses(
            (status = 200, description = "Page image", content_type = "image/*"),
            (status = 404, description = "Page not found")
        )
    )]
    pub async fn get_page() {}

    /// Get text content of a novel chapter
    #[utoipa::path(
        get,
        path = "/api/contents/{id}/chapters/{chapter}/text",
        tag = "chapters",
        params(
            ("id" = i64, Path, description = "Content ID"),
            ("chapter" = i32, Path, description = "Chapter index (0-based)")
        ),
        responses(
            (status = 200, description = "Chapter text", body = ChapterTextResponse),
            (status = 404, description = "Chapter not found")
        )
    )]
    pub async fn get_chapter_text() {}

    // ========================================================================
    // Progress endpoints
    // ========================================================================

    /// Get overall content progress
    #[utoipa::path(
        get,
        path = "/api/contents/{id}/progress",
        tag = "progress",
        security(("bearer_auth" = [])),
        params(
            ("id" = i64, Path, description = "Content ID")
        ),
        responses(
            (status = 200, description = "Content progress", body = ContentProgressResponse),
            (status = 401, description = "Not authenticated")
        )
    )]
    pub async fn get_content_progress() {}

    /// Get chapter progress
    #[utoipa::path(
        get,
        path = "/api/chapters/{id}/progress",
        tag = "progress",
        security(("bearer_auth" = [])),
        params(
            ("id" = i64, Path, description = "Chapter ID")
        ),
        responses(
            (status = 200, description = "Chapter progress", body = Option<ProgressResponse>),
            (status = 401, description = "Not authenticated")
        )
    )]
    pub async fn get_chapter_progress() {}

    /// Update chapter progress
    #[utoipa::path(
        put,
        path = "/api/chapters/{id}/progress",
        tag = "progress",
        security(("bearer_auth" = [])),
        params(
            ("id" = i64, Path, description = "Chapter ID")
        ),
        request_body = UpdateProgressWithPercentageRequest,
        responses(
            (status = 200, description = "Progress updated", body = ProgressResponse),
            (status = 400, description = "Invalid progress values"),
            (status = 401, description = "Not authenticated"),
            (status = 404, description = "Chapter not found")
        )
    )]
    pub async fn update_chapter_progress() {}

    // ========================================================================
    // Bangumi endpoints
    // ========================================================================

    /// Search for content on Bangumi.tv
    #[utoipa::path(
        get,
        path = "/api/bangumi/search",
        tag = "bangumi",
        params(
            ("q" = String, Query, description = "Search query")
        ),
        responses(
            (status = 200, description = "Search results", body = Vec<BangumiSearchResult>)
        )
    )]
    pub async fn search_bangumi() {}

    // ========================================================================
    // Scan queue endpoints
    // ========================================================================

    /// Submit a scan task for a library
    #[utoipa::path(
        post,
        path = "/api/libraries/{id}/scan",
        tag = "scan-tasks",
        params(
            ("id" = i64, Path, description = "Library ID")
        ),
        responses(
            (status = 200, description = "Scan task submitted", body = SubmitScanResponse),
            (status = 404, description = "Library not found")
        )
    )]
    pub async fn submit_scan() {}

    /// Get a scan task by ID
    #[utoipa::path(
        get,
        path = "/api/scan-tasks/{id}",
        tag = "scan-tasks",
        params(
            ("id" = String, Path, description = "Task ID (UUID)")
        ),
        responses(
            (status = 200, description = "Task details", body = ScanTask),
            (status = 404, description = "Task not found")
        )
    )]
    pub async fn get_task() {}

    /// List all scan tasks
    #[utoipa::path(
        get,
        path = "/api/scan-tasks",
        tag = "scan-tasks",
        params(
            ("limit" = Option<usize>, Query, description = "Maximum number of history tasks to return (default: 50)")
        ),
        responses(
            (status = 200, description = "List of tasks", body = ListTasksResponse)
        )
    )]
    pub async fn list_tasks() {}

    /// Cancel a scan task
    #[utoipa::path(
        delete,
        path = "/api/scan-tasks/{id}",
        tag = "scan-tasks",
        params(
            ("id" = String, Path, description = "Task ID (UUID)")
        ),
        responses(
            (status = 200, description = "Task cancelled", body = ScanTask),
            (status = 404, description = "Task not found")
        )
    )]
    pub async fn cancel_task() {}
}
