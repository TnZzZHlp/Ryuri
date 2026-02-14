import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { useContentStore } from "./useContentStore";
import { useAuthStore } from "./useAuthStore";
import { createReaderApi } from "@/api/reader";
import { createProgressApi } from "@/api/progress";
import { createContentApi } from "@/api/content";
import { ApiClient } from "@/api/client";
import type { Chapter, EpubSpineItem } from "@/api/types";
import { useDebounceFn } from "@vueuse/core";

export type ReaderMode = "scroll" | "paged";

export const useReaderStore = defineStore("reader", () => {
    // Dependencies
    const authStore = useAuthStore();
    const contentStore = useContentStore();
    const apiClient = new ApiClient({
        baseUrl: import.meta.env.VITE_API_BASE_URL || "",
        getToken: () => authStore.token,
    });
    const readerApi = createReaderApi(apiClient);
    const progressApi = createProgressApi(apiClient);
    const contentApi = createContentApi(apiClient);

    // State
    const chapters = ref<Chapter[]>([]);
    const currentContentId = ref<number | null>(null);
    const currentChapterId = ref<number | null>(null);
    const loading = ref(false);
    const pageUrls = ref<Map<number, string>>(new Map());
    const failedPages = ref<Set<number>>(new Set());
    const loadingPages = ref<Set<number>>(new Set());
    const endOfChapter = ref(false);
    const pages = ref<number[]>([]); // Buffer for scroll mode
    const readerMode = ref<ReaderMode>(
        (localStorage.getItem("reader_mode") as ReaderMode) || "paged",
    );
    const currentPage = ref(0); // For paged mode (also tracks current reading pos in scroll)
    const PRELOAD_BUFFER = 5;

    // EPUB-specific state
    const epubSpine = ref<EpubSpineItem[]>([]);
    const epubCurrentSpineIndex = ref(0);
    const epubHtmlContent = ref("");
    const epubSpineLoading = ref(false);

    // Computed
    const isNovel = computed(() => currentChapter.value?.file_type === "epub");

    const currentChapter = computed(() =>
        chapters.value.find((c) => c.id === currentChapterId.value),
    );

    const currentChapterIndex = computed(() =>
        chapters.value.findIndex((c) => c.id === currentChapterId.value),
    );

    const prevChapter = computed(() => {
        if (currentChapterIndex.value > 0) {
            return chapters.value[currentChapterIndex.value - 1];
        }
        return null;
    });

    const nextChapter = computed(() => {
        if (
            currentChapterIndex.value !== -1 &&
            currentChapterIndex.value < chapters.value.length - 1
        ) {
            return chapters.value[currentChapterIndex.value + 1];
        }
        return null;
    });

    const epubHasNext = computed(
        () => epubCurrentSpineIndex.value < epubSpine.value.length - 1,
    );
    const epubHasPrev = computed(() => epubCurrentSpineIndex.value > 0);

    // Actions
    const setMode = (mode: ReaderMode) => {
        readerMode.value = mode;
        localStorage.setItem("reader_mode", mode);

        if (mode === "paged") {
            loadPage(currentPage.value);
            // Preload next
            for (let i = 1; i <= PRELOAD_BUFFER; i++) {
                loadPage(currentPage.value + i);
            }
        } else {
            // Scroll mode buffer init
            if (pages.value.length < PRELOAD_BUFFER) {
                const newPages = [];
                for (let i = 0; i < PRELOAD_BUFFER; i++) {
                    const p = currentPage.value + i;
                    if (
                        currentChapter.value &&
                        p >= currentChapter.value.page_count
                    )
                        break;
                    newPages.push(p);
                }
                pages.value =
                    newPages.length > 0 ? newPages : [currentPage.value];
                pages.value.forEach((p) => loadPage(p));
            }
        }
    };

    const saveProgress = useDebounceFn(async (pageIndex: number) => {
        if (!currentChapter.value || !currentChapterId.value) return;
        const total = currentChapter.value.page_count;
        const percentage = total > 0 ? ((pageIndex + 1) / total) * 100 : 0;

        try {
            await progressApi.updateChapterProgress(
                currentChapterId.value,
                pageIndex,
                percentage,
            );
        } catch (e) {
            console.warn("Failed to save progress", e);
        }
    }, 1000);

    const saveNovelProgress = useDebounceFn(async (percentage: number) => {
        if (!currentChapterId.value) return;

        try {
            await progressApi.updateChapterProgress(
                currentChapterId.value,
                0,
                Math.min(100, percentage),
            );
        } catch (e) {
            console.warn("Failed to save novel progress", e);
        }
    }, 1000);

    const loadPage = (pageIndex: number) => {
        if (!currentContentId.value || !currentChapter.value) return;

        // Skip if already loaded
        if (pageUrls.value.has(pageIndex)) return;

        if (pageIndex >= currentChapter.value.page_count) return;

        const url = readerApi.getPageImage(
            currentContentId.value,
            currentChapter.value.id,
            pageIndex,
        );

        pageUrls.value.set(pageIndex, url);

        // Preload image in background
        const img = new Image();
        img.src = url;
    };

    /**
     * Rewrite relative URLs in EPUB XHTML content to point at the backend resource endpoint.
     * Handles src, href, xlink:href attributes and url() in inline styles.
     */
    const rewriteEpubUrls = (
        html: string,
        contentId: number,
        chapterId: number,
        currentResourcePath: string,
    ): string => {
        // Determine the base directory of the current spine resource
        const lastSlash = currentResourcePath.lastIndexOf("/");
        const baseDir =
            lastSlash >= 0
                ? currentResourcePath.substring(0, lastSlash + 1)
                : "";

        const resolveUrl = (relativeUrl: string): string => {
            // Skip absolute URLs, data URIs, and fragment-only refs
            if (
                relativeUrl.startsWith("http://") ||
                relativeUrl.startsWith("https://") ||
                relativeUrl.startsWith("data:") ||
                relativeUrl.startsWith("#") ||
                relativeUrl.startsWith("mailto:")
            ) {
                return relativeUrl;
            }

            // Resolve relative path against base directory
            let resolved = baseDir + relativeUrl;

            // Normalize path (collapse ../ segments)
            const parts = resolved.split("/");
            const normalized: string[] = [];
            for (const part of parts) {
                if (part === "..") {
                    normalized.pop();
                } else if (part !== "." && part !== "") {
                    normalized.push(part);
                }
            }
            resolved = normalized.join("/");

            return readerApi.getEpubResourceUrl(contentId, chapterId, resolved);
        };

        // Rewrite src, href, xlink:href attributes
        let result = html.replace(
            /((?:src|href|xlink:href)\s*=\s*)(["'])((?:(?!\2).)+)\2/gi,
            (_match, prefix: string, quote: string, url: string) => {
                return `${prefix}${quote}${resolveUrl(url)}${quote}`;
            },
        );

        // Rewrite url() references in inline styles
        result = result.replace(
            /url\(\s*(["']?)((?:(?!\1\)).)+)\1\s*\)/gi,
            (_match, quote: string, url: string) => {
                return `url(${quote}${resolveUrl(url)}${quote})`;
            },
        );

        return result;
    };

    /**
     * Load an EPUB spine page by index, fetching its XHTML content and rewriting URLs.
     */
    const loadEpubSpinePage = async (index: number) => {
        if (!currentContentId.value || !currentChapterId.value) return;
        if (index < 0 || index >= epubSpine.value.length) return;

        epubSpineLoading.value = true;
        epubCurrentSpineIndex.value = index;
        const spineItem = epubSpine.value[index];
        if (!spineItem) return;

        try {
            // Fetch the XHTML content via the resource endpoint
            const url = readerApi.getEpubResourceUrl(
                currentContentId.value,
                currentChapterId.value,
                spineItem.path,
            );
            const response = await fetch(url);
            let html = await response.text();

            // Rewrite internal URLs
            html = rewriteEpubUrls(
                html,
                currentContentId.value,
                currentChapterId.value,
                spineItem.path,
            );

            // Extract body content from the XHTML
            const bodyMatch = html.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
            epubHtmlContent.value = bodyMatch?.[1] ?? html;

            // Save progress
            const percentage =
                epubSpine.value.length > 0
                    ? ((index + 1) / epubSpine.value.length) * 100
                    : 0;
            saveNovelProgress(percentage);
        } catch (e) {
            console.error("Failed to load EPUB spine page:", e);
            epubHtmlContent.value = `<p style="color: #ff6b6b;">Failed to load content</p>`;
        } finally {
            epubSpineLoading.value = false;
        }
    };

    const epubNextPage = () => {
        if (epubHasNext.value) {
            loadEpubSpinePage(epubCurrentSpineIndex.value + 1);
        }
    };

    const epubPrevPage = () => {
        if (epubHasPrev.value) {
            loadEpubSpinePage(epubCurrentSpineIndex.value - 1);
        }
    };

    const loadMorePages = () => {
        const maxPage = Math.max(...pages.value);
        const nextPage = maxPage + 1;

        if (
            currentChapter.value &&
            nextPage < currentChapter.value.page_count &&
            !pages.value.includes(nextPage) &&
            !failedPages.value.has(nextPage)
        ) {
            pages.value.push(nextPage);
            loadPage(nextPage);
        }
    };

    const loadChapter = async (contentId: number, chapterId: number) => {
        // Reset state if chapter changed
        if (
            currentChapterId.value !== chapterId ||
            currentContentId.value !== contentId
        ) {
            // Cleanup old URLs
            pageUrls.value.clear();

            failedPages.value.clear();
            loadingPages.value.clear();
            endOfChapter.value = false;
            pages.value = [];
            epubSpine.value = [];
            epubHtmlContent.value = "";
            epubCurrentSpineIndex.value = 0;
        }

        currentContentId.value = contentId;
        currentChapterId.value = chapterId;
        loading.value = true;

        try {
            // Load chapters list if needed
            if (contentStore.chapters.get(contentId)) {
                chapters.value = contentStore.chapters.get(contentId)!;
            } else {
                chapters.value = await contentApi.listChapters(contentId);
                chapters.value.sort((a, b) => a.sort_order - b.sort_order);
            }

            // Fetch Progress
            let startPage = 0;
            try {
                const progresses =
                    await progressApi.getChapterProgress(chapterId);
                const progress = progresses.find(
                    (p) => p.chapter_id === chapterId,
                );
                if (progress) {
                    startPage = progress.position;
                }
            } catch {
                // Ignore progress load errors
            }

            currentPage.value = startPage;

            // Branch based on content type
            if (isNovel.value) {
                // Novel EPUB: fetch spine and load first page
                if (currentChapter.value) {
                    try {
                        epubSpine.value = await readerApi.getEpubSpine(
                            contentId,
                            currentChapter.value.id,
                        );
                        // Load the first spine page
                        if (epubSpine.value.length > 0) {
                            await loadEpubSpinePage(0);
                        }
                    } catch (e) {
                        console.error("Failed to load EPUB spine:", e);
                    }
                }
                loading.value = false;
            } else {
                // Comic: load page images
                loading.value = false;

                if (readerMode.value === "scroll") {
                    const initialPages = [];
                    const endPage = startPage + PRELOAD_BUFFER;

                    for (let i = 0; i <= endPage; i++) {
                        if (
                            currentChapter.value &&
                            i >= currentChapter.value.page_count
                        )
                            break;
                        initialPages.push(i);
                    }

                    if (initialPages.length === 0) initialPages.push(0);
                    pages.value = initialPages;

                    for (let i = 0; i < PRELOAD_BUFFER; i++) {
                        loadPage(startPage + i);
                    }

                    if (startPage > 0) loadPage(startPage - 1);
                } else {
                    loadPage(startPage);
                    for (let i = 1; i <= PRELOAD_BUFFER; i++) {
                        loadPage(startPage + i);
                    }
                }
            }
        } catch (e) {
            console.error(e);
            loading.value = false;
            throw e;
        }
    };

    return {
        // State
        chapters,
        currentContentId,
        currentChapterId,
        loading,
        pageUrls,
        failedPages,
        loadingPages,
        endOfChapter,
        pages,
        readerMode,
        currentPage,

        // EPUB state
        epubSpine,
        epubCurrentSpineIndex,
        epubHtmlContent,
        epubSpineLoading,

        // Computed
        isNovel,
        currentChapter,
        currentChapterIndex,
        prevChapter,
        nextChapter,
        epubHasNext,
        epubHasPrev,

        // Actions
        loadChapter,
        loadPage,
        loadMorePages,
        loadEpubSpinePage,
        epubNextPage,
        epubPrevPage,
        saveProgress,
        saveNovelProgress,
        setMode,

        // Constants
        PRELOAD_BUFFER,
    };
});
