//! Library scanning and scan queue service.
//!
//! This module provides:
//! - `ScanService`: Functionality to scan library paths for content, detect content folders,
//!   identify chapters, and generate thumbnails.
//! - `ScanQueueService`: Manages a queue of scan tasks with priority-based ordering,
//!   deduplication, and task status tracking.

use sqlx::{Pool, Sqlite};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::extractors::{ComicArchiveExtractor, NovelArchiveExtractor, natural_sort_key};
use crate::models::{
    Content, ContentType, NewChapter, NewContent, QueuedTask, ScanPath, ScanTask, TaskPriority,
    TaskResult, TaskStatus,
};
use crate::repository::content::{ChapterRepository, ContentRepository};
use crate::repository::library::ScanPathRepository;
use crate::services::bangumi::BangumiService;

// ============================================================================
// ScanResult
// ============================================================================

/// Result of a library scan operation.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Newly added content items.
    pub added: Vec<Content>,
    /// IDs of content items that were removed (folder no longer exists).
    pub removed: Vec<i64>,
    /// Content items that failed metadata scraping, with error messages.
    pub failed_scrape: Vec<(Content, String)>,
}

// ============================================================================
// ScanService
// ============================================================================

/// Service for scanning library paths and importing content.
pub struct ScanService {
    pool: Pool<Sqlite>,
    bangumi_service: Option<Arc<BangumiService>>,
}

impl ScanService {
    /// Create a new scan service.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            bangumi_service: None,
        }
    }

    /// Create a new scan service with Bangumi integration for auto-scraping.
    pub fn with_bangumi(pool: Pool<Sqlite>, bangumi_service: Arc<BangumiService>) -> Self {
        Self {
            pool,
            bangumi_service: Some(bangumi_service),
        }
    }

    /// Set the Bangumi service for auto-scraping.
    pub fn set_bangumi_service(&mut self, bangumi_service: Arc<BangumiService>) {
        self.bangumi_service = Some(bangumi_service);
    }

    /// Scan all paths in a library and import/update content.
    ///
    /// Requirements: 2.1
    #[instrument(skip(self), fields(library_id = library_id))]
    pub async fn scan_library(&self, library_id: i64) -> Result<ScanResult> {
        let scan_paths = ScanPathRepository::list_by_library(&self.pool, library_id).await?;

        let mut result = ScanResult::default();

        for scan_path in scan_paths {
            let path_result = self.scan_path(&scan_path).await?;
            result.added.extend(path_result.added);
            result.removed.extend(path_result.removed);
            result.failed_scrape.extend(path_result.failed_scrape);
        }

        Ok(result)
    }

    /// Scan a single scan path and import/update content.
    #[instrument(skip(self), fields(scan_path_id = scan_path.id, path = %scan_path.path))]
    pub async fn scan_path(&self, scan_path: &ScanPath) -> Result<ScanResult> {
        info!(path = ?scan_path, "Scanning...");

        let mut result = ScanResult::default();
        let base_path = Path::new(&scan_path.path);

        // Check if the scan path exists
        if !base_path.exists() {
            return Err(AppError::FileSystem(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Scan path does not exist: {}", scan_path.path),
            )));
        }

        // Get existing content folder paths for this scan path
        let existing_paths: HashSet<String> =
            ContentRepository::get_folder_paths_by_scan_path(&self.pool, scan_path.id)
                .await?
                .into_iter()
                .collect();

        // Scan for content folders
        let discovered_folders = self.discover_content_folders(base_path)?;
        let discovered_paths: HashSet<String> = discovered_folders
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        // Find removed content (exists in DB but not on disk)
        for existing_path in &existing_paths {
            if !discovered_paths.contains(existing_path) {
                // Content folder was removed from disk
                if let Some(content) = ContentRepository::find_by_folder_path(
                    &self.pool,
                    scan_path.library_id,
                    existing_path,
                )
                .await?
                {
                    ContentRepository::delete(&self.pool, content.id).await?;
                    result.removed.push(content.id);
                }
            }
        }

        // Find new content (exists on disk but not in DB)
        for folder_path in discovered_folders {
            let folder_path_str = folder_path.to_string_lossy().to_string();

            if !existing_paths.contains(&folder_path_str) {
                // New content folder found
                match self.import_content_folder(scan_path, &folder_path).await {
                    Ok((content, scrape_error)) => {
                        if let Some(error_msg) = scrape_error {
                            // Content was imported but metadata scraping failed
                            result.failed_scrape.push((content.clone(), error_msg));
                        }
                        result.added.push(content);
                    }
                    Err(e) => {
                        // Log error but continue scanning
                        error!(folder_path = ?folder_path, error = %e, "Failed to import content");
                    }
                }
            }
        }

        Ok(result)
    }

    /// Discover content folders within a scan path.
    /// Content folders are immediate subdirectories that contain archive files.
    fn discover_content_folders(&self, base_path: &Path) -> Result<Vec<PathBuf>> {
        let mut content_folders = Vec::new();

        let entries = std::fs::read_dir(base_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if this directory contains any supported archive files
                if self.has_archive_files(&path)? {
                    content_folders.push(path);
                }
            }
        }

        // Sort folders by name using natural sort
        content_folders.sort_by_key(|p| {
            natural_sort_key(&p.file_name().unwrap_or_default().to_string_lossy())
        });

        Ok(content_folders)
    }

    /// Check if a directory contains any supported archive files.
    fn has_archive_files(&self, dir: &Path) -> Result<bool> {
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && (ComicArchiveExtractor::is_supported(&path)
                    || NovelArchiveExtractor::is_supported(&path))
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Import a content folder into the database.
    ///
    /// Returns the imported content and an optional error message if metadata scraping failed.
    ///
    /// Requirements: 2.2, 2.3, 2.4, 8.1, 8.2, 8.3
    async fn import_content_folder(
        &self,
        scan_path: &ScanPath,
        folder_path: &Path,
    ) -> Result<(Content, Option<String>)> {
        // Derive title from folder name (Requirement 2.4)
        let title = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::BadRequest("Invalid folder name".to_string()))?
            .to_string();

        // Detect content type and find chapters
        let (content_type, chapters) = self.detect_content_type_and_chapters(folder_path)?;

        // Auto-scrape metadata from Bangumi if service is available (Requirements: 8.1, 8.2, 8.3)
        let (metadata, scrape_error) = self.auto_scrape_metadata(&title).await;

        // Create the content record
        let new_content = NewContent {
            library_id: scan_path.library_id,
            scan_path_id: scan_path.id,
            content_type,
            title,
            folder_path: folder_path.to_string_lossy().to_string(),
            chapter_count: chapters.len() as i32,
            thumbnail: None,
            metadata,
        };

        let content = ContentRepository::create(&self.pool, new_content).await?;

        // Create chapter records
        let new_chapters: Vec<NewChapter> = chapters
            .into_iter()
            .enumerate()
            .map(|(idx, (chapter_title, file_path, page_count))| NewChapter {
                content_id: content.id,
                title: chapter_title,
                file_path,
                sort_order: idx as i32,
                page_count,
            })
            .collect();

        ChapterRepository::create_batch(&self.pool, new_chapters).await?;

        // Generate thumbnail
        let thumbnail = self.generate_thumbnail(&content, folder_path).await;
        if let Ok(Some(thumb_data)) = thumbnail {
            ContentRepository::update_thumbnail(&self.pool, content.id, Some(thumb_data)).await?;
        }

        // Fetch the updated content with thumbnail
        let final_content = ContentRepository::find_by_id(&self.pool, content.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created content".to_string()))?;

        Ok((final_content, scrape_error))
    }

    /// Auto-scrape metadata from Bangumi for a content title.
    ///
    /// Returns the metadata JSON blob if successful, or None with an error message if failed.
    ///
    /// Requirements: 8.1, 8.2, 8.3
    async fn auto_scrape_metadata(
        &self,
        title: &str,
    ) -> (Option<serde_json::Value>, Option<String>) {
        let Some(ref bangumi_service) = self.bangumi_service else {
            // No Bangumi service configured, skip scraping
            return (None, None);
        };

        match bangumi_service.auto_scrape(title).await {
            Ok(Some(metadata)) => {
                // Successfully scraped metadata (Requirement 8.2)
                (Some(metadata), None)
            }
            Ok(None) => {
                // No results found (Requirement 8.3)
                let error_msg = format!("No Bangumi results found for '{}'", title);
                debug!(title = %title, "No Bangumi results found");
                (None, Some(error_msg))
            }
            Err(e) => {
                // Scraping failed (Requirement 8.3)
                let error_msg = format!("Failed to scrape metadata for '{}': {}", title, e);
                warn!(title = %title, error = %e, "Failed to scrape metadata");
                (None, Some(error_msg))
            }
        }
    }

    /// Detect content type based on archive files and return sorted chapters.
    ///
    /// Requirements: 2.2, 2.3
    fn detect_content_type_and_chapters(
        &self,
        folder_path: &Path,
    ) -> Result<(ContentType, Vec<(String, String, i32)>)> {
        let mut comic_files = Vec::new();
        let mut novel_files = Vec::new();

        let entries = std::fs::read_dir(folder_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if ComicArchiveExtractor::is_supported(&path) {
                    comic_files.push(path);
                } else if NovelArchiveExtractor::is_supported(&path) {
                    novel_files.push(path);
                }
            }
        }

        // Determine content type based on which type has more files
        // If equal, prefer comics
        let (content_type, files) =
            if comic_files.len() >= novel_files.len() && !comic_files.is_empty() {
                (ContentType::Comic, comic_files)
            } else if !novel_files.is_empty() {
                (ContentType::Novel, novel_files)
            } else {
                return Err(AppError::BadRequest(
                    "No supported archive files found in folder".to_string(),
                ));
            };

        // Sort files by filename using natural sort
        let mut files = files;
        files.sort_by_key(|p| {
            natural_sort_key(&p.file_name().unwrap_or_default().to_string_lossy())
        });

        // Create chapter entries (title derived from filename without extension)
        // And calculate page count
        let mut chapters: Vec<(String, String, i32)> = Vec::with_capacity(files.len());

        for path in files {
            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            let file_path = path.to_string_lossy().to_string();

            // Calculate page count based on content type
            let page_count = match content_type {
                ContentType::Comic => {
                    // Start a best effort to get page count, default to 0 on error (will be calculated on demand later)
                    ComicArchiveExtractor::page_count(&path).unwrap_or(0) as i32
                }
                ContentType::Novel => {
                    NovelArchiveExtractor::chapter_count(&path).unwrap_or(0) as i32
                }
            };

            chapters.push((title, file_path, page_count));
        }

        Ok((content_type, chapters))
    }

    /// Generate a thumbnail for content.
    ///
    /// Requirements: 2.5, 2.6
    async fn generate_thumbnail(
        &self,
        content: &Content,
        folder_path: &Path,
    ) -> Result<Option<Vec<u8>>> {
        match content.content_type {
            ContentType::Comic => self.generate_comic_thumbnail(folder_path),
            ContentType::Novel => self.generate_novel_thumbnail(folder_path),
        }
    }

    /// Generate thumbnail for comics from the first page of the first chapter.
    ///
    /// Requirements: 2.5
    fn generate_comic_thumbnail(&self, folder_path: &Path) -> Result<Option<Vec<u8>>> {
        // Find the first comic archive file
        let entries = std::fs::read_dir(folder_path)?;
        let mut comic_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && ComicArchiveExtractor::is_supported(p))
            .collect();

        if comic_files.is_empty() {
            return Ok(None);
        }

        // Sort to get the first chapter
        comic_files.sort_by_key(|p| {
            natural_sort_key(&p.file_name().unwrap_or_default().to_string_lossy())
        });

        let first_chapter = &comic_files[0];

        // Extract the first image from the first chapter
        let image_data = ComicArchiveExtractor::extract_first_image(first_chapter)?;

        // Resize and compress the thumbnail
        let thumbnail = self.compress_thumbnail(&image_data)?;

        Ok(Some(thumbnail))
    }

    /// Generate default thumbnail for novels.
    ///
    /// Requirements: 2.6
    fn generate_novel_thumbnail(&self, folder_path: &Path) -> Result<Option<Vec<u8>>> {
        // Check if there's a cover image in the folder
        let cover_names = ["cover.jpg", "cover.jpeg", "cover.png", "cover.webp"];

        for cover_name in cover_names {
            let cover_path = folder_path.join(cover_name);
            if cover_path.exists() {
                let image_data = std::fs::read(&cover_path)?;
                let thumbnail = self.compress_thumbnail(&image_data)?;
                return Ok(Some(thumbnail));
            }
        }

        // Check for EPUB files which might have embedded covers
        let entries = std::fs::read_dir(folder_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            let is_epub = path.is_file()
                && path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| ext.to_lowercase() == "epub")
                    .unwrap_or(false);

            if let (true, Ok(Some(cover))) = (is_epub, self.extract_epub_cover(&path)) {
                let thumbnail = self.compress_thumbnail(&cover)?;
                return Ok(Some(thumbnail));
            }
        }

        // No cover found, return None (will use default placeholder in frontend)
        Ok(None)
    }

    /// Extract cover image from an EPUB file.
    fn extract_epub_cover(&self, epub_path: &Path) -> Result<Option<Vec<u8>>> {
        let mut doc = epub::doc::EpubDoc::new(epub_path)
            .map_err(|e| AppError::Archive(format!("Failed to open EPUB: {}", e)))?;

        // Try to get the cover image
        if let Some((cover_data, _mime)) = doc.get_cover() {
            return Ok(Some(cover_data));
        }

        Ok(None)
    }

    /// Compress and resize an image for use as a thumbnail.
    fn compress_thumbnail(&self, image_data: &[u8]) -> Result<Vec<u8>> {
        use image::ImageReader;
        use std::io::Cursor;

        // Load the image
        let img = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| AppError::Internal(format!("Failed to read image format: {}", e)))?
            .decode()
            .map_err(|e| AppError::Internal(format!("Failed to decode image: {}", e)))?;

        // Resize to thumbnail size (max 300px width, maintaining aspect ratio)
        let thumbnail = img.thumbnail(300, 450);

        // Encode as JPEG with quality 80
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        thumbnail
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| AppError::Internal(format!("Failed to encode thumbnail: {}", e)))?;

        Ok(buffer)
    }
}

// ============================================================================
// ScanQueueService
// ============================================================================

/// Service for managing the scan task queue.
///
/// The `ScanQueueService` provides:
/// - Task submission with automatic deduplication
/// - Priority-based task ordering (High > Normal)
/// - Task status tracking and querying
/// - Task cancellation
/// - History retention for completed tasks
/// - Background worker for processing tasks
///
/// Requirements: 1.2, 1.3, 2.2, 6.1, 6.2
pub struct ScanQueueService {
    /// Pending tasks queue ordered by priority and creation time.
    pending_queue: Arc<RwLock<BinaryHeap<QueuedTask>>>,
    /// All tasks indexed by task ID (includes history).
    tasks: Arc<RwLock<HashMap<Uuid, ScanTask>>>,
    /// Mapping from library ID to active task ID for deduplication.
    /// Only contains pending or running tasks.
    library_tasks: Arc<RwLock<HashMap<i64, Uuid>>>,
    /// Scan service for executing scans.
    scan_service: Option<Arc<ScanService>>,
    /// Broadcast sender for shutdown signal.
    shutdown_tx: broadcast::Sender<()>,
    /// Worker task handle.
    worker_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Notify channel to wake up worker when new tasks are added.
    task_notify: Arc<tokio::sync::Notify>,
}

impl ScanQueueService {
    /// Creates a new scan queue service without a scan service.
    ///
    /// This constructor is useful for testing or when the scan service
    /// will be set later. The worker will not be started.
    ///
    /// Requirements: 1.2
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            library_tasks: Arc::new(RwLock::new(HashMap::new())),
            scan_service: None,
            shutdown_tx,
            worker_handle: Arc::new(RwLock::new(None)),
            task_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Creates a new scan queue service with a scan service and starts the worker.
    ///
    /// The worker will process tasks from the queue in priority order.
    ///
    /// Requirements: 1.2, 1.3
    pub fn with_scan_service(scan_service: Arc<ScanService>) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            library_tasks: Arc::new(RwLock::new(HashMap::new())),
            scan_service: Some(scan_service),
            shutdown_tx,
            worker_handle: Arc::new(RwLock::new(None)),
            task_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Starts the background worker that processes tasks from the queue.
    ///
    /// This should be called after the service is created to begin processing.
    /// The worker runs in a separate tokio task and processes tasks in priority order.
    ///
    /// Requirements: 1.3, 2.2, 6.1, 6.2
    pub async fn start_worker(&self) {
        let Some(scan_service) = self.scan_service.clone() else {
            warn!("Cannot start worker: no scan service configured");
            return;
        };

        let pending_queue = Arc::clone(&self.pending_queue);
        let tasks = Arc::clone(&self.tasks);
        let library_tasks = Arc::clone(&self.library_tasks);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let task_notify = Arc::clone(&self.task_notify);

        let handle = tokio::spawn(async move {
            info!("Scan queue worker started");

            loop {
                // Wait for either a new task notification or shutdown
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Scan queue worker received shutdown signal");
                        break;
                    }
                    _ = task_notify.notified() => {
                        // Process all available tasks
                        Self::process_pending_tasks(
                            &pending_queue,
                            &tasks,
                            &library_tasks,
                            &scan_service,
                            &mut shutdown_rx,
                        ).await;
                    }
                }
            }

            info!("Scan queue worker stopped");
        });

        let mut worker_handle = self.worker_handle.write().await;
        *worker_handle = Some(handle);
    }

    /// Processes all pending tasks in the queue.
    ///
    /// Requirements: 1.3, 2.2, 6.1, 6.2
    #[instrument(skip_all)]
    async fn process_pending_tasks(
        pending_queue: &Arc<RwLock<BinaryHeap<QueuedTask>>>,
        tasks: &Arc<RwLock<HashMap<Uuid, ScanTask>>>,
        library_tasks: &Arc<RwLock<HashMap<i64, Uuid>>>,
        scan_service: &Arc<ScanService>,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) {
        loop {
            // Pop the next task from the queue
            let queued_task = {
                let mut queue = pending_queue.write().await;
                queue.pop()
            };

            let Some(queued_task) = queued_task else {
                // No more tasks to process
                break;
            };

            // Check if task was cancelled before we start
            let (task_id, library_id) = {
                let tasks_guard = tasks.read().await;
                if let Some(task) = tasks_guard.get(&queued_task.task_id) {
                    if task.status == TaskStatus::Cancelled {
                        debug!(task_id = %queued_task.task_id, "Skipping cancelled task");
                        continue;
                    }
                    (task.id, task.library_id)
                } else {
                    // Task was removed, skip it
                    continue;
                }
            };

            // Update task status to Running
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(task) = tasks_guard.get_mut(&task_id) {
                    task.status = TaskStatus::Running;
                    task.started_at = Some(chrono::Utc::now());
                    debug!(task_id = %task_id, library_id = library_id, "Starting scan task");
                }
            }

            // Execute the scan with cancellation support
            let scan_result = tokio::select! {
                result = scan_service.scan_library(library_id) => {
                    Some(result)
                }
                _ = shutdown_rx.recv() => {
                    // Shutdown requested during scan
                    info!(task_id = %task_id, "Scan interrupted by shutdown");
                    None
                }
            };

            // Update task with result
            {
                let mut tasks_guard = tasks.write().await;
                let mut library_tasks_guard = library_tasks.write().await;

                if let Some(task) = tasks_guard.get_mut(&task_id) {
                    // Check if task was cancelled while running
                    if task.status == TaskStatus::Cancelled {
                        debug!(task_id = %task_id, "Task was cancelled during execution");
                        // Already marked as cancelled, just clean up
                        library_tasks_guard.remove(&library_id);
                        continue;
                    }

                    task.completed_at = Some(chrono::Utc::now());

                    match scan_result {
                        Some(Ok(result)) => {
                            // Scan completed successfully (Requirements: 6.1)
                            task.status = TaskStatus::Completed;
                            task.result = Some(TaskResult {
                                added_count: result.added.len() as i32,
                                removed_count: result.removed.len() as i32,
                                failed_scrape_count: result.failed_scrape.len() as i32,
                            });
                            info!(
                                task_id = %task_id,
                                library_id = library_id,
                                added = result.added.len(),
                                removed = result.removed.len(),
                                "Scan task completed"
                            );
                        }
                        Some(Err(e)) => {
                            // Scan failed (Requirements: 6.2)
                            task.status = TaskStatus::Failed;
                            task.error = Some(e.to_string());
                            error!(
                                task_id = %task_id,
                                library_id = library_id,
                                error = %e,
                                "Scan task failed"
                            );
                        }
                        None => {
                            // Shutdown interrupted the scan
                            task.status = TaskStatus::Cancelled;
                            task.error = Some("Scan interrupted by shutdown".to_string());
                        }
                    }

                    // Remove from library_tasks mapping
                    library_tasks_guard.remove(&library_id);
                }
            }

            // Check for shutdown after each task
            if shutdown_rx.try_recv().is_ok() {
                info!("Shutdown signal received, stopping task processing");
                break;
            }
        }
    }

    /// Submits a scan task for a library.
    ///
    /// If a task already exists for the library (pending or running), returns
    /// the existing task ID. If the new request has higher priority than an
    /// existing pending task, upgrades the task's priority.
    ///
    /// Requirements: 1.1, 4.1, 4.2, 4.3
    pub async fn submit_task(&self, library_id: i64, priority: TaskPriority) -> Uuid {
        let mut library_tasks = self.library_tasks.write().await;
        let mut tasks = self.tasks.write().await;
        let mut pending_queue = self.pending_queue.write().await;

        // Check for existing task (deduplication)
        if let Some(&existing_task_id) = library_tasks.get(&library_id)
            && let Some(existing_task) = tasks.get_mut(&existing_task_id)
        {
            // If existing task is pending and new priority is higher, upgrade
            if existing_task.status == TaskStatus::Pending && priority > existing_task.priority {
                existing_task.priority = priority;
                // Rebuild the queue to reflect the priority change
                self.rebuild_queue_internal(&tasks, &mut pending_queue);
            }
            // Return existing task ID (Requirements 4.1, 4.2)
            return existing_task_id;
        }

        // Create new task
        let task = ScanTask::new(library_id, priority);
        let task_id = task.id;

        // Add to pending queue
        let queued_task = QueuedTask::from_scan_task(&task);
        pending_queue.push(queued_task);

        // Store task and mapping
        tasks.insert(task_id, task);
        library_tasks.insert(library_id, task_id);

        // Notify worker that a new task is available
        drop(pending_queue);
        drop(tasks);
        drop(library_tasks);
        self.task_notify.notify_one();

        task_id
    }

    /// Rebuilds the pending queue from the tasks map.
    ///
    /// This is needed when a task's priority changes, as BinaryHeap
    /// doesn't support updating priorities in place.
    fn rebuild_queue_internal(
        &self,
        tasks: &HashMap<Uuid, ScanTask>,
        pending_queue: &mut BinaryHeap<QueuedTask>,
    ) {
        pending_queue.clear();
        for task in tasks.values() {
            if task.status == TaskStatus::Pending {
                pending_queue.push(QueuedTask::from_scan_task(task));
            }
        }
    }

    /// Gets a task by its ID.
    ///
    /// Requirements: 2.1
    pub async fn get_task(&self, task_id: Uuid) -> Option<ScanTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned()
    }

    /// Gets the current task for a library.
    ///
    /// Returns the active (pending or running) task for the library, if any.
    pub async fn get_library_task(&self, library_id: i64) -> Option<ScanTask> {
        let library_tasks = self.library_tasks.read().await;
        let tasks = self.tasks.read().await;

        library_tasks
            .get(&library_id)
            .and_then(|task_id| tasks.get(task_id).cloned())
    }

    /// Cancels a task.
    ///
    /// - For pending tasks: removes from queue and sets status to Cancelled
    /// - For running tasks: signals cancellation (handled by worker)
    /// - For completed/failed/cancelled tasks: returns an error
    ///
    /// Requirements: 3.1, 3.2, 3.3
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let mut library_tasks = self.library_tasks.write().await;
        let mut pending_queue = self.pending_queue.write().await;

        let task = tasks
            .get_mut(&task_id)
            .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

        match task.status {
            TaskStatus::Pending => {
                // Remove from pending queue and update status
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(chrono::Utc::now());

                // Remove from library_tasks mapping
                library_tasks.remove(&task.library_id);

                // Rebuild queue without the cancelled task
                self.rebuild_queue_internal(&tasks, &mut pending_queue);

                Ok(())
            }
            TaskStatus::Running => {
                // For running tasks, just mark as cancelled
                // The worker will check this flag and stop
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(chrono::Utc::now());

                // Remove from library_tasks mapping
                library_tasks.remove(&task.library_id);

                Ok(())
            }
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => Err(
                AppError::BadRequest(format!("Cannot cancel task with status {:?}", task.status)),
            ),
        }
    }

    /// Lists all processing tasks.
    pub async fn list_processing(&self) -> Vec<ScanTask> {
        let tasks = self.tasks.read().await;

        let mut result: Vec<ScanTask> = tasks
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .cloned()
            .collect();

        // Sort by created_at (asc)
        result.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        result
    }

    /// Lists all pending tasks in priority order.
    pub async fn list_pending(&self) -> Vec<ScanTask> {
        let tasks = self.tasks.read().await;

        // Get Pending and Running tasks from the tasks map
        let mut result: Vec<ScanTask> = tasks
            .values()
            .filter(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::Running))
            .cloned()
            .collect();

        // Sort by priority (desc) then created_at (asc)
        result.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        result
    }

    /// Lists task history (completed, failed, cancelled tasks).
    ///
    /// Returns tasks from the last 24 hours by default.
    ///
    /// Requirements: 2.3, 6.3
    pub async fn list_history(&self, limit: usize) -> Vec<ScanTask> {
        let tasks = self.tasks.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);

        let mut history: Vec<ScanTask> = tasks
            .values()
            .filter(|t| {
                matches!(
                    t.status,
                    TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                ) && t.created_at >= cutoff
            })
            .cloned()
            .collect();

        // Sort by completed_at descending (most recent first)
        history.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));

        history.truncate(limit);
        history
    }

    /// Returns the number of pending tasks.
    pub async fn pending_count(&self) -> usize {
        let pending_queue = self.pending_queue.read().await;
        pending_queue.len()
    }

    /// Shuts down the scan queue service gracefully.
    ///
    /// Sends a shutdown signal to the worker and waits for it to complete.
    /// Any currently running task will be allowed to finish or will be
    /// marked as cancelled.
    ///
    /// Requirements: 3.2
    pub async fn shutdown(&self) {
        info!("Shutting down scan queue service");

        // Send shutdown signal to worker
        let _ = self.shutdown_tx.send(());

        // Wait for worker to finish
        let mut worker_handle = self.worker_handle.write().await;
        if let Some(handle) = worker_handle.take() {
            // Give the worker some time to finish gracefully
            match tokio::time::timeout(std::time::Duration::from_secs(30), handle).await {
                Ok(Ok(())) => {
                    info!("Scan queue worker shut down gracefully");
                }
                Ok(Err(e)) => {
                    error!("Scan queue worker panicked: {:?}", e);
                }
                Err(_) => {
                    warn!("Scan queue worker did not shut down within timeout");
                }
            }
        }
    }

    /// Checks if the worker is currently running.
    pub async fn is_worker_running(&self) -> bool {
        let worker_handle = self.worker_handle.read().await;
        if let Some(handle) = worker_handle.as_ref() {
            !handle.is_finished()
        } else {
            false
        }
    }
}

impl Default for ScanQueueService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl ScanQueueService {
    /// Sets the status of a task for testing purposes.
    ///
    /// This method is only available in test builds.
    pub async fn set_task_status_for_test(
        &self,
        task_id: Uuid,
        status: TaskStatus,
        error: Option<String>,
    ) {
        let mut tasks = self.tasks.write().await;
        let mut library_tasks = self.library_tasks.write().await;
        let mut pending_queue = self.pending_queue.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            let library_id = task.library_id;
            task.status = status;
            task.completed_at = Some(chrono::Utc::now());
            task.error = error;

            // If task is no longer active, remove from library_tasks mapping
            if matches!(
                status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ) {
                library_tasks.remove(&library_id);
                // Rebuild queue without this task
                self.rebuild_queue_internal(&tasks, &mut pending_queue);
            }
        }
    }

    /// Sets the created_at timestamp of a task for testing purposes.
    ///
    /// This method is only available in test builds.
    pub async fn set_task_created_at_for_test(
        &self,
        task_id: Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
    ) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.created_at = created_at;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_creates_empty_service() {
        let service = ScanQueueService::new();
        assert_eq!(service.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_submit_task_creates_pending_task() {
        let service = ScanQueueService::new();
        let task_id = service.submit_task(1, TaskPriority::Normal).await;

        let task = service.get_task(task_id).await.unwrap();
        assert_eq!(task.library_id, 1);
        assert_eq!(task.priority, TaskPriority::Normal);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_submit_duplicate_returns_existing() {
        let service = ScanQueueService::new();
        let task_id1 = service.submit_task(1, TaskPriority::Normal).await;
        let task_id2 = service.submit_task(1, TaskPriority::Normal).await;

        assert_eq!(task_id1, task_id2);
        assert_eq!(service.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_submit_duplicate_with_higher_priority_upgrades() {
        let service = ScanQueueService::new();
        let task_id = service.submit_task(1, TaskPriority::Normal).await;

        // Submit again with higher priority
        let task_id2 = service.submit_task(1, TaskPriority::High).await;

        assert_eq!(task_id, task_id2);

        let task = service.get_task(task_id).await.unwrap();
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn test_cancel_pending_task() {
        let service = ScanQueueService::new();
        let task_id = service.submit_task(1, TaskPriority::Normal).await;

        service.cancel_task(task_id).await.unwrap();

        let task = service.get_task(task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
        assert_eq!(service.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_cancel_completed_task_fails() {
        let service = ScanQueueService::new();
        let task_id = service.submit_task(1, TaskPriority::Normal).await;

        // Manually set task to completed
        {
            let mut tasks = service.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Completed;
            }
        }

        let result = service.cancel_task(task_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_pending_includes_running_tasks() {
        let service = ScanQueueService::new();
        let task_id = service.submit_task(1, TaskPriority::Normal).await;

        // Manually set task to Running
        {
            let mut tasks = service.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Running;
            }
        }

        let pending = service.list_pending().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, task_id);
        assert_eq!(pending[0].status, TaskStatus::Running);
    }
}
