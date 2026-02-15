<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Button } from '@/components/ui/button'
import { rewriteCssUrlsWithBase } from '@/lib/utils'
import { ChevronRight } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import type { Chapter, EpubSpineItem } from '@/api/types'

const { t } = useI18n()

interface Props {
    chapter: Chapter | undefined
    epubHtmlContent: string
    epubStylesheets: string[]
    epubInlineStyles: string[]
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
const shadowHostRef = ref<HTMLElement | null>(null)
const shadowRootRef = ref<ShadowRoot | null>(null)

const epubHasNext = computed(() => props.epubCurrentSpineIndex < props.epubSpine.length - 1)

const fallbackShadowStyles = `
:host {
    display: block;
    color: #e0ddd5;
    font-family: 'Georgia', 'Noto Serif SC', 'Source Han Serif CN', serif;
    line-height: 2;
}

:host *,
:host *::before,
:host *::after {
    box-sizing: border-box;
}

.epub-content-container {
    color: #e0ddd5;
}

.epub-body p {
    color: #e0ddd5;
    margin-bottom: 1em;
}

.epub-body h1,
.epub-body h2,
.epub-body h3,
.epub-body h4,
.epub-body h5,
.epub-body h6 {
    color: #ffffff;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

.epub-body a {
    color: #8ab4f8;
}

.epub-body img {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1em auto;
}

.epub-body span,
.epub-body div,
.epub-body li,
.epub-body td,
.epub-body th,
.epub-body blockquote {
    color: #e0ddd5;
}

.epub-body blockquote {
    border-left: 3px solid #555;
    padding-left: 1em;
    margin-left: 0;
    font-style: italic;
}

.epub-body pre,
.epub-body code {
    background: #1a1a1a;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
}

.epub-body hr {
    border: none;
    border-top: 1px solid #444;
    margin: 2em 0;
}

.epub-body table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}

.epub-body td,
.epub-body th {
    border: 1px solid #444;
    padding: 0.5em;
}
`

const ensureShadowRoot = (): ShadowRoot | null => {
    if (!shadowHostRef.value) return null
    if (!shadowRootRef.value || shadowRootRef.value.host !== shadowHostRef.value) {
        shadowRootRef.value = shadowHostRef.value.shadowRoot || shadowHostRef.value.attachShadow({ mode: 'open' })
    }
    return shadowRootRef.value
}

const stylesheetContentCache = new Map<string, Promise<string>>()
let renderRequestId = 0

const rewriteStylesheetCssUrls = (cssText: string, stylesheetHref: string) => {
    return rewriteCssUrlsWithBase(cssText, stylesheetHref)
}

const fetchStylesheetText = (stylesheetHref: string): Promise<string> => {
    const cachedPromise = stylesheetContentCache.get(stylesheetHref)
    if (cachedPromise) return cachedPromise

    const fetchPromise = fetch(stylesheetHref)
        .then(async (response) => {
            if (!response.ok) {
                throw new Error(`Failed to fetch stylesheet: ${response.status}`)
            }
            const cssText = await response.text()
            return rewriteStylesheetCssUrls(cssText, stylesheetHref)
        })
        .catch((e) => {
            console.warn(`Failed to load EPUB stylesheet: ${stylesheetHref}`, e)
            throw e
        })

    stylesheetContentCache.set(stylesheetHref, fetchPromise)
    return fetchPromise
}

const renderEpubContent = async () => {
    const requestId = ++renderRequestId
    const shadowRoot = ensureShadowRoot()
    if (!shadowRoot) return

    const stylesheets = await Promise.all(
        props.epubStylesheets.map(async (href) => {
            const stylesheetHref = href.trim()
            if (!stylesheetHref) return null

            try {
                const cssText = await fetchStylesheetText(stylesheetHref)
                return { cssText, href: stylesheetHref, useLinkFallback: false }
            } catch {
                return { cssText: '', href: stylesheetHref, useLinkFallback: true }
            }
        }),
    )

    if (requestId !== renderRequestId) return

    const fragment = document.createDocumentFragment()

    const fallbackStyle = document.createElement('style')
    fallbackStyle.textContent = fallbackShadowStyles
    fragment.appendChild(fallbackStyle)

    stylesheets.forEach((stylesheet) => {
        if (!stylesheet) return
        if (stylesheet.useLinkFallback) {
            const link = document.createElement('link')
            link.rel = 'stylesheet'
            link.href = stylesheet.href
            fragment.appendChild(link)
            return
        }

        const style = document.createElement('style')
        style.textContent = stylesheet.cssText
        fragment.appendChild(style)
    })

    props.epubInlineStyles.forEach((cssText) => {
        const styleText = cssText.trim()
        if (!styleText) return
        const style = document.createElement('style')
        style.textContent = styleText
        fragment.appendChild(style)
    })

    const contentContainer = document.createElement('div')
    contentContainer.className = 'epub-content-container'

    const body = document.createElement('div')
    body.className = 'epub-body'
    body.innerHTML = props.epubHtmlContent

    contentContainer.appendChild(body)
    fragment.appendChild(contentContainer)

    shadowRoot.replaceChildren(fragment)
}

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

watch(
    () => [props.epubHtmlContent, props.epubStylesheets, props.epubInlineStyles],
    () => {
        void renderEpubContent()
    },
    { immediate: true, deep: true },
)

watch(
    () => props.chapter?.id,
    () => {
        renderRequestId += 1
        stylesheetContentCache.clear()
    },
)

watch(
    () => [props.loading, props.epubSpineLoading, shadowHostRef.value],
    ([loading, spineLoading]) => {
        if (!loading && !spineLoading) {
            void renderEpubContent()
        }
    },
    { immediate: true },
)

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
            <div ref="shadowHostRef" class="epub-shadow-host"></div>

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
.epub-shadow-host {
    display: block;
    width: 100%;
}
</style>
