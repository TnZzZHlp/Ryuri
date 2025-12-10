//! File system watch service.
//!
//! This module provides functionality to monitor file system changes
//! in library scan paths and automatically update content when files
//! are added or removed.
//!
//! Requirements: 1.9, 1.10, 1.11

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::error::Result;
use crate::repository::library::ScanPathRepository;
use crate::services::scan::ScanService;

/// Handle for a running watcher that can be used to stop it.
struct WatcherHandle {
    /// The watcher instance.
    _watcher: RecommendedWatcher,
    /// Paths being watched by this watcher.
    watched_paths: Vec<PathBuf>,
}

/// Service for monitoring file system changes in library scan paths.
///
/// The WatchService monitors directories for changes and triggers
/// library rescans when content is added or removed.
pub struct WatchService {
    pool: Pool<Sqlite>,
    scan_service: Arc<ScanService>,
    /// Map of library_id to watcher handle.
    watchers: Arc<RwLock<HashMap<i64, WatcherHandle>>>,
}

impl WatchService {
    /// Create a new watch service.
    pub fn new(pool: Pool<Sqlite>, scan_service: Arc<ScanService>) -> Self {
        Self {
            pool,
            scan_service,
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching a library's scan paths for file system changes.
    ///
    /// Requirements: 1.9
    pub async fn start_watching(&self, library_id: i64) -> Result<()> {
        // Check if already watching
        {
            let watchers = self.watchers.read().await;
            if watchers.contains_key(&library_id) {
                return Ok(()); // Already watching
            }
        }

        // Get scan paths for the library
        let scan_paths = ScanPathRepository::list_by_library(&self.pool, library_id).await?;

        if scan_paths.is_empty() {
            return Ok(()); // Nothing to watch
        }

        // Create a channel for receiving events
        let (tx, mut rx) = mpsc::channel::<Event>(100);

        // Create the watcher
        let watcher =
            notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Only send events for create/remove operations
                    match event.kind {
                        EventKind::Create(_) | EventKind::Remove(_) => {
                            let _ = tx.blocking_send(event);
                        }
                        _ => {}
                    }
                }
            })
            .map_err(|e| {
                crate::error::AppError::Internal(format!("Failed to create watcher: {}", e))
            })?;

        let mut watcher = watcher;
        let mut watched_paths = Vec::new();

        // Add all scan paths to the watcher
        for scan_path in &scan_paths {
            let path = PathBuf::from(&scan_path.path);
            if path.exists() {
                if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                    eprintln!("Failed to watch path {:?}: {}", path, e);
                } else {
                    watched_paths.push(path);
                }
            }
        }

        // Store the watcher handle
        {
            let mut watchers = self.watchers.write().await;
            watchers.insert(
                library_id,
                WatcherHandle {
                    _watcher: watcher,
                    watched_paths,
                },
            );
        }

        // Spawn a task to handle events
        let scan_service = Arc::clone(&self.scan_service);
        let lib_id = library_id;
        tokio::spawn(async move {
            while let Some(_event) = rx.recv().await {
                // Debounce: wait a bit for more events to settle
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Drain any additional events that came in
                while rx.try_recv().is_ok() {}

                // Trigger a rescan of the library
                // Requirements: 1.10, 1.11
                if let Err(e) = scan_service.scan_library(lib_id).await {
                    eprintln!("Watch service: failed to rescan library {}: {}", lib_id, e);
                }
            }
        });

        Ok(())
    }

    /// Stop watching a library's scan paths.
    ///
    /// Requirements: 1.9
    pub async fn stop_watching(&self, library_id: i64) -> Result<()> {
        let mut watchers = self.watchers.write().await;
        watchers.remove(&library_id);
        Ok(())
    }

    /// Check if a library is currently being watched.
    pub async fn is_watching(&self, library_id: i64) -> bool {
        let watchers = self.watchers.read().await;
        watchers.contains_key(&library_id)
    }

    /// Get the list of paths being watched for a library.
    pub async fn get_watched_paths(&self, library_id: i64) -> Vec<PathBuf> {
        let watchers = self.watchers.read().await;
        watchers
            .get(&library_id)
            .map(|h| h.watched_paths.clone())
            .unwrap_or_default()
    }

    /// Update watched paths for a library (e.g., when scan paths change).
    ///
    /// This stops the current watcher and starts a new one with updated paths.
    pub async fn refresh_watching(&self, library_id: i64) -> Result<()> {
        // Only refresh if we're currently watching
        if self.is_watching(library_id).await {
            self.stop_watching(library_id).await?;
            self.start_watching(library_id).await?;
        }
        Ok(())
    }

    /// Stop all watchers (for shutdown).
    pub async fn stop_all(&self) {
        let mut watchers = self.watchers.write().await;
        watchers.clear();
    }
}
