//! Application state configuration.
//!
//! This module provides the unified application state that is shared
//! across all handlers.

use sqlx::{Pool, Sqlite};
use std::sync::Arc;

use crate::services::auth::{AuthConfig, AuthService, middleware::HasAuthService};
use crate::services::bangumi::BangumiService;
use crate::services::library::LibraryService;
use crate::services::progress::ProgressService;
use crate::services::scan::ScanService;
use crate::services::scan_queue::ScanQueueService;
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
    /// Scan queue service for asynchronous scan task management.
    pub scan_queue_service: Arc<ScanQueueService>,
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

        // Create scan queue service with scan service reference
        let scan_queue_service = Arc::new(ScanQueueService::with_scan_service(Arc::clone(
            &scan_service,
        )));

        // Create scheduler service with scan queue for task submission
        let scheduler_service = Arc::new(SchedulerService::new(Arc::clone(&scan_queue_service)));

        Self {
            pool,
            auth_service,
            library_service,
            scan_service,
            progress_service,
            bangumi_service,
            watch_service,
            scan_queue_service,
            scheduler_service,
        }
    }
}
