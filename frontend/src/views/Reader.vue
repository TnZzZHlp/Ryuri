<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useContentStore } from '@/stores/useContentStore'
import { createContentApi } from '@/api/content'
import { createReaderApi } from '@/api/reader'
import { createProgressApi } from '@/api/progress'
import { ApiClient } from '@/api/client'
import { useAuthStore } from '@/stores/useAuthStore'
import { Button } from '@/components/ui/button'
import { useDebounceFn } from '@vueuse/core'
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
    DropdownMenuLabel,
    DropdownMenuSeparator
} from '@/components/ui/dropdown-menu'
import {
    ArrowLeft,
    ChevronLeft,
    ChevronRight,
    Settings,
    AlignJustify,
    Columns
} from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { Progress } from '@/components/ui/progress'
import type { Chapter } from '@/api/types'

// Initialize APIs
const authStore = useAuthStore()
const apiClient = new ApiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
    getToken: () => authStore.token,
})
const contentApi = createContentApi(apiClient)
const readerApi = createReaderApi(apiClient)
const progressApi = createProgressApi(apiClient)

const route = useRoute()
const router = useRouter()
const contentStore = useContentStore()

// State
const libraryId = computed(() => Number(route.params.libraryId))
const contentId = computed(() => Number(route.params.contentId))
const chapterId = computed(() => Number(route.params.chapterId))
const chapters = ref<Chapter[]>([])
const loading = ref(true)
const PRELOAD_BUFFER = 5
const pages = ref<number[]>([0, 1, 2, 3, 4]) // Buffer for scroll mode
const pageUrls = ref<Map<number, string>>(new Map())
const failedPages = ref<Set<number>>(new Set())
const endOfChapter = ref(false)
const showControls = ref(true)
const readingProgress = ref(0)

// Reader Settings
type ReaderMode = 'scroll' | 'paged'
const readerMode = ref<ReaderMode>(localStorage.getItem('reader_mode') as ReaderMode || 'paged')
const currentPage = ref(0) // For paged mode

// Computed
const currentChapter = computed(() =>
    chapters.value.find(c => c.id === chapterId.value)
)

const currentChapterIndex = computed(() =>
    chapters.value.findIndex(c => c.id === chapterId.value)
)

const prevChapter = computed(() => {
    if (currentChapterIndex.value > 0) {
        return chapters.value[currentChapterIndex.value - 1]
    }
    return null
})

const nextChapter = computed(() => {
    if (currentChapterIndex.value < chapters.value.length - 1) {
        return chapters.value[currentChapterIndex.value + 1]
    }
    return null
})

// Progress Saving
const saveProgress = useDebounceFn(async (pageIndex: number) => {
    if (!currentChapter.value) return
    const total = currentChapter.value.page_count
    const percentage = total > 0 ? ((pageIndex + 1) / total) * 100 : 0

    try {
        await progressApi.updateChapterProgress(chapterId.value, pageIndex, percentage)
    } catch (e) {
        console.warn('Failed to save progress', e)
    }
}, 1000)

// Methods
const setMode = (mode: ReaderMode) => {
    readerMode.value = mode
    localStorage.setItem('reader_mode', mode)
    // If switching to paged, ensure current page is loaded
    if (mode === 'paged') {
        loadPage(currentPage.value)
        // Preload next
        for (let i = 1; i <= PRELOAD_BUFFER; i++) {
            loadPage(currentPage.value + i)
        }
        readingProgress.value = 0
    } else {
        // If switching to scroll, ensure we have a buffer
        if (pages.value.length < PRELOAD_BUFFER) {
            // Rebuild buffer around current page
            const newPages = []
            for (let i = 0; i < PRELOAD_BUFFER; i++) {
                const p = currentPage.value + i
                if (currentChapter.value && p >= currentChapter.value.page_count) break
                newPages.push(p)
            }
            pages.value = newPages.length > 0 ? newPages : [currentPage.value]
            pages.value.forEach(p => loadPage(p))
        }
        updateProgress()
    }
}

const updateProgress = () => {
    if (readerMode.value === 'scroll') {
        const scrollTop = window.scrollY
        const docHeight = document.documentElement.scrollHeight
        const winHeight = window.innerHeight
        const total = docHeight - winHeight
        readingProgress.value = total > 0 ? (scrollTop / total) * 100 : 0

        // Find visible page
        const images = document.querySelectorAll('img[data-page-index]')
        let bestMatch = -1
        let minDiff = Infinity

        images.forEach(img => {
            const rect = img.getBoundingClientRect()
            // We want the image closest to the vertical center of the viewport
            const imgCenter = rect.top + rect.height / 2
            const diff = Math.abs(imgCenter - (window.innerHeight / 2))

            if (diff < minDiff) {
                minDiff = diff
                bestMatch = Number(img.getAttribute('data-page-index'))
            }
        })

        if (bestMatch !== -1) {
            currentPage.value = bestMatch
            saveProgress(bestMatch)
        }
    }
}

const loadPage = async (pageIndex: number) => {
    if (pageUrls.value.has(pageIndex) || failedPages.value.has(pageIndex) || !currentChapter.value) return

    if (pageIndex >= currentChapter.value.page_count) return

    try {
        const blob = await readerApi.getPageImage(contentId.value, currentChapter.value.sort_order, pageIndex)
        const url = URL.createObjectURL(blob)
        pageUrls.value.set(pageIndex, url)
    } catch (e) {
        handleImageError(pageIndex)
    }
}

const handleImageLoad = (pageIndex: number) => {
    if (readerMode.value === 'scroll') {
        // Update progress when new images load as dimensions change
        updateProgress()
    }

    if (readerMode.value !== 'scroll') return

    // Extend buffer if we are close to the end
    const maxPage = Math.max(...pages.value)

    if (pageIndex >= maxPage - PRELOAD_BUFFER + 1 && !endOfChapter.value) {
        const nextPage = maxPage + 1
        if (currentChapter.value && nextPage < currentChapter.value.page_count && !pages.value.includes(nextPage) && !failedPages.value.has(nextPage)) {
            pages.value.push(nextPage)
            loadPage(nextPage)
        }
    }
}

const handleImageError = (pageIndex: number) => {
    failedPages.value.add(pageIndex)

    // Assume 404/Error means end of chapter
    // Only mark end if it's a "reasonable" error (e.g. sequential)

    if (readerMode.value === 'scroll') {
        endOfChapter.value = true
        pages.value = pages.value.filter(p => p < pageIndex)
    } else {
        // In paged mode, if current page fails, try to handle end
        if (pageIndex === currentPage.value) {
            endOfChapter.value = true
        }
    }
}

const loadData = async () => {
    loading.value = true
    try {
        // Check store first
        if (contentStore.chapters.get(contentId.value)) {
            chapters.value = contentStore.chapters.get(contentId.value)!
        } else {
            chapters.value = await contentApi.listChapters(contentId.value)
            chapters.value.sort((a, b) => a.sort_order - b.sort_order)
        }

        // Fetch Progress
        let startPage = 0
        try {
            const progress = await progressApi.getChapterProgress(chapterId.value)
            if (progress) {
                startPage = progress.position
            }
        } catch (e) {
            // Ignore progress load errors
        }

        // Start loading initial pages
        loading.value = false
        if (readerMode.value === 'scroll') {
            const initialPages = []
            for (let i = 0; i < PRELOAD_BUFFER; i++) {
                const p = startPage + i
                if (currentChapter.value && p >= currentChapter.value.page_count) break
                initialPages.push(p)
            }
            // Fallback if empty or startPage was invalid, just load 0
            if (initialPages.length === 0) initialPages.push(0)

            pages.value = initialPages
            pages.value.forEach(p => loadPage(p))

            // Note: Scroll position restoration is not strictly handled here as images load asynchronously.
            // But we start loading from the last read page, which is the most important part.
        } else {
            currentPage.value = startPage
            loadPage(currentPage.value)
            for (let i = 1; i <= PRELOAD_BUFFER; i++) {
                loadPage(currentPage.value + i)
            }
        }

    } catch (e) {
        toast.error('Failed to load chapter')
        console.error(e)
        loading.value = false
    }
}

const navigateToChapter = (chapter: Chapter) => {
    router.replace({
        name: 'Reader',
        params: {
            contentId: contentId.value,
            chapterId: chapter.id
        }
    })
}

const toggleControls = () => {
    showControls.value = !showControls.value
}

// Paged Navigation
const nextPage = () => {
    if (endOfChapter.value) {
        if (nextChapter.value) navigateToChapter(nextChapter.value)
        return
    }

    const next = currentPage.value + 1

    // Check if next page failed
    if (failedPages.value.has(next)) {
        endOfChapter.value = true
        return
    }

    currentPage.value = next
    loadPage(next)
    for (let i = 1; i <= PRELOAD_BUFFER; i++) {
        loadPage(next + i)
    }
    window.scrollTo(0, 0)
    saveProgress(currentPage.value)
}

const prevPage = () => {
    if (currentPage.value > 0) {
        currentPage.value--
        window.scrollTo(0, 0)
        saveProgress(currentPage.value)
    } else {
        if (prevChapter.value) navigateToChapter(prevChapter.value)
    }
    endOfChapter.value = false
}

const handlePageClick = (e: MouseEvent) => {
    // Click left 30% -> prev, right 30% -> next, center -> toggle controls
    const width = window.innerWidth
    const x = e.clientX

    if (x < width * 0.3) {
        prevPage()
    } else if (x > width * 0.7) {
        nextPage()
    } else {
        toggleControls()
    }
}

// Watchers
watch(() => route.params.chapterId, () => {
    // Reset state for new chapter
    // Cleanup old URLs
    pageUrls.value.forEach(url => URL.revokeObjectURL(url))
    pageUrls.value.clear()

    // pages reset will be handled in loadData mostly, but we reset here for UI
    pages.value = []
    for (let i = 0; i < PRELOAD_BUFFER; i++) pages.value.push(i)

    currentPage.value = 0
    failedPages.value.clear()
    endOfChapter.value = false
    readingProgress.value = 0
    window.scrollTo(0, 0)

    // Trigger data reload
    loadData()
})

onMounted(() => {
    loadData()
    window.addEventListener('keydown', handleKeydown)
    window.addEventListener('scroll', updateProgress)
})

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
    window.removeEventListener('scroll', updateProgress)
    pageUrls.value.forEach(url => URL.revokeObjectURL(url))
})

const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === 'ArrowLeft') {
        if (readerMode.value === 'paged') {
            prevPage()
        } else {
            if (prevChapter.value) navigateToChapter(prevChapter.value)
        }
    } else if (e.key === 'ArrowRight') {
        if (readerMode.value === 'paged') {
            nextPage()
        } else {
            if (nextChapter.value) navigateToChapter(nextChapter.value)
        }
    } else if (e.key === 'Escape') {
        router.push(`/content/${contentId.value}`)
    } else if (e.key === ' ' || e.key === 'Space') { // Spacebar
        e.preventDefault()
        if (readerMode.value === 'paged') {
            nextPage()
        }
    }
}
</script>

<template>
    <div class="fixed inset-0 bg-black text-white overflow-auto">
        <!-- Reader Content -->

        <!-- Scroll Mode -->
        <div v-if="readerMode === 'scroll'" class="mx-auto max-w-4xl min-h-screen" @click="toggleControls">
            <div v-if="loading" class="flex items-center justify-center h-screen">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
            </div>

            <div v-else class="flex flex-col items-center pb-24">
                <template v-for="page in pages" :key="page">
                    <img v-if="pageUrls.has(page)" :src="pageUrls.get(page)" :data-page-index="page"
                        class="w-full h-auto object-contain max-h-screen mb-1" @load="handleImageLoad(page)"
                        alt="Comic page" />
                    <div v-else class="w-full aspect-2/3 flex items-center justify-center bg-gray-900 mb-1">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-600"></div>
                    </div>
                </template>

                <!-- End of Chapter / Navigation -->
                <div v-if="endOfChapter" class="py-12 flex flex-col items-center gap-4 w-full">
                    <p class="text-gray-400">End of Chapter</p>
                    <div class="flex gap-4">
                        <Button v-if="prevChapter" variant="secondary" @click.stop="navigateToChapter(prevChapter)">
                            <ChevronLeft class="mr-2 h-4 w-4" /> Previous Chapter
                        </Button>
                        <Button v-if="nextChapter" variant="default" @click.stop="navigateToChapter(nextChapter)">
                            Next Chapter
                            <ChevronRight class="ml-2 h-4 w-4" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Paged Mode -->
        <div v-else class="h-screen w-full flex items-center justify-center overflow-hidden" @click="handlePageClick">
            <div v-if="loading" class="flex items-center justify-center">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
            </div>

            <div v-else-if="endOfChapter" class="flex flex-col items-center gap-6 p-8">
                <p class="text-xl text-gray-400">End of Chapter</p>
                <div class="flex flex-col gap-4 min-w-[200px]">
                    <Button v-if="nextChapter" size="lg" variant="default" @click.stop="navigateToChapter(nextChapter)">
                        Next Chapter
                        <ChevronRight class="ml-2 h-4 w-4" />
                    </Button>
                    <Button v-if="prevChapter" variant="secondary" @click.stop="navigateToChapter(prevChapter)">
                        <ChevronLeft class="mr-2 h-4 w-4" /> Previous Chapter
                    </Button>
                    <Button variant="outline" @click.stop="prevPage">
                        Re-read Page
                    </Button>
                </div>
            </div>

            <div v-else class="relative h-full w-full flex items-center justify-center">
                <img v-if="pageUrls.has(currentPage)" :src="pageUrls.get(currentPage)"
                    class="max-w-full max-h-full object-contain" alt="Comic page" />
                <div v-else class="flex flex-col items-center gap-4">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
                    <p class="text-gray-400">Loading Page {{ currentPage + 1 }}...</p>
                    <Button v-if="failedPages.has(currentPage)" variant="destructive"
                        @click="loadPage(currentPage)">Retry</Button>
                </div>
            </div>
        </div>

        <!-- Top Bar -->
        <div class="fixed top-0 left-0 right-0 h-16 bg-black/80 backdrop-blur-sm border-b border-white/10 flex items-center px-4 transition-transform duration-300 z-50"
            :class="showControls ? 'translate-y-0' : '-translate-y-full'">
            <Button variant="ghost" size="icon" @click="router.push(`/library/${libraryId}/content/${contentId}`)">
                <ArrowLeft class="h-5 w-5 text-white" />
            </Button>
            <div class="ml-4 flex-1 overflow-hidden">
                <h1 class="text-sm font-medium truncate text-white">
                    {{ currentChapter?.title || 'Chapter ' + (currentChapterIndex + 1) }}
                </h1>
            </div>

            <!-- Settings Menu -->
            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                        <Settings class="h-5 w-5 text-white" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                    <DropdownMenuLabel>Reading Mode</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem @click="setMode('scroll')">
                        <AlignJustify class="mr-2 h-4 w-4" />
                        <span>Scroll (Webtoon)</span>
                        <span v-if="readerMode === 'scroll'" class="ml-auto text-xs">✓</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="setMode('paged')">
                        <Columns class="mr-2 h-4 w-4" />
                        <span>Paged</span>
                        <span v-if="readerMode === 'paged'" class="ml-auto text-xs">✓</span>
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>

        <!-- Bottom Bar (Navigation) -->
        <div class="fixed bottom-0 left-0 right-0 h-16 bg-black/80 backdrop-blur-sm border-t border-white/10 flex items-center justify-between px-4 transition-transform duration-300 z-50"
            :class="showControls ? 'translate-y-0' : 'translate-y-full'">
            <Progress v-if="readerMode === 'scroll'" :model-value="readingProgress"
                class="absolute top-0 left-0 right-0 h-1 rounded-none bg-white/20" />

            <Button variant="ghost" size="sm" :disabled="!prevChapter"
                @click="prevChapter && navigateToChapter(prevChapter)" class="text-white hover:text-white/80">
                <ChevronLeft class="mr-1 h-4 w-4" /> Prev
            </Button>

            <div class="flex flex-col items-center">
                <span class="text-xs text-gray-400">
                    {{ readerMode === 'paged' ? (currentPage + 1) + ' / ' + (currentChapter?.page_count || '?') :
                        `${currentChapterIndex + 1} / ${chapters.length}` }}
                </span>
            </div>

            <Button variant="ghost" size="sm" :disabled="!nextChapter"
                @click="nextChapter && navigateToChapter(nextChapter)" class="text-white hover:text-white/80">
                Next
                <ChevronRight class="ml-1 h-4 w-4" />
            </Button>
        </div>
    </div>
</template>