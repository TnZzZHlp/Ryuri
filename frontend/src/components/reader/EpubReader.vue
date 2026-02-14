<script setup lang="ts">
import { ref, computed } from 'vue'
import { Button } from '@/components/ui/button'
import { ChevronRight } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import type { Chapter, EpubSpineItem } from '@/api/types'

const { t } = useI18n()

interface Props {
    chapter: Chapter | undefined
    epubHtmlContent: string
    epubSpine: EpubSpineItem[]
    epubCurrentSpineIndex: number
    epubSpineLoading: boolean
    loading: boolean
    prevChapter: Chapter | null | undefined
    nextChapter: Chapter | null | undefined
}

const props = defineProps<Props>()

const emit = defineEmits<{
    click: [e: MouseEvent]
    navigateToChapter: [chapter: Chapter]
}>()

const containerRef = ref<HTMLElement | null>(null)

const epubHasNext = computed(() => props.epubCurrentSpineIndex < props.epubSpine.length - 1)

// Progress calculation for epub
const epubProgress = computed(() => {
    if (props.epubSpine.length === 0) return 0
    return ((props.epubCurrentSpineIndex + 1) / props.epubSpine.length) * 100
})

// Scroll-based progress for epub within current page
const updateProgress = () => {
    if (!containerRef.value) return 0
    
    const scrollTop = containerRef.value.scrollTop
    const docHeight = containerRef.value.scrollHeight
    const winHeight = containerRef.value.clientHeight
    const total = docHeight - winHeight
    const scrollPercent = total > 0 ? (scrollTop / total) * 100 : 0

    if (props.epubSpine.length > 0) {
        const spinePercent = (props.epubCurrentSpineIndex / props.epubSpine.length) * 100
        const pageContribution = (1 / props.epubSpine.length) * (scrollPercent / 100) * 100
        return spinePercent + pageContribution
    }
    return scrollPercent
}

defineExpose({
    containerRef,
    epubProgress,
    updateProgress,
})
</script>

<template>
    <div
        ref="containerRef"
        class="epub-reader-wrapper min-h-screen w-full"
        @click="$emit('click', $event)"
    >
        <div v-if="loading || epubSpineLoading" class="flex items-center justify-center h-screen">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
        </div>

        <div v-else class="epub-content-container mx-auto max-w-3xl px-6 py-12">
            <!-- eslint-disable-next-line vue/no-v-html -->
            <div class="epub-body" v-html="epubHtmlContent"></div>

            <!-- Spine page navigation at bottom -->
            <div v-if="!epubHasNext && nextChapter" class="py-12 flex flex-col items-center gap-4">
                <p class="text-gray-400">{{ t('reader.end_of_chapter') }}</p>
                <Button variant="default" @click.stop="$emit('navigateToChapter', nextChapter)">
                    {{ t('reader.next_chapter') }}
                    <ChevronRight class="ml-2 h-4 w-4" />
                </Button>
            </div>
        </div>
    </div>
</template>

<style scoped>
/* Custom EPUB body styling */
.epub-content-container {
    font-family: 'Georgia', 'Noto Serif SC', 'Source Han Serif CN', serif;
    line-height: 2;
    color: #e0ddd5;
}

/* Style the injected EPUB HTML content */
.epub-body :deep(p) {
    color: #e0ddd5;
    margin-bottom: 1em;
}

.epub-body :deep(h1),
.epub-body :deep(h2),
.epub-body :deep(h3),
.epub-body :deep(h4),
.epub-body :deep(h5),
.epub-body :deep(h6) {
    color: #ffffff;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

.epub-body :deep(a) {
    color: #8ab4f8;
}

.epub-body :deep(img) {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1em auto;
}

.epub-body :deep(span),
.epub-body :deep(div),
.epub-body :deep(li),
.epub-body :deep(td),
.epub-body :deep(th),
.epub-body :deep(blockquote) {
    color: #e0ddd5;
}

.epub-body :deep(blockquote) {
    border-left: 3px solid #555;
    padding-left: 1em;
    margin-left: 0;
    font-style: italic;
}

.epub-body :deep(pre),
.epub-body :deep(code) {
    background: #1a1a1a;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
}

.epub-body :deep(hr) {
    border: none;
    border-top: 1px solid #444;
    margin: 2em 0;
}

.epub-body :deep(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}

.epub-body :deep(td),
.epub-body :deep(th) {
    border: 1px solid #444;
    padding: 0.5em;
}
</style>
