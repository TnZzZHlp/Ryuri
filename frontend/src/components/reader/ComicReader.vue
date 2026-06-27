<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { Button } from '@/components/ui/button'
import { Spinner } from '@/components/ui/spinner'
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import type { Chapter } from '@/api/types'

// Swipe configuration
const SWIPE_THRESHOLD = 50 // Minimum distance for swipe
const SWIPE_VELOCITY_THRESHOLD = 0.3 // Minimum velocity for quick swipe

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
    navigateBack: []
    saveProgress: [page: number]
}>()

// Refs
const containerRef = ref<HTMLElement | null>(null)
const pageRefs = new Map<number, HTMLElement>()
const visibilityMap = new Map<number, number>()
let observer: IntersectionObserver | null = null

// Swipe state for paged mode
const touchStartX = ref(0)
const touchStartY = ref(0)
const touchEndX = ref(0)
const touchEndY = ref(0)
const isSwiping = ref(false)
const swipeOffset = ref(0)
const isAnimating = ref(false)
const swipeDirection = ref<'left' | 'right' | null>(null)

// Computed
const showPagedEndOfChapter = computed(() => {
    if (!props.chapter) return false
    return props.currentPage >= props.chapter.page_count
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

const preloadFromPage = (page: number) => {
    emit('loadPage', page)
    for (let i = 1; i <= props.preloadBuffer; i++) {
        emit('loadPage', page + i)
    }
}

const nextPage = () => {
    if (!props.chapter || props.chapter.page_count <= 0) return

    if (props.currentPage >= props.chapter.page_count) {
        if (props.nextChapter) emit('navigateToChapter', props.nextChapter)
        return
    }

    const lastPageIndex = props.chapter.page_count - 1
    if (props.currentPage >= lastPageIndex) {
        emit('setCurrentPage', props.chapter.page_count)
        return
    }

    const next = props.currentPage + 1

    if (props.failedPages.has(next)) {
        return
    }

    emit('setCurrentPage', next)
    preloadFromPage(next)
    containerRef.value?.scrollTo(0, 0)
    emit('saveProgress', next)
}

const prevPage = () => {
    if (props.currentPage > 0) {
        const prev = props.currentPage - 1
        emit('setCurrentPage', prev)
        preloadFromPage(prev)
        containerRef.value?.scrollTo(0, 0)
        emit('saveProgress', prev)
    } else if (props.prevChapter) {
        emit('navigateToChapter', props.prevChapter)
    }
}

// Touch handlers for swipe navigation in paged mode
const handleTouchStart = (e: TouchEvent) => {
    if (props.readerMode !== 'paged') return
    touchStartX.value = e.touches[0].clientX
    touchStartY.value = e.touches[0].clientY
    touchEndX.value = touchStartX.value
    touchEndY.value = touchStartY.value
    isSwiping.value = false
    swipeOffset.value = 0
}

const handleTouchMove = (e: TouchEvent) => {
    if (props.readerMode !== 'paged' || isAnimating.value) return

    touchEndX.value = e.touches[0].clientX
    touchEndY.value = e.touches[0].clientY

    const deltaX = touchEndX.value - touchStartX.value
    const deltaY = touchEndY.value - touchStartY.value

    // Only consider horizontal swipes (ignore if vertical movement is dominant)
    if (Math.abs(deltaX) > Math.abs(deltaY) && Math.abs(deltaX) > 10) {
        e.preventDefault() // Prevent page scroll during swipe
        isSwiping.value = true
        swipeOffset.value = deltaX
        // Determine swipe direction for showing adjacent page
        swipeDirection.value = deltaX < 0 ? 'left' : 'right'
    }
}

const handleTouchEnd = () => {
    if (props.readerMode !== 'paged' || !isSwiping.value || isAnimating.value) {
        swipeOffset.value = 0
        isSwiping.value = false
        swipeDirection.value = null
        return
    }

    const deltaX = touchEndX.value - touchStartX.value
    const velocity = Math.abs(deltaX) / SWIPE_THRESHOLD
    const direction = swipeDirection.value

    // Check if swipe meets threshold or was quick enough
    if (Math.abs(deltaX) > SWIPE_THRESHOLD || velocity > SWIPE_VELOCITY_THRESHOLD) {
        // Swipe left (next page) or swipe right (prev page)
        if (deltaX < 0 && direction === 'left') {
            // Swipe left - go to next page
            animatePageTransition('left')
        } else if (deltaX > 0 && direction === 'right') {
            // Swipe right - go to prev page
            animatePageTransition('right')
        } else {
            // Cancelled, reset
            resetSwipe()
        }
    } else {
        // Not enough swipe, animate back
        resetSwipe()
    }
}

const resetSwipe = () => {
    isAnimating.value = true
    swipeOffset.value = 0
    setTimeout(() => {
        isAnimating.value = false
        isSwiping.value = false
        swipeDirection.value = null
    }, 200)
}

const animatePageTransition = (direction: 'left' | 'right') => {
    isAnimating.value = true
    const containerWidth = containerRef.value?.offsetWidth ?? 500

    // Complete the swipe animation
    swipeOffset.value = direction === 'left' ? -containerWidth : containerWidth

    setTimeout(() => {
        if (direction === 'left') {
            nextPage()
        } else {
            prevPage()
        }
        // Reset immediately after page change (no animation on reset)
        swipeOffset.value = 0
        isSwiping.value = false
        isAnimating.value = false
        swipeDirection.value = null
    }, 150) // Match CSS transition duration
}

// Computed for adjacent pages in paged mode
const prevPageUrl = computed(() => {
    if (!props.chapter || props.currentPage <= 0) return null
    const prev = props.currentPage - 1
    return props.pageUrls.get(prev) ?? null
})

const currentPageUrl = computed(() => {
    return props.pageUrls.get(props.currentPage) ?? null
})

const nextPageUrl = computed(() => {
    if (!props.chapter || props.currentPage >= props.chapter.page_count - 1) return null
    const next = props.currentPage + 1
    return props.pageUrls.get(next) ?? null
})

// Watchers
watch(
    () => props.readerMode,
    (newMode) => {
        if (newMode === 'scroll') {
            nextTick(initObserver)
        } else {
            observer?.disconnect()
            observer = null
            // Preload adjacent pages for paged mode
            preloadAdjacentPages()
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
        } else {
            // Preload adjacent pages for paged mode
            preloadAdjacentPages()
        }
    },
)

// Watch currentPage changes in paged mode to preload adjacent pages
watch(
    () => props.currentPage,
    () => {
        if (props.readerMode === 'paged' && props.chapter) {
            preloadAdjacentPages()
        }
    },
)

// Preload adjacent pages for paged mode
const preloadAdjacentPages = () => {
    if (!props.chapter) return

    // Preload previous page
    if (props.currentPage > 0) {
        emit('loadPage', props.currentPage - 1)
    }

    // Preload current page
    emit('loadPage', props.currentPage)

    // Preload next page
    if (props.currentPage < props.chapter.page_count - 1) {
        emit('loadPage', props.currentPage + 1)
    }
}

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
    <div v-if="readerMode === 'scroll'" ref="containerRef" class="mx-auto max-w-4xl min-h-svh"
        @click="$emit('pageClick', $event)" @scroll="$emit('scroll')">
        <div v-if="loading" class="flex items-center justify-center h-screen">
            <Spinner class="size-12 text-white" />
        </div>

        <div v-else class="flex flex-col items-center pb-24">
            <template v-for="page in pages" :key="page">
                <img v-if="pageUrls.has(page)" :src="pageUrls.get(page)" :data-page-index="page"
                    :ref="(el) => setPageRef(el, page)" class="w-full h-auto object-contain max-h-screen mb-1"
                    @load="$emit('scroll')" alt="Comic page" />
                <div v-else :data-page-index="page" :ref="(el) => setPageRef(el, page)"
                    class="w-full aspect-2/3 flex items-center justify-center bg-gray-900 mb-1">
                    <Spinner class="size-8 text-gray-600" />
                </div>
            </template>

            <!-- End of Chapter / Navigation -->
            <div v-if="endOfChapter" class="py-12 flex flex-col items-center gap-4 w-full">
                <p class="text-gray-400">{{ t('reader.end_of_chapter') }}</p>
                <div class="flex gap-4">
                    <Button v-if="prevChapter" variant="secondary"
                        @click.stop="$emit('navigateToChapter', prevChapter)">
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
    <div v-else ref="containerRef" class="h-full w-full flex items-center justify-center overflow-hidden touch-pan-y"
        @click="$emit('pageClick', $event)" @touchstart="handleTouchStart" @touchmove="handleTouchMove"
        @touchend="handleTouchEnd">
        <div v-if="loading" class="flex items-center justify-center">
            <Spinner class="size-12 text-white" />
        </div>

        <div v-else-if="showPagedEndOfChapter" class="flex flex-col items-center gap-6 p-8">
            <p class="text-xl text-gray-400">{{ t('reader.end_of_chapter') }}</p>
            <div class="flex flex-col gap-4 min-w-50">
                <Button v-if="nextChapter" size="lg" variant="default"
                    @click.stop="$emit('navigateToChapter', nextChapter)">
                    {{ t('reader.next_chapter') }}
                    <ChevronRight class="ml-2 h-4 w-4" />
                </Button>
                <Button variant="secondary" @click.stop="$emit('navigateBack')">
                    {{ t('reader.exit_to_content') }}
                </Button>
            </div>
        </div>

        <!-- Page container with slide effect -->
        <div v-else class="relative h-full w-full overflow-hidden">
            <!-- Previous page (shown when swiping right) -->
            <div v-if="prevPageUrl && (swipeDirection === 'right' || swipeOffset > 0)"
                class="absolute inset-0 flex items-center justify-center" :style="{
                    transform: `translateX(${swipeOffset - (containerRef?.offsetWidth ?? 0)}px)`,
                    transition: isAnimating ? 'transform 0.15s ease-out' : 'none',
                    zIndex: 10
                }">
                <img :src="prevPageUrl" class="max-w-full max-h-full object-contain select-none" alt="Previous page"
                    draggable="false" />
            </div>

            <!-- Current page -->
            <div class="absolute inset-0 flex items-center justify-center" :style="{
                transform: `translateX(${swipeOffset}px)`,
                transition: isAnimating ? 'transform 0.15s ease-out' : 'none',
                zIndex: 20
            }">
                <img v-if="currentPageUrl" :src="currentPageUrl"
                    class="max-w-full max-h-full object-contain select-none" alt="Comic page" draggable="false" />
            </div>

            <!-- Next page (shown when swiping left) -->
            <div v-if="nextPageUrl && (swipeDirection === 'left' || swipeOffset < 0)"
                class="absolute inset-0 flex items-center justify-center" :style="{
                    transform: `translateX(${swipeOffset + (containerRef?.offsetWidth ?? 0)}px)`,
                    transition: isAnimating ? 'transform 0.15s ease-out' : 'none',
                    zIndex: 10
                }">
                <img :src="nextPageUrl" class="max-w-full max-h-full object-contain select-none" alt="Next page"
                    draggable="false" />
            </div>
        </div>
    </div>
</template>
