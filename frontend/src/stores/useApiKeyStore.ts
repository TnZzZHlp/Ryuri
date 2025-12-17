/**
 * API Key Store - Pinia store for API key state management
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiClient } from '@/api/client'
import { createApiKeyApi, type ApiKeyApi } from '@/api/apikey'
import type { ApiKeyResponse, CreateApiKeyRequest } from '@/api/types'
import { useAuthStore } from './useAuthStore'

// Lazy-initialized API instance
let apiClient: ApiClient | null = null
let apiKeyApi: ApiKeyApi | null = null

function getApiClient(getToken: () => string | null): ApiClient {
    if (!apiClient) {
        apiClient = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken,
        })
    }
    return apiClient
}

function getApiKeyApi(getToken: () => string | null): ApiKeyApi {
    if (!apiKeyApi) {
        apiKeyApi = createApiKeyApi(getApiClient(getToken))
    }
    return apiKeyApi
}

export const useApiKeyStore = defineStore('apiKey', () => {
    // State
    const apiKeys = ref<ApiKeyResponse[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Internal helper to get token from auth store
    function getToken(): string | null {
        const authStore = useAuthStore()
        return authStore.token
    }

    // Actions

    /**
     * Fetches all API keys and caches them in the store.
     */
    async function fetchApiKeys(): Promise<void> {
        loading.value = true
        error.value = null
        try {
            const response = await getApiKeyApi(getToken).list()
            apiKeys.value = response
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to retrieve API keys.'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Creates a new API key and refreshes the cache.
     */
    async function createApiKey(request: CreateApiKeyRequest): Promise<ApiKeyResponse> {
        loading.value = true
        error.value = null
        try {
            const newKey = await getApiKeyApi(getToken).create(request)
            // Add to cache directly or refresh list
            apiKeys.value.push(newKey)
            return newKey
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to create API key'
            throw e
        } finally {
            loading.value = false
        }
    }

    /**
     * Deletes an API key and removes it from the cache.
     */
    async function deleteApiKey(id: number): Promise<void> {
        loading.value = true
        error.value = null
        try {
            await getApiKeyApi(getToken).delete(id)
            // Remove from cache
            apiKeys.value = apiKeys.value.filter((key) => key.id !== id)
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to delete API key'
            throw e
        } finally {
            loading.value = false
        }
    }

    return {
        // State
        apiKeys,
        loading,
        error,
        // Actions
        fetchApiKeys,
        createApiKey,
        deleteApiKey,
    }
})
