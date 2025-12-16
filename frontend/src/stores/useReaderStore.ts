import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { useContentStore } from "./useContentStore";
import { useAuthStore } from "./useAuthStore";
import { createReaderApi } from "@/api/reader";
import { createProgressApi } from "@/api/progress";
import { createContentApi } from "@/api/content";
import { ApiClient } from "@/api/client";
import type { Chapter } from "@/api/types";
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
        (localStorage.getItem("reader_mode") as ReaderMode) || "paged"
    );
    const currentPage = ref(0); // For paged mode (also tracks current reading pos in scroll)
    const PRELOAD_BUFFER = 5;

    // Computed
    const currentChapter = computed(() =>
        chapters.value.find((c) => c.id === currentChapterId.value)
    );

    const currentChapterIndex = computed(() =>
        chapters.value.findIndex((c) => c.id === currentChapterId.value)
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
                percentage
            );
        } catch (e) {
            console.warn("Failed to save progress", e);
        }
    }, 1000);

    const loadPage = (pageIndex: number) => {
        if (!currentContentId.value || !currentChapter.value) return;

        // Skip if already loaded
        if (pageUrls.value.has(pageIndex)) return;

        if (pageIndex >= currentChapter.value.page_count) return;

        const url = readerApi.getPageImage(
            currentContentId.value,
            currentChapter.value.sort_order,
            pageIndex
        );

        pageUrls.value.set(pageIndex, url);
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
                const progress = await progressApi.getChapterProgress(
                    chapterId
                );
                if (progress) {
                    startPage = progress.position;
                }
            } catch (e) {
                // Ignore progress load errors
            }

            currentPage.value = startPage;

            // Start loading initial pages
            loading.value = false;

            if (readerMode.value === "scroll") {
                const initialPages = [];
                for (let i = 0; i < PRELOAD_BUFFER; i++) {
                    const p = startPage + i;
                    if (
                        currentChapter.value &&
                        p >= currentChapter.value.page_count
                    )
                        break;
                    initialPages.push(p);
                }
                // Fallback if empty or startPage was invalid, just load 0
                if (initialPages.length === 0) initialPages.push(0);

                pages.value = initialPages;
                pages.value.forEach((p) => loadPage(p));
            } else {
                loadPage(startPage);
                for (let i = 1; i <= PRELOAD_BUFFER; i++) {
                    loadPage(startPage + i);
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

        // Computed
        currentChapter,
        currentChapterIndex,
        prevChapter,
        nextChapter,

        // Actions
        loadChapter,
        loadPage,
        saveProgress,
        setMode,

        // Constants
        PRELOAD_BUFFER,
    };
});
