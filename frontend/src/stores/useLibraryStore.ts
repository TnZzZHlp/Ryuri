/**
 * Library Store - Pinia store for library state management
 *
 * Manages library list caching, current library selection, and CRUD operations.
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
} from '@/api/types'
import { useAuthStore } from './useAuthStore'

// Lazy-initialized API instance
let apiClient: ApiClient | null = null
let libraryApi: LibraryApi | null = null

function getLibraryApi(getToken: () => string | null): LibraryApi {
  if (!libraryApi) {
    apiClient = new ApiClient({
      baseUrl: import.meta.env.VITE_API_BASE_URL || '',
      getToken,
    })
    libraryApi = createLibraryApi(apiClient)
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
      error.value = e instanceof Error ? e.message : '获取库列表失败'
      throw e
    } finally {
      loading.value = false
    }
  }

  /**
   * Selects a library by ID and sets it as the current library.
   * **Implements: Requirement 3.3**
   */
  function selectLibrary(id: number): void {
    const library = libraries.value.find((lib) => lib.id === id)
    if (library) {
      currentLibrary.value = library
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
      error.value = e instanceof Error ? e.message : '创建库失败'
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
      // Update currentLibrary if it was the one updated
      if (currentLibrary.value?.id === id) {
        const updated = libraries.value.find((lib) => lib.id === id)
        if (updated) {
          currentLibrary.value = updated
        }
      }
      return updatedLibrary
    } catch (e) {
      error.value = e instanceof Error ? e.message : '更新库失败'
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
      error.value = e instanceof Error ? e.message : '删除库失败'
      throw e
    } finally {
      loading.value = false
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
    selectLibrary,
    createLibrary,
    updateLibrary,
    deleteLibrary,
  }
})
