/**
 * Frontend API Layer
 *
 * Unified export module for all API functionality.
 * Provides type-safe TypeScript interfaces for backend communication.
 *
 * **Implements: Requirements 1.1**
 */

// ============================================================================
// Type Exports
// ============================================================================

export {
    // Error class
    ApiError,
    // Enums
    ContentType,
    TaskPriority,
    TaskStatus,
    // Auth types
    type LoginRequest,
    type LoginResponse,
    type UserResponse,
    type UpdateUserRequest,
    type UpdatePasswordRequest,
    // Library types
    type Library,
    type LibraryWithStats,
    type CreateLibraryRequest,
    type UpdateLibraryRequest,
    type ScanPath,
    // Content types
    type ContentResponse,
    type Chapter,
    type ChapterTextResponse,
    type UpdateMetadataRequest,
    // Progress types
    type ProgressResponse,
    type ContentProgressResponse,
    type UpdateProgressRequest,
    // Scan queue types
    type TaskProgress,
    type TaskResult,
    type ScanTask,
    type SubmitScanResponse,
    // Bangumi types
    type BangumiSearchResult,
    // ApiKey types
    type ApiKeyResponse,
    type CreateApiKeyRequest,
} from "./types";

// ============================================================================
// Client Exports
// ============================================================================

export {
    ApiClient,
    type ApiClientConfig,
    type RequestOptions,
    buildUrl,
    appendQueryParams,
    buildAuthHeader,
    getDefaultClient,
    resetDefaultClient,
} from "./client";

// ============================================================================
// API Module Exports
// ============================================================================

export { createAuthApi, type AuthApi } from "./auth";
export {
    createLibraryApi,
    type LibraryApi,
    type AddScanPathRequest,
} from "./library";
export { createContentApi, type ContentApi } from "./content";
export { createReaderApi, type ReaderApi } from "./reader";
export { createProgressApi, type ProgressApi } from "./progress";
export { createBangumiApi, type BangumiApi } from "./bangumi";
export { createApiKeyApi, type ApiKeyApi } from "./apikey";

// ============================================================================
// Unified API Interface
// ============================================================================

import { ApiClient, type ApiClientConfig } from "./client";
import { createAuthApi, type AuthApi } from "./auth";
import { createLibraryApi, type LibraryApi } from "./library";
import { createContentApi, type ContentApi } from "./content";
import { createReaderApi, type ReaderApi } from "./reader";
import { createProgressApi, type ProgressApi } from "./progress";
import { createBangumiApi, type BangumiApi } from "./bangumi";
import { createApiKeyApi, type ApiKeyApi } from "./apikey";

/**
 * Unified API interface containing all API modules.
 */
export interface Api {
    /** The underlying API client */
    client: ApiClient;
    /** Authentication API */
    auth: AuthApi;
    /** Library management API */
    library: LibraryApi;
    /** Content management API */
    content: ContentApi;
    /** Reader API for content retrieval */
    reader: ReaderApi;
    /** Progress tracking API */
    progress: ProgressApi;
    /** Bangumi metadata search API */
    bangumi: BangumiApi;
    /** API Key management API */
    apiKey: ApiKeyApi;
}

/**
 * Creates a unified API instance with all modules.
 *
 * @param config - API client configuration
 * @returns A unified Api object containing all API modules
 *
 * @example
 * ```typescript
 * const api = createApi({
 *   baseUrl: 'http://localhost:3000',
 *   getToken: () => localStorage.getItem('token')
 * });
 *
 * // Use individual modules
 * const user = await api.auth.login('username', 'password');
 * const libraries = await api.library.list();
 * ```
 */
export function createApi(config: ApiClientConfig): Api {
    const client = new ApiClient(config);

    return {
        client,
        auth: createAuthApi(client),
        library: createLibraryApi(client),
        content: createContentApi(client),
        reader: createReaderApi(client),
        progress: createProgressApi(client),
        bangumi: createBangumiApi(client),
        apiKey: createApiKeyApi(client),
    };
}

// ============================================================================
// Default API Instance
// ============================================================================

let defaultApi: Api | null = null;

/**
 * Initializes the default API instance.
 *
 * @param config - API client configuration
 * @returns The initialized Api instance
 */
export function initializeApi(config: ApiClientConfig): Api {
    defaultApi = createApi(config);
    return defaultApi;
}

/**
 * Gets the default API instance.
 *
 * @throws Error if the API has not been initialized
 * @returns The default Api instance
 */
export function getApi(): Api {
    if (!defaultApi) {
        throw new Error(
            "API not initialized. Call initializeApi(config) first."
        );
    }
    return defaultApi;
}

/**
 * Resets the default API instance (useful for testing).
 */
export function resetApi(): void {
    defaultApi = null;
}
