/**
 * Content API Module
 *
 * Provides functions for content management including listing, searching,
 * CRUD operations, chapter management, and scan triggering.
 *
 * **Implements: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7**
 */

import { ApiClient } from "./client";
import type {
    ContentResponse,
    Chapter,
    SubmitScanResponse,
    UpdateMetadataRequest,
} from "./types";

/**
 * Content API interface.
 */
export interface ContentApi {
    list(libraryId: number): Promise<ContentResponse[]>;
    search(libraryId: number, query: string): Promise<ContentResponse[]>;
    get(id: number): Promise<ContentResponse>;
    delete(id: number): Promise<void>;
    updateMetadata(
        id: number,
        metadata: unknown | null
    ): Promise<ContentResponse>;
    listChapters(contentId: number): Promise<Chapter[]>;
    triggerScan(libraryId: number): Promise<SubmitScanResponse>;
}

/**
 * Creates a Content API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns A ContentApi implementation
 */
export function createContentApi(client: ApiClient): ContentApi {
    return {
        /**
         * Lists all contents in a library.
         *
         * **Implements: Requirement 4.1**
         *
         * @param libraryId - The library ID
         * @returns Array of content items
         */
        async list(libraryId: number): Promise<ContentResponse[]> {
            return client.get<ContentResponse[]>(
                `/api/libraries/${libraryId}/contents`
            );
        },

        /**
         * Searches contents by title within a library.
         *
         * **Implements: Requirement 4.3**
         *
         * @param libraryId - The library ID
         * @param query - The search query string
         * @returns Array of matching content items
         */
        async search(
            libraryId: number,
            query: string
        ): Promise<ContentResponse[]> {
            return client.get<ContentResponse[]>(
                `/api/libraries/${libraryId}/search`,
                {
                    params: { q: query },
                }
            );
        },

        /**
         * Gets a specific content by ID.
         *
         * **Implements: Requirement 4.4**
         *
         * @param id - The content ID
         * @returns The content details
         */
        async get(id: number): Promise<ContentResponse> {
            return client.get<ContentResponse>(`/api/contents/${id}`);
        },

        /**
         * Deletes a content and all associated chapters.
         *
         * **Implements: Requirement 4.5**
         *
         * @param id - The content ID
         */
        async delete(id: number): Promise<void> {
            await client.delete<void>(`/api/contents/${id}`);
        },

        /**
         * Updates the metadata for a content item.
         *
         * **Implements: Requirement 4.6**
         *
         * @param id - The content ID
         * @param metadata - The metadata to set (or null to clear)
         * @returns The updated content
         */
        async updateMetadata(
            id: number,
            metadata: unknown | null
        ): Promise<ContentResponse> {
            const request: UpdateMetadataRequest = { metadata };
            return client.put<ContentResponse>(
                `/api/contents/${id}/metadata`,
                request
            );
        },

        /**
         * Lists all chapters for a content.
         *
         * **Implements: Requirement 4.7**
         *
         * @param contentId - The content ID
         * @returns Array of chapters
         */
        async listChapters(contentId: number): Promise<Chapter[]> {
            return client.get<Chapter[]>(`/api/contents/${contentId}/chapters`);
        },

        /**
         * Triggers a library scan with high priority.
         *
         * **Implements: Requirement 4.2**
         *
         * @param libraryId - The library ID to scan
         * @returns The scan task response with task ID
         */
        async triggerScan(libraryId: number): Promise<SubmitScanResponse> {
            return client.post<SubmitScanResponse>(
                `/api/libraries/${libraryId}/scan`
            );
        },
    };
}
