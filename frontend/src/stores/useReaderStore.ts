import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { useContentStore } from "./useContentStore";
import { useAuthStore } from "./useAuthStore";
import { createReaderApi } from "@/api/reader";
import { createProgressApi } from "@/api/progress";
import { createContentApi } from "@/api/content";
import { ApiClient } from "@/api/client";
import {
    isLooseEpubPathMatch,
    isSkippableResourceUrl,
    normalizeEpubPath,
    resolveRelativeUrlWithBaseQuery,
    splitEpubSrc,
} from "@/lib/utils";
import { parseEpubToc } from "@/lib/epubTocParser";
import type { Chapter, EpubSpineItem, EpubTocItem } from "@/api/types";
import { useDebounceFn } from "@vueuse/core";

export type ReaderMode = "scroll" | "paged";

interface EpubSpineContent {
    body: string;
    stylesheets: string[];
    inlineStyles: string[];
}

export const useReaderStore = defineStore("reader", () => {
    const MIN_PRELOAD_BUFFER = 1;
    const MAX_PRELOAD_BUFFER = 20;
    const storedPreloadBuffer = Number(
        localStorage.getItem("reader_preload_buffer"),
    );

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
    const preloadBuffer = ref<number>(
        Number.isFinite(storedPreloadBuffer)
            ? Math.min(
                  MAX_PRELOAD_BUFFER,
                  Math.max(MIN_PRELOAD_BUFFER, Math.floor(storedPreloadBuffer)),
              )
            : 5,
    );
    const currentPage = ref(0); // For paged mode (also tracks current reading pos in scroll)

    // EPUB-specific state
    const epubSpine = ref<EpubSpineItem[]>([]);
    const epubToc = ref<EpubTocItem[]>([]);
    const epubCurrentSpineIndex = ref(0);
    const epubHtmlContent = ref("");
    const epubStylesheets = ref<string[]>([]);
    const epubInlineStyles = ref<string[]>([]);
    const epubSpineLoading = ref(false);
    const epubTocLoading = ref(false);
    const epubSpineContentCache = ref<Map<number, EpubSpineContent>>(new Map());
    const epubTocCache = ref<Map<number, EpubTocItem[]>>(new Map());
    const epubSpinePrefetching = ref<Set<number>>(new Set());
    const epubSpineInFlightFetches = ref<
        Map<number, Promise<EpubSpineContent>>
    >(new Map());
    const epubResourcePrefetched = new Set<string>();
    const epubResourcePrefetching = new Map<string, Promise<void>>();
    const epubStylesheetAnalysisPrefetching = new Map<string, Promise<void>>();
    let epubLoadRequestId = 0;
    let epubTocRequestId = 0;
    let epubChapterGeneration = 0;

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
            for (let i = 1; i <= preloadBuffer.value; i++) {
                loadPage(currentPage.value + i);
            }
        } else {
            // Scroll mode buffer init
            if (pages.value.length < preloadBuffer.value) {
                const newPages = [];
                for (let i = 0; i < preloadBuffer.value; i++) {
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

    const setPreloadBuffer = (value: number) => {
        const normalized = Math.min(
            MAX_PRELOAD_BUFFER,
            Math.max(MIN_PRELOAD_BUFFER, Math.floor(value)),
        );
        preloadBuffer.value = normalized;
        localStorage.setItem("reader_preload_buffer", String(normalized));

        if (!currentChapter.value) return;

        if (isNovel.value) {
            preloadEpubSpineAround(epubCurrentSpineIndex.value);
            return;
        }

        if (readerMode.value === "paged") {
            loadPage(currentPage.value);
            for (let i = 1; i <= preloadBuffer.value; i++) {
                loadPage(currentPage.value + i);
            }
            return;
        }

        const maxPage = currentChapter.value.page_count - 1;
        const startPage = Math.max(0, currentPage.value - 1);
        const endPage = Math.min(
            maxPage,
            currentPage.value + preloadBuffer.value,
        );
        const nextPages: number[] = [];

        for (let i = startPage; i <= endPage; i++) {
            nextPages.push(i);
            loadPage(i);
        }

        pages.value = nextPages.length > 0 ? nextPages : [currentPage.value];
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

    const saveNovelProgress = useDebounceFn(
        async (position: number, percentage: number) => {
            if (!currentChapterId.value) return;

            try {
                await progressApi.updateChapterProgress(
                    currentChapterId.value,
                    Math.max(0, Math.floor(position)),
                    Math.min(100, Math.max(0, percentage)),
                );
            } catch (e) {
                console.warn("Failed to save novel progress", e);
            }
        },
        1000,
    );

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
    const prefetchEpubResource = (
        url: string,
        generation = epubChapterGeneration,
    ) => {
        if (generation !== epubChapterGeneration) return;
        if (
            !url ||
            url.startsWith("data:") ||
            url.startsWith("blob:") ||
            url.startsWith("javascript:") ||
            url.startsWith("#")
        ) {
            return;
        }
        if (epubResourcePrefetched.has(url)) return;

        const existing = epubResourcePrefetching.get(url);
        if (existing) return;

        const prefetchPromise = fetch(url)
            .then(() => {
                if (generation !== epubChapterGeneration) return;
                epubResourcePrefetched.add(url);
            })
            .catch((e) => {
                console.warn(`Failed to preload EPUB resource: ${url}`, e);
            })
            .finally(() => {
                if (epubResourcePrefetching.get(url) === prefetchPromise) {
                    epubResourcePrefetching.delete(url);
                }
            });

        epubResourcePrefetching.set(url, prefetchPromise);
    };

    const prefetchEpubResourcesFromHtml = (
        html: string,
        generation = epubChapterGeneration,
    ) => {
        if (generation !== epubChapterGeneration) return;
        const parser = new DOMParser();
        const doc = parser.parseFromString(html, "text/html");
        const urls = new Set<string>();
        const stylesheetUrls = new Set<string>();
        const addUrl = (url: string | null | undefined) => {
            if (!url) return;
            const normalized = url.trim();
            if (!normalized) return;
            urls.add(normalized);
        };

        doc.querySelectorAll(
            "img[src], source[src], audio[src], video[src], track[src]",
        ).forEach((el) => {
            addUrl(el.getAttribute("src"));
        });

        doc.querySelectorAll("link[rel~='stylesheet'][href]").forEach((el) => {
            const stylesheetUrl = el.getAttribute("href");
            if (!stylesheetUrl) return;
            const normalized = stylesheetUrl.trim();
            if (!normalized) return;
            stylesheetUrls.add(normalized);
        });

        doc.querySelectorAll("[srcset]").forEach((el) => {
            const srcset = el.getAttribute("srcset");
            if (!srcset) return;
            srcset.split(",").forEach((entry) => {
                const candidate = entry.trim().split(/\s+/)[0];
                addUrl(candidate);
            });
        });

        const cssUrlPattern = /url\(\s*(["']?)(.*?)\1\s*\)/gi;
        doc.querySelectorAll("[style]").forEach((el) => {
            const style = el.getAttribute("style");
            if (!style) return;
            let match: RegExpExecArray | null = cssUrlPattern.exec(style);
            while (match) {
                addUrl(match[2]);
                match = cssUrlPattern.exec(style);
            }
            cssUrlPattern.lastIndex = 0;
        });

        doc.querySelectorAll("style").forEach((el) => {
            const cssText = el.textContent || "";
            let match: RegExpExecArray | null = cssUrlPattern.exec(cssText);
            while (match) {
                addUrl(match[2]);
                match = cssUrlPattern.exec(cssText);
            }
            cssUrlPattern.lastIndex = 0;
        });

        stylesheetUrls.forEach((stylesheetUrl) => {
            prefetchEpubResource(stylesheetUrl, generation);
            const existing =
                epubStylesheetAnalysisPrefetching.get(stylesheetUrl);
            if (existing) return;

            const prefetchPromise = fetch(stylesheetUrl)
                .then(async (response) => {
                    if (generation !== epubChapterGeneration) return;
                    if (!response.ok) return;

                    const cssText = await response.text();
                    const cssUrlPattern = /url\(\s*(["']?)(.*?)\1\s*\)/gi;
                    let match: RegExpExecArray | null =
                        cssUrlPattern.exec(cssText);

                    while (match) {
                        const candidate = match[2]?.trim();
                        if (candidate && !isSkippableResourceUrl(candidate)) {
                            const resolved = resolveRelativeUrlWithBaseQuery(
                                candidate,
                                stylesheetUrl,
                            );
                            if (resolved) {
                                prefetchEpubResource(resolved, generation);
                            }
                        }
                        match = cssUrlPattern.exec(cssText);
                    }
                })
                .catch((e) => {
                    console.warn(
                        `Failed to analyze EPUB stylesheet resources: ${stylesheetUrl}`,
                        e,
                    );
                })
                .finally(() => {
                    if (
                        epubStylesheetAnalysisPrefetching.get(stylesheetUrl) ===
                        prefetchPromise
                    ) {
                        epubStylesheetAnalysisPrefetching.delete(stylesheetUrl);
                    }
                });

            epubStylesheetAnalysisPrefetching.set(
                stylesheetUrl,
                prefetchPromise,
            );
        });

        urls.forEach((url) => prefetchEpubResource(url, generation));
    };

    const extractEpubSpineContent = (
        rewrittenHtml: string,
    ): EpubSpineContent => {
        const parser = new DOMParser();
        const doc = parser.parseFromString(rewrittenHtml, "text/html");

        const stylesheets: string[] = [];
        const seenStylesheets = new Set<string>();
        doc.querySelectorAll("link[rel~='stylesheet'][href]").forEach((el) => {
            const href = el.getAttribute("href")?.trim();
            if (!href || seenStylesheets.has(href)) return;
            seenStylesheets.add(href);
            stylesheets.push(href);
        });

        const inlineStyles: string[] = [];
        doc.querySelectorAll("head style").forEach((el) => {
            const cssText = el.textContent?.trim();
            if (!cssText) return;
            inlineStyles.push(cssText);
        });

        const body = doc.body?.innerHTML ?? rewrittenHtml;

        return {
            body,
            stylesheets,
            inlineStyles,
        };
    };

    const fetchEpubSpineContent = async (
        index: number,
        generation = epubChapterGeneration,
    ): Promise<EpubSpineContent> => {
        if (generation !== epubChapterGeneration) {
            throw new Error("Stale EPUB fetch context");
        }
        if (!currentContentId.value || !currentChapterId.value) {
            throw new Error("Missing EPUB context");
        }
        if (index < 0 || index >= epubSpine.value.length) {
            throw new Error(`EPUB spine index out of range: ${index}`);
        }

        const cachedContent = epubSpineContentCache.value.get(index);
        if (cachedContent) return cachedContent;
        const inFlight = epubSpineInFlightFetches.value.get(index);
        if (inFlight) return inFlight;

        const spineItem = epubSpine.value[index];
        if (!spineItem) {
            throw new Error(`Missing EPUB spine item: ${index}`);
        }

        const fetchPromise = (async () => {
            const url = readerApi.getEpubResourceUrl(
                currentContentId.value!,
                currentChapterId.value!,
                spineItem.path,
            );
            const response = await fetch(url);
            const html = await response.text();

            const rewrittenHtml = rewriteEpubUrls(
                html,
                currentContentId.value!,
                currentChapterId.value!,
                spineItem.path,
            );
            prefetchEpubResourcesFromHtml(rewrittenHtml, generation);

            const content = extractEpubSpineContent(rewrittenHtml);

            if (generation !== epubChapterGeneration) {
                throw new Error("Stale EPUB fetch context");
            }

            epubSpineContentCache.value.set(index, content);
            return content;
        })().finally(() => {
            if (epubSpineInFlightFetches.value.get(index) === fetchPromise) {
                epubSpineInFlightFetches.value.delete(index);
            }
        });

        epubSpineInFlightFetches.value.set(index, fetchPromise);
        return fetchPromise;
    };

    const preloadEpubSpineAround = (fromIndex: number) => {
        if (!currentContentId.value || !currentChapterId.value) return;
        if (fromIndex < 0 || fromIndex >= epubSpine.value.length) return;

        const generation = epubChapterGeneration;
        for (let i = 1; i <= preloadBuffer.value; i++) {
            const targets = [fromIndex + i, fromIndex - i];
            for (const preloadIndex of targets) {
                if (
                    preloadIndex < 0 ||
                    preloadIndex >= epubSpine.value.length
                ) {
                    continue;
                }
                if (epubSpineContentCache.value.has(preloadIndex)) continue;
                if (epubSpinePrefetching.value.has(preloadIndex)) continue;

                epubSpinePrefetching.value.add(preloadIndex);
                void fetchEpubSpineContent(preloadIndex, generation)
                    .catch((e) => {
                        console.warn(
                            `Failed to preload EPUB spine page ${preloadIndex}:`,
                            e,
                        );
                    })
                    .finally(() => {
                        epubSpinePrefetching.value.delete(preloadIndex);
                    });
            }
        }
    };

    const loadEpubSpinePage = async (index: number) => {
        if (!currentContentId.value || !currentChapterId.value) return;
        if (index < 0 || index >= epubSpine.value.length) return;

        const requestId = ++epubLoadRequestId;
        epubSpineLoading.value = true;
        epubCurrentSpineIndex.value = index;

        try {
            const content = await fetchEpubSpineContent(index);
            if (requestId !== epubLoadRequestId) return;
            epubHtmlContent.value = content.body;
            epubStylesheets.value = content.stylesheets;
            epubInlineStyles.value = content.inlineStyles;

            // Save progress
            const percentage =
                epubSpine.value.length > 0
                    ? ((index + 1) / epubSpine.value.length) * 100
                    : 0;
            saveNovelProgress(index, percentage);

            preloadEpubSpineAround(index);
        } catch (e) {
            console.error("Failed to load EPUB spine page:", e);
            if (requestId === epubLoadRequestId) {
                epubHtmlContent.value = `<p style="color: #ff6b6b;">Failed to load content</p>`;
                epubStylesheets.value = [];
                epubInlineStyles.value = [];
            }
        } finally {
            if (requestId === epubLoadRequestId) {
                epubSpineLoading.value = false;
            }
        }
    };

    const loadEpubToc = async (contentId: number, chapterId: number) => {
        const cachedToc = epubTocCache.value.get(chapterId);
        if (cachedToc) {
            epubToc.value = cachedToc;
            epubTocLoading.value = false;
            return;
        }

        const requestId = ++epubTocRequestId;
        epubTocLoading.value = true;
        try {
            const fetchResource = async (path: string): Promise<string> => {
                const url = readerApi.getEpubResourceUrl(
                    contentId,
                    chapterId,
                    path,
                );
                const response = await fetch(url);
                if (!response.ok) {
                    throw new Error(
                        `Failed to fetch EPUB resource: ${response.status}`,
                    );
                }
                return response.text();
            };
            const toc = await parseEpubToc(fetchResource);
            if (requestId !== epubTocRequestId) return;
            epubToc.value = toc;
            epubTocCache.value.set(chapterId, toc);
        } catch (e) {
            if (requestId === epubTocRequestId) {
                epubToc.value = [];
            }
            console.warn("Failed to load EPUB TOC:", e);
        } finally {
            if (requestId === epubTocRequestId) {
                epubTocLoading.value = false;
            }
        }
    };

    const findEpubSpineIndexByPath = (targetPath: string): number => {
        const normalizedTarget =
            normalizeEpubPath(targetPath).split("?")[0] ?? "";
        if (!normalizedTarget) return -1;

        const exactIndex = epubSpine.value.findIndex((item) => {
            const spinePath = normalizeEpubPath(item.path).split("?")[0] ?? "";
            return spinePath === normalizedTarget;
        });
        if (exactIndex >= 0) return exactIndex;

        const looseIndex = epubSpine.value.findIndex((item) => {
            const spinePath = normalizeEpubPath(item.path).split("?")[0] ?? "";
            return isLooseEpubPathMatch(spinePath, normalizedTarget);
        });
        return looseIndex;
    };

    const jumpToEpubTocEntry = async (
        entry: EpubTocItem,
    ): Promise<string | undefined> => {
        const splitSrc = splitEpubSrc(entry.src || "");
        const entryPath = normalizeEpubPath(splitSrc.path).split("?")[0] ?? "";
        const fragment =
            splitSrc.fragment?.replace(/^#/, "").trim() || undefined;

        if (!entryPath) {
            return fragment;
        }

        const targetIndex = findEpubSpineIndexByPath(entryPath);
        if (targetIndex < 0) {
            console.warn(`EPUB TOC target not found in spine: ${entryPath}`);
            return fragment;
        }

        await loadEpubSpinePage(targetIndex);
        return fragment;
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
            epubToc.value = [];
            epubHtmlContent.value = "";
            epubStylesheets.value = [];
            epubInlineStyles.value = [];
            epubCurrentSpineIndex.value = 0;
            epubTocLoading.value = false;
            epubSpineContentCache.value.clear();
            epubSpinePrefetching.value.clear();
            epubSpineInFlightFetches.value.clear();
            epubResourcePrefetched.clear();
            epubResourcePrefetching.clear();
            epubStylesheetAnalysisPrefetching.clear();
            epubChapterGeneration += 1;
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
            let startPercentage = 0;
            try {
                const progresses =
                    await progressApi.getChapterProgress(chapterId);
                const progress = progresses.find(
                    (p) => p.chapter_id === chapterId,
                );
                if (progress) {
                    startPage = progress.position;
                    startPercentage = progress.percentage;
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
                        void loadEpubToc(contentId, currentChapter.value.id);
                        // Resume from saved spine position
                        if (epubSpine.value.length > 0) {
                            let startSpineIndex = Math.min(
                                Math.max(0, startPage),
                                epubSpine.value.length - 1,
                            );
                            if (startSpineIndex === 0 && startPercentage > 0) {
                                const percentageIndex = Math.floor(
                                    (Math.min(
                                        100,
                                        Math.max(0, startPercentage),
                                    ) /
                                        100) *
                                        epubSpine.value.length,
                                );
                                startSpineIndex = Math.min(
                                    Math.max(0, percentageIndex),
                                    epubSpine.value.length - 1,
                                );
                            }
                            await loadEpubSpinePage(startSpineIndex);
                            preloadEpubSpineAround(startSpineIndex);
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
                    const endPage = startPage + preloadBuffer.value;

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

                    for (let i = 0; i < preloadBuffer.value; i++) {
                        loadPage(startPage + i);
                    }

                    if (startPage > 0) loadPage(startPage - 1);
                } else {
                    loadPage(startPage);
                    for (let i = 1; i <= preloadBuffer.value; i++) {
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
        preloadBuffer,
        currentPage,

        // EPUB state
        epubSpine,
        epubToc,
        epubCurrentSpineIndex,
        epubHtmlContent,
        epubStylesheets,
        epubInlineStyles,
        epubSpineLoading,
        epubTocLoading,

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
        loadEpubToc,
        jumpToEpubTocEntry,
        epubNextPage,
        epubPrevPage,
        saveProgress,
        saveNovelProgress,
        setMode,
        setPreloadBuffer,

        // Constants
        MIN_PRELOAD_BUFFER,
        MAX_PRELOAD_BUFFER,
    };
});
