<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useScanTaskStore } from '@/stores/useScanTaskStore'
import { useLibraryStore } from '@/stores/useLibraryStore'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { toast } from 'vue-sonner'
import type { ScanTask } from '@/api/types'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Dialog,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
    DialogDescription,
    DialogScrollContent
} from '@/components/ui/dialog'
import { FileText, BookOpen, File } from 'lucide-vue-next'
import { useIntervalFn } from '@vueuse/core'
import { useI18n } from 'vue-i18n'

const scanTaskStore = useScanTaskStore()
const libraryStore = useLibraryStore()
const { t } = useI18n()

// Component state
const cancellingTaskId = ref<string | null>(null)

// Fetch tasks on mount
onMounted(async () => {
    await Promise.all([
        scanTaskStore.fetchTasks(),
        libraryStore.fetchLibraries()
    ])
})

// Poll tasks every 3 seconds
useIntervalFn(() => {
    scanTaskStore.fetchTasks(50, true)
}, 3000)

// Computed properties
const libraryNameMap = computed(() => {
    const map = new Map<number, string>()
    libraryStore.libraries.forEach(lib => {
        map.set(lib.id, lib.name)
    })
    return map
})

const pendingTasks = computed(() => scanTaskStore.pendingTasks)
const processingTasks = computed(() => scanTaskStore.processingTasks)
const historyTasks = computed(() => scanTaskStore.historyTasks)
const loading = computed(() => scanTaskStore.loading)

// Helper functions
function getLibraryName(libraryId: number): string {
    return libraryNameMap.value.get(libraryId) || t('scan_queue.unknown_library')
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

function getStatusLabel(status: string): string {
    switch (status) {
        case 'Pending': return t('scan_queue.status_pending')
        case 'Running': return t('scan_queue.status_running')
        case 'Completed': return t('scan_queue.status_completed')
        case 'Failed': return t('scan_queue.status_failed')
        case 'Cancelled': return t('scan_queue.status_cancelled')
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
        toast.error(t('scan_queue.cancel_fail'), {
            description: e instanceof Error ? e.message : 'Unknown error'
        })
    } finally {
        cancellingTaskId.value = null
    }
}
</script>

<template>
    <div class="flex flex-col h-[calc(100vh-7rem)] p-6 gap-8">
        <!-- Page Title -->
        <h1 class="text-2xl font-bold shrink-0">{{ t('scan_queue.title') }}</h1>

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
            <div v-if="pendingTasks.length === 0 && historyTasks.length === 0 && processingTasks.length === 0"
                class="flex flex-col items-center justify-center flex-1 py-20 text-muted-foreground">
                <p class="text-lg">{{ t('scan_queue.no_tasks') }}</p>
                <p class="text-sm">{{ t('scan_queue.no_tasks_desc') }}</p>
            </div>

            <template v-else>
                <!-- Pending Tasks Section -->
                <section class="space-y-4 shrink-0">
                    <!-- Empty pending state -->
                    <div v-if="pendingTasks.length === 0 && processingTasks.length === 0"
                        class="text-muted-foreground py-4 text-center border rounded-lg">
                        {{ t('scan_queue.no_active_tasks') }}
                    </div>

                    <!-- Pending task table -->
                    <div v-else class="rounded-md border">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead class="w-[150px]">
                                        {{ t('scan_queue.status') }}
                                    </TableHead>
                                    <TableHead>
                                        {{ t('scan_queue.library') }}
                                    </TableHead>
                                    <TableHead>
                                        {{ t('scan_queue.created_at') }}
                                    </TableHead>
                                    <TableHead>
                                        {{ t('scan_queue.progress') }}
                                    </TableHead>
                                    <TableHead class="text-right">
                                        {{ t('scan_queue.actions') }}
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <TableRow v-for="task in [...pendingTasks, ...processingTasks]" :key="task.id">
                                    <TableCell>
                                        <span
                                            :class="['px-2 py-1 rounded-full text-xs font-medium', getStatusBadgeClass(task.status)]">
                                            {{ getStatusLabel(task.status) }}
                                        </span>
                                    </TableCell>
                                    <TableCell>
                                        {{ getLibraryName(task.library_id) }}
                                    </TableCell>
                                    <TableCell>
                                        {{ formatTime(task.created_at) }}
                                    </TableCell>
                                    <TableCell>
                                        <div v-if="task.status === 'Running' && task.progress" class="w-48">
                                            <div class="flex items-center justify-between text-sm mb-1">
                                                <span>{{ t('scan_queue.scan_progress') }}</span>
                                                <span>{{ calculateProgress(task) }}%</span>
                                            </div>
                                            <div class="w-full bg-gray-200 rounded-full h-2 dark:bg-gray-700">
                                                <div class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                                                    :style="{ width: `${calculateProgress(task)}%` }"></div>
                                            </div>
                                            <div class="text-xs text-muted-foreground mt-1">
                                                {{ t('scan_queue.paths_scanned', {
                                                    scanned: task.progress.scanned_paths,
                                                    total: task.progress.total_paths
                                                }) }}
                                            </div>
                                        </div>
                                        <span v-else class="text-muted-foreground text-sm">-</span>
                                    </TableCell>
                                    <TableCell class="text-right">
                                        <Button v-if="canCancel(task)" variant="outline" size="sm"
                                            :disabled="cancellingTaskId === task.id" @click="handleCancel(task.id)">
                                            {{ cancellingTaskId === task.id ? t('scan_queue.cancelling') :
                                                t('scan_queue.cancel') }}
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            </TableBody>
                        </Table>
                    </div>
                </section>

                <!-- History Tasks Section -->
                <section v-if="historyTasks.length > 0" class="flex flex-col flex-1 min-h-0 gap-4">
                    <h2 class="text-lg font-semibold shrink-0">{{ t('scan_queue.history') }}</h2>

                    <div class="rounded-md border flex-1 overflow-auto relative">
                        <table class="w-full caption-bottom text-sm">
                            <TableHeader class="sticky top-0 z-10 bg-background shadow-sm">
                                <TableRow>
                                    <TableHead class="w-[150px]">
                                        {{ t('scan_queue.status') }}
                                    </TableHead>
                                    <TableHead>
                                        {{ t('scan_queue.library') }}
                                    </TableHead>
                                    <TableHead>
                                        {{ t('scan_queue.created_at') }}
                                    </TableHead>
                                    <TableHead>
                                        {{ t('scan_queue.result') }}
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <TableRow v-for="task in historyTasks" :key="task.id">
                                    <TableCell>
                                        <span
                                            :class="['px-2 py-1 rounded-full text-xs font-medium', getStatusBadgeClass(task.status)]">
                                            {{ getStatusLabel(task.status) }}
                                        </span>
                                    </TableCell>
                                    <TableCell>
                                        {{ getLibraryName(task.library_id) }}
                                    </TableCell>
                                    <TableCell>
                                        {{ formatTime(task.created_at) }}
                                    </TableCell>
                                    <TableCell>
                                        <!-- Result for completed tasks -->
                                        <div v-if="task.status === 'Completed' && task.result"
                                            class="flex flex-col gap-1 text-sm">

                                            <span v-if="task.result.failed_scrape_count > 0"
                                                class="text-yellow-600 dark:text-yellow-400">
                                                {{ t('scan_queue.failed_scrapes', {
                                                    count:
                                                        task.result.failed_scrape_count
                                                }) }}
                                            </span>

                                            <!-- Details Button -->
                                            <div
                                                v-else-if="(task.result.added_contents && task.result.added_contents.length > 0) || (task.result.added_chapters && task.result.added_chapters.length > 0)">
                                                <Dialog>
                                                    <DialogTrigger as-child>
                                                        <Button variant="outline" size="sm" class="h-7 text-xs">
                                                            <FileText class="w-3 h-3 mr-1" />
                                                            {{ t('scan_queue.details') }}
                                                        </Button>
                                                    </DialogTrigger>
                                                    <DialogScrollContent class="max-w-2xl">
                                                        <DialogHeader>
                                                            <DialogTitle>{{ t('scan_queue.scan_results_details') }}
                                                            </DialogTitle>
                                                            <DialogDescription>
                                                                {{ getLibraryName(task.library_id) }} - {{
                                                                    formatTime(task.created_at) }}
                                                            </DialogDescription>
                                                        </DialogHeader>

                                                        <div class="space-y-6">
                                                            <!-- Added Content -->
                                                            <div
                                                                v-if="task.result.added_contents && task.result.added_contents.length > 0">
                                                                <h3 class="text-sm font-medium mb-2 flex items-center">
                                                                    <BookOpen class="w-4 h-4 mr-2" />
                                                                    {{ t('scan_queue.added_content_list') }} ({{
                                                                        task.result.added_contents.length }})
                                                                </h3>
                                                                <div
                                                                    class="bg-muted/50 rounded-md p-2 text-sm max-h-[200px] overflow-y-auto">
                                                                    <div v-for="(content, idx) in task.result.added_contents"
                                                                        :key="idx"
                                                                        class="py-1 border-b last:border-0 border-border/50">
                                                                        <div class="font-medium">{{ content.content_name
                                                                        }}</div>
                                                                        <div
                                                                            class="text-xs text-muted-foreground truncate">
                                                                            {{ content.path }}</div>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <!-- Added Chapters -->
                                                            <div
                                                                v-if="task.result.added_chapters && task.result.added_chapters.length > 0">
                                                                <h3 class="text-sm font-medium mb-2 flex items-center">
                                                                    <File class="w-4 h-4 mr-2" />
                                                                    {{ t('scan_queue.added_chapters_list') }} ({{
                                                                        task.result.added_chapters.length }})
                                                                </h3>
                                                                <div
                                                                    class="bg-muted/50 rounded-md p-2 text-sm max-h-[200px] overflow-y-auto">
                                                                    <div v-for="(chapter, idx) in task.result.added_chapters"
                                                                        :key="idx"
                                                                        class="py-1 border-b last:border-0 border-border/50">
                                                                        <div class="flex justify-between gap-2">
                                                                            <span class="font-medium">{{
                                                                                chapter.chapter_name }}</span>
                                                                            <span
                                                                                class="text-muted-foreground text-xs">{{
                                                                                    chapter.content_name }}</span>
                                                                        </div>
                                                                        <div
                                                                            class="text-xs text-muted-foreground truncate">
                                                                            {{ chapter.path }}</div>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </DialogScrollContent>
                                                </Dialog>
                                            </div>

                                            <span v-else class="text-muted-foreground">
                                                {{ t('scan_queue.no_new_content') }}
                                            </span>

                                        </div>

                                        <!-- Error for failed tasks -->
                                        <div v-else-if="task.status === 'Failed' && task.error"
                                            class="text-sm text-red-600 dark:text-red-400">
                                            {{ t('scan_queue.error_prefix', { error: task.error }) }}
                                        </div>
                                        <span v-else class="text-muted-foreground text-sm">-</span>
                                    </TableCell>
                                </TableRow>
                            </TableBody>
                        </table>
                    </div>
                </section>
            </template>
        </template>
    </div>
</template>
