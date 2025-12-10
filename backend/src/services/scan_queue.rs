//! Scan queue service for asynchronous scan task management.
//!
//! This module provides the `ScanQueueService` which manages a queue of scan tasks,
//! supporting priority-based ordering, deduplication, and task status tracking.

use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::{QueuedTask, ScanTask, TaskPriority, TaskResult, TaskStatus};
use crate::services::scan::ScanService;

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
        if let Some(&existing_task_id) = library_tasks.get(&library_id) {
            if let Some(existing_task) = tasks.get_mut(&existing_task_id) {
                // If existing task is pending and new priority is higher, upgrade
                if existing_task.status == TaskStatus::Pending && priority > existing_task.priority
                {
                    existing_task.priority = priority;
                    // Rebuild the queue to reflect the priority change
                    self.rebuild_queue_internal(&tasks, &mut pending_queue);
                }
                // Return existing task ID (Requirements 4.1, 4.2)
                return existing_task_id;
            }
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

    /// Lists all pending tasks in priority order.
    pub async fn list_pending(&self) -> Vec<ScanTask> {
        let tasks = self.tasks.read().await;
        let pending_queue = self.pending_queue.read().await;

        // Get tasks in priority order by iterating the heap
        let mut result: Vec<ScanTask> = pending_queue
            .iter()
            .filter_map(|qt| tasks.get(&qt.task_id).cloned())
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
}
