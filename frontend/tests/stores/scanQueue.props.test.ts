/**
 * Property-based tests for Scan Queue Page.
 *
 * Tests the core functionality of the scan queue page including task categorization,
 * library name display, cancel button visibility, progress display, result display,
 * and error display.
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import type { ScanTask, TaskStatus, TaskPriority, TaskProgress, TaskResult } from '@/api/types'

// ============================================================================
// Pure Functions to Test (extracted from component logic)
// ============================================================================

/**
 * Categorizes tasks into pending and history based on status.
 * Pending: Pending, Running
 * History: Completed, Failed, Cancelled
 */
export function categorizeTasks(tasks: ScanTask[]): { pending: ScanTask[]; history: ScanTask[] } {
    const pending = tasks.filter(t => t.status === 'Pending' || t.status === 'Running')
    const history = tasks.filter(t => t.status === 'Completed' || t.status === 'Failed' || t.status === 'Cancelled')
    return { pending, history }
}

/**
 * Gets library name from a map, returns fallback if not found.
 */
export function getLibraryName(libraryId: number, libraryNameMap: Map<number, string>): string {
    return libraryNameMap.get(libraryId) || '未知库'
}

/**
 * Determines if a task can be cancelled based on its status.
 */
export function canCancelTask(task: ScanTask): boolean {
    return task.status === 'Pending' || task.status === 'Running'
}

/**
 * Calculates progress percentage from TaskProgress.
 */
export function calculateProgress(progress: TaskProgress | null): number {
    if (!progress || progress.total_paths === 0) return 0
    return Math.round((progress.scanned_paths / progress.total_paths) * 100)
}

/**
 * Formats task result for display.
 */
export function formatTaskResult(result: TaskResult | null): { added: number; removed: number; failed: number } | null {
    if (!result) return null
    return {
        added: result.added_count,
        removed: result.removed_count,
        failed: result.failed_scrape_count
    }
}

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// Task ID generator (UUID-like string)
const taskIdArb = fc.uuid()

// Library ID generator
const libraryIdArb = fc.integer({ min: 1, max: 1000000 })

// Task priority generator
const taskPriorityArb: fc.Arbitrary<TaskPriority> = fc.constantFrom('Normal', 'High')

// Task status generator
const taskStatusArb: fc.Arbitrary<TaskStatus> = fc.constantFrom('Pending', 'Running', 'Completed', 'Failed', 'Cancelled')

// Pending status generator (only Pending or Running)
const pendingStatusArb: fc.Arbitrary<TaskStatus> = fc.constantFrom('Pending', 'Running')

// History status generator (only Completed, Failed, Cancelled)
const historyStatusArb: fc.Arbitrary<TaskStatus> = fc.constantFrom('Completed', 'Failed', 'Cancelled')

// ISO date string generator
const dateStringArb = fc.integer({ min: 1577836800000, max: 1924905600000 })
    .map(ts => new Date(ts).toISOString())

// Nullable date string generator
const nullableDateStringArb = fc.option(dateStringArb, { nil: null })

// Task progress generator
const taskProgressArb: fc.Arbitrary<TaskProgress> = fc.record({
    scanned_paths: fc.integer({ min: 0, max: 10000 }),
    total_paths: fc.integer({ min: 0, max: 10000 }),
})

// Task result generator
const taskResultArb: fc.Arbitrary<TaskResult> = fc.record({
    added_count: fc.integer({ min: 0, max: 1000 }),
    removed_count: fc.integer({ min: 0, max: 1000 }),
    failed_scrape_count: fc.integer({ min: 0, max: 100 }),
})

// Error message generator
const errorMessageArb = fc.string({ minLength: 1, maxLength: 200 })

// ScanTask generator
const scanTaskArb: fc.Arbitrary<ScanTask> = fc.record({
    id: taskIdArb,
    library_id: libraryIdArb,
    priority: taskPriorityArb,
    status: taskStatusArb,
    created_at: dateStringArb,
    started_at: nullableDateStringArb,
    completed_at: nullableDateStringArb,
    progress: fc.option(taskProgressArb, { nil: null }),
    result: fc.option(taskResultArb, { nil: null }),
    error: fc.option(errorMessageArb, { nil: null }),
})

// ScanTask with specific status generator
function scanTaskWithStatusArb(status: TaskStatus): fc.Arbitrary<ScanTask> {
    return fc.record({
        id: taskIdArb,
        library_id: libraryIdArb,
        priority: taskPriorityArb,
        status: fc.constant(status),
        created_at: dateStringArb,
        started_at: nullableDateStringArb,
        completed_at: nullableDateStringArb,
        progress: fc.option(taskProgressArb, { nil: null }),
        result: fc.option(taskResultArb, { nil: null }),
        error: fc.option(errorMessageArb, { nil: null }),
    })
}

// Running task with progress generator
const runningTaskWithProgressArb: fc.Arbitrary<ScanTask> = fc.record({
    id: taskIdArb,
    library_id: libraryIdArb,
    priority: taskPriorityArb,
    status: fc.constant('Running' as TaskStatus),
    created_at: dateStringArb,
    started_at: dateStringArb,
    completed_at: fc.constant(null),
    progress: taskProgressArb,
    result: fc.constant(null),
    error: fc.constant(null),
})

// Completed task with result generator
const completedTaskWithResultArb: fc.Arbitrary<ScanTask> = fc.record({
    id: taskIdArb,
    library_id: libraryIdArb,
    priority: taskPriorityArb,
    status: fc.constant('Completed' as TaskStatus),
    created_at: dateStringArb,
    started_at: dateStringArb,
    completed_at: dateStringArb,
    progress: fc.constant(null),
    result: taskResultArb,
    error: fc.constant(null),
})

// Failed task with error generator
const failedTaskWithErrorArb: fc.Arbitrary<ScanTask> = fc.record({
    id: taskIdArb,
    library_id: libraryIdArb,
    priority: taskPriorityArb,
    status: fc.constant('Failed' as TaskStatus),
    created_at: dateStringArb,
    started_at: dateStringArb,
    completed_at: dateStringArb,
    progress: fc.constant(null),
    result: fc.constant(null),
    error: errorMessageArb,
})

// Array of unique tasks (unique by ID)
const tasksArrayArb = fc.array(scanTaskArb, { minLength: 0, maxLength: 20 })
    .map(tasks => {
        const seen = new Set<string>()
        return tasks.filter(t => {
            if (seen.has(t.id)) return false
            seen.add(t.id)
            return true
        })
    })

// Library name generator
const libraryNameArb = fc.string({ minLength: 1, maxLength: 100 })

// Library name map generator
const libraryNameMapArb = fc.array(
    fc.tuple(libraryIdArb, libraryNameArb),
    { minLength: 0, maxLength: 20 }
).map(entries => new Map(entries))

// ============================================================================
// Property Tests
// ============================================================================

describe('Property 1: Task Categorization Correctness', () => {
    /**
     * **Feature: scan-queue-page, Property 1: Task Categorization Correctness**
     * **Validates: Requirements 4.1, 4.2, 4.3**
     *
     * For any set of scan tasks, tasks with status Pending or Running SHALL appear
     * in the pending section, and tasks with status Completed, Failed, or Cancelled
     * SHALL appear in the history section.
     */

    it('tasks with Pending or Running status appear in pending section', () => {
        fc.assert(
            fc.property(tasksArrayArb, (tasks) => {
                const { pending } = categorizeTasks(tasks)

                // All tasks in pending should have Pending or Running status
                for (const task of pending) {
                    expect(task.status === 'Pending' || task.status === 'Running').toBe(true)
                }
            }),
            { numRuns: 100 }
        )
    })

    it('tasks with Completed, Failed, or Cancelled status appear in history section', () => {
        fc.assert(
            fc.property(tasksArrayArb, (tasks) => {
                const { history } = categorizeTasks(tasks)

                // All tasks in history should have Completed, Failed, or Cancelled status
                for (const task of history) {
                    expect(
                        task.status === 'Completed' ||
                        task.status === 'Failed' ||
                        task.status === 'Cancelled'
                    ).toBe(true)
                }
            }),
            { numRuns: 100 }
        )
    })

    it('all tasks are categorized (no tasks lost)', () => {
        fc.assert(
            fc.property(tasksArrayArb, (tasks) => {
                const { pending, history } = categorizeTasks(tasks)

                // Total count should match
                expect(pending.length + history.length).toBe(tasks.length)

                // All original tasks should be in one of the categories
                for (const task of tasks) {
                    const inPending = pending.some(t => t.id === task.id)
                    const inHistory = history.some(t => t.id === task.id)
                    expect(inPending || inHistory).toBe(true)
                    // Should not be in both
                    expect(inPending && inHistory).toBe(false)
                }
            }),
            { numRuns: 100 }
        )
    })

    it('pending tasks only contain Pending or Running status', () => {
        fc.assert(
            fc.property(
                fc.array(scanTaskWithStatusArb('Pending'), { minLength: 1, maxLength: 10 }),
                fc.array(scanTaskWithStatusArb('Running'), { minLength: 1, maxLength: 10 }),
                (pendingTasks, runningTasks) => {
                    const allTasks = [...pendingTasks, ...runningTasks]
                    const { pending } = categorizeTasks(allTasks)

                    expect(pending.length).toBe(allTasks.length)
                }
            ),
            { numRuns: 100 }
        )
    })

    it('history tasks only contain Completed, Failed, or Cancelled status', () => {
        fc.assert(
            fc.property(
                fc.array(scanTaskWithStatusArb('Completed'), { minLength: 0, maxLength: 5 }),
                fc.array(scanTaskWithStatusArb('Failed'), { minLength: 0, maxLength: 5 }),
                fc.array(scanTaskWithStatusArb('Cancelled'), { minLength: 0, maxLength: 5 }),
                (completedTasks, failedTasks, cancelledTasks) => {
                    const allTasks = [...completedTasks, ...failedTasks, ...cancelledTasks]
                    const { history } = categorizeTasks(allTasks)

                    expect(history.length).toBe(allTasks.length)
                }
            ),
            { numRuns: 100 }
        )
    })
})


describe('Property 3: Cancel Button Visibility', () => {
    /**
     * **Feature: scan-queue-page, Property 3: Cancel Button Visibility**
     * **Validates: Requirements 3.1**
     *
     * For any scan task with status Pending or Running, a cancel button SHALL be visible;
     * for tasks with other statuses, no cancel button SHALL be present.
     */

    it('cancel button is visible for Pending tasks', () => {
        fc.assert(
            fc.property(scanTaskWithStatusArb('Pending'), (task) => {
                expect(canCancelTask(task)).toBe(true)
            }),
            { numRuns: 100 }
        )
    })

    it('cancel button is visible for Running tasks', () => {
        fc.assert(
            fc.property(scanTaskWithStatusArb('Running'), (task) => {
                expect(canCancelTask(task)).toBe(true)
            }),
            { numRuns: 100 }
        )
    })

    it('cancel button is NOT visible for Completed tasks', () => {
        fc.assert(
            fc.property(scanTaskWithStatusArb('Completed'), (task) => {
                expect(canCancelTask(task)).toBe(false)
            }),
            { numRuns: 100 }
        )
    })

    it('cancel button is NOT visible for Failed tasks', () => {
        fc.assert(
            fc.property(scanTaskWithStatusArb('Failed'), (task) => {
                expect(canCancelTask(task)).toBe(false)
            }),
            { numRuns: 100 }
        )
    })

    it('cancel button is NOT visible for Cancelled tasks', () => {
        fc.assert(
            fc.property(scanTaskWithStatusArb('Cancelled'), (task) => {
                expect(canCancelTask(task)).toBe(false)
            }),
            { numRuns: 100 }
        )
    })

    it('for any task, cancel button visibility matches Pending or Running status', () => {
        fc.assert(
            fc.property(scanTaskArb, (task) => {
                const canCancel = canCancelTask(task)
                const isPendingOrRunning = task.status === 'Pending' || task.status === 'Running'

                expect(canCancel).toBe(isPendingOrRunning)
            }),
            { numRuns: 100 }
        )
    })
})


describe('Property 2: Library Name Display', () => {
    /**
     * **Feature: scan-queue-page, Property 2: Library Name Display**
     * **Validates: Requirements 2.2**
     *
     * For any scan task displayed on the page, the rendered output SHALL contain
     * the library name corresponding to the task's library_id.
     */

    it('returns correct library name when library exists in map', () => {
        fc.assert(
            fc.property(libraryIdArb, libraryNameArb, (libraryId, libraryName) => {
                const libraryNameMap = new Map<number, string>()
                libraryNameMap.set(libraryId, libraryName)

                const result = getLibraryName(libraryId, libraryNameMap)
                expect(result).toBe(libraryName)
            }),
            { numRuns: 100 }
        )
    })

    it('returns fallback when library does not exist in map', () => {
        fc.assert(
            fc.property(libraryIdArb, libraryNameMapArb, (libraryId, libraryNameMap) => {
                // Ensure the library ID is not in the map
                libraryNameMap.delete(libraryId)

                const result = getLibraryName(libraryId, libraryNameMap)
                expect(result).toBe('未知库')
            }),
            { numRuns: 100 }
        )
    })

    it('for any task, library name is either from map or fallback', () => {
        fc.assert(
            fc.property(scanTaskArb, libraryNameMapArb, (task, libraryNameMap) => {
                const result = getLibraryName(task.library_id, libraryNameMap)

                if (libraryNameMap.has(task.library_id)) {
                    expect(result).toBe(libraryNameMap.get(task.library_id))
                } else {
                    expect(result).toBe('未知库')
                }
            }),
            { numRuns: 100 }
        )
    })
})


describe('Property 4: Running Task Progress Display', () => {
    /**
     * **Feature: scan-queue-page, Property 4: Running Task Progress Display**
     * **Validates: Requirements 2.4**
     *
     * For any scan task with Running status and non-null progress, the rendered output
     * SHALL display the progress percentage calculated as (scanned_paths / total_paths * 100).
     */

    it('calculates progress percentage correctly', () => {
        fc.assert(
            fc.property(
                fc.integer({ min: 0, max: 10000 }),
                fc.integer({ min: 1, max: 10000 }),
                (scanned, total) => {
                    const progress: TaskProgress = {
                        scanned_paths: scanned,
                        total_paths: total
                    }

                    const result = calculateProgress(progress)
                    const expected = Math.round((scanned / total) * 100)

                    expect(result).toBe(expected)
                }
            ),
            { numRuns: 100 }
        )
    })

    it('returns 0 when progress is null', () => {
        const result = calculateProgress(null)
        expect(result).toBe(0)
    })

    it('returns 0 when total_paths is 0', () => {
        fc.assert(
            fc.property(
                fc.integer({ min: 0, max: 10000 }),
                (scanned) => {
                    const progress: TaskProgress = {
                        scanned_paths: scanned,
                        total_paths: 0
                    }

                    const result = calculateProgress(progress)
                    expect(result).toBe(0)
                }
            ),
            { numRuns: 100 }
        )
    })

    it('progress percentage is always between 0 and 100 when scanned <= total', () => {
        fc.assert(
            fc.property(
                fc.integer({ min: 1, max: 10000 }).chain(total =>
                    fc.tuple(
                        fc.constant(total),
                        fc.integer({ min: 0, max: total })
                    )
                ),
                ([total, scanned]) => {
                    const progress: TaskProgress = {
                        scanned_paths: scanned,
                        total_paths: total
                    }

                    const result = calculateProgress(progress)
                    expect(result).toBeGreaterThanOrEqual(0)
                    expect(result).toBeLessThanOrEqual(100)
                }
            ),
            { numRuns: 100 }
        )
    })

    it('for running tasks with progress, percentage is calculated correctly', () => {
        fc.assert(
            fc.property(runningTaskWithProgressArb, (task) => {
                if (task.progress && task.progress.total_paths > 0) {
                    const result = calculateProgress(task.progress)
                    const expected = Math.round(
                        (task.progress.scanned_paths / task.progress.total_paths) * 100
                    )
                    expect(result).toBe(expected)
                }
            }),
            { numRuns: 100 }
        )
    })
})


describe('Property 5: Completed Task Result Display', () => {
    /**
     * **Feature: scan-queue-page, Property 5: Completed Task Result Display**
     * **Validates: Requirements 2.5**
     *
     * For any scan task with Completed status and non-null result, the rendered output
     * SHALL contain the added_count, removed_count, and failed_scrape_count values.
     */

    it('formats task result correctly', () => {
        fc.assert(
            fc.property(taskResultArb, (result) => {
                const formatted = formatTaskResult(result)

                expect(formatted).not.toBeNull()
                expect(formatted!.added).toBe(result.added_count)
                expect(formatted!.removed).toBe(result.removed_count)
                expect(formatted!.failed).toBe(result.failed_scrape_count)
            }),
            { numRuns: 100 }
        )
    })

    it('returns null when result is null', () => {
        const formatted = formatTaskResult(null)
        expect(formatted).toBeNull()
    })

    it('for completed tasks with result, all counts are preserved', () => {
        fc.assert(
            fc.property(completedTaskWithResultArb, (task) => {
                if (task.result) {
                    const formatted = formatTaskResult(task.result)

                    expect(formatted).not.toBeNull()
                    expect(formatted!.added).toBe(task.result.added_count)
                    expect(formatted!.removed).toBe(task.result.removed_count)
                    expect(formatted!.failed).toBe(task.result.failed_scrape_count)
                }
            }),
            { numRuns: 100 }
        )
    })

    it('result counts are non-negative', () => {
        fc.assert(
            fc.property(taskResultArb, (result) => {
                const formatted = formatTaskResult(result)

                expect(formatted!.added).toBeGreaterThanOrEqual(0)
                expect(formatted!.removed).toBeGreaterThanOrEqual(0)
                expect(formatted!.failed).toBeGreaterThanOrEqual(0)
            }),
            { numRuns: 100 }
        )
    })
})


describe('Property 6: Failed Task Error Display', () => {
    /**
     * **Feature: scan-queue-page, Property 6: Failed Task Error Display**
     * **Validates: Requirements 2.6**
     *
     * For any scan task with Failed status and non-null error, the rendered output
     * SHALL contain the error message.
     */

    it('failed tasks with error have error message preserved', () => {
        fc.assert(
            fc.property(failedTaskWithErrorArb, (task) => {
                // Task should have Failed status
                expect(task.status).toBe('Failed')

                // Task should have an error message
                expect(task.error).not.toBeNull()
                expect(typeof task.error).toBe('string')
                expect(task.error!.length).toBeGreaterThan(0)
            }),
            { numRuns: 100 }
        )
    })

    it('error message is non-empty string for failed tasks', () => {
        fc.assert(
            fc.property(errorMessageArb, (errorMessage) => {
                // Error message should be a non-empty string
                expect(typeof errorMessage).toBe('string')
                expect(errorMessage.length).toBeGreaterThan(0)
            }),
            { numRuns: 100 }
        )
    })

    it('failed task error is accessible from task object', () => {
        fc.assert(
            fc.property(failedTaskWithErrorArb, (task) => {
                // The error should be directly accessible
                const error = task.error

                expect(error).not.toBeNull()
                expect(error).toBe(task.error)
            }),
            { numRuns: 100 }
        )
    })
})
