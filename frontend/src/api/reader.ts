/**
 * Reader API Module
 *
 * Provides functions for reading content including comic page URLs,
 * EPUB spine/resource access.
 */

import { ApiClient, buildUrl } from "./client";
import type { EpubSpineItem } from "./types";

/**
 * Reader API interface.
 */
export interface ReaderApi {
    getPageUrl(contentId: number, chapter: number, page: number): string;
    getPageImage(contentId: number, chapterId: number, page: number): string;
    getEpubSpine(
        contentId: number,
        chapterId: number,
    ): Promise<EpubSpineItem[]>;
    getEpubResourceUrl(
        contentId: number,
        chapterId: number,
        resourcePath: string,
    ): string;
}

/**
 * Creates a Reader API instance using the provided API client.
 */
export function createReaderApi(client: ApiClient): ReaderApi {
    return {
        getPageUrl(contentId: number, chapter: number, page: number): string {
            const path = `/api/contents/${contentId}/chapters/${chapter}/pages/${page}`;
            return buildUrl(client.baseUrl, path);
        },

        getPageImage(
            contentId: number,
            chapterId: number,
            page: number,
        ): string {
            return client.buildAuthenticatedUrl(
                `/api/contents/${contentId}/chapters/${chapterId}/pages/${page}`,
            );
        },

        async getEpubSpine(
            contentId: number,
            chapterId: number,
        ): Promise<EpubSpineItem[]> {
            return client.get<EpubSpineItem[]>(
                `/api/contents/${contentId}/chapters/${chapterId}/epub-spine`,
            );
        },

        getEpubResourceUrl(
            contentId: number,
            chapterId: number,
            resourcePath: string,
        ): string {
            return client.buildAuthenticatedUrl(
                `/api/contents/${contentId}/chapters/${chapterId}/epub/${resourcePath}`,
            );
        },
    };
}
