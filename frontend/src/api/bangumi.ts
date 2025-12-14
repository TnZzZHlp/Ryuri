/**
 * Bangumi API Module
 *
 * Provides functions for searching metadata on Bangumi.tv.
 *
 * **Implements: Requirements 7.1**
 */

import { ApiClient } from "./client";
import type { BangumiSearchResult } from "./types";

/**
 * Bangumi API interface.
 */
export interface BangumiApi {
    search(query: string): Promise<BangumiSearchResult[]>;
}

/**
 * Creates a Bangumi API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns A BangumiApi implementation
 */
export function createBangumiApi(client: ApiClient): BangumiApi {
    return {
        /**
         * Searches for content on Bangumi.tv by keyword.
         *
         * **Implements: Requirement 7.1**
         *
         * @param query - The search query string
         * @returns Array of matching BangumiSearchResult objects
         */
        async search(query: string): Promise<BangumiSearchResult[]> {
            return client.get<BangumiSearchResult[]>("/api/bangumi/search", {
                params: { q: query },
            });
        },
    };
}
