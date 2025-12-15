/**
 * Progress API Module
 *
 * Provides functions for reading progress management including
 * content progress retrieval, chapter progress retrieval, and progress updates.
 *
 * **Implements: Requirements 6.1, 6.2, 6.3**
 */

import { ApiClient } from "./client";
import type { ProgressResponse, ContentProgressResponse, ContentResponse } from "./types";

/**
 * Progress API interface.
 */
export interface ProgressApi {
    getContentProgress(contentId: number): Promise<ContentProgressResponse>;
    getRecentProgress(limit?: number): Promise<ContentResponse[]>;
    getChapterProgress(chapterId: number): Promise<ProgressResponse | null>;
    updateChapterProgress(
        chapterId: number,
        position: number,
        percentage?: number
    ): Promise<ProgressResponse>;
}

/**
 * Creates a Progress API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns A ProgressApi implementation
 */
export function createProgressApi(client: ApiClient): ProgressApi {
    return {
        /**
         * Gets the overall reading progress for a content.
         *
         * **Implements: Requirement 6.1**
         *
         * @param contentId - The content ID
         * @returns The aggregated content progress
         */
        async getContentProgress(
            contentId: number
        ): Promise<ContentProgressResponse> {
            return client.get<ContentProgressResponse>(
                `/api/contents/${contentId}/progress`
            );
        },

        /**
         * Gets the most recently read contents.
         *
         * @param limit - Max number of items to return (default 5)
         * @returns List of content items
         */
        async getRecentProgress(limit?: number): Promise<ContentResponse[]> {
            return client.get<ContentResponse[]>('/api/progress/recent', {
                params: { limit },
            });
        },

        /**
         * Gets the reading progress for a specific chapter.
         *
         * **Implements: Requirement 6.2**
         *
         * @param chapterId - The chapter ID
         * @returns The chapter progress or null if not found
         */
        async getChapterProgress(
            chapterId: number
        ): Promise<ProgressResponse | null> {
            return client.get<ProgressResponse | null>(
                `/api/chapters/${chapterId}/progress`
            );
        },

        /**
         * Updates the reading progress for a specific chapter.
         *
         * **Implements: Requirement 6.3**
         *
         * @param chapterId - The chapter ID
         * @param position - Current position within the chapter
         * @param percentage - Optional percentage (0.0 to 100.0)
         * @returns The updated progress
         */
        async updateChapterProgress(
            chapterId: number,
            position: number,
            percentage?: number
        ): Promise<ProgressResponse> {
            const body: { position: number; percentage?: number } = {
                position,
            };
            if (percentage !== undefined) {
                body.percentage = percentage;
            }
            return client.put<ProgressResponse>(
                `/api/chapters/${chapterId}/progress`,
                body
            );
        },
    };
}
