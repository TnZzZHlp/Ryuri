<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useReaderStore } from '@/stores/useReaderStore'
import { toast } from 'vue-sonner'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { Dialog } from '@/components/ui/dialog'
import ReaderControls from './ReaderControls.vue'
import ReaderSettingsDialog from './ReaderSettingsDialog.vue'
import ComicReader from './ComicReader.vue'
import EpubReader from './EpubReader.vue'
import type { Chapter } from '@/api/types'

const { t } = useI18n()
const router = useRouter()
const readerStore = useReaderStore()

const {
    chapters,
    loading,
    pageUrls,
    failedPages,
    endOfChapter,
    pages,
    readerMode,
    preloadBuffer,
    currentPage,
    currentChapter,
    currentChapterIndex,
    prevChapter,
    nextChapter,
    isNovel,
    epubHtmlContent,
    epubSpineLoading,
    epubCurrentSpineIndex,
    epubSpine,
} = storeToRefs(readerStore)

// Props
interface Props {
    libraryId: number
    contentId: number
    chapterId: number
}

const props = defineProps<Props>()

// State
const showControls = ref(true)
const showSettingsDialog = ref(false)
const readingProgress = ref(0)

// Refs
const comicReaderRef = ref<InstanceType<typeof ComicReader> | null>(null)
const epubReaderRef = ref<InstanceType<typeof EpubReader> | null>(null)

// Computed
const epubProgress = computed(() => {
    if (epubSpine.value.length === 0) return 0
    return ((epubCurrentSpineIndex.value + 1) / epubSpine.value.length) * 100
})

// Methods
const loadData = async () => {
    try {
        await readerStore.loadChapter(props.contentId, props.chapterId)
        if (!isNovel.value && readerMode.value === 'scroll') {
            nextTick(() => {
                comicReaderRef.value?.scrollToPage(currentPage.value)
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
            contentId: props.contentId,
            chapterId: chapter.id,
        },
    })
}

const navigateBack = () => {
    router.push(`/library/${props.libraryId}/content/${props.contentId}`)
}

const openSettings = () => {
    showSettingsDialog.value = true
}

const closeSettings = () => {
    showSettingsDialog.value = false
}

const toggleControls = () => {
    showControls.value = !showControls.value
}

const updateProgress = () => {
    if (isNovel.value) {
        readingProgress.value = epubReaderRef.value?.updateProgress() || epubProgress.value
    } else if (readerMode.value === 'scroll' && comicReaderRef.value?.containerRef) {
        const container = comicReaderRef.value.containerRef
        const scrollTop = container.scrollTop
        const docHeight = container.scrollHeight
        const winHeight = container.clientHeight
        const total = docHeight - winHeight
        readingProgress.value = total > 0 ? (scrollTop / total) * 100 : 0
    }
}

// Page click handling
const handlePageClick = (e: MouseEvent) => {
    const width = window.innerWidth
    const x = e.clientX

    if (x < width * 0.3) {
        comicReaderRef.value?.prevPage()
        showControls.value = false
    } else if (x > width * 0.7) {
        comicReaderRef.value?.nextPage()
        showControls.value = false
    } else {
        toggleControls()
    }
}

// EPUB click handling
const handleEpubClick = (e: MouseEvent) => {
    const width = window.innerWidth
    const x = e.clientX

    if (x < width * 0.3) {
        readerStore.epubPrevPage()
        epubReaderRef.value?.containerRef?.scrollTo(0, 0)
        showControls.value = false
    } else if (x > width * 0.7) {
        readerStore.epubNextPage()
        epubReaderRef.value?.containerRef?.scrollTo(0, 0)
        showControls.value = false
    } else {
        toggleControls()
    }
}

// Keyboard handling
const handleKeydown = (e: KeyboardEvent) => {
    if (isNovel.value) {
        if (e.key === 'ArrowLeft') {
            readerStore.epubPrevPage()
            epubReaderRef.value?.containerRef?.scrollTo(0, 0)
        } else if (e.key === 'ArrowRight') {
            readerStore.epubNextPage()
            epubReaderRef.value?.containerRef?.scrollTo(0, 0)
        } else if (e.key === 'Escape') {
            navigateBack()
        }
        showControls.value = false
        return
    }

    if (e.key === 'ArrowLeft') {
        if (readerMode.value === 'paged') {
            comicReaderRef.value?.prevPage()
        } else if (prevChapter.value) {
            navigateToChapter(prevChapter.value)
        }
    } else if (e.key === 'ArrowRight') {
        if (readerMode.value === 'paged') {
            comicReaderRef.value?.nextPage()
        } else if (nextChapter.value) {
            navigateToChapter(nextChapter.value)
        }
    } else if (e.key === 'Escape') {
        navigateBack()
    } else if (e.key === ' ' || e.key === 'Space') {
        e.preventDefault()
        if (readerMode.value === 'paged') {
            comicReaderRef.value?.nextPage()
        }
    }

    showControls.value = false
}

const preventSelection = (e: Event) => {
    e.preventDefault()
}

// Watchers
watch(
    () => props.chapterId,
    () => {
        readingProgress.value = 0
        loadData()
    },
)

watch(currentPage, (newPage) => {
    if (!isNovel.value && readerMode.value === 'paged' && currentChapter.value && currentChapter.value.page_count > 0) {
        readingProgress.value = ((newPage + 1) / currentChapter.value.page_count) * 100
    }
})

watch(epubCurrentSpineIndex, () => {
    readingProgress.value = epubProgress.value
})

onMounted(() => {
    loadData()
    window.addEventListener('keydown', handleKeydown)
    document.addEventListener('selectstart', preventSelection)
})

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
    document.removeEventListener('selectstart', preventSelection)
})
</script>

<template>
    <div ref="containerRef" class="fixed inset-0 bg-black text-white overflow-auto select-none" @scroll="updateProgress">
        <!-- EPUB Reader -->
        <EpubReader
            v-if="isNovel"
            ref="epubReaderRef"
            :chapter="currentChapter"
            :epub-html-content="epubHtmlContent"
            :epub-spine="epubSpine"
            :epub-current-spine-index="epubCurrentSpineIndex"
            :epub-spine-loading="epubSpineLoading"
            :loading="loading"
            :prev-chapter="prevChapter"
            :next-chapter="nextChapter"
            @click="handleEpubClick"
            @navigate-to-chapter="navigateToChapter"
        />

        <!-- Comic Reader -->
        <ComicReader
            v-else
            ref="comicReaderRef"
            :chapter="currentChapter"
            :page-urls="pageUrls"
            :failed-pages="failedPages"
            :current-page="currentPage"
            :reader-mode="readerMode"
            :pages="pages"
            :loading="loading"
            :preload-buffer="preloadBuffer"
            :end-of-chapter="endOfChapter"
            :prev-chapter="prevChapter"
            :next-chapter="nextChapter"
            @page-click="handlePageClick"
            @scroll="updateProgress"
            @set-current-page="(p) => (currentPage = p)"
            @load-page="readerStore.loadPage"
            @load-more-pages="readerStore.loadMorePages"
            @navigate-to-chapter="navigateToChapter"
            @save-progress="readerStore.saveProgress"
        />

        <!-- Controls -->
        <ReaderControls
            :show="showControls"
            :current-chapter="currentChapter"
            :current-chapter-index="currentChapterIndex"
            :chapters="chapters"
            :is-novel="isNovel"
            :reader-mode="readerMode"
            :reading-progress="readingProgress"
            :current-page="currentPage"
            :epub-current-spine-index="epubCurrentSpineIndex"
            :epub-spine-length="epubSpine.length"
            :prev-chapter="prevChapter"
            :next-chapter="nextChapter"
            @navigate-back="navigateBack"
            @navigate-to-chapter="navigateToChapter"
            @set-mode="readerStore.setMode"
            @open-settings="openSettings"
        />

        <!-- Settings Dialog -->
        <Dialog :open="showSettingsDialog" @update:open="showSettingsDialog = $event">
            <ReaderSettingsDialog
                :reader-mode="readerMode"
                :preload-buffer="preloadBuffer"
                :min-preload-buffer="readerStore.MIN_PRELOAD_BUFFER"
                :max-preload-buffer="readerStore.MAX_PRELOAD_BUFFER"
                @close="closeSettings"
                @update:reader-mode="readerStore.setMode"
                @update:preload-buffer="readerStore.setPreloadBuffer"
            />
        </Dialog>
    </div>
</template>
