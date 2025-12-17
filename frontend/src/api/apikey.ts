/**
 * API Key Module
 *
 * Provides functions for managing user API keys.
 */

import { ApiClient } from "./client";
import type { ApiKeyResponse, CreateApiKeyRequest } from "./types";

/**
 * API Key API interface.
 */
export interface ApiKeyApi {
    list(): Promise<ApiKeyResponse[]>;
    create(request: CreateApiKeyRequest): Promise<ApiKeyResponse>;
    delete(id: number): Promise<void>;
}

/**
 * Creates an API Key API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns An ApiKeyApi implementation
 */
export function createApiKeyApi(client: ApiClient): ApiKeyApi {
    return {
        /**
         * Get all API keys for the current user.
         */
        async list(): Promise<ApiKeyResponse[]> {
            return client.get<ApiKeyResponse[]>("/api/api-keys");
        },

        /**
         * Create a new API key.
         */
        async create(request: CreateApiKeyRequest): Promise<ApiKeyResponse> {
            return client.post<ApiKeyResponse>("/api/api-keys", request);
        },

        /**
         * Delete an API key by ID.
         */
        async delete(id: number): Promise<void> {
            return client.delete<void>(`/api/api-keys/${id}`);
        },
    };
}