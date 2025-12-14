//! Property tests for scan queue data models and service.
//!
//! This module contains property-based tests for the scan queue ordering logic
//! and the ScanQueueService functionality.

use backend::models::{QueuedTask, TaskPriority, TaskStatus};
use backend::services::scan_queue::ScanQueueService;
use chrono::{Duration, Utc};
use proptest::prelude::*;
use std::collections::BinaryHeap;
use tokio::runtime::Runtime;
use uuid::Uuid;

// ============================================================================
// Test Utilities
// ============================================================================

/// Strategy to generate a random TaskPriority.
fn arb_priority() -> impl Strategy<Value = TaskPriority> {
    prop_oneof![Just(TaskPriority::Normal), Just(TaskPriority::High),]
}

/// Strategy to generate a random time offset in seconds (for creating different timestamps).
fn arb_time_offset_secs() -> impl Strategy<Value = i64> {
    0i64..3600 // Up to 1 hour offset
}

// ============================================================================
// Property 2: Queue Ordering by Priority and Time
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 2: Queue Ordering by Priority and Time**
    /// **Validates: Requirements 1.3, 5.3**
    ///
    /// For any set of tasks with different priorities and creation times,
    /// when tasks are dequeued, High priority tasks should be processed before
    /// Normal priority tasks, and within the same priority, earlier tasks
    /// should be processed first.
    #[test]
    fn queue_ordering_by_priority_and_time(
        num_high in 1usize..10,
        num_normal in 1usize..10,
    ) {
        let base_time = Utc::now();

        // Create high priority tasks with varying timestamps
        let high_tasks: Vec<QueuedTask> = (0..num_high)
            .map(|i| {
                QueuedTask::new(
                    Uuid::new_v4(),
                    TaskPriority::High,
                    base_time + Duration::seconds(i as i64 * 10),
                )
            })
            .collect();

        // Create normal priority tasks with varying timestamps
        let normal_tasks: Vec<QueuedTask> = (0..num_normal)
            .map(|i| {
                QueuedTask::new(
                    Uuid::new_v4(),
                    TaskPriority::Normal,
                    base_time + Duration::seconds(i as i64 * 10),
                )
            })
            .collect();

        // Add all tasks to a BinaryHeap
        let mut heap: BinaryHeap<QueuedTask> = BinaryHeap::new();
        for task in &high_tasks {
            heap.push(task.clone());
        }
        for task in &normal_tasks {
            heap.push(task.clone());
        }

        // Pop all tasks and verify ordering
        let mut popped: Vec<QueuedTask> = Vec::new();
        while let Some(task) = heap.pop() {
            popped.push(task);
        }

        // Verify: All high priority tasks come before all normal priority tasks
        let high_count = popped.iter().take_while(|t| t.priority == TaskPriority::High).count();
        prop_assert_eq!(high_count, num_high, "All high priority tasks should come first");

        // Verify: Within high priority, tasks are ordered by creation time (earliest first)
        let high_portion: Vec<_> = popped.iter().take(num_high).collect();
        for i in 1..high_portion.len() {
            prop_assert!(
                high_portion[i - 1].created_at <= high_portion[i].created_at,
                "High priority tasks should be ordered by creation time"
            );
        }

        // Verify: Within normal priority, tasks are ordered by creation time (earliest first)
        let normal_portion: Vec<_> = popped.iter().skip(num_high).collect();
        for i in 1..normal_portion.len() {
            prop_assert!(
                normal_portion[i - 1].created_at <= normal_portion[i].created_at,
                "Normal priority tasks should be ordered by creation time"
            );
        }
    }

    /// **Feature: scan-queue, Property 2: Queue Ordering by Priority and Time**
    /// **Validates: Requirements 1.3, 5.3**
    ///
    /// For any two tasks, the one with higher priority should be popped first.
    /// If priorities are equal, the one created earlier should be popped first.
    #[test]
    fn pairwise_ordering_consistency(
        priority1 in arb_priority(),
        priority2 in arb_priority(),
        offset1 in arb_time_offset_secs(),
        offset2 in arb_time_offset_secs(),
    ) {
        let base_time = Utc::now();
        let task1 = QueuedTask::new(
            Uuid::new_v4(),
            priority1,
            base_time + Duration::seconds(offset1),
        );
        let task2 = QueuedTask::new(
            Uuid::new_v4(),
            priority2,
            base_time + Duration::seconds(offset2),
        );

        let mut heap: BinaryHeap<QueuedTask> = BinaryHeap::new();
        heap.push(task1.clone());
        heap.push(task2.clone());

        let first = heap.pop().unwrap();
        let _second = heap.pop().unwrap();

        // Verify ordering based on priority first, then time
        if priority1 != priority2 {
            // Different priorities: higher priority comes first
            if priority1 > priority2 {
                prop_assert_eq!(first.priority, priority1, "Higher priority task should come first");
            } else {
                prop_assert_eq!(first.priority, priority2, "Higher priority task should come first");
            }
        } else {
            // Same priority: earlier created task comes first
            let earlier_time = std::cmp::min(
                base_time + Duration::seconds(offset1),
                base_time + Duration::seconds(offset2),
            );
            prop_assert_eq!(first.created_at, earlier_time, "Earlier task should come first when priorities are equal");
        }
    }

    /// **Feature: scan-queue, Property 2: Queue Ordering by Priority and Time**
    /// **Validates: Requirements 1.3, 5.3**
    ///
    /// For any sequence of random tasks added to the queue, the dequeue order
    /// should always satisfy: no Normal priority task is dequeued while High
    /// priority tasks remain in the queue.
    #[test]
    fn no_priority_inversion(task_count in 2usize..20) {
        let base_time = Utc::now();

        // Generate random tasks
        let mut tasks: Vec<QueuedTask> = Vec::new();
        for i in 0..task_count {
            let priority = if i % 3 == 0 {
                TaskPriority::High
            } else {
                TaskPriority::Normal
            };
            tasks.push(QueuedTask::new(
                Uuid::new_v4(),
                priority,
                base_time + Duration::seconds(i as i64),
            ));
        }

        // Shuffle and add to heap
        let mut heap: BinaryHeap<QueuedTask> = BinaryHeap::new();
        for task in tasks {
            heap.push(task);
        }

        // Track if we've seen a Normal priority task
        let mut seen_normal = false;

        while let Some(task) = heap.pop() {
            if task.priority == TaskPriority::Normal {
                seen_normal = true;
            }
            if seen_normal && task.priority == TaskPriority::High {
                prop_assert!(false, "High priority task found after Normal priority task - priority inversion!");
            }
        }
    }
}

// ============================================================================
// Property 1: Task Creation Adds to Pending Queue
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 1: Task Creation Adds to Pending Queue**
    /// **Validates: Requirements 1.1, 1.2**
    ///
    /// For any library_id and priority, when submit_task is called, the returned
    /// task_id should correspond to a task in the queue with status Pending
    /// (unless deduplicated to an existing task).
    #[test]
    fn task_creation_adds_to_pending_queue(
        library_id in 1i64..1000,
        priority in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit a task
            let task_id = service.submit_task(library_id, priority).await;

            // Verify the task exists and has correct properties
            let task = service.get_task(task_id).await;
            prop_assert!(task.is_some(), "Task should exist after submission");

            let task = task.unwrap();
            prop_assert_eq!(task.id, task_id, "Task ID should match");
            prop_assert_eq!(task.library_id, library_id, "Library ID should match");
            prop_assert_eq!(task.priority, priority, "Priority should match");
            prop_assert_eq!(task.status, TaskStatus::Pending, "Status should be Pending");

            // Verify the task is in the pending queue
            let pending_count = service.pending_count().await;
            prop_assert!(pending_count >= 1, "Pending queue should have at least one task");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 1: Task Creation Adds to Pending Queue**
    /// **Validates: Requirements 1.1, 1.2**
    ///
    /// For any sequence of unique library IDs, submitting tasks should result
    /// in exactly that many tasks in the pending queue.
    #[test]
    fn multiple_task_creation_increases_queue_size(
        num_tasks in 1usize..20,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit tasks for different libraries
            for i in 0..num_tasks {
                service.submit_task(i as i64, TaskPriority::Normal).await;
            }

            // Verify queue size matches number of unique libraries
            let pending_count = service.pending_count().await;
            prop_assert_eq!(pending_count, num_tasks, "Queue size should match number of unique tasks");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 7: Deduplication Returns Existing Task
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 7: Deduplication Returns Existing Task**
    /// **Validates: Requirements 4.1, 4.2**
    ///
    /// For any library_id that already has a pending task, calling submit_task
    /// should return the existing task_id instead of creating a new task.
    #[test]
    fn deduplication_returns_existing_task(
        library_id in 1i64..1000,
        priority1 in arb_priority(),
        priority2 in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit first task
            let task_id1 = service.submit_task(library_id, priority1).await;

            // Submit second task for same library
            let task_id2 = service.submit_task(library_id, priority2).await;

            // Both should return the same task ID
            prop_assert_eq!(task_id1, task_id2, "Duplicate submission should return same task ID");

            // Queue should only have one task
            let pending_count = service.pending_count().await;
            prop_assert_eq!(pending_count, 1, "Queue should have exactly one task after duplicate submission");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 7: Deduplication Returns Existing Task**
    /// **Validates: Requirements 4.1, 4.2**
    ///
    /// For any set of library IDs with duplicates, the final queue size should
    /// equal the number of unique library IDs.
    #[test]
    fn deduplication_maintains_unique_library_tasks(
        submissions in prop::collection::vec((1i64..100, arb_priority()), 1..30),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit all tasks
            for (library_id, priority) in &submissions {
                service.submit_task(*library_id, *priority).await;
            }

            // Count unique library IDs
            let unique_libraries: std::collections::HashSet<i64> =
                submissions.iter().map(|(lib_id, _)| *lib_id).collect();

            // Queue size should equal unique library count
            let pending_count = service.pending_count().await;
            prop_assert_eq!(
                pending_count,
                unique_libraries.len(),
                "Queue size should equal number of unique libraries"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 8: Priority Upgrade on Duplicate Submission
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 8: Priority Upgrade on Duplicate Submission**
    /// **Validates: Requirements 4.3**
    ///
    /// For any library_id with a pending Normal priority task, submitting a High
    /// priority task for the same library should upgrade the existing task's
    /// priority to High.
    #[test]
    fn priority_upgrade_on_duplicate_submission(
        library_id in 1i64..1000,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit Normal priority task first
            let task_id = service.submit_task(library_id, TaskPriority::Normal).await;

            // Verify initial priority
            let task = service.get_task(task_id).await.unwrap();
            prop_assert_eq!(task.priority, TaskPriority::Normal, "Initial priority should be Normal");

            // Submit High priority task for same library
            let task_id2 = service.submit_task(library_id, TaskPriority::High).await;

            // Should return same task ID
            prop_assert_eq!(task_id, task_id2, "Should return same task ID");

            // Priority should be upgraded
            let task = service.get_task(task_id).await.unwrap();
            prop_assert_eq!(task.priority, TaskPriority::High, "Priority should be upgraded to High");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 8: Priority Upgrade on Duplicate Submission**
    /// **Validates: Requirements 4.3**
    ///
    /// Priority should never be downgraded - submitting a lower priority task
    /// for a library with a higher priority pending task should keep the higher priority.
    #[test]
    fn priority_never_downgrades(
        library_id in 1i64..1000,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit High priority task first
            let task_id = service.submit_task(library_id, TaskPriority::High).await;

            // Submit Normal priority task for same library
            let task_id2 = service.submit_task(library_id, TaskPriority::Normal).await;

            // Should return same task ID
            prop_assert_eq!(task_id, task_id2, "Should return same task ID");

            // Priority should remain High (not downgraded)
            let task = service.get_task(task_id).await.unwrap();
            prop_assert_eq!(task.priority, TaskPriority::High, "Priority should remain High");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 3: Task Query Returns Correct Status
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 3: Task Query Returns Correct Status**
    /// **Validates: Requirements 2.1**
    ///
    /// For any task that was submitted, querying by its task_id should return
    /// the task with the correct current status.
    #[test]
    fn task_query_returns_correct_status(
        library_id in 1i64..1000,
        priority in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit a task
            let task_id = service.submit_task(library_id, priority).await;

            // Query the task by ID
            let task = service.get_task(task_id).await;
            prop_assert!(task.is_some(), "Task should be found by ID");

            let task = task.unwrap();
            prop_assert_eq!(task.id, task_id, "Task ID should match");
            prop_assert_eq!(task.library_id, library_id, "Library ID should match");
            prop_assert_eq!(task.priority, priority, "Priority should match");
            prop_assert_eq!(task.status, TaskStatus::Pending, "Initial status should be Pending");

            // Query by library ID
            let library_task = service.get_library_task(library_id).await;
            prop_assert!(library_task.is_some(), "Task should be found by library ID");

            let library_task = library_task.unwrap();
            prop_assert_eq!(library_task.id, task_id, "Task ID should match when queried by library");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 3: Task Query Returns Correct Status**
    /// **Validates: Requirements 2.1**
    ///
    /// For any non-existent task ID, querying should return None.
    #[test]
    fn task_query_nonexistent_returns_none(
        library_id in 1i64..1000,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Query a random non-existent task ID
            let random_id = Uuid::new_v4();
            let task = service.get_task(random_id).await;
            prop_assert!(task.is_none(), "Non-existent task should return None");

            // Query a non-existent library
            let library_task = service.get_library_task(library_id).await;
            prop_assert!(library_task.is_none(), "Non-existent library task should return None");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 3: Task Query Returns Correct Status**
    /// **Validates: Requirements 2.1**
    ///
    /// For any set of submitted tasks, each task should be queryable and return
    /// consistent information.
    #[test]
    fn multiple_tasks_query_consistency(
        num_tasks in 1usize..20,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit multiple tasks
            let mut task_ids: Vec<(i64, Uuid)> = Vec::new();
            for i in 0..num_tasks {
                let library_id = i as i64;
                let task_id = service.submit_task(library_id, TaskPriority::Normal).await;
                task_ids.push((library_id, task_id));
            }

            // Verify each task is queryable
            for (library_id, task_id) in &task_ids {
                let task = service.get_task(*task_id).await;
                prop_assert!(task.is_some(), "Task {} should exist", task_id);

                let task = task.unwrap();
                prop_assert_eq!(task.library_id, *library_id, "Library ID should match");

                let library_task = service.get_library_task(*library_id).await;
                prop_assert!(library_task.is_some(), "Library task should exist");
                prop_assert_eq!(library_task.unwrap().id, *task_id, "Task IDs should match");
            }

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 5: Cancel Pending Task Changes Status
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 5: Cancel Pending Task Changes Status**
    /// **Validates: Requirements 3.1**
    ///
    /// For any task with Pending status, calling cancel_task should change its
    /// status to Cancelled and remove it from the pending queue.
    #[test]
    fn cancel_pending_task_changes_status(
        library_id in 1i64..1000,
        priority in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit a task
            let task_id = service.submit_task(library_id, priority).await;

            // Verify task is pending
            let task = service.get_task(task_id).await.unwrap();
            prop_assert_eq!(task.status, TaskStatus::Pending, "Task should be Pending before cancel");

            // Cancel the task
            let result = service.cancel_task(task_id).await;
            prop_assert!(result.is_ok(), "Cancel should succeed for pending task");

            // Verify status changed to Cancelled
            let task = service.get_task(task_id).await.unwrap();
            prop_assert_eq!(task.status, TaskStatus::Cancelled, "Task should be Cancelled after cancel");
            prop_assert!(task.completed_at.is_some(), "Completed_at should be set");

            // Verify task is removed from pending queue
            let pending_count = service.pending_count().await;
            prop_assert_eq!(pending_count, 0, "Pending queue should be empty after cancel");

            // Verify library_tasks mapping is cleared
            let library_task = service.get_library_task(library_id).await;
            prop_assert!(library_task.is_none(), "Library task mapping should be cleared");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 5: Cancel Pending Task Changes Status**
    /// **Validates: Requirements 3.1**
    ///
    /// For any set of pending tasks, cancelling one should only affect that task
    /// and leave others unchanged.
    #[test]
    fn cancel_one_task_preserves_others(
        num_tasks in 2usize..10,
        cancel_index in 0usize..10,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit multiple tasks
            let mut task_ids: Vec<Uuid> = Vec::new();
            for i in 0..num_tasks {
                let task_id = service.submit_task(i as i64, TaskPriority::Normal).await;
                task_ids.push(task_id);
            }

            // Cancel one task (use modulo to ensure valid index)
            let index_to_cancel = cancel_index % num_tasks;
            let task_to_cancel = task_ids[index_to_cancel];
            service.cancel_task(task_to_cancel).await.unwrap();

            // Verify cancelled task has Cancelled status
            let cancelled_task = service.get_task(task_to_cancel).await.unwrap();
            prop_assert_eq!(cancelled_task.status, TaskStatus::Cancelled, "Cancelled task should have Cancelled status");

            // Verify other tasks are still Pending
            for (i, task_id) in task_ids.iter().enumerate() {
                if i != index_to_cancel {
                    let task = service.get_task(*task_id).await.unwrap();
                    prop_assert_eq!(task.status, TaskStatus::Pending, "Other tasks should remain Pending");
                }
            }

            // Verify pending count is reduced by 1
            let pending_count = service.pending_count().await;
            prop_assert_eq!(pending_count, num_tasks - 1, "Pending count should be reduced by 1");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 6: Cancel Completed Task Returns Error
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 6: Cancel Completed Task Returns Error**
    /// **Validates: Requirements 3.3**
    ///
    /// For any task with Completed status, calling cancel_task should return an error.
    #[test]
    fn cancel_completed_task_returns_error(
        library_id in 1i64..1000,
        priority in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit a task
            let task_id = service.submit_task(library_id, priority).await;

            // Set task to Completed using test helper
            service.set_task_status_for_test(task_id, TaskStatus::Completed, None).await;

            // Try to cancel - should fail
            let result = service.cancel_task(task_id).await;
            prop_assert!(result.is_err(), "Cancel should fail for completed task");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 6: Cancel Completed Task Returns Error**
    /// **Validates: Requirements 3.3**
    ///
    /// For any task with Failed status, calling cancel_task should return an error.
    #[test]
    fn cancel_failed_task_returns_error(
        library_id in 1i64..1000,
        priority in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit a task
            let task_id = service.submit_task(library_id, priority).await;

            // Set task to Failed using test helper
            service.set_task_status_for_test(task_id, TaskStatus::Failed, Some("Test error".to_string())).await;

            // Try to cancel - should fail
            let result = service.cancel_task(task_id).await;
            prop_assert!(result.is_err(), "Cancel should fail for failed task");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 6: Cancel Completed Task Returns Error**
    /// **Validates: Requirements 3.3**
    ///
    /// For any task with Cancelled status, calling cancel_task again should return an error.
    #[test]
    fn cancel_already_cancelled_task_returns_error(
        library_id in 1i64..1000,
        priority in arb_priority(),
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit a task
            let task_id = service.submit_task(library_id, priority).await;

            // Cancel the task
            service.cancel_task(task_id).await.unwrap();

            // Try to cancel again - should fail
            let result = service.cancel_task(task_id).await;
            prop_assert!(result.is_err(), "Cancel should fail for already cancelled task");

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 6: Cancel Completed Task Returns Error**
    /// **Validates: Requirements 3.3**
    ///
    /// For any non-existent task ID, calling cancel_task should return an error.
    #[test]
    fn cancel_nonexistent_task_returns_error(
        _dummy in 0i32..100,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Try to cancel a non-existent task
            let random_id = Uuid::new_v4();
            let result = service.cancel_task(random_id).await;
            prop_assert!(result.is_err(), "Cancel should fail for non-existent task");

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 4: History Query Returns All Recent Tasks
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: scan-queue, Property 4: History Query Returns All Recent Tasks**
    /// **Validates: Requirements 2.3, 6.3**
    ///
    /// For any set of submitted tasks that are completed within the retention period,
    /// querying history should return all those tasks.
    #[test]
    fn history_query_returns_recent_completed_tasks(
        num_tasks in 1usize..10,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit and complete multiple tasks
            let mut task_ids: Vec<Uuid> = Vec::new();
            for i in 0..num_tasks {
                let task_id = service.submit_task(i as i64, TaskPriority::Normal).await;
                task_ids.push(task_id);
            }

            // Complete all tasks
            for task_id in &task_ids {
                service.set_task_status_for_test(*task_id, TaskStatus::Completed, None).await;
            }

            // Query history
            let history = service.list_history(100).await;

            // All completed tasks should be in history
            prop_assert_eq!(
                history.len(),
                num_tasks,
                "History should contain all completed tasks"
            );

            // Verify all task IDs are present
            for task_id in &task_ids {
                let found = history.iter().any(|t| t.id == *task_id);
                prop_assert!(found, "Task {} should be in history", task_id);
            }

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 4: History Query Returns All Recent Tasks**
    /// **Validates: Requirements 2.3, 6.3**
    ///
    /// For any set of tasks with different statuses (Completed, Failed, Cancelled),
    /// all non-pending tasks should appear in history.
    #[test]
    fn history_includes_all_terminal_statuses(
        num_completed in 1usize..5,
        num_failed in 1usize..5,
        num_cancelled in 1usize..5,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            let mut library_id = 0i64;
            let mut completed_ids: Vec<Uuid> = Vec::new();
            let mut failed_ids: Vec<Uuid> = Vec::new();
            let mut cancelled_ids: Vec<Uuid> = Vec::new();

            // Create completed tasks
            for _ in 0..num_completed {
                let task_id = service.submit_task(library_id, TaskPriority::Normal).await;
                service.set_task_status_for_test(task_id, TaskStatus::Completed, None).await;
                completed_ids.push(task_id);
                library_id += 1;
            }

            // Create failed tasks
            for _ in 0..num_failed {
                let task_id = service.submit_task(library_id, TaskPriority::Normal).await;
                service.set_task_status_for_test(task_id, TaskStatus::Failed, Some("Test error".to_string())).await;
                failed_ids.push(task_id);
                library_id += 1;
            }

            // Create cancelled tasks
            for _ in 0..num_cancelled {
                let task_id = service.submit_task(library_id, TaskPriority::Normal).await;
                service.cancel_task(task_id).await.unwrap();
                cancelled_ids.push(task_id);
                library_id += 1;
            }

            // Query history
            let history = service.list_history(100).await;

            let expected_count = num_completed + num_failed + num_cancelled;
            prop_assert_eq!(
                history.len(),
                expected_count,
                "History should contain all terminal tasks"
            );

            // Verify all completed tasks are in history
            for task_id in &completed_ids {
                let found = history.iter().any(|t| t.id == *task_id && t.status == TaskStatus::Completed);
                prop_assert!(found, "Completed task {} should be in history", task_id);
            }

            // Verify all failed tasks are in history
            for task_id in &failed_ids {
                let found = history.iter().any(|t| t.id == *task_id && t.status == TaskStatus::Failed);
                prop_assert!(found, "Failed task {} should be in history", task_id);
            }

            // Verify all cancelled tasks are in history
            for task_id in &cancelled_ids {
                let found = history.iter().any(|t| t.id == *task_id && t.status == TaskStatus::Cancelled);
                prop_assert!(found, "Cancelled task {} should be in history", task_id);
            }

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 4: History Query Returns All Recent Tasks**
    /// **Validates: Requirements 2.3, 6.3**
    ///
    /// Pending tasks should NOT appear in history.
    #[test]
    fn history_excludes_pending_tasks(
        num_pending in 1usize..10,
        num_completed in 1usize..10,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            let mut library_id = 0i64;

            // Create pending tasks
            let mut pending_ids: Vec<Uuid> = Vec::new();
            for _ in 0..num_pending {
                let task_id = service.submit_task(library_id, TaskPriority::Normal).await;
                pending_ids.push(task_id);
                library_id += 1;
            }

            // Create completed tasks
            for _ in 0..num_completed {
                let task_id = service.submit_task(library_id, TaskPriority::Normal).await;
                service.set_task_status_for_test(task_id, TaskStatus::Completed, None).await;
                library_id += 1;
            }

            // Query history
            let history = service.list_history(100).await;

            // History should only contain completed tasks
            prop_assert_eq!(
                history.len(),
                num_completed,
                "History should only contain completed tasks, not pending"
            );

            // Verify no pending tasks are in history
            for task_id in &pending_ids {
                let found = history.iter().any(|t| t.id == *task_id);
                prop_assert!(!found, "Pending task {} should NOT be in history", task_id);
            }

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 4: History Query Returns All Recent Tasks**
    /// **Validates: Requirements 2.3, 6.3**
    ///
    /// History should be sorted by completed_at in descending order (most recent first).
    #[test]
    fn history_sorted_by_completion_time(
        num_tasks in 2usize..10,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit and complete multiple tasks
            for i in 0..num_tasks {
                let task_id = service.submit_task(i as i64, TaskPriority::Normal).await;
                service.set_task_status_for_test(task_id, TaskStatus::Completed, None).await;
                // Small delay to ensure different completion times
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            // Query history
            let history = service.list_history(100).await;

            // Verify sorted by completed_at descending
            for i in 1..history.len() {
                let prev_completed = history[i - 1].completed_at;
                let curr_completed = history[i].completed_at;
                prop_assert!(
                    prev_completed >= curr_completed,
                    "History should be sorted by completed_at descending"
                );
            }

            Ok(())
        })?;
    }

    /// **Feature: scan-queue, Property 4: History Query Returns All Recent Tasks**
    /// **Validates: Requirements 2.3, 6.3**
    ///
    /// History limit parameter should cap the number of returned tasks.
    #[test]
    fn history_respects_limit(
        num_tasks in 5usize..20,
        limit in 1usize..10,
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let service = ScanQueueService::new();

            // Submit and complete multiple tasks
            for i in 0..num_tasks {
                let task_id = service.submit_task(i as i64, TaskPriority::Normal).await;
                service.set_task_status_for_test(task_id, TaskStatus::Completed, None).await;
            }

            // Query history with limit
            let history = service.list_history(limit).await;

            // History should respect the limit
            let expected_count = std::cmp::min(num_tasks, limit);
            prop_assert_eq!(
                history.len(),
                expected_count,
                "History should respect the limit parameter"
            );

            Ok(())
        })?;
    }
}
