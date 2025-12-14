# Requirements Document

## Introduction

本文档定义了扫描队列页面的需求规格。扫描队列页面是一个用于展示和管理内容库扫描任务的前端界面，允许用户查看待处理任务、正在运行的任务以及历史任务记录，并提供取消任务的功能。

## Glossary

- **Scan_Queue_Page**: 扫描队列页面，用于展示和管理扫描任务的前端视图组件
- **ScanTask**: 扫描任务，表示一个内容库扫描操作的数据实体
- **TaskStatus**: 任务状态，包括 Pending（等待中）、Running（运行中）、Completed（已完成）、Failed（失败）、Cancelled（已取消）
- **TaskPriority**: 任务优先级，包括 Normal（普通）和 High（高优先级）
- **TaskProgress**: 任务进度，包含已扫描路径数和总路径数
- **TaskResult**: 任务结果，包含新增、删除和抓取失败的内容数量
- **ScanTaskStore**: Pinia 状态管理存储，用于管理扫描任务的状态和操作

## Requirements

### Requirement 1

**User Story:** As a user, I want to view all scan tasks in a unified page, so that I can monitor the scanning progress of my content libraries.

#### Acceptance Criteria

1. WHEN a user navigates to the scan queue page THEN the Scan_Queue_Page SHALL display a list of all pending and history tasks
2. WHEN the Scan_Queue_Page loads THEN the Scan_Queue_Page SHALL fetch tasks from the ScanTaskStore
3. WHEN tasks are loading THEN the Scan_Queue_Page SHALL display a loading skeleton placeholder
4. WHEN no tasks exist THEN the Scan_Queue_Page SHALL display an empty state message

### Requirement 2

**User Story:** As a user, I want to see the status and details of each scan task, so that I can understand the current state of scanning operations.

#### Acceptance Criteria

1. WHEN displaying a ScanTask THEN the Scan_Queue_Page SHALL show the task status with appropriate visual indicators (color/icon)
2. WHEN displaying a ScanTask THEN the Scan_Queue_Page SHALL show the associated library name
3. WHEN displaying a ScanTask THEN the Scan_Queue_Page SHALL show the task creation time in a human-readable format
4. WHEN a ScanTask has Running status THEN the Scan_Queue_Page SHALL display the TaskProgress as a progress bar or percentage
5. WHEN a ScanTask has Completed status THEN the Scan_Queue_Page SHALL display the TaskResult summary (added/removed/failed counts)
6. WHEN a ScanTask has Failed status THEN the Scan_Queue_Page SHALL display the error message

### Requirement 3

**User Story:** As a user, I want to cancel pending or running scan tasks, so that I can stop unnecessary scanning operations.

#### Acceptance Criteria

1. WHEN a ScanTask has Pending or Running status THEN the Scan_Queue_Page SHALL display a cancel button for that task
2. WHEN a user clicks the cancel button THEN the Scan_Queue_Page SHALL call the cancelTask action from ScanTaskStore
3. WHEN a cancel operation succeeds THEN the Scan_Queue_Page SHALL update the task display to show Cancelled status
4. WHEN a cancel operation fails THEN the Scan_Queue_Page SHALL display an error notification

### Requirement 4

**User Story:** As a user, I want to distinguish between pending tasks and completed tasks, so that I can focus on active operations.

#### Acceptance Criteria

1. WHEN displaying tasks THEN the Scan_Queue_Page SHALL separate pending tasks from history tasks in distinct sections
2. WHEN displaying the pending section THEN the Scan_Queue_Page SHALL show tasks with Pending or Running status
3. WHEN displaying the history section THEN the Scan_Queue_Page SHALL show tasks with Completed, Failed, or Cancelled status
4. WHEN the pending section is empty THEN the Scan_Queue_Page SHALL display a message indicating no active tasks

### Requirement 5

**User Story:** As a user, I want the scan queue page to be accessible from the main navigation, so that I can easily access it from anywhere in the application.

#### Acceptance Criteria

1. WHEN the application router is configured THEN the router SHALL include a route for the scan queue page at path "/scan-tasks"
2. WHEN a user is authenticated THEN the Scan_Queue_Page SHALL be accessible within the main layout
3. WHEN the sidebar navigation is displayed THEN the sidebar SHALL include a navigation item for the scan queue page
