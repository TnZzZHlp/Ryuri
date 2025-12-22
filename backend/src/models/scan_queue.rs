//! Scan queue data models.
//!
//! This module contains data structures for the scan queue system,
//! including task priority, status, progress, and result types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use uuid::Uuid;

/// Task priority for scan operations.
///
/// Higher priority tasks are processed before lower priority tasks.
/// Manual scans have High priority, scheduled scans have Normal priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Normal priority for scheduled scans.
    Normal = 0,
    /// High priority for manual scans.
    High = 1,
}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as i32).cmp(&(*other as i32))
    }
}

/// Task status for scan operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting in the queue.
    Pending,
    /// Task is currently being executed.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed with an error.
    Failed,
    /// Task was cancelled by user.
    Cancelled,
}

/// Progress information for a running scan task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Number of paths that have been scanned.
    pub scanned_paths: i32,
    /// Total number of paths to scan.
    pub total_paths: i32,
}

/// Result information for a completed scan task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskResult {
    /// Number of content items added during the scan.
    pub added_count: i32,
    /// Number of content items removed during the scan.
    pub removed_count: i32,
    /// Number of items that failed to scrape metadata.
    pub failed_scrape_count: i32,
}

/// A scan task representing a queued or executed scan operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanTask {
    /// Unique identifier for the task.
    pub id: Uuid,
    /// ID of the library being scanned.
    pub library_id: i64,
    /// Priority of the task.
    pub priority: TaskPriority,
    /// Current status of the task.
    pub status: TaskStatus,
    /// Timestamp when the task was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the task started executing.
    pub started_at: Option<DateTime<Utc>>,
    /// Timestamp when the task completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Progress information for running tasks.
    pub progress: Option<TaskProgress>,
    /// Result information for completed tasks.
    pub result: Option<TaskResult>,
    /// Error message for failed tasks.
    pub error: Option<String>,
}

impl ScanTask {
    /// Creates a new pending scan task.
    pub fn new(library_id: i64, priority: TaskPriority) -> Self {
        Self {
            id: Uuid::new_v4(),
            library_id,
            priority,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: None,
            result: None,
            error: None,
        }
    }
}

/// A queued task entry for priority queue ordering.
///
/// This struct is used internally by the queue to maintain
/// proper ordering based on priority and creation time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedTask {
    /// Task identifier.
    pub task_id: Uuid,
    /// Task priority.
    pub priority: TaskPriority,
    /// Task creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl QueuedTask {
    /// Creates a new queued task entry.
    pub fn new(task_id: Uuid, priority: TaskPriority, created_at: DateTime<Utc>) -> Self {
        Self {
            task_id,
            priority,
            created_at,
        }
    }

    /// Creates a queued task entry from a ScanTask.
    pub fn from_scan_task(task: &ScanTask) -> Self {
        Self {
            task_id: task.id,
            priority: task.priority,
            created_at: task.created_at,
        }
    }
}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // For BinaryHeap (max-heap), "greater" elements are popped first.
        // We want: High priority before Normal, and earlier times before later times.
        // So: High > Normal (natural order), and earlier > later (reversed time order)
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::High > TaskPriority::Normal);
    }

    #[test]
    fn test_task_priority_serialize() {
        let high = TaskPriority::High;
        let json = serde_json::to_string(&high).unwrap();
        assert_eq!(json, "\"High\"");

        let normal = TaskPriority::Normal;
        let json = serde_json::to_string(&normal).unwrap();
        assert_eq!(json, "\"Normal\"");
    }

    #[test]
    fn test_task_status_serialize() {
        let pending = TaskStatus::Pending;
        let json = serde_json::to_string(&pending).unwrap();
        assert_eq!(json, "\"Pending\"");
    }

    #[test]
    fn test_scan_task_new() {
        let task = ScanTask::new(1, TaskPriority::High);
        assert_eq!(task.library_id, 1);
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
        assert!(task.progress.is_none());
        assert!(task.result.is_none());
        assert!(task.error.is_none());
    }

    #[test]
    fn test_queued_task_ordering_by_priority() {
        let now = Utc::now();
        let high_task = QueuedTask::new(Uuid::new_v4(), TaskPriority::High, now);
        let normal_task = QueuedTask::new(Uuid::new_v4(), TaskPriority::Normal, now);

        // High priority should be "greater" so it gets popped first from BinaryHeap (max-heap)
        assert!(high_task > normal_task);
    }

    #[test]
    fn test_queued_task_ordering_by_time() {
        let now = Utc::now();
        let earlier = now - Duration::seconds(10);

        let earlier_task = QueuedTask::new(Uuid::new_v4(), TaskPriority::Normal, earlier);
        let later_task = QueuedTask::new(Uuid::new_v4(), TaskPriority::Normal, now);

        // Earlier task should be "greater" so it gets popped first from BinaryHeap (max-heap)
        assert!(earlier_task > later_task);
    }

    #[test]
    fn test_queued_task_from_scan_task() {
        let scan_task = ScanTask::new(1, TaskPriority::High);
        let queued = QueuedTask::from_scan_task(&scan_task);

        assert_eq!(queued.task_id, scan_task.id);
        assert_eq!(queued.priority, scan_task.priority);
        assert_eq!(queued.created_at, scan_task.created_at);
    }
}
