<script setup lang="ts">
import { ref, onUnmounted, computed, watch, onMounted, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useReaderStore } from '@/stores/useReaderStore'
import { Button } from '@/components/ui/button'
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
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'

const route = useRoute()
const router = useRouter()
const readerStore = useReaderStore()
const { t } = useI18n()
const {
    chapters,
    loading,
    pageUrls,
    failedPages,
    endOfChapter,
    pages,
    readerMode,
    currentPage,
    currentChapter,
    currentChapterIndex,
    prevChapter,
    nextChapter,
    isNovel,
    chapterText,
    textLoading
} = storeToRefs(readerStore)

// State
const libraryId = computed(() => Number(route.params.libraryId))
const contentId = computed(() => Number(route.params.contentId))
const chapterId = computed(() => Number(route.params.chapterId))

const showControls = ref(true)
const readingProgress = ref(0)
const containerRef = ref<HTMLElement | null>(null)

// Intersection Observer State
const pageRefs = new Map<number, HTMLElement>()
const visibilityMap = new Map<number, number>()
let observer: IntersectionObserver | null = null

const renderedPages = computed(() => {
    const pagesList: number[] = []
    if (!currentChapter.value) return pagesList

    // Previous page
    if (currentPage.value > 0) {
        pagesList.push(currentPage.value - 1)
    }

    // Current page
    pagesList.push(currentPage.value)

    // Next pages (buffer)
    for (let i = 1; i <= readerStore.PRELOAD_BUFFER; i++) {
        const p = currentPage.value + i
        if (p < currentChapter.value.page_count) {
            pagesList.push(p)
        }
    }

    return pagesList
})

// Novel text paragraphs
const textParagraphs = computed(() => {
    if (!chapterText.value) return []
    return chapterText.value.split('\n').filter(p => p.trim().length > 0)
})

// Methods
const scrollToPage = (pageIndex: number) => {
    if (!containerRef.value) return

    // We need to wait for render
    nextTick(() => {
        const el = pageRefs.get(pageIndex)
        if (el) {
            el.scrollIntoView({ block: 'start' })
        }
    })
}

const initObserver = () => {
    if (observer) observer.disconnect()

    if (readerMode.value !== 'scroll') return

    observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            const el = entry.target as HTMLElement
            const index = Number(el.dataset.pageIndex)

            if (entry.isIntersecting) {
                visibilityMap.set(index, entry.intersectionRatio)
                // Lazy load image if visible
                readerStore.loadPage(index)
            } else {
                visibilityMap.delete(index)
            }

            if (entry.isIntersecting) {
                const maxPage = Math.max(...pages.value)
                if (index >= maxPage - readerStore.PRELOAD_BUFFER && !endOfChapter.value) {
                    loadMorePages()
                }
            }
        })

        let bestPage = -1
        let maxRatio = 0
        for (const [page, ratio] of visibilityMap.entries()) {
            if (ratio > maxRatio) {
                maxRatio = ratio
                bestPage = page
            }
        }

        if (bestPage !== -1 && bestPage !== currentPage.value) {
            currentPage.value = bestPage
            readerStore.saveProgress(bestPage)
        }
    }, {
        threshold: [0, 0.1, 0.5, 0.8, 1.0],
        rootMargin: '200px'
    })

    pageRefs.forEach(el => observer?.observe(el))
}

const setPageRef = (el: any, page: number) => {
    if (el) {
        const htmlEl = el as HTMLElement
        if (pageRefs.get(page) !== htmlEl) {
            pageRefs.set(page, htmlEl)
            observer?.observe(htmlEl)
        }
    } else {
        const oldEl = pageRefs.get(page)
        if (oldEl) {
            observer?.unobserve(oldEl)
            pageRefs.delete(page)
            visibilityMap.delete(page)
        }
    }
}

const loadMorePages = () => {
    const maxPage = Math.max(...pages.value)
    const nextPage = maxPage + 1

    if (currentChapter.value && nextPage < currentChapter.value.page_count && !pages.value.includes(nextPage) && !failedPages.value.has(nextPage)) {
        pages.value.push(nextPage)
        readerStore.loadPage(nextPage)
    }
}

const updateProgress = () => {
    if (!containerRef.value) return

    if (isNovel.value) {
        // Novel: track scroll percentage
        const scrollTop = containerRef.value.scrollTop
        const docHeight = containerRef.value.scrollHeight
        const winHeight = containerRef.value.clientHeight
        const total = docHeight - winHeight
        const percentage = total > 0 ? (scrollTop / total) * 100 : 0
        readingProgress.value = percentage
        readerStore.saveNovelProgress(percentage)
    } else if (readerMode.value === 'scroll') {
        const scrollTop = containerRef.value.scrollTop
        const docHeight = containerRef.value.scrollHeight
        const winHeight = containerRef.value.clientHeight
        const total = docHeight - winHeight
        readingProgress.value = total > 0 ? (scrollTop / total) * 100 : 0
    }
}

const loadData = async () => {
    try {
        await readerStore.loadChapter(contentId.value, chapterId.value)
        if (!isNovel.value && readerMode.value === 'scroll') {
            nextTick(() => {
                initObserver()
                // If we have progress, scroll to it
                if (currentPage.value > 0) {
                    scrollToPage(currentPage.value)
                }
            })
        }
    } catch {
        toast.error(t('reader.loading_fail'))
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
const nextPageFn = () => {
    if (endOfChapter.value) {
        if (nextChapter.value) navigateToChapter(nextChapter.value)
        return
    }

    const next = currentPage.value + 1

    if (currentChapter.value && next >= currentChapter.value.page_count) {
        endOfChapter.value = true
        return
    }

    if (failedPages.value.has(next)) {
        endOfChapter.value = true
        return
    }

    currentPage.value = next
    readerStore.loadPage(next)
    for (let i = 1; i <= readerStore.PRELOAD_BUFFER; i++) {
        readerStore.loadPage(next + i)
    }
    containerRef.value?.scrollTo(0, 0)
    readerStore.saveProgress(currentPage.value)
}

const prevPageFn = () => {
    if (currentPage.value > 0 && !endOfChapter.value) {
        currentPage.value--
        readerStore.loadPage(currentPage.value)
        containerRef.value?.scrollTo(0, 0)
        readerStore.saveProgress(currentPage.value)
    } else if (endOfChapter.value) {
        endOfChapter.value = false
    } else {
        if (prevChapter.value) navigateToChapter(prevChapter.value)
    }
    endOfChapter.value = false
}

const handlePageClick = (e: MouseEvent) => {
    const width = window.innerWidth
    const x = e.clientX

    if (x < width * 0.3) {
        prevPageFn()
        showControls.value = false
    } else if (x > width * 0.7) {
        nextPageFn()
        showControls.value = false
    } else {
        toggleControls()
    }
}

// Watchers
watch(() => route.params.chapterId, () => {
    readingProgress.value = 0
    containerRef.value?.scrollTo(0, 0)
    loadData()
})

watch(currentPage, (newPage) => {
    if (!isNovel.value && readerMode.value === 'paged' && currentChapter.value && currentChapter.value.page_count > 0) {
        readingProgress.value = ((newPage + 1) / currentChapter.value.page_count) * 100
    }
})

watch(readerMode, (newMode) => {
    if (newMode === 'scroll') {
        nextTick(initObserver)
    } else {
        observer?.disconnect()
        observer = null
    }
})

const preventSelection = (e: Event) => {
    // Allow text selection in novel mode
    if (isNovel.value) return
    e.preventDefault()
}

onMounted(() => {
    loadData()
    window.addEventListener('keydown', handleKeydown)
    document.addEventListener('selectstart', preventSelection)
})

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
    document.removeEventListener('selectstart', preventSelection)
    observer?.disconnect()
})

const handleKeydown = (e: KeyboardEvent) => {
    if (isNovel.value) {
        // Novel mode: left/right for chapter navigation, Escape to exit
        if (e.key === 'ArrowLeft') {
            if (prevChapter.value) navigateToChapter(prevChapter.value)
        } else if (e.key === 'ArrowRight') {
            if (nextChapter.value) navigateToChapter(nextChapter.value)
        } else if (e.key === 'Escape') {
            router.push(`/content/${contentId.value}`)
        }
        showControls.value = false
        return
    }

    if (e.key === 'ArrowLeft') {
        if (readerMode.value === 'paged') {
            prevPageFn()
        } else {
            if (prevChapter.value) navigateToChapter(prevChapter.value)
        }

    } else if (e.key === 'ArrowRight') {
        if (readerMode.value === 'paged') {
            nextPageFn()
        } else {
            if (nextChapter.value) navigateToChapter(nextChapter.value)
        }
    } else if (e.key === 'Escape') {
        router.push(`/content/${contentId.value}`)
    } else if (e.key === ' ' || e.key === 'Space') {
        e.preventDefault()
        if (readerMode.value === 'paged') {
            nextPageFn()
        }
    }

    showControls.value = false
}
</script>

<template>
    <div ref="containerRef" class="fixed inset-0 bg-black text-white overflow-auto" @scroll="updateProgress">

        <!-- Reader Content -->

        <!-- Novel Text Reader Mode -->
        <div v-if="isNovel" class="novel-reader mx-auto max-w-3xl min-h-screen px-6 md:px-12 py-20"
            @click="toggleControls">
            <div v-if="loading || textLoading" class="flex items-center justify-center h-screen">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
            </div>

            <div v-else-if="chapterText" class="novel-text-content">
                <p v-for="(paragraph, index) in textParagraphs" :key="index" class="novel-paragraph">
                    {{ paragraph }}
                </p>

                <!-- End of Chapter Navigation -->
                <div class="py-16 flex flex-col items-center gap-4 w-full border-t border-white/10 mt-12">
                    <p class="text-gray-400">{{ t('reader.end_of_chapter') }}</p>
                    <div class="flex gap-4">
                        <Button v-if="prevChapter" variant="secondary" @click.stop="navigateToChapter(prevChapter)">
                            <ChevronLeft class="mr-2 h-4 w-4" /> {{ t('reader.prev_chapter') }}
                        </Button>
                        <Button v-if="nextChapter" variant="default" @click.stop="navigateToChapter(nextChapter)">
                            {{ t('reader.next_chapter') }}
                            <ChevronRight class="ml-2 h-4 w-4" />
                        </Button>
                    </div>
                </div>
            </div>

            <div v-else class="flex items-center justify-center h-screen text-gray-400">
                <p>{{ t('reader.text_load_fail') }}</p>
            </div>
        </div>

        <!-- Comic: Scroll Mode -->
        <div v-else-if="readerMode === 'scroll'" class="mx-auto max-w-4xl min-h-screen" @click="toggleControls">
            <div v-if="loading" class="flex items-center justify-center h-screen">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
            </div>

            <div v-else class="flex flex-col items-center pb-24">
                <template v-for="page in pages" :key="page">
                    <img v-if="pageUrls.has(page)" :src="pageUrls.get(page)" :data-page-index="page"
                        :ref="(el) => setPageRef(el, page)" class="w-full h-auto object-contain max-h-screen mb-1"
                        @load="updateProgress" alt="Comic page" />
                    <div v-else :data-page-index="page" :ref="(el) => setPageRef(el, page)"
                        class="w-full aspect-2/3 flex items-center justify-center bg-gray-900 mb-1">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-600"></div>
                    </div>
                </template>

                <!-- End of Chapter / Navigation -->
                <div v-if="endOfChapter" class="py-12 flex flex-col items-center gap-4 w-full">
                    <p class="text-gray-400">{{ t('reader.end_of_chapter') }}</p>
                    <div class="flex gap-4">
                        <Button v-if="prevChapter" variant="secondary" @click.stop="navigateToChapter(prevChapter)">
                            <ChevronLeft class="mr-2 h-4 w-4" /> {{ t('reader.prev_chapter') }}
                        </Button>
                        <Button v-if="nextChapter" variant="default" @click.stop="navigateToChapter(nextChapter)">
                            {{ t('reader.next_chapter') }}
                            <ChevronRight class="ml-2 h-4 w-4" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Comic: Paged Mode -->
        <div v-else class="h-screen w-full flex items-center justify-center overflow-hidden" @click="handlePageClick">
            <div v-if="loading" class="flex items-center justify-center">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
            </div>

            <div v-else-if="endOfChapter" class="flex flex-col items-center gap-6 p-8">
                <p class="text-xl text-gray-400">{{ t('reader.end_of_chapter') }}</p>
                <div class="flex flex-col gap-4 min-w-50">
                    <Button v-if="nextChapter" size="lg" variant="default" @click.stop="navigateToChapter(nextChapter)">
                        {{ t('reader.next_chapter') }}
                        <ChevronRight class="ml-2 h-4 w-4" />
                    </Button>
                    <Button v-if="prevChapter" variant="secondary" @click.stop="navigateToChapter(prevChapter)">
                        <ChevronLeft class="mr-2 h-4 w-4" /> {{ t('reader.prev_chapter') }}
                    </Button>
                    <Button variant="outline" @click.stop="router.push(`/library/${libraryId}/content/${contentId}`)">
                        {{ t('reader.exit_to_content') }}
                    </Button>
                </div>
            </div>

            <div v-else class="relative h-full w-full flex items-center justify-center">
                <template v-for="page in renderedPages" :key="page">
                    <img v-if="pageUrls.has(page)" :src="pageUrls.get(page)" v-show="page === currentPage"
                        class="max-w-full max-h-full object-contain" alt="Comic page" />
                </template>
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
                    {{ currentChapter?.title || t('reader.chapter_title_fallback', { index: currentChapterIndex + 1 })
                    }}
                </h1>
            </div>

            <!-- Settings Menu (hidden for novels) -->
            <DropdownMenu v-if="!isNovel">
                <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                        <Settings class="h-5 w-5 text-white" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                    <DropdownMenuLabel>{{ t('reader.reading_mode') }}</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem @click="readerStore.setMode('scroll')">
                        <AlignJustify class="mr-2 h-4 w-4" />
                        <span>{{ t('reader.mode_scroll') }}</span>
                        <span v-if="readerMode === 'scroll'" class="ml-auto text-xs">✓</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="readerStore.setMode('paged')">
                        <Columns class="mr-2 h-4 w-4" />
                        <span>{{ t('reader.mode_paged') }}</span>
                        <span v-if="readerMode === 'paged'" class="ml-auto text-xs">✓</span>
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>

        <!-- Bottom Bar (Navigation) -->
        <div class="fixed bottom-0 left-0 right-0 h-16 bg-black/80 backdrop-blur-sm border-t border-white/10 flex items-center justify-between px-4 transition-transform duration-300 z-50"
            :class="showControls ? 'translate-y-0' : 'translate-y-full'">
            <Progress :model-value="readingProgress"
                class="absolute top-0 left-0 right-0 h-1 rounded-none bg-white/20" />

            <Button variant="ghost" size="sm" :disabled="!prevChapter"
                @click="prevChapter && navigateToChapter(prevChapter)" class="text-white hover:text-white/80">
                <ChevronLeft class="mr-1 h-4 w-4" /> {{ t('reader.prev') }}
            </Button>

            <div class="flex flex-col items-center">
                <span class="text-xs text-gray-400">
                    <template v-if="isNovel">
                        {{ `${currentChapterIndex + 1} / ${chapters.length}` }}
                    </template>
                    <template v-else>
                        {{ readerMode === 'paged' ? (currentPage + 1) + ' / ' + (currentChapter?.page_count || '?') :
                            `${currentChapterIndex + 1} / ${chapters.length}` }}
                    </template>
                </span>
            </div>

            <Button variant="ghost" size="sm" :disabled="!nextChapter"
                @click="nextChapter && navigateToChapter(nextChapter)" class="text-white hover:text-white/80">

                {{ t('reader.next') }}

                <ChevronRight class="ml-1 h-4 w-4" />

            </Button>

        </div>

    </div>

</template>

<style scoped>
.novel-reader {
    font-family: 'Georgia', 'Noto Serif SC', 'Source Han Serif CN', serif;
}

.novel-text-content {
    color: #e0ddd5;
    line-height: 2;
    font-size: 1.125rem;
}

.novel-paragraph {
    text-indent: 2em;
    margin-bottom: 0.75em;
}

@media (max-width: 768px) {
    .novel-text-content {
        font-size: 1rem;
        line-height: 1.9;
    }
}
</style>