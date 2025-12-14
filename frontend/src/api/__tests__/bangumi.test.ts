/**
 * Unit tests for Bangumi API module.
 *
 * Tests Bangumi metadata search functionality.
 *
 * **Validates: Requirements 7.1**
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiClient } from "../client";
import { createBangumiApi } from "../bangumi";
import { ApiError } from "../types";
import type { BangumiSearchResult } from "../types";

describe("Bangumi API", () => {
    let mockFetch: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        mockFetch = vi.fn();
        vi.stubGlobal("fetch", mockFetch);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    /**
     * Helper to create a mock successful response.
     */
    function mockSuccessResponse<T>(data: T) {
        return {
            ok: true,
            status: 200,
            headers: new Headers({ "content-length": "100" }),
            json: async () => data,
        };
    }

    /**
     * Helper to create a mock error response.
     */
    function mockErrorResponse(status: number, message: string) {
        return {
            ok: false,
            status,
            statusText: "Error",
            text: async () => JSON.stringify({ error: message }),
        };
    }

    describe("search", () => {
        /**
         * **Validates: Requirement 7.1**
         * WHEN a user searches on Bangumi with a query string, THE Bangumi_API SHALL
         * return matching BangumiSearchResult objects with metadata
         */

        it("sends search request with query parameter", async () => {
            const mockResults: BangumiSearchResult[] = [
                {
                    id: 12345,
                    name: "Test Anime",
                    name_cn: "测试动画",
                    summary: "A test anime summary",
                    image: "https://example.com/image.jpg",
                },
                {
                    id: 67890,
                    name: "Another Anime",
                    name_cn: null,
                    summary: null,
                    image: null,
                },
            ];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResults));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const bangumiApi = createBangumiApi(client);

            const result = await bangumiApi.search("test anime");

            expect(result).toEqual(mockResults);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe(
                "http://localhost:3000/api/bangumi/search?q=test+anime"
            );
            expect(init.method).toBe("GET");

            const headers = init.headers as Record<string, string>;
            expect(headers["Authorization"]).toBe("Bearer valid.jwt.token");
        });

        it("returns empty array when no results found", async () => {
            const mockResults: BangumiSearchResult[] = [];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResults));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const bangumiApi = createBangumiApi(client);

            const result = await bangumiApi.search("nonexistent");

            expect(result).toEqual([]);
            expect(result).toHaveLength(0);
        });

        it("handles special characters in query", async () => {
            const mockResults: BangumiSearchResult[] = [];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResults));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const bangumiApi = createBangumiApi(client);

            await bangumiApi.search("进击的巨人");

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toContain("q=");
            expect(url).toContain(encodeURIComponent("进击的巨人"));
        });

        it("throws ApiError when not authenticated", async () => {
            mockFetch.mockResolvedValue(mockErrorResponse(401, "Unauthorized"));

            const client = new ApiClient({ baseUrl: "http://localhost:3000" });
            const bangumiApi = createBangumiApi(client);

            await expect(bangumiApi.search("test")).rejects.toThrow(ApiError);

            try {
                await bangumiApi.search("test");
            } catch (error) {
                expect(error).toBeInstanceOf(ApiError);
                expect((error as ApiError).isUnauthorized()).toBe(true);
            }
        });

        it("throws ApiError on server error", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(500, "Internal server error")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const bangumiApi = createBangumiApi(client);

            await expect(bangumiApi.search("test")).rejects.toThrow(ApiError);

            try {
                await bangumiApi.search("test");
            } catch (error) {
                expect(error).toBeInstanceOf(ApiError);
                expect((error as ApiError).isServerError()).toBe(true);
            }
        });
    });
});
