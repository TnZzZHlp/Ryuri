//! Application router and state configuration.
//!
//! This module provides the unified application state and router configuration
//! for the Axum web server.

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::warn;

use crate::services::auth::{AuthConfig, AuthService, middleware::HasAuthService};
use crate::services::bangumi::BangumiService;
use crate::services::library::LibraryService;
use crate::services::progress::ProgressService;
use crate::services::scan::ScanService;
use crate::services::scheduler::SchedulerService;
use crate::services::watch::WatchService;

/// Unified application state containing all services.
///
/// This state is shared across all handlers and provides access to
/// the database pool and all business logic services.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool.
    pub pool: Pool<Sqlite>,
    /// Authentication service for user management and JWT handling.
    pub auth_service: Arc<AuthService>,
    /// Library management service.
    pub library_service: Arc<LibraryService>,
    /// Content scanning service.
    pub scan_service: Arc<ScanService>,
    /// Reading progress service.
    pub progress_service: Arc<ProgressService>,
    /// Bangumi metadata service.
    pub bangumi_service: Arc<BangumiService>,
    /// File system watch service for auto-detecting changes.
    pub watch_service: Arc<WatchService>,
    /// Scheduled scanning service.
    pub scheduler_service: Arc<SchedulerService>,
}

impl HasAuthService for AppState {
    fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }
}

/// Configuration for the application.
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    /// Authentication configuration.
    pub auth: AuthConfig,
    /// Optional Bangumi API key for metadata scraping.
    pub bangumi_api_key: Option<String>,
}

impl AppState {
    /// Create a new application state with all services initialized.
    ///
    /// # Arguments
    /// * `pool` - SQLite database connection pool
    /// * `config` - Application configuration
    pub fn new(pool: Pool<Sqlite>, config: AppConfig) -> Self {
        // Create auth service
        let auth_service = Arc::new(AuthService::new(pool.clone(), config.auth));

        // Create library service
        let library_service = Arc::new(LibraryService::new(pool.clone()));

        // Create Bangumi service
        let bangumi_service = Arc::new(BangumiService::new(config.bangumi_api_key));

        // Create scan service with Bangumi integration
        let scan_service = Arc::new(ScanService::with_bangumi(
            pool.clone(),
            Arc::clone(&bangumi_service),
        ));

        // Create progress service
        let progress_service = Arc::new(ProgressService::new(pool.clone()));

        // Create watch service
        let watch_service = Arc::new(WatchService::new(pool.clone(), Arc::clone(&scan_service)));

        // Create scheduler service
        let scheduler_service = Arc::new(SchedulerService::new(Arc::clone(&scan_service)));

        Self {
            pool,
            auth_service,
            library_service,
            scan_service,
            progress_service,
            bangumi_service,
            watch_service,
            scheduler_service,
        }
    }
}

/// Create the application router with all routes configured.
///
/// # Arguments
/// * `state` - The application state containing all services
///
/// # Returns
/// A configured Axum router with all API endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Auth routes
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/me", get(auth_get_me).put(auth_update_me))
        .route("/api/auth/password", put(auth_update_password))
        // Library routes
        .route("/api/libraries", get(list_libraries).post(create_library))
        .route(
            "/api/libraries/{id}",
            get(get_library).put(update_library).delete(delete_library),
        )
        .route(
            "/api/libraries/{id}/paths",
            get(list_scan_paths).post(add_scan_path),
        )
        .route(
            "/api/libraries/{id}/paths/{path_id}",
            delete(remove_scan_path),
        )
        .route("/api/libraries/{id}/contents", get(list_contents))
        .route("/api/libraries/{id}/scan", post(scan_library))
        .route("/api/libraries/{id}/search", get(search_contents))
        // Content routes
        .route(
            "/api/contents/{id}",
            get(get_content).delete(delete_content),
        )
        .route("/api/contents/{id}/metadata", put(update_metadata))
        .route("/api/contents/{id}/chapters", get(list_chapters))
        .route("/api/contents/{id}/progress", get(get_content_progress))
        .route(
            "/api/contents/{id}/chapters/{chapter}/pages/{page}",
            get(get_page),
        )
        .route(
            "/api/contents/{id}/chapters/{chapter}/text",
            get(get_chapter_text),
        )
        // Chapter progress routes
        .route(
            "/api/chapters/{id}/progress",
            get(get_chapter_progress).put(update_chapter_progress),
        )
        // Bangumi routes
        .route("/api/bangumi/search", get(search_bangumi))
        .with_state(state)
}

/// Create the router with CORS middleware configured.
///
/// # Arguments
/// * `state` - The application state containing all services
///
/// # Returns
/// A configured Axum router with CORS enabled
pub fn create_router_with_cors(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    create_router(state).layer(cors)
}

// ============================================================================
// Handler wrapper functions that adapt the unified AppState to handler needs
// ============================================================================

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::error::Result;
use crate::handlers::content::{
    ChapterTextParams, ChapterTextResponse, PageParams, SearchQuery, UpdateMetadataRequest,
};
use crate::handlers::library::{AddScanPathRequest, ScanPathParams};
use crate::handlers::progress::UpdateProgressWithPercentageRequest;
use crate::models::{
    Chapter, ContentProgressResponse, ContentResponse, CreateLibraryRequest, Library,
    LibraryWithStats, LoginRequest, LoginResponse, ProgressResponse, ScanPath,
    UpdateLibraryRequest, UpdatePasswordRequest, UpdateUserRequest, UserResponse,
};
use crate::services::auth::middleware::AuthUser;
use crate::services::bangumi::BangumiSearchResult;
use crate::services::content::ContentService;
use crate::services::scan::ScanResult;

// ============================================================================
// Auth handlers
// ============================================================================

/// POST /api/auth/login
async fn auth_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let (user, token) = state.auth_service.login(req.username, req.password).await?;
    Ok(Json(LoginResponse {
        user: UserResponse::from(user),
        token,
    }))
}

/// GET /api/auth/me
async fn auth_get_me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>> {
    let user = state
        .auth_service
        .get_user(auth_user.user_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;
    Ok(Json(UserResponse::from(user)))
}

/// PUT /api/auth/me
async fn auth_update_me(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    let user = state
        .auth_service
        .update_user(auth_user.user_id, req)
        .await?;
    Ok(Json(UserResponse::from(user)))
}

/// PUT /api/auth/password
async fn auth_update_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdatePasswordRequest>,
) -> Result<Json<()>> {
    state
        .auth_service
        .update_password(auth_user.user_id, req.old_password, req.new_password)
        .await?;
    Ok(Json(()))
}

// ============================================================================
// Library handlers
// ============================================================================

/// GET /api/libraries
async fn list_libraries(State(state): State<AppState>) -> Result<Json<Vec<LibraryWithStats>>> {
    let libraries = state.library_service.list().await?;
    Ok(Json(libraries))
}

/// POST /api/libraries
async fn create_library(
    State(state): State<AppState>,
    Json(req): Json<CreateLibraryRequest>,
) -> Result<Json<Library>> {
    let scan_interval = req.scan_interval.unwrap_or(0);
    let watch_mode = req.watch_mode.unwrap_or(false);

    let library = state.library_service.create(req).await?;

    // Start scheduler if scan_interval is set (Requirement 1.8)
    if scan_interval > 0
        && let Err(e) = state
            .scheduler_service
            .schedule_scan(library.id, scan_interval)
            .await
    {
        warn!(library_id = library.id, error = %e, "Failed to schedule scan for library");
    }

    // Start watch service if watch_mode is enabled (Requirement 1.9)
    if watch_mode && let Err(e) = state.watch_service.start_watching(library.id).await {
        warn!(library_id = library.id, error = %e, "Failed to start watching library");
    }

    Ok(Json(library))
}

/// GET /api/libraries/{id}
async fn get_library(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<LibraryWithStats>> {
    let library = state
        .library_service
        .get_with_stats(id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Library with id {} not found", id))
        })?;
    Ok(Json(library))
}

/// PUT /api/libraries/{id}
async fn update_library(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLibraryRequest>,
) -> Result<Json<Library>> {
    let new_scan_interval = req.scan_interval;
    let new_watch_mode = req.watch_mode;

    let library = state.library_service.update(id, req).await?;

    // Update scheduler if scan_interval changed (Requirement 1.8)
    if let Some(interval) = new_scan_interval {
        if interval > 0 {
            if let Err(e) = state.scheduler_service.schedule_scan(id, interval).await {
                warn!(library_id = id, error = %e, "Failed to update scan schedule for library");
            }
        } else if let Err(e) = state.scheduler_service.cancel_scan(id).await {
            warn!(library_id = id, error = %e, "Failed to cancel scan schedule for library");
        }
    }

    // Update watch service if watch_mode changed (Requirement 1.9)
    if let Some(watch_mode) = new_watch_mode {
        if watch_mode {
            if let Err(e) = state.watch_service.start_watching(id).await {
                warn!(library_id = id, error = %e, "Failed to start watching library");
            }
        } else if let Err(e) = state.watch_service.stop_watching(id).await {
            warn!(library_id = id, error = %e, "Failed to stop watching library");
        }
    }

    Ok(Json(library))
}

/// DELETE /api/libraries/{id}
async fn delete_library(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<()>> {
    // Stop scheduler before deleting (Requirement 1.8)
    if let Err(e) = state.scheduler_service.cancel_scan(id).await {
        warn!(library_id = id, error = %e, "Failed to cancel scan schedule for library");
    }

    // Stop watch service before deleting (Requirement 1.9)
    if let Err(e) = state.watch_service.stop_watching(id).await {
        warn!(library_id = id, error = %e, "Failed to stop watching library");
    }

    state.library_service.delete(id).await?;
    Ok(Json(()))
}

/// GET /api/libraries/{id}/paths
async fn list_scan_paths(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<Vec<ScanPath>>> {
    let paths = state.library_service.list_scan_paths(library_id).await?;
    Ok(Json(paths))
}

/// POST /api/libraries/{id}/paths
async fn add_scan_path(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
    Json(req): Json<AddScanPathRequest>,
) -> Result<Json<ScanPath>> {
    let scan_path = state
        .library_service
        .add_scan_path(library_id, req.path)
        .await?;

    // Refresh watch service to include new path (Requirement 1.9)
    if let Err(e) = state.watch_service.refresh_watching(library_id).await {
        warn!(library_id = library_id, error = %e, "Failed to refresh watching for library");
    }

    Ok(Json(scan_path))
}

/// DELETE /api/libraries/{id}/paths/{path_id}
async fn remove_scan_path(
    State(state): State<AppState>,
    Path(params): Path<ScanPathParams>,
) -> Result<Json<()>> {
    state
        .library_service
        .remove_scan_path(params.id, params.path_id)
        .await?;

    // Refresh watch service to remove the path (Requirement 1.9)
    if let Err(e) = state.watch_service.refresh_watching(params.id).await {
        warn!(library_id = params.id, error = %e, "Failed to refresh watching for library");
    }

    Ok(Json(()))
}

// ============================================================================
// Content handlers
// ============================================================================

/// Response for scan operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanResponse {
    pub added_count: usize,
    pub removed_count: usize,
    pub failed_scrape_count: usize,
    pub added: Vec<ContentResponse>,
    pub removed: Vec<i64>,
}

impl From<ScanResult> for ScanResponse {
    fn from(result: ScanResult) -> Self {
        Self {
            added_count: result.added.len(),
            removed_count: result.removed.len(),
            failed_scrape_count: result.failed_scrape.len(),
            added: result
                .added
                .into_iter()
                .map(ContentResponse::from)
                .collect(),
            removed: result.removed,
        }
    }
}

/// GET /api/libraries/{id}/contents
async fn list_contents(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<Vec<ContentResponse>>> {
    let contents = ContentService::list_contents(&state.pool, library_id).await?;
    let responses: Vec<ContentResponse> = contents.into_iter().map(ContentResponse::from).collect();
    Ok(Json(responses))
}

/// POST /api/libraries/{id}/scan
async fn scan_library(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<ScanResponse>> {
    let result = state.scan_service.scan_library(library_id).await?;
    Ok(Json(ScanResponse::from(result)))
}

/// GET /api/libraries/{id}/search
async fn search_contents(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<ContentResponse>>> {
    let contents = ContentService::search_contents(&state.pool, library_id, &query.q).await?;
    let responses: Vec<ContentResponse> = contents.into_iter().map(ContentResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/contents/{id}
async fn get_content(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ContentResponse>> {
    let content = ContentService::get_content(&state.pool, id).await?;
    Ok(Json(ContentResponse::from(content)))
}

/// DELETE /api/contents/{id}
async fn delete_content(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<()>> {
    ContentService::delete_content(&state.pool, id).await?;
    Ok(Json(()))
}

/// PUT /api/contents/{id}/metadata
async fn update_metadata(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMetadataRequest>,
) -> Result<Json<ContentResponse>> {
    let content = ContentService::update_metadata(&state.pool, id, request.metadata).await?;
    Ok(Json(ContentResponse::from(content)))
}

/// GET /api/contents/{id}/chapters
async fn list_chapters(
    State(state): State<AppState>,
    Path(content_id): Path<i64>,
) -> Result<Json<Vec<Chapter>>> {
    let chapters = ContentService::list_chapters(&state.pool, content_id).await?;
    Ok(Json(chapters))
}

/// GET /api/contents/{id}/chapters/{chapter}/pages/{page}
async fn get_page(
    State(state): State<AppState>,
    Path(params): Path<PageParams>,
) -> Result<impl IntoResponse> {
    let image_data =
        ContentService::get_page(&state.pool, params.id, params.chapter, params.page).await?;

    // Detect image type from magic bytes
    let content_type = detect_image_type(&image_data);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(image_data))
        .unwrap())
}

/// Detect image type from magic bytes.
fn detect_image_type(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        "image/gif"
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

/// GET /api/contents/{id}/chapters/{chapter}/text
async fn get_chapter_text(
    State(state): State<AppState>,
    Path(params): Path<ChapterTextParams>,
) -> Result<Json<ChapterTextResponse>> {
    let text = ContentService::get_chapter_text(&state.pool, params.id, params.chapter).await?;
    Ok(Json(ChapterTextResponse { text }))
}

// ============================================================================
// Progress handlers
// ============================================================================

/// GET /api/contents/{id}/progress
async fn get_content_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(content_id): Path<i64>,
) -> Result<Json<ContentProgressResponse>> {
    let progress = state
        .progress_service
        .get_aggregated_content_progress(auth_user.user_id, content_id)
        .await?;
    Ok(Json(progress))
}

/// GET /api/chapters/{id}/progress
async fn get_chapter_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(chapter_id): Path<i64>,
) -> Result<Json<Option<ProgressResponse>>> {
    let progress = state
        .progress_service
        .get_chapter_progress_response(auth_user.user_id, chapter_id)
        .await?;
    Ok(Json(progress))
}

/// PUT /api/chapters/{id}/progress
async fn update_chapter_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(chapter_id): Path<i64>,
    Json(req): Json<UpdateProgressWithPercentageRequest>,
) -> Result<Json<ProgressResponse>> {
    let progress = if let Some(percentage) = req.percentage {
        state
            .progress_service
            .update_progress_with_percentage(
                auth_user.user_id,
                chapter_id,
                req.position,
                percentage,
            )
            .await?
    } else {
        state
            .progress_service
            .update_progress(auth_user.user_id, chapter_id, req.position)
            .await?
    };
    Ok(Json(ProgressResponse::from(progress)))
}

// ============================================================================
// Bangumi handlers
// ============================================================================

/// Query parameters for Bangumi search.
#[derive(Debug, Clone, Deserialize)]
pub struct BangumiSearchQuery {
    pub q: String,
}

/// GET /api/bangumi/search
async fn search_bangumi(
    State(state): State<AppState>,
    Query(query): Query<BangumiSearchQuery>,
) -> Result<Json<Vec<BangumiSearchResult>>> {
    let results = state.bangumi_service.search(&query.q).await?;
    Ok(Json(results))
}
