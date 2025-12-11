/**
 * @deprecated This composable is deprecated and will be removed in a future version.
 * Please use `useAuthStore` from `@/stores/useAuthStore` instead.
 *
 * Migration guide:
 * ```typescript
 * // Before (deprecated)
 * import { useAuth } from '@/composables/useAuth'
 * const { isAuthenticated, user, login, logout } = useAuth()
 *
 * // After (recommended)
 * import { useAuthStore } from '@/stores/useAuthStore'
 * const authStore = useAuthStore()
 * // Access: authStore.isAuthenticated, authStore.user, authStore.login(), authStore.logout()
 * ```
 *
 * The Pinia store provides:
 * - Vue DevTools integration for state debugging
 * - Consistent state management patterns
 * - Better TypeScript support
 */

import { ref, computed } from "vue";
import { ApiClient } from "@/api/client";
import { createAuthApi, type AuthApi } from "@/api/auth";
import type {
    LoginResponse,
    UserResponse,
    UpdateUserRequest,
    UpdatePasswordRequest,
} from "@/api/types";

const TOKEN_KEY = "auth_token";

// 全局状态（在模块级别，所有 useAuth() 调用共享）
const token = ref<string | null>(localStorage.getItem(TOKEN_KEY));
const user = ref<UserResponse | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

// 延迟初始化的 API 实例
let apiClient: ApiClient | null = null;
let authApi: AuthApi | null = null;

function getAuthApi(): AuthApi {
    if (!authApi) {
        apiClient = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || "",
            getToken: () => token.value,
        });
        authApi = createAuthApi(apiClient);
    }
    return authApi;
}

/**
 * @deprecated Use `useAuthStore` from `@/stores/useAuthStore` instead.
 */
export function useAuth() {
    if (import.meta.env.DEV) {
        console.warn(
            '[DEPRECATED] useAuth() is deprecated. Please migrate to useAuthStore() from "@/stores/useAuthStore".'
        );
    }

    const isAuthenticated = computed(() => !!token.value);

    function setToken(newToken: string) {
        token.value = newToken;
        localStorage.setItem(TOKEN_KEY, newToken);
    }

    function clearToken() {
        token.value = null;
        user.value = null;
        localStorage.removeItem(TOKEN_KEY);
    }

    function getToken() {
        return token.value;
    }

    async function login(
        username: string,
        password: string
    ): Promise<LoginResponse> {
        loading.value = true;
        try {
            const response = await getAuthApi().login(username, password);
            setToken(response.token);
            user.value = response.user;
            return response;
        } catch (e) {
            throw e;
        } finally {
            loading.value = false;
        }
    }

    function logout() {
        clearToken();
    }

    async function fetchUser(): Promise<UserResponse> {
        loading.value = true;
        error.value = null;
        try {
            const response = await getAuthApi().getMe();
            user.value = response;
            return response;
        } catch (e) {
            error.value = e instanceof Error ? e.message : "获取用户信息失败";
            throw e;
        } finally {
            loading.value = false;
        }
    }

    async function updateUser(
        request: UpdateUserRequest
    ): Promise<UserResponse> {
        loading.value = true;
        error.value = null;
        try {
            const response = await getAuthApi().updateMe(request);
            user.value = response;
            return response;
        } catch (e) {
            error.value = e instanceof Error ? e.message : "更新用户信息失败";
            throw e;
        } finally {
            loading.value = false;
        }
    }

    async function updatePassword(
        request: UpdatePasswordRequest
    ): Promise<void> {
        loading.value = true;
        error.value = null;
        try {
            await getAuthApi().updatePassword(request);
        } catch (e) {
            error.value = e instanceof Error ? e.message : "修改密码失败";
            throw e;
        } finally {
            loading.value = false;
        }
    }

    return {
        // 状态
        isAuthenticated,
        token,
        user,
        loading,
        error,
        // 方法
        setToken,
        clearToken,
        getToken,
        login,
        logout,
        fetchUser,
        updateUser,
        updatePassword,
    };
}
