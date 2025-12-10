# Requirements Document

## Introduction

本功能将扫描目录操作改造为基于队列的异步执行模式。当前的扫描服务是同步执行的，当用户触发扫描或定时扫描时，扫描操作会阻塞直到完成。通过引入扫描队列，可以实现任务的异步执行、优先级管理、状态追踪和重复任务去重等功能。

## Glossary

- **ScanQueue**: 扫描队列，用于管理待执行的扫描任务
- **ScanTask**: 扫描任务，表示一个待执行或正在执行的扫描操作
- **TaskPriority**: 任务优先级，决定任务在队列中的执行顺序
- **TaskStatus**: 任务状态，包括 Pending（等待中）、Running（执行中）、Completed（已完成）、Failed（失败）、Cancelled（已取消）
- **Library**: 媒体库，包含多个扫描路径的内容集合
- **ScanPath**: 扫描路径，媒体库中的一个具体目录路径

## Requirements

### Requirement 1

**User Story:** As a user, I want scan operations to be queued and executed asynchronously, so that the API responds immediately without waiting for the scan to complete.

#### Acceptance Criteria

1. WHEN a user triggers a library scan THEN the ScanQueue SHALL create a new ScanTask and return a task identifier immediately
2. WHEN a ScanTask is created THEN the ScanQueue SHALL add the task to the pending queue for background processing
3. WHILE a ScanTask is in the queue THEN the ScanQueue SHALL process tasks in priority order followed by creation time order

### Requirement 2

**User Story:** As a user, I want to query the status of scan tasks, so that I can monitor the progress of ongoing scans.

#### Acceptance Criteria

1. WHEN a user queries a task by identifier THEN the ScanQueue SHALL return the current TaskStatus and progress information
2. WHEN a ScanTask is running THEN the ScanQueue SHALL track and report the number of items scanned and total items
3. WHEN a user queries all tasks THEN the ScanQueue SHALL return a list of recent tasks with their statuses

### Requirement 3

**User Story:** As a user, I want to cancel pending scan tasks, so that I can stop unnecessary scans before they execute.

#### Acceptance Criteria

1. WHEN a user cancels a pending ScanTask THEN the ScanQueue SHALL remove the task from the queue and set its status to Cancelled
2. WHEN a user attempts to cancel a running ScanTask THEN the ScanQueue SHALL signal the task to stop and set its status to Cancelled
3. IF a user attempts to cancel a completed or already cancelled task THEN the ScanQueue SHALL return an error indicating the task cannot be cancelled

### Requirement 4

**User Story:** As a user, I want duplicate scan requests to be deduplicated, so that the same library is not scanned multiple times unnecessarily.

#### Acceptance Criteria

1. WHEN a scan request is submitted for a library that already has a pending task THEN the ScanQueue SHALL return the existing task identifier instead of creating a new task
2. WHEN a scan request is submitted for a library that is currently being scanned THEN the ScanQueue SHALL return the running task identifier
3. WHEN a scan request with higher priority is submitted for a library with a pending lower-priority task THEN the ScanQueue SHALL upgrade the existing task priority

### Requirement 5

**User Story:** As a user, I want manual scan requests to have higher priority than scheduled scans, so that my explicit actions are processed first.

#### Acceptance Criteria

1. WHEN a user manually triggers a scan THEN the ScanQueue SHALL assign High priority to the task
2. WHEN a scheduled scan is triggered THEN the ScanQueue SHALL assign Normal priority to the task
3. WHILE processing the queue THEN the ScanQueue SHALL execute High priority tasks before Normal priority tasks

### Requirement 6

**User Story:** As a developer, I want the scan queue to persist task history, so that users can view past scan results.

#### Acceptance Criteria

1. WHEN a ScanTask completes THEN the ScanQueue SHALL store the task result including added, removed, and failed items count
2. WHEN a ScanTask fails THEN the ScanQueue SHALL store the error message with the task record
3. WHEN querying task history THEN the ScanQueue SHALL return tasks from the last 24 hours by default
