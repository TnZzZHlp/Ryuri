//! Scheduled scanning service.
//!
//! This module provides functionality to schedule periodic library scans
//! based on configured scan intervals.
//!
//! Requirements: 1.8, 5.2

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument, warn};
use sqlx::{Pool, Sqlite};
use rust_i18n::t;

use crate::error::Result;
use crate::models::TaskPriority;
use crate::repository::library::LibraryRepository;
use crate::services::scan_queue::ScanQueueService;

/// Information about a scheduled scan task.
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// The library ID being scanned.
    pub library_id: i64,
    /// Scan interval in minutes.
    pub interval_minutes: i32,
    /// Time of the next scheduled scan.
    pub next_scan_time: DateTime<Utc>,
}

/// Handle for a running scheduled task.
struct TaskHandle {
    /// The task information.
    task: ScheduledTask,
    /// Handle to cancel the task.
    cancel_handle: tokio::sync::oneshot::Sender<()>,
    /// The spawned task handle.
    _join_handle: JoinHandle<()>,
}

/// Service for scheduling periodic library scans.
///
/// The SchedulerService manages scheduled scan tasks for libraries
/// with non-zero scan intervals. It submits tasks to the ScanQueueService
/// with Normal priority for background processing.
///
/// Requirements: 5.2
pub struct SchedulerService {
    scan_queue_service: Arc<ScanQueueService>,
    /// Map of library_id to task handle.
    tasks: Arc<RwLock<HashMap<i64, TaskHandle>>>,
}

impl SchedulerService {
    /// Create a new scheduler service.
    ///
    /// # Arguments
    /// * `scan_queue_service` - The scan queue service for submitting tasks
    pub fn new(scan_queue_service: Arc<ScanQueueService>) -> Self {
        Self {
            scan_queue_service,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Schedule periodic scanning for a library.
    ///
    /// If interval_minutes is 0 or negative, any existing schedule is cancelled.
    ///
    /// Requirements: 1.8
    #[instrument(skip(self), fields(library_id = library_id, interval_minutes = interval_minutes))]
    pub async fn schedule_scan(&self, library_id: i64, interval_minutes: i32) -> Result<()> {
        // Cancel any existing schedule first
        self.cancel_scan(library_id).await?;

        // Don't schedule if interval is 0 or negative
        if interval_minutes <= 0 {
            return Ok(());
        }

        let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
        let next_scan_time = Utc::now() + chrono::Duration::minutes(interval_minutes as i64);

        // Create a cancellation channel
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let scan_queue_service = Arc::clone(&self.scan_queue_service);
        let lib_id = library_id;
        let tasks = Arc::clone(&self.tasks);
        let interval_mins = interval_minutes;

        // Spawn the periodic scan task
        let join_handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            // Skip the first tick (immediate)
            interval_timer.tick().await;

            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        // Update next scan time
                        {
                            let mut tasks_guard = tasks.write().await;
                            if let Some(handle) = tasks_guard.get_mut(&lib_id) {
                                handle.task.next_scan_time = Utc::now() + chrono::Duration::minutes(interval_mins as i64);
                            }
                        }

                        // Submit scan task to queue with Normal priority (Requirements: 5.2)
                        let task_id = scan_queue_service.submit_task(lib_id, TaskPriority::Normal).await;
                        info!(library_id = lib_id, task_id = %task_id, "{}", t!("scheduler.task_submitted"));
                        debug!(library_id = lib_id, "{}", t!("scheduler.task_queued"));
                    }
                    _ = &mut cancel_rx => {
                        // Cancellation requested
                        break;
                    }
                }
            }
        });

        // Store the task handle
        let task = ScheduledTask {
            library_id,
            interval_minutes,
            next_scan_time,
        };

        let handle = TaskHandle {
            task,
            cancel_handle: cancel_tx,
            _join_handle: join_handle,
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(library_id, handle);
        }

        Ok(())
    }

    /// Cancel scheduled scanning for a library.
    ///
    /// Requirements: 1.8
    #[instrument(skip(self), fields(library_id = library_id))]
    pub async fn cancel_scan(&self, library_id: i64) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.remove(&library_id) {
            // Send cancellation signal (ignore if receiver is dropped)
            let _ = handle.cancel_handle.send(());
        }
        Ok(())
    }

    /// Get the next scheduled scan time for a library.
    pub async fn get_next_scan_time(&self, library_id: i64) -> Option<DateTime<Utc>> {
        let tasks = self.tasks.read().await;
        tasks.get(&library_id).map(|h| h.task.next_scan_time)
    }

    /// Check if a library has a scheduled scan.
    pub async fn is_scheduled(&self, library_id: i64) -> bool {
        let tasks = self.tasks.read().await;
        tasks.contains_key(&library_id)
    }

    /// Get the scheduled task info for a library.
    pub async fn get_scheduled_task(&self, library_id: i64) -> Option<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&library_id).map(|h| h.task.clone())
    }

    /// Get all scheduled tasks.
    pub async fn list_scheduled_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().map(|h| h.task.clone()).collect()
    }

    /// Update the scan interval for a library.
    ///
    /// This cancels the existing schedule and creates a new one with the updated interval.
    pub async fn update_interval(&self, library_id: i64, interval_minutes: i32) -> Result<()> {
        self.schedule_scan(library_id, interval_minutes).await
    }

    /// Restore scheduled scans from the database.
    pub async fn restore_schedules(&self, pool: &Pool<Sqlite>) {
        info!("{}", t!("scheduler.restoring_schedules"));
        match LibraryRepository::list(pool).await {
            Ok(libraries) => {
                let mut count = 0;
                for lib in libraries {
                    if lib.scan_interval > 0 {
                        if let Err(e) = self.schedule_scan(lib.id, lib.scan_interval).await {
                            warn!(
                                library_id = lib.id,
                                error = %e,
                                "{}", t!("scheduler.restore_failed")
                            );
                        } else {
                            count += 1;
                        }
                    }
                }
                info!(count = count, "{}", t!("scheduler.scans_restored"));
            }
            Err(e) => {
                error!(error = %e, "{}", t!("scheduler.load_libraries_failed"));
            }
        }
    }

    /// Cancel all scheduled scans (for shutdown).
    pub async fn cancel_all(&self) {
        let mut tasks = self.tasks.write().await;
        for (_, handle) in tasks.drain() {
            let _ = handle.cancel_handle.send(());
        }
    }
}
