/**
 * Unit tests for Progress API module.
 *
 * Tests progress management API functions including content progress retrieval,
 * chapter progress retrieval, and progress updates.
 *
 * **Validates: Requirements 6.1, 6.2, 6.3**
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiClient } from "../client";
import { createProgressApi } from "../progress";
import { ApiError } from "../types";
import type { ProgressResponse, ContentProgressResponse } from "../types";

describe("Progress API", () => {
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

    describe("getContentProgress", () => {
        /**
         * **Validates: Requirement 6.1**
         * WHEN a user requests content progress by content ID, THE Progress_API
         * SHALL return the overall Progress for that content
         */

        it("retrieves overall progress for a content", async () => {
            const mockProgress: ContentProgressResponse = {
                content_id: 1,
                total_chapters: 10,
                completed_chapters: 5,
                current_chapter_id: 6,
                overall_percentage: 50.0,
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockProgress));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            const result = await progressApi.getContentProgress(1);

            expect(result).toEqual(mockProgress);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/contents/1/progress");
            expect(init.method).toBe("GET");
        });

        it("throws ApiError when content not found", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(404, "Content not found")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            await expect(progressApi.getContentProgress(999)).rejects.toThrow(
                ApiError
            );
        });
    });

    describe("getChapterProgress", () => {
        /**
         * **Validates: Requirement 6.2**
         * WHEN a user requests chapter progress by chapter ID, THE Progress_API
         * SHALL return the Progress for that chapter
         */

        it("retrieves progress for a chapter", async () => {
            const mockProgress: ProgressResponse = {
                chapter_id: 5,
                position: 42,
                percentage: 75.5,
                updated_at: "2024-01-15T10:30:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockProgress));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            const result = await progressApi.getChapterProgress(5);

            expect(result).toEqual(mockProgress);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/chapters/5/progress");
            expect(init.method).toBe("GET");
        });

        it("returns null when no progress exists for chapter", async () => {
            mockFetch.mockResolvedValue(mockSuccessResponse(null));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            const result = await progressApi.getChapterProgress(5);

            expect(result).toBeNull();
        });
    });

    describe("updateChapterProgress", () => {
        /**
         * **Validates: Requirement 6.3**
         * WHEN a user updates chapter progress, THE Progress_API SHALL send
         * the update request and return the updated Progress object
         */

        it("updates chapter progress with position only", async () => {
            const mockProgress: ProgressResponse = {
                chapter_id: 5,
                position: 100,
                percentage: 0.0,
                updated_at: "2024-01-15T11:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockProgress));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            const result = await progressApi.updateChapterProgress(5, 100);

            expect(result).toEqual(mockProgress);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/chapters/5/progress");
            expect(init.method).toBe("PUT");
            expect(JSON.parse(init.body as string)).toEqual({ position: 100 });
        });

        it("updates chapter progress with position and percentage", async () => {
            const mockProgress: ProgressResponse = {
                chapter_id: 5,
                position: 100,
                percentage: 85.5,
                updated_at: "2024-01-15T11:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockProgress));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            const result = await progressApi.updateChapterProgress(
                5,
                100,
                85.5
            );

            expect(result).toEqual(mockProgress);

            const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(JSON.parse(init.body as string)).toEqual({
                position: 100,
                percentage: 85.5,
            });
        });

        it("throws ApiError when chapter not found", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(404, "Chapter not found")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const progressApi = createProgressApi(client);

            await expect(
                progressApi.updateChapterProgress(999, 50)
            ).rejects.toThrow(ApiError);
        });
    });
});
