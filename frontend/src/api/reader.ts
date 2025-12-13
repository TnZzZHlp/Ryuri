/**
 * Reader API Module
 *
 * Provides functions for reading content including comic page URLs
 * and novel chapter text retrieval.
 *
 * **Implements: Requirements 5.1, 5.2**
 */

import { ApiClient, buildUrl } from "./client";
import type { ChapterTextResponse } from "./types";

/**
 * Reader API interface.
 */
export interface ReaderApi {
    getPageUrl(contentId: number, chapter: number, page: number): string;
    getChapterText(
        contentId: number,
        chapter: number
    ): Promise<ChapterTextResponse>;
    getPageImage(contentId: number, chapter: number, page: number): Promise<Blob>;
}

/**
 * Creates a Reader API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns A ReaderApi implementation
 */
export function createReaderApi(client: ApiClient): ReaderApi {
    return {
        /**
         * Gets the URL for a comic page image.
         *
         * This function returns a URL string directly without making an HTTP call.
         * The URL can be used in an <img> tag or for direct image fetching.
         *
         * **Implements: Requirement 5.1**
         *
         * @param contentId - The content ID
         * @param chapter - The chapter index (0-based)
         * @param page - The page index (0-based)
         * @returns The URL string for the page image
         */
        getPageUrl(contentId: number, chapter: number, page: number): string {
            const path = `/api/contents/${contentId}/chapters/${chapter}/pages/${page}`;
            return buildUrl(client.baseUrl, path);
        },

        /**
         * Gets the text content of a novel chapter.
         *
         * **Implements: Requirement 5.2**
         *
         * @param contentId - The content ID
         * @param chapter - The chapter index (0-based)
         * @returns The chapter text content
         */
        async getChapterText(
            contentId: number,
            chapter: number
        ): Promise<ChapterTextResponse> {
            return client.get<ChapterTextResponse>(
                `/api/contents/${contentId}/chapters/${chapter}/text`
            );
        },

        async getPageImage(contentId: number, chapter: number, page: number): Promise<Blob> {
            return client.getBlob(`/api/contents/${contentId}/chapters/${chapter}/pages/${page}`);
        }
    };
}
