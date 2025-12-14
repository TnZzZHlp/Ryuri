/**
 * Auth Store - Pinia store for authentication state management
 *
 * Manages user authentication state including token, user info, and auth operations.
 *
 * **Implements: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ApiClient } from '@/api/client'
import { createAuthApi, type AuthApi } from '@/api/auth'
import type {
    LoginResponse,
    UserResponse,
    UpdateUserRequest,
    UpdatePasswordRequest,
} from '@/api/types'

const TOKEN_KEY = 'auth_token'

// Lazy-initialized API instance
let apiClient: ApiClient | null = null
let authApi: AuthApi | null = null

function getAuthApi(getToken: () => string | null): AuthApi {
    if (!authApi) {
        apiClient = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken,
        })
        authApi = createAuthApi(apiClient)
    }
    return authApi
}

export const useAuthStore = defineStore('auth', () => {
    // State - restore token from localStorage on initialization
    const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))
    const user = ref<UserResponse | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Getters
    const isAuthenticated = computed(() => !!token.value)

    // Internal helpers
    function setToken(newToken: string) {
        token.value = newToken
        localStorage.setItem(TOKEN_KEY, newToken)
    }

    function clearToken() {
        token.value = null
        user.value = null
        localStorage.removeItem(TOKEN_KEY)
    }

    // Actions
    async function login(username: string, password: string): Promise<LoginResponse> {
        loading.value = true
        error.value = null
        try {
            const response = await getAuthApi(() => token.value).login(username, password)
            // Update token and user atomically
            setToken(response.token)
            user.value = response.user
            return response
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Login failed'
            throw e
        } finally {
            loading.value = false
        }
    }

    function logout() {
        clearToken()
    }

    async function fetchUser(): Promise<UserResponse> {
        loading.value = true
        error.value = null
        try {
            const response = await getAuthApi(() => token.value).getMe()
            user.value = response
            return response
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to retrieve user information'
            throw e
        } finally {
            loading.value = false
        }
    }

    async function updateUser(request: UpdateUserRequest): Promise<UserResponse> {
        loading.value = true
        error.value = null
        try {
            const response = await getAuthApi(() => token.value).updateMe(request)
            user.value = response
            return response
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to update user information'
            throw e
        } finally {
            loading.value = false
        }
    }

    async function updatePassword(request: UpdatePasswordRequest): Promise<void> {
        loading.value = true
        error.value = null
        try {
            await getAuthApi(() => token.value).updatePassword(request)
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to change password'
            throw e
        } finally {
            loading.value = false
        }
    }

    return {
        // State
        token,
        user,
        loading,
        error,
        // Getters
        isAuthenticated,
        // Actions
        login,
        logout,
        fetchUser,
        updateUser,
        updatePassword,
    }
})

// Export TOKEN_KEY for testing purposes
export { TOKEN_KEY }
