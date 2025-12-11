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

    // Internal helper to get token from auth store
    function getToken(): string | null {
        const authStore = useAuthStore()
        return authStore.token
    }

    // Actions

    /**
     * Fetches contents for a library and caches them.
     * **Implements: Requirements 2.1, 2.2, 2.3**
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
            return response
        } catch (e) {
            // **Implements: Requirement 6.1** - Set error state on failure
            error.value = e instanceof Error ? e.message : '获取内容列表失败'
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
            error.value = e instanceof Error ? e.message : '搜索内容失败'
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
            error.value = e instanceof Error ? e.message : '获取章节列表失败'
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
     * **Implements: Requirements 5.1, 5.2**
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

            // **Implements: Requirement 5.2** - Clear currentContent if it was deleted
            if (currentContent.value?.id === id) {
                currentContent.value = null
            }
        } catch (e) {
            // **Implements: Requirement 6.1** - Set error state on failure
            error.value = e instanceof Error ? e.message : '删除内容失败'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Invalidates the cache for a specific library or all libraries.
     * **Implements: Requirement 2.3**
     *
     * @param libraryId - Optional library ID to invalidate. If not provided, clears all caches.
     */
    function invalidateCache(libraryId?: number): void {
        if (libraryId !== undefined) {
            contents.value.delete(libraryId)
        } else {
            contents.value.clear()
            chapters.value.clear()
        }
    }

    return {
        // State
        contents,
        currentContent,
        chapters,
        loading,
        error,
        // Getters
        contentsByLibrary,
        currentChapters,
        // Actions
        fetchContents,
        searchContents,
        selectContent,
        clearCurrentContent,
        deleteContent,
        invalidateCache,
    }
})
