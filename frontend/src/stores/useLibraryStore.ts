/**
 * Library Store - Pinia store for library state management
 *
 * Manages library list caching and CRUD operations.
 *
 * **Implements: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ApiClient } from '@/api/client'
import { createLibraryApi, type LibraryApi } from '@/api/library'
import type {
    Library,
    LibraryWithStats,
    CreateLibraryRequest,
    UpdateLibraryRequest,
    ScanPath,
} from '@/api/types'
import { useAuthStore } from './useAuthStore'

// Lazy-initialized API instance
let apiClient: ApiClient | null = null
let libraryApi: LibraryApi | null = null

function getApiClient(getToken: () => string | null): ApiClient {
    if (!apiClient) {
        apiClient = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken,
        })
    }
    return apiClient
}

function getLibraryApi(getToken: () => string | null): LibraryApi {
    if (!libraryApi) {
        libraryApi = createLibraryApi(getApiClient(getToken))
    }
    return libraryApi
}

export const useLibraryStore = defineStore('library', () => {
    // State
    const libraries = ref<LibraryWithStats[]>([])
    const currentLibrary = ref<LibraryWithStats | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Getters
    const libraryById = computed(() => {
        return (id: number): LibraryWithStats | undefined => {
            return libraries.value.find((lib) => lib.id === id)
        }
    })

    // Internal helper to get token from auth store
    function getToken(): string | null {
        const authStore = useAuthStore()
        return authStore.token
    }

    // Actions

    /**
     * Fetches all libraries and caches them in the store.
     * **Implements: Requirement 3.2**
     */
    async function fetchLibraries(): Promise<void> {
        loading.value = true
        error.value = null
        try {
            const response = await getLibraryApi(getToken).list()
            libraries.value = response
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to retrieve library list.'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Fetches scan paths for a specific library.
     * **Implements: Requirement 3.6**
     */
    async function fetchScanPaths(libraryId: number): Promise<ScanPath[]> {
        try {
            return await getLibraryApi(getToken).listScanPaths(libraryId)
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to retrieve the scan path.'
            throw e
        }
    }

    /**
     * Adds a scan path to a library.
     * **Implements: Requirement 3.7**
     */
    async function addScanPath(libraryId: number, path: string): Promise<ScanPath> {
        try {
            return await getLibraryApi(getToken).addScanPath(libraryId, path)
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to add scan path.'
            throw e
        }
    }

    /**
     * Removes a scan path from a library.
     * **Implements: Requirement 3.8**
     */
    async function removeScanPath(libraryId: number, pathId: number): Promise<void> {
        try {
            await getLibraryApi(getToken).removeScanPath(libraryId, pathId)
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to delete the scan path.'
            throw e
        }
    }

    /**
     * Creates a new library and refreshes the cache.
     * **Implements: Requirement 3.4**
     */
    async function createLibrary(request: CreateLibraryRequest): Promise<Library> {
        loading.value = true
        error.value = null
        try {
            const newLibrary = await getLibraryApi(getToken).create(request)
            // Refresh the libraries list to get updated stats
            await fetchLibraries()
            return newLibrary
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to create library'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Updates an existing library and refreshes the cache.
     * **Implements: Requirement 3.4**
     */
    async function updateLibrary(id: number, request: UpdateLibraryRequest): Promise<Library> {
        loading.value = true
        error.value = null
        try {
            const updatedLibrary = await getLibraryApi(getToken).update(id, request)
            // Refresh the libraries list to get updated stats
            await fetchLibraries()
            return updatedLibrary
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Library update failed'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Deletes a library and removes it from the cache.
     * **Implements: Requirement 3.5**
     */
    async function deleteLibrary(id: number): Promise<void> {
        loading.value = true
        error.value = null
        try {
            await getLibraryApi(getToken).delete(id)
            // Remove from cache
            libraries.value = libraries.value.filter((lib) => lib.id !== id)
            // Clear currentLibrary if it was the deleted one
            if (currentLibrary.value?.id === id) {
                currentLibrary.value = null
            }
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Deletion of database failed'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Selects a library by ID and sets it as the current library.
     * **Implements: Requirement 3.1**
     */
    function selectLibrary(id: number): void {
        const library = libraries.value.find((lib) => lib.id === id)
        if (library) {
            currentLibrary.value = library
        }
    }

    return {
        // State
        libraries,
        currentLibrary,
        loading,
        error,
        // Getters
        libraryById,
        // Actions
        fetchLibraries,
        fetchScanPaths,
        addScanPath,
        removeScanPath,
        createLibrary,
        updateLibrary,
        deleteLibrary,
        selectLibrary,
    }
})
