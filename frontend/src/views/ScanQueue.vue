<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useScanTaskStore } from '@/stores/useScanTaskStore'
import { useLibraryStore } from '@/stores/useLibraryStore'
import { Card, CardHeader, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { toast } from 'vue-sonner'
import type { ScanTask } from '@/api/types'

const scanTaskStore = useScanTaskStore()
const libraryStore = useLibraryStore()

// Component state
const cancellingTaskId = ref<string | null>(null)

// Fetch tasks on mount
onMounted(async () => {
    await Promise.all([
        scanTaskStore.fetchTasks(),
        libraryStore.fetchLibraries()
    ])
})

// Computed properties
const libraryNameMap = computed(() => {
    const map = new Map<number, string>()
    libraryStore.libraries.forEach(lib => {
        map.set(lib.id, lib.name)
    })
    return map
})

const pendingTasks = computed(() => scanTaskStore.pendingTasks)
const historyTasks = computed(() => scanTaskStore.historyTasks)
const loading = computed(() => scanTaskStore.loading)

// Helper functions
function getLibraryName(libraryId: number): string {
    return libraryNameMap.value.get(libraryId) || '未知库'
}

function formatTime(dateString: string): string {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
    })
}

function getStatusBadgeClass(status: string): string {
    switch (status) {
        case 'Pending':
            return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
        case 'Running':
            return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'
        case 'Completed':
            return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
        case 'Failed':
            return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
        case 'Cancelled':
            return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
        default:
            return 'bg-gray-100 text-gray-800'
    }
}

function getStatusText(status: string): string {
    switch (status) {
        case 'Pending': return '等待中'
        case 'Running': return '运行中'
        case 'Completed': return '已完成'
        case 'Failed': return '失败'
        case 'Cancelled': return '已取消'
        default: return status
    }
}

function calculateProgress(task: ScanTask): number {
    if (!task.progress || task.progress.total_paths === 0) return 0
    return Math.round((task.progress.scanned_paths / task.progress.total_paths) * 100)
}

function canCancel(task: ScanTask): boolean {
    return task.status === 'Pending' || task.status === 'Running'
}

async function handleCancel(taskId: string) {
    cancellingTaskId.value = taskId
    try {
        await scanTaskStore.cancelTask(taskId)
    } catch (e) {
        toast.error('取消任务失败', {
            description: e instanceof Error ? e.message : '未知错误'
        })
    } finally {
        cancellingTaskId.value = null
    }
}
</script>

<template>
    <div class="p-6 space-y-8">
        <!-- Page Title -->
        <h1 class="text-2xl font-bold">扫描队列</h1>

        <!-- Loading State -->
        <template v-if="loading">
            <div class="space-y-4">
                <Skeleton class="h-8 w-32" />
                <div class="grid gap-4">
                    <Skeleton v-for="i in 3" :key="i" class="h-32 w-full rounded-lg" />
                </div>
            </div>
        </template>

        <!-- Content -->
        <template v-else>
            <!-- Empty State - No tasks at all -->
            <div v-if="pendingTasks.length === 0 && historyTasks.length === 0"
                class="flex flex-col items-center justify-center py-20 text-muted-foreground">
                <p class="text-lg">暂无扫描任务</p>
                <p class="text-sm">在库设置中触发扫描以添加任务</p>
            </div>

            <template v-else>
                <!-- Pending Tasks Section -->
                <section class="space-y-4">
                    <h2 class="text-lg font-semibold">待处理任务</h2>

                    <!-- Empty pending state -->
                    <div v-if="pendingTasks.length === 0"
                        class="text-muted-foreground py-4 text-center border rounded-lg">
                        当前没有活动任务
                    </div>

                    <!-- Pending task cards -->
                    <div v-else class="grid gap-4">
                        <Card v-for="task in pendingTasks" :key="task.id" class="py-4">
                            <CardHeader class="pb-2">
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <span
                                            :class="['px-2 py-1 rounded-full text-xs font-medium', getStatusBadgeClass(task.status)]">
                                            {{ getStatusText(task.status) }}
                                        </span>
                                        <span class="font-medium">{{ getLibraryName(task.library_id) }}</span>
                                    </div>
                                    <Button v-if="canCancel(task)" variant="outline" size="sm"
                                        :disabled="cancellingTaskId === task.id" @click="handleCancel(task.id)">
                                        {{ cancellingTaskId === task.id ? '取消中...' : '取消' }}
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent>
                                <div class="text-sm text-muted-foreground">
                                    创建时间: {{ formatTime(task.created_at) }}
                                </div>
                                <!-- Progress for running tasks -->
                                <div v-if="task.status === 'Running' && task.progress" class="mt-3">
                                    <div class="flex items-center justify-between text-sm mb-1">
                                        <span>扫描进度</span>
                                        <span>{{ calculateProgress(task) }}%</span>
                                    </div>
                                    <div class="w-full bg-gray-200 rounded-full h-2 dark:bg-gray-700">
                                        <div class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                                            :style="{ width: `${calculateProgress(task)}%` }"></div>
                                    </div>
                                    <div class="text-xs text-muted-foreground mt-1">
                                        {{ task.progress.scanned_paths }} / {{ task.progress.total_paths }} 路径
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </section>

                <!-- History Tasks Section -->
                <section v-if="historyTasks.length > 0" class="space-y-4">
                    <h2 class="text-lg font-semibold">历史记录</h2>

                    <div class="grid gap-4">
                        <Card v-for="task in historyTasks" :key="task.id" class="py-4">
                            <CardHeader class="pb-2">
                                <div class="flex items-center gap-3">
                                    <span
                                        :class="['px-2 py-1 rounded-full text-xs font-medium', getStatusBadgeClass(task.status)]">
                                        {{ getStatusText(task.status) }}
                                    </span>
                                    <span class="font-medium">{{ getLibraryName(task.library_id) }}</span>
                                </div>
                            </CardHeader>
                            <CardContent>
                                <div class="text-sm text-muted-foreground">
                                    创建时间: {{ formatTime(task.created_at) }}
                                </div>

                                <!-- Result for completed tasks -->
                                <div v-if="task.status === 'Completed' && task.result" class="mt-3 flex gap-4 text-sm">
                                    <span class="text-green-600 dark:text-green-400">
                                        新增: {{ task.result.added_count }}
                                    </span>
                                    <span class="text-red-600 dark:text-red-400">
                                        删除: {{ task.result.removed_count }}
                                    </span>
                                    <span v-if="task.result.failed_scrape_count > 0"
                                        class="text-yellow-600 dark:text-yellow-400">
                                        抓取失败: {{ task.result.failed_scrape_count }}
                                    </span>
                                </div>

                                <!-- Error for failed tasks -->
                                <div v-if="task.status === 'Failed' && task.error"
                                    class="mt-3 text-sm text-red-600 dark:text-red-400">
                                    错误: {{ task.error }}
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </section>
            </template>
        </template>
    </div>
</template>
