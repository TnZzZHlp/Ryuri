<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { Button } from '@/components/ui/button'
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import type { Chapter } from '@/api/types'

const { t } = useI18n()

interface Props {
    chapter: Chapter | undefined
    pageUrls: Map<number, string>
    failedPages: Set<number>
    currentPage: number
    readerMode: 'scroll' | 'paged'
    pages: number[]
    loading: boolean
    preloadBuffer: number
    endOfChapter: boolean
    prevChapter: Chapter | null | undefined
    nextChapter: Chapter | null | undefined
}

const props = defineProps<Props>()

const emit = defineEmits<{
    pageClick: [e: MouseEvent]
    scroll: []
    setCurrentPage: [page: number]
    loadPage: [page: number]
    loadMorePages: []
    navigateToChapter: [chapter: Chapter]
    saveProgress: [page: number]
}>()

// Refs
const containerRef = ref<HTMLElement | null>(null)
const pageRefs = new Map<number, HTMLElement>()
const visibilityMap = new Map<number, number>()
let observer: IntersectionObserver | null = null

// Computed
const renderedPages = computed(() => {
    const pagesList: number[] = []
    if (!props.chapter) return pagesList

    if (props.currentPage > 0) {
        pagesList.push(props.currentPage - 1)
    }

    pagesList.push(props.currentPage)

    for (let i = 1; i <= props.preloadBuffer; i++) {
        const p = props.currentPage + i
        if (p < props.chapter.page_count) {
            pagesList.push(p)
        }
    }

    return pagesList
})

// Methods
const scrollToPage = (pageIndex: number) => {
    if (!containerRef.value) return

    nextTick(() => {
        const el = pageRefs.get(pageIndex)
        if (el) {
            el.scrollIntoView({ block: 'start' })
        }
    })
}

const initObserver = () => {
    if (observer) observer.disconnect()

    if (props.readerMode !== 'scroll') return

    observer = new IntersectionObserver(
        (entries) => {
            entries.forEach((entry) => {
                const el = entry.target as HTMLElement
                const index = Number(el.dataset.pageIndex)

                if (entry.isIntersecting) {
                    visibilityMap.set(index, entry.intersectionRatio)
                    emit('loadPage', index)
                } else {
                    visibilityMap.delete(index)
                }

                if (entry.isIntersecting) {
                    const maxPage = Math.max(...props.pages)
                    if (index >= maxPage - props.preloadBuffer && !props.endOfChapter) {
                        emit('loadMorePages')
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

            if (bestPage !== -1 && bestPage !== props.currentPage) {
                emit('setCurrentPage', bestPage)
                emit('saveProgress', bestPage)
            }
        },
        {
            threshold: [0, 0.1, 0.5, 0.8, 1.0],
            rootMargin: '200px',
        },
    )

    pageRefs.forEach((el) => observer?.observe(el))
}

const setPageRef = (el: unknown, page: number) => {
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

const nextPage = () => {
    if (props.endOfChapter) {
        if (props.nextChapter) emit('navigateToChapter', props.nextChapter)
        return
    }

    const next = props.currentPage + 1

    if (props.chapter && next >= props.chapter.page_count) {
        emit('setCurrentPage', props.currentPage)
        return
    }

    if (props.failedPages.has(next)) {
        return
    }

    emit('setCurrentPage', next)
    emit('loadPage', next)
    for (let i = 1; i <= props.preloadBuffer; i++) {
        emit('loadPage', next + i)
    }
    containerRef.value?.scrollTo(0, 0)
    emit('saveProgress', next)
}

const prevPage = () => {
    if (props.currentPage > 0) {
        const prev = props.currentPage - 1
        emit('setCurrentPage', prev)
        emit('loadPage', prev)
        containerRef.value?.scrollTo(0, 0)
        emit('saveProgress', prev)
    } else if (props.prevChapter) {
        emit('navigateToChapter', props.prevChapter)
    }
}

// Watchers
watch(
    () => props.readerMode,
    (newMode) => {
        if (newMode === 'scroll') {
            nextTick(initObserver)
        } else {
            observer?.disconnect()
            observer = null
        }
    },
)

watch(
    () => props.chapter,
    () => {
        if (props.readerMode === 'scroll') {
            nextTick(() => {
                initObserver()
                if (props.currentPage > 0) {
                    scrollToPage(props.currentPage)
                }
            })
        }
    },
)

onUnmounted(() => {
    observer?.disconnect()
})

defineExpose({
    containerRef,
    scrollToPage,
    nextPage,
    prevPage,
})
</script>

<template>
    <!-- Scroll Mode -->
    <div
        v-if="readerMode === 'scroll'"
        ref="containerRef"
        class="mx-auto max-w-4xl min-h-screen"
        @click="$emit('pageClick', $event)"
        @scroll="$emit('scroll')"
    >
        <div v-if="loading" class="flex items-center justify-center h-screen">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
        </div>

        <div v-else class="flex flex-col items-center pb-24">
            <template v-for="page in pages" :key="page">
                <img
                    v-if="pageUrls.has(page)"
                    :src="pageUrls.get(page)"
                    :data-page-index="page"
                    :ref="(el) => setPageRef(el, page)"
                    class="w-full h-auto object-contain max-h-screen mb-1"
                    @load="$emit('scroll')"
                    alt="Comic page"
                />
                <div
                    v-else
                    :data-page-index="page"
                    :ref="(el) => setPageRef(el, page)"
                    class="w-full aspect-2/3 flex items-center justify-center bg-gray-900 mb-1"
                >
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-600"></div>
                </div>
            </template>

            <!-- End of Chapter / Navigation -->
            <div v-if="endOfChapter" class="py-12 flex flex-col items-center gap-4 w-full">
                <p class="text-gray-400">{{ t('reader.end_of_chapter') }}</p>
                <div class="flex gap-4">
                    <Button v-if="prevChapter" variant="secondary" @click.stop="$emit('navigateToChapter', prevChapter)">
                        <ChevronLeft class="mr-2 h-4 w-4" /> {{ t('reader.prev_chapter') }}
                    </Button>
                    <Button v-if="nextChapter" variant="default" @click.stop="$emit('navigateToChapter', nextChapter)">
                        {{ t('reader.next_chapter') }}
                        <ChevronRight class="ml-2 h-4 w-4" />
                    </Button>
                </div>
            </div>
        </div>
    </div>

    <!-- Paged Mode -->
    <div
        v-else
        ref="containerRef"
        class="h-screen w-full flex items-center justify-center overflow-hidden"
        @click="$emit('pageClick', $event)"
    >
        <div v-if="loading" class="flex items-center justify-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
        </div>

        <div v-else-if="endOfChapter" class="flex flex-col items-center gap-6 p-8">
            <p class="text-xl text-gray-400">{{ t('reader.end_of_chapter') }}</p>
            <div class="flex flex-col gap-4 min-w-50">
                <Button
                    v-if="nextChapter"
                    size="lg"
                    variant="default"
                    @click.stop="$emit('navigateToChapter', nextChapter)"
                >
                    {{ t('reader.next_chapter') }}
                    <ChevronRight class="ml-2 h-4 w-4" />
                </Button>
                <Button
                    v-if="prevChapter"
                    variant="secondary"
                    @click.stop="$emit('navigateToChapter', prevChapter)"
                >
                    <ChevronLeft class="mr-2 h-4 w-4" /> {{ t('reader.prev_chapter') }}
                </Button>
            </div>
        </div>

        <div v-else class="relative h-full w-full flex items-center justify-center">
            <template v-for="page in renderedPages" :key="page">
                <img
                    v-if="pageUrls.has(page)"
                    :src="pageUrls.get(page)"
                    v-show="page === currentPage"
                    class="max-w-full max-h-full object-contain"
                    alt="Comic page"
                />
            </template>
        </div>
    </div>
</template>
