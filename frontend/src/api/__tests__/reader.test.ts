/**
 * Unit tests for Reader API module.
 *
 * Tests reader API functions including page URL generation
 * and chapter text retrieval.
 *
 * **Validates: Requirements 5.1, 5.2**
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiClient } from "../client";
import { createReaderApi } from "../reader";
import { ApiError } from "../types";
import type { ChapterTextResponse } from "../types";

describe("Reader API", () => {
    let mockFetch: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        mockFetch = vi.fn();
        vi.stubGlobal("fetch", mockFetch);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    function mockSuccessResponse<T>(data: T) {
        return {
            ok: true,
            status: 200,
            headers: new Headers({ "content-length": "100" }),
            json: async () => data,
        };
    }

    function mockErrorResponse(status: number, message: string) {
        return {
            ok: false,
            status,
            statusText: "Error",
            text: async () => JSON.stringify({ error: message }),
        };
    }

    describe("getPageUrl", () => {
        /**
         * **Validates: Requirement 5.1**
         * WHEN a user requests a comic page by chapter ID and page number,
         * THE Reader_API SHALL return the page image URL
         */

        it("generates correct URL for a comic page", () => {
            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            const url = readerApi.getPageUrl(1, 0, 0);

            expect(url).toBe(
                "http://localhost:3000/api/contents/1/chapters/0/pages/0"
            );
        });

        it("generates correct URL with different content, chapter, and page indices", () => {
            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            const url = readerApi.getPageUrl(42, 5, 10);

            expect(url).toBe(
                "http://localhost:3000/api/contents/42/chapters/5/pages/10"
            );
        });

        it("handles base URL with trailing slash", () => {
            const client = new ApiClient({
                baseUrl: "http://localhost:3000/",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            const url = readerApi.getPageUrl(1, 0, 0);

            expect(url).toBe(
                "http://localhost:3000/api/contents/1/chapters/0/pages/0"
            );
        });

        it("does not make any HTTP calls", () => {
            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            readerApi.getPageUrl(1, 0, 0);

            expect(mockFetch).not.toHaveBeenCalled();
        });
    });

    describe("getChapterText", () => {
        /**
         * **Validates: Requirement 5.2**
         * WHEN a user requests novel chapter text by chapter ID,
         * THE Reader_API SHALL return the chapter text content as a string
         */

        it("retrieves chapter text content", async () => {
            const mockResponse: ChapterTextResponse = {
                text: "This is the content of chapter 1. It contains the story text.",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResponse));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            const result = await readerApi.getChapterText(1, 0);

            expect(result).toEqual(mockResponse);
            expect(result.text).toBe(
                "This is the content of chapter 1. It contains the story text."
            );

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe(
                "http://localhost:3000/api/contents/1/chapters/0/text"
            );
            expect(init.method).toBe("GET");
        });

        it("retrieves chapter text for different content and chapter indices", async () => {
            const mockResponse: ChapterTextResponse = {
                text: "Chapter 5 content here.",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResponse));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            const result = await readerApi.getChapterText(42, 5);

            expect(result.text).toBe("Chapter 5 content here.");

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toBe(
                "http://localhost:3000/api/contents/42/chapters/5/text"
            );
        });

        it("throws ApiError when chapter not found", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(404, "Chapter not found")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            await expect(readerApi.getChapterText(999, 0)).rejects.toThrow(
                ApiError
            );
        });

        it("throws ApiError when content not found", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(404, "Content not found")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const readerApi = createReaderApi(client);

            await expect(readerApi.getChapterText(999, 0)).rejects.toThrow(
                ApiError
            );
        });

        it("includes authorization header in request", async () => {
            const mockResponse: ChapterTextResponse = {
                text: "Protected content.",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResponse));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "my-auth-token",
            });
            const readerApi = createReaderApi(client);

            await readerApi.getChapterText(1, 0);

            const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
            const headers = init.headers as Record<string, string>;
            expect(headers["Authorization"]).toBe("Bearer my-auth-token");
        });
    });
});
