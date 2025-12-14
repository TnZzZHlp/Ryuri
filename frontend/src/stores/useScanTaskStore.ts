/**
 * Scan Task Store - Pinia store for scan task state management
 *
 * Manages scan task queue, status tracking, and scan operations.
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ApiClient } from '@/api/client'
import type { ScanTask, SubmitScanResponse } from '@/api/types'
import { useAuthStore } from './useAuthStore'

/**
 * Response for listing scan tasks.
 */
interface ListTasksResponse {
    pending: ScanTask[]
    history: ScanTask[]
}

// Lazy-initialized API client
let apiClient: ApiClient | null = null

function getApiClient(getToken: () => string | null): ApiClient {
    if (!apiClient) {
        apiClient = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken,
        })
    }
    return apiClient
}

export const useScanTaskStore = defineStore('scanTask', () => {
    // State
    const pendingTasks = ref<ScanTask[]>([])
    const historyTasks = ref<ScanTask[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Getters
    const allTasks = computed(() => [...pendingTasks.value, ...historyTasks.value])

    const taskById = computed(() => {
        return (id: string): ScanTask | undefined => {
            return allTasks.value.find((task) => task.id === id)
        }
    })

    const taskByLibraryId = computed(() => {
        return (libraryId: number): ScanTask | undefined => {
            // Return the most recent task for this library (pending first, then history)
            return (
                pendingTasks.value.find((task) => task.library_id === libraryId) ||
                historyTasks.value.find((task) => task.library_id === libraryId)
            )
        }
    })

    const hasPendingTasks = computed(() => pendingTasks.value.length > 0)

    // Internal helper to get token from auth store
    function getToken(): string | null {
        const authStore = useAuthStore()
        return authStore.token
    }

    // Actions

    /**
     * Fetches all scan tasks (pending and history).
     */
    async function fetchTasks(limit: number = 50): Promise<void> {
        loading.value = true
        error.value = null
        try {
            const client = getApiClient(getToken)
            const response = await client.get<ListTasksResponse>('/api/scan-tasks', {
                params: { limit: limit.toString() },
            })
            pendingTasks.value = response.pending
            historyTasks.value = response.history
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to fetch scan tasks'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Fetches a single scan task by ID.
     */
    async function fetchTask(taskId: string): Promise<ScanTask> {
        try {
            const client = getApiClient(getToken)
            const task = await client.get<ScanTask>(`/api/scan-tasks/${taskId}`)

            // Update local state
            updateTaskInState(task)

            return task
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to fetch scan task'
            throw e
        }
    }

    /**
     * Triggers a scan for a library.
     */
    async function triggerScan(libraryId: number): Promise<SubmitScanResponse> {
        try {
            const client = getApiClient(getToken)
            const response = await client.post<SubmitScanResponse>(
                `/api/libraries/${libraryId}/scan`
            )

            // Add the new task to pending if not already present
            const existingIndex = pendingTasks.value.findIndex((t) => t.id === response.task.id)
            if (existingIndex === -1) {
                pendingTasks.value.unshift(response.task)
            } else {
                pendingTasks.value[existingIndex] = response.task
            }

            return response
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to trigger scan'
            throw e
        }
    }

    /**
     * Cancels a scan task.
     */
    async function cancelTask(taskId: string): Promise<ScanTask> {
        try {
            const client = getApiClient(getToken)
            const task = await client.delete<ScanTask>(`/api/scan-tasks/${taskId}`)

            // Update local state
            updateTaskInState(task)

            return task
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to cancel scan task'
            throw e
        }
    }

    /**
     * Updates a task in the local state based on its status.
     */
    function updateTaskInState(task: ScanTask): void {
        // Remove from pending if exists
        const pendingIndex = pendingTasks.value.findIndex((t) => t.id === task.id)
        if (pendingIndex !== -1) {
            pendingTasks.value.splice(pendingIndex, 1)
        }

        // Remove from history if exists
        const historyIndex = historyTasks.value.findIndex((t) => t.id === task.id)
        if (historyIndex !== -1) {
            historyTasks.value.splice(historyIndex, 1)
        }

        // Add to appropriate list based on status
        if (task.status === 'Pending' || task.status === 'Running') {
            pendingTasks.value.unshift(task)
        } else {
            historyTasks.value.unshift(task)
        }
    }

    /**
     * Clears all tasks from the store.
     */
    function clearTasks(): void {
        pendingTasks.value = []
        historyTasks.value = []
    }

    return {
        // State
        pendingTasks,
        historyTasks,
        loading,
        error,
        // Getters
        allTasks,
        taskById,
        taskByLibraryId,
        hasPendingTasks,
        // Actions
        fetchTasks,
        fetchTask,
        triggerScan,
        cancelTask,
        clearTasks,
    }
})
