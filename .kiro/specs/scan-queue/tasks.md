# Implementation Plan

- [x] 1. Create scan queue data models





  - [x] 1.1 Define TaskPriority, TaskStatus, ScanTask, TaskProgress, TaskResult types in `backend/src/models/scan_queue.rs`


    - Implement Serialize/Deserialize for API responses
    - Implement Ord for QueuedTask to support priority queue ordering
    - _Requirements: 1.1, 1.3, 5.1, 5.2_
  - [x] 1.2 Write property test for queue ordering


    - **Property 2: Queue Ordering by Priority and Time**
    - **Validates: Requirements 1.3, 5.3**


- [x] 2. Implement ScanQueueService core functionality




  - [x] 2.1 Create `backend/src/services/scan_queue.rs` with basic structure


    - Initialize pending_queue, tasks, library_tasks data structures
    - Implement new() constructor
    - _Requirements: 1.2_
  - [x] 2.2 Implement submit_task with deduplication logic

    - Check library_tasks for existing task
    - Handle priority upgrade for duplicate submissions
    - Create new task if no existing task
    - _Requirements: 1.1, 4.1, 4.2, 4.3_
  - [x] 2.3 Write property test for task creation


    - **Property 1: Task Creation Adds to Pending Queue**
    - **Validates: Requirements 1.1, 1.2**
  - [x] 2.4 Write property test for deduplication

    - **Property 7: Deduplication Returns Existing Task**
    - **Validates: Requirements 4.1, 4.2**
  - [x] 2.5 Write property test for priority upgrade

    - **Property 8: Priority Upgrade on Duplicate Submission**
    - **Validates: Requirements 4.3**


- [x] 3. Implement task query and cancel operations




  - [x] 3.1 Implement get_task and get_library_task methods


    - Return task by ID or library ID
    - _Requirements: 2.1_

  - [x] 3.2 Implement cancel_task method

    - Validate task status before cancellation
    - Remove from pending queue if pending
    - Set cancellation flag for running tasks
    - _Requirements: 3.1, 3.2, 3.3_
  - [x] 3.3 Write property test for task query


    - **Property 3: Task Query Returns Correct Status**
    - **Validates: Requirements 2.1**
  - [x] 3.4 Write property test for cancel pending task

    - **Property 5: Cancel Pending Task Changes Status**
    - **Validates: Requirements 3.1**
  - [x] 3.5 Write property test for cancel completed task error

    - **Property 6: Cancel Completed Task Returns Error**
    - **Validates: Requirements 3.3**


- [x] 4. Implement history and listing operations




  - [x] 4.1 Implement list_pending and list_history methods


    - Filter by time for history (24 hours default)
    - Sort results appropriately
    - _Requirements: 2.3, 6.3_
  - [x] 4.2 Write property test for history query


    - **Property 4: History Query Returns All Recent Tasks**
    - **Validates: Requirements 2.3, 6.3**


- [x] 5. Implement background worker




  - [x] 5.1 Create worker task that processes queue


    - Pop tasks from priority queue
    - Execute scan via ScanService
    - Update task status and result
    - Handle cancellation signals
    - _Requirements: 1.3, 2.2, 6.1, 6.2_
  - [x] 5.2 Implement shutdown method for graceful termination


    - Send cancel signal to worker
    - Wait for current task to complete
    - _Requirements: 3.2_


- [x] 6. Integrate with existing services




  - [x] 6.1 Add ScanQueueService to AppState


    - Update `backend/src/state.rs`
    - Initialize queue service with scan service reference
    - _Requirements: 1.1_

  - [x] 6.2 Update SchedulerService to use queue

    - Modify `backend/src/services/scheduler.rs` to submit tasks to queue with Normal priority
    - _Requirements: 5.2_

  - [x] 6.3 Export scan_queue module from services/mod.rs

    - _Requirements: 1.1_

- [x] 7. Create API endpoints






  - [x] 7.1 Add scan queue handlers in `backend/src/handlers/scan_queue.rs`

    - POST /api/libraries/{id}/scan - submit scan task (High priority)
    - GET /api/scan-tasks/{id} - get task status
    - GET /api/scan-tasks - list all tasks
    - DELETE /api/scan-tasks/{id} - cancel task
    - _Requirements: 1.1, 2.1, 2.3, 3.1_

  - [x] 7.2 Register routes in router

    - Update `backend/src/router.rs`
    - _Requirements: 1.1_

- [x] 8. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.
