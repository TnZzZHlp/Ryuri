/**
 * Content Store - Pinia store for content state management
 *
 * Manages content list caching by library, current content selection,
 * chapter management, and search functionality.
 *
 * **Implements: Requirements 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1, 3.2, 4.1, 4.2, 4.3, 5.1, 5.2, 6.1, 6.2**
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ApiClient } from '@/api/client'
import { createContentApi, type ContentApi } from '@/api/content'
import type { ContentResponse, Chapter } from '@/api/types'
import { useAuthStore } from './useAuthStore'

// Lazy-initialized API instance
let apiClient: ApiClient | null = null
let contentApi: ContentApi | null = null

function getContentApi(getToken: () => string | null): ContentApi {
    if (!contentApi) {
        apiClient = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken,
        })
        contentApi = createContentApi(apiClient)
    }
    return contentApi
}

export const useContentStore = defineStore('content', () => {
    // State
    // **Implements: Requirement 1.1**
    const contents = ref<Map<number, ContentResponse[]>>(new Map())
    const currentContent = ref<ContentResponse | null>(null)
    const chapters = ref<Map<number, Chapter[]>>(new Map())
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Thumbnail state
    // **Implements: Requirements 3.4, 3.5**
    const thumbnailUrls = ref<Map<number, string>>(new Map())
    const thumbnailLoading = ref<Set<number>>(new Set())

    // Getters
    // **Implements: Requirement 1.2**
    const contentsByLibrary = computed(() => {
        return (libraryId: number): ContentResponse[] => {
            return contents.value.get(libraryId) ?? []
        }
    })

    // **Implements: Requirement 1.3**
    const currentChapters = computed((): Chapter[] => {
        if (!currentContent.value) return []
        return chapters.value.get(currentContent.value.id) ?? []
    })

    // Thumbnail getters
    // **Implements: Requirements 3.4, 3.5**
    /**
     * Gets the thumbnail URL for a content ID.
     * Returns null if the thumbnail is not loaded.
     */
    const getThumbnailUrl = computed(() => {
        return (contentId: number): string | null => {
            return thumbnailUrls.value.get(contentId) ?? null
        }
    })

    /**
     * Checks if a thumbnail is currently loading.
     */
    const isThumbnailLoading = computed(() => {
        return (contentId: number): boolean => {
            return thumbnailLoading.value.has(contentId)
        }
    })

    // Internal helper to get token from auth store
    function getToken(): string | null {
        const authStore = useAuthStore()
        return authStore.token
    }

    // Actions

    /**
     * Fetches contents for a library and caches them.
     * Automatically triggers thumbnail preloading after fetching.
     * **Implements: Requirements 1.1, 2.1, 2.2, 2.3, 4.1, 4.3, 5.5**
     *
     * @param libraryId - The library ID to fetch contents for
     * @param force - If true, bypasses cache and fetches fresh data
     * @returns Array of content items
     */
    async function fetchContents(libraryId: number, force = false): Promise<ContentResponse[]> {
        // **Implements: Requirement 6.2** - Clear error on new operation
        error.value = null

        // **Implements: Requirement 2.2** - Return cached data if available and not forced
        if (!force && contents.value.has(libraryId)) {
            return contents.value.get(libraryId)!
        }

        loading.value = true
        try {
            const response = await getContentApi(getToken).list(libraryId)
            // **Implements: Requirement 2.1** - Cache results by library ID
            contents.value.set(libraryId, response)

            // **Implements: Requirements 1.1, 4.3, 5.5** - Auto-trigger thumbnail preload (non-blocking)
            preloadThumbnails(response).catch(err => {
                console.warn('Failed to preload thumbnails:', err)
            })

            return response
        } catch (e) {
            // **Implements: Requirement 6.1** - Set error state on failure
            error.value = e instanceof Error ? e.message : 'Failed to fetch content list'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Searches contents within a library.
     * **Implements: Requirements 3.1, 3.2**
     *
     * @param libraryId - The library ID to search in
     * @param query - The search query string
     * @returns Array of matching content items
     */
    async function searchContents(libraryId: number, query: string): Promise<ContentResponse[]> {
        // **Implements: Requirement 6.2** - Clear error on new operation
        error.value = null

        // **Implements: Requirement 3.2** - Return cached contents for empty query
        if (!query.trim()) {
            return contents.value.get(libraryId) ?? []
        }

        loading.value = true
        try {
            // **Implements: Requirement 3.1** - Call search API
            return await getContentApi(getToken).search(libraryId, query)
        } catch (e) {
            // **Implements: Requirement 6.1** - Set error state on failure
            error.value = e instanceof Error ? e.message : 'Search content failed'
            return []
        } finally {
            loading.value = false
        }
    }

    /**
     * Selects a content and fetches its chapters.
     * **Implements: Requirements 4.1, 4.2, 4.3**
     *
     * @param content - The content to select
     */
    async function selectContent(content: ContentResponse): Promise<void> {
        // **Implements: Requirement 6.2** - Clear error on new operation
        error.value = null

        // **Implements: Requirement 4.1** - Set currentContent
        currentContent.value = content

        // **Implements: Requirement 4.3** - Use cached chapters if available
        if (chapters.value.has(content.id)) {
            return
        }

        loading.value = true
        try {
            // **Implements: Requirement 4.2** - Fetch and cache chapters
            const chapterList = await getContentApi(getToken).listChapters(content.id)
            chapters.value.set(content.id, chapterList)
        } catch (e) {
            // **Implements: Requirement 6.1** - Set error state on failure
            error.value = e instanceof Error ? e.message : 'Failed to retrieve the chapter list'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Clears the current content selection.
     * **Implements: Requirement 4.1**
     */
    function clearCurrentContent(): void {
        currentContent.value = null
    }

    /**
     * Deletes a content and removes it from cache.
     * **Implements: Requirements 5.1, 5.2, 5.3**
     *
     * @param id - The content ID to delete
     */
    async function deleteContent(id: number): Promise<void> {
        // **Implements: Requirement 6.2** - Clear error on new operation
        error.value = null

        loading.value = true
        try {
            await getContentApi(getToken).delete(id)

            // **Implements: Requirement 5.1** - Remove from cache
            for (const [libraryId, contentList] of contents.value.entries()) {
                const filtered = contentList.filter(c => c.id !== id)
                if (filtered.length !== contentList.length) {
                    contents.value.set(libraryId, filtered)
                }
            }

            // Remove chapters from cache
            chapters.value.delete(id)

            // **Implements: Requirement 5.3** - Clear thumbnail cache for deleted content
            invalidateThumbnailCache(id)

            // **Implements: Requirement 5.2** - Clear currentContent if it was deleted
            if (currentContent.value?.id === id) {
                currentContent.value = null
            }
        } catch (e) {
            // **Implements: Requirement 6.1** - Set error state on failure
            error.value = e instanceof Error ? e.message : 'Failed to delete content'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Invalidates the cache for a specific library or all libraries.
     * Also clears associated thumbnail caches and revokes Object URLs.
     * **Implements: Requirements 2.3, 5.4**
     *
     * @param libraryId - Optional library ID to invalidate. If not provided, clears all caches.
     */
    function invalidateCache(libraryId?: number): void {
        if (libraryId !== undefined) {
            // **Implements: Requirement 5.4** - Clear thumbnails for contents in this library
            const libraryContents = contents.value.get(libraryId) ?? []
            libraryContents.forEach(content => {
                invalidateThumbnailCache(content.id)
            })
            contents.value.delete(libraryId)
        } else {
            // **Implements: Requirement 5.4** - Clear all thumbnails
            invalidateThumbnailCache()
            contents.value.clear()
            chapters.value.clear()
        }
    }

    /**
     * Loads a single thumbnail for a content.
     * If already cached or loading, returns immediately.
     * **Implements: Requirements 1.2, 1.4, 3.1**
     *
     * @param contentId - The content ID to load thumbnail for
     */
    async function loadThumbnail(contentId: number): Promise<void> {
        // **Implements: Requirement 3.1** - Return if already cached
        if (thumbnailUrls.value.has(contentId)) {
            return
        }

        // **Implements: Requirement 1.4** - Prevent duplicate requests
        if (thumbnailLoading.value.has(contentId)) {
            return
        }

        thumbnailLoading.value.add(contentId)

        try {
            // **Implements: Requirement 1.2** - Fetch thumbnail blob
            const blob = await getContentApi(getToken).getThumbnail(contentId)
            // **Implements: Requirement 3.1** - Create Object URL and cache
            const url = URL.createObjectURL(blob)
            thumbnailUrls.value.set(contentId, url)
        } catch (error) {
            // **Implements: Requirement 1.5** - Silent error handling
            console.warn(`Failed to load thumbnail for content ${contentId}:`, error)
        } finally {
            thumbnailLoading.value.delete(contentId)
        }
    }

    /**
     * Preloads thumbnails for multiple contents.
     * Loads all thumbnails concurrently, with error isolation.
     * **Implements: Requirements 1.5, 3.1**
     *
     * @param contentList - Array of contents to preload thumbnails for
     */
    async function preloadThumbnails(contentList: ContentResponse[]): Promise<void> {
        // **Implements: Requirement 1.1** - Filter contents with thumbnails
        const loadPromises = contentList
            .filter(content => content.has_thumbnail)
            .map(content => loadThumbnail(content.id))

        // **Implements: Requirement 1.5** - Use allSettled for error isolation
        await Promise.allSettled(loadPromises)
    }

    /**
     * Invalidates thumbnail cache for a specific content or all contents.
     * Properly releases Object URLs to prevent memory leaks.
     * **Implements: Requirements 3.3, 3.5**
     *
     * @param contentId - Optional content ID to invalidate. If not provided, clears all thumbnail caches.
     */
    function invalidateThumbnailCache(contentId?: number): void {
        if (contentId !== undefined) {
            // **Implements: Requirement 3.3** - Release single Object URL
            const url = thumbnailUrls.value.get(contentId)
            if (url) {
                URL.revokeObjectURL(url)
                thumbnailUrls.value.delete(contentId)
            }
        } else {
            // **Implements: Requirement 3.3** - Release all Object URLs
            for (const url of thumbnailUrls.value.values()) {
                URL.revokeObjectURL(url)
            }
            thumbnailUrls.value.clear()
        }
    }

    return {
        // State
        contents,
        currentContent,
        chapters,
        loading,
        error,
        thumbnailUrls,
        thumbnailLoading,
        // Getters
        contentsByLibrary,
        currentChapters,
        getThumbnailUrl,
        isThumbnailLoading,
        // Actions
        fetchContents,
        searchContents,
        selectContent,
        clearCurrentContent,
        deleteContent,
        invalidateCache,
        loadThumbnail,
        preloadThumbnails,
        invalidateThumbnailCache,
    }
})
