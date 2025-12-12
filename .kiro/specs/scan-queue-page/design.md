# Design Document: Scan Queue Page

## Overview

扫描队列页面是一个 Vue 3 组件，用于展示和管理内容库扫描任务。该页面将使用现有的 `useScanTaskStore` Pinia store 来获取和管理任务数据，并提供直观的用户界面来监控扫描进度和管理任务。

## Architecture

```mermaid
graph TB
    subgraph Frontend
        SQP[ScanQueue.vue Page]
        STS[useScanTaskStore]
        LS[useLibraryStore]
        AC[ApiClient]
    end
    
    subgraph Backend
        API[/api/scan-tasks]
    end
    
    SQP --> STS
    SQP --> LS
    STS --> AC
    LS --> AC
    AC --> API
```

页面采用组合式 API (Composition API) 和 `<script setup>` 语法，遵循项目现有的代码风格。

## Components and Interfaces

### ScanQueue.vue

主页面组件，负责：
- 从 `useScanTaskStore` 获取任务数据
- 从 `useLibraryStore` 获取库名称映射
- 渲染任务列表（分为待处理和历史两个区域）
- 处理取消任务操作

```typescript
// 组件内部状态
interface ComponentState {
  cancellingTaskId: string | null  // 正在取消的任务ID
}

// 计算属性
const libraryNameMap: ComputedRef<Map<number, string>>
const pendingTasks: ComputedRef<ScanTask[]>
const historyTasks: ComputedRef<ScanTask[]>
```

### UI Components Used

- `Card`, `CardHeader`, `CardContent` - 任务卡片容器
- `Skeleton` - 加载占位符
- `Button` - 取消按钮
- `Sonner` (toast) - 错误通知

## Data Models

### ScanTask (existing)

```typescript
interface ScanTask {
  id: string
  library_id: number
  priority: TaskPriority
  status: TaskStatus
  created_at: string
  started_at: string | null
  completed_at: string | null
  progress: TaskProgress | null
  result: TaskResult | null
  error: string | null
}
```

### TaskProgress (existing)

```typescript
interface TaskProgress {
  scanned_paths: number
  total_paths: number
}
```

### TaskResult (existing)

```typescript
interface TaskResult {
  added_count: number
  removed_count: number
  failed_scrape_count: number
}
```

### TaskStatus (existing)

```typescript
type TaskStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Task Categorization Correctness

*For any* set of scan tasks, tasks with status Pending or Running SHALL appear in the pending section, and tasks with status Completed, Failed, or Cancelled SHALL appear in the history section.

**Validates: Requirements 4.1, 4.2, 4.3**

### Property 2: Library Name Display

*For any* scan task displayed on the page, the rendered output SHALL contain the library name corresponding to the task's library_id.

**Validates: Requirements 2.2**

### Property 3: Cancel Button Visibility

*For any* scan task with status Pending or Running, a cancel button SHALL be visible; for tasks with other statuses, no cancel button SHALL be present.

**Validates: Requirements 3.1**

### Property 4: Running Task Progress Display

*For any* scan task with Running status and non-null progress, the rendered output SHALL display the progress percentage calculated as (scanned_paths / total_paths * 100).

**Validates: Requirements 2.4**

### Property 5: Completed Task Result Display

*For any* scan task with Completed status and non-null result, the rendered output SHALL contain the added_count, removed_count, and failed_scrape_count values.

**Validates: Requirements 2.5**

### Property 6: Failed Task Error Display

*For any* scan task with Failed status and non-null error, the rendered output SHALL contain the error message.

**Validates: Requirements 2.6**

## Error Handling

| Error Scenario | Handling Strategy |
|----------------|-------------------|
| Network error during task fetch | Display error message via store's error state, show retry option |
| Cancel task fails | Display toast notification with error message |
| Library not found for task | Display "Unknown Library" as fallback |

## Testing Strategy

### Property-Based Testing

使用 `fast-check` 库进行属性测试，验证组件的核心逻辑：

1. **Task Categorization** - 生成随机任务列表，验证分类逻辑正确性
2. **Cancel Button Logic** - 生成随机状态的任务，验证取消按钮显示逻辑
3. **Progress Calculation** - 生成随机进度数据，验证百分比计算

每个属性测试配置运行至少 100 次迭代。

### Unit Testing

使用 `vitest` 进行单元测试：

1. 组件挂载时调用 fetchTasks
2. 加载状态显示骨架屏
3. 空状态显示提示信息
4. 取消按钮点击触发 cancelTask
5. 路由配置正确
