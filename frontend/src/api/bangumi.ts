/**
 * Bangumi API Module
 *
 * Provides functions for searching metadata on Bangumi.tv.
 *
 * **Implements: Requirements 7.1**
 */

import { ApiClient } from "./client";

/**
 * Bangumi API interface.
 */
export interface BangumiApi {
    searchSubjects(options: Record<string, any>): Promise<any[]>;
}

/**
 * Creates a Bangumi API instance using the provided API client.
 *
 * @param _client - The API client (unused in new implementation but kept for signature compatibility)
 * @returns A BangumiApi implementation
 */
export function createBangumiApi(_client: ApiClient): BangumiApi {
    return {
        /**
         * Searches for subjects using Bangumi v0 API.
         *
         * @param options - Search options
         * @returns Array of raw subject data
         */
        async searchSubjects(options: Record<string, any>): Promise<any[]> {
            const response = await fetch("https://api.bgm.tv/v0/search/subjects", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(options),
            });

            if (!response.ok) {
                throw new Error(`Bangumi API error: ${response.status}`);
            }

            const result = await response.json();
            return result.data;
        },
    };
}
