/**
 * Unit tests for Content API module.
 *
 * Tests content management API functions including listing, searching,
 * CRUD operations, chapter management, and scan triggering.
 *
 * **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7**
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiClient } from "../client";
import { createContentApi } from "../content";
import { ApiError, ContentType } from "../types";
import type {
    ContentResponse,
    Chapter,
    SubmitScanResponse,
    ScanTask,
} from "../types";

describe("Content API", () => {
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

    describe("list", () => {
        /**
         * **Validates: Requirement 4.1**
         * WHEN a user requests contents for a library, THE Content_API SHALL
         * return all Content objects in that library
         */

        it("retrieves all contents in a library", async () => {
            const mockContents: ContentResponse[] = [
                {
                    id: 1,
                    library_id: 1,
                    content_type: ContentType.Comic,
                    title: "One Piece",
                    chapter_count: 1000,
                    has_thumbnail: true,
                    metadata: { author: "Oda" },
                    created_at: "2024-01-01T00:00:00Z",
                },
                {
                    id: 2,
                    library_id: 1,
                    content_type: ContentType.Novel,
                    title: "Mushoku Tensei",
                    chapter_count: 25,
                    has_thumbnail: false,
                    metadata: null,
                    created_at: "2024-01-02T00:00:00Z",
                },
            ];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockContents));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.list(1);

            expect(result).toEqual(mockContents);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries/1/contents");
            expect(init.method).toBe("GET");
        });
    });

    describe("search", () => {
        /**
         * **Validates: Requirement 4.3**
         * WHEN a user searches for content with a query string, THE Content_API
         * SHALL return matching Content objects
         */

        it("searches contents by query string", async () => {
            const mockContents: ContentResponse[] = [
                {
                    id: 1,
                    library_id: 1,
                    content_type: ContentType.Comic,
                    title: "One Piece",
                    chapter_count: 1000,
                    has_thumbnail: true,
                    metadata: null,
                    created_at: "2024-01-01T00:00:00Z",
                },
            ];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockContents));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.search(1, "piece");

            expect(result).toEqual(mockContents);

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toBe(
                "http://localhost:3000/api/libraries/1/search?q=piece"
            );
        });

        it("returns empty array when no matches found", async () => {
            mockFetch.mockResolvedValue(mockSuccessResponse([]));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.search(1, "nonexistent");

            expect(result).toEqual([]);
        });
    });

    describe("get", () => {
        /**
         * **Validates: Requirement 4.4**
         * WHEN a user requests a specific content by ID, THE Content_API SHALL
         * return the Content details
         */

        it("retrieves a content by ID", async () => {
            const mockContent: ContentResponse = {
                id: 1,
                library_id: 1,
                content_type: ContentType.Comic,
                title: "One Piece",
                chapter_count: 1000,
                has_thumbnail: true,
                metadata: { author: "Oda", year: 1997 },
                created_at: "2024-01-01T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockContent));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.get(1);

            expect(result).toEqual(mockContent);

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toBe("http://localhost:3000/api/contents/1");
        });

        it("throws ApiError when content not found", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(404, "Content not found")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            await expect(contentApi.get(999)).rejects.toThrow(ApiError);
        });
    });

    describe("delete", () => {
        /**
         * **Validates: Requirement 4.5**
         * WHEN a user deletes a content, THE Content_API SHALL send
         * the deletion request and return success status
         */

        it("deletes a content", async () => {
            mockFetch.mockResolvedValue(mockSuccessResponse(null));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            await contentApi.delete(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/contents/1");
            expect(init.method).toBe("DELETE");
        });
    });

    describe("updateMetadata", () => {
        /**
         * **Validates: Requirement 4.6**
         * WHEN a user updates content metadata, THE Content_API SHALL send
         * the update request and return the updated Content object
         */

        it("updates content metadata with new values", async () => {
            const metadata = { author: "Oda", year: 1997, genre: "Adventure" };
            const mockContent: ContentResponse = {
                id: 1,
                library_id: 1,
                content_type: ContentType.Comic,
                title: "One Piece",
                chapter_count: 1000,
                has_thumbnail: true,
                metadata,
                created_at: "2024-01-01T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockContent));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.updateMetadata(1, metadata);

            expect(result).toEqual(mockContent);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/contents/1/metadata");
            expect(init.method).toBe("PUT");
            expect(JSON.parse(init.body as string)).toEqual({ metadata });
        });

        it("clears metadata when null is passed", async () => {
            const mockContent: ContentResponse = {
                id: 1,
                library_id: 1,
                content_type: ContentType.Comic,
                title: "One Piece",
                chapter_count: 1000,
                has_thumbnail: true,
                metadata: null,
                created_at: "2024-01-01T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockContent));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.updateMetadata(1, null);

            expect(result.metadata).toBeNull();

            const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(JSON.parse(init.body as string)).toEqual({ metadata: null });
        });
    });

    describe("listChapters", () => {
        /**
         * **Validates: Requirement 4.7**
         * WHEN a user requests chapters for a content, THE Content_API SHALL
         * return all Chapter objects for that content
         */

        it("retrieves all chapters for a content", async () => {
            const mockChapters: Chapter[] = [
                {
                    id: 1,
                    content_id: 1,
                    title: "Chapter 1",
                    file_path: "/path/to/chapter1.cbz",
                    sort_order: 0,
                    page_count: 20
                },
                {
                    id: 2,
                    content_id: 1,
                    title: "Chapter 2",
                    file_path: "/path/to/chapter2.cbz",
                    sort_order: 1,
                    page_count: 22
                },
            ];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockChapters));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.listChapters(1);

            expect(result).toEqual(mockChapters);

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toBe("http://localhost:3000/api/contents/1/chapters");
        });
    });

    describe("triggerScan", () => {
        /**
         * **Validates: Requirement 4.2**
         * WHEN a user triggers a library scan, THE Content_API SHALL send
         * the scan request and return the scan results
         */

        it("triggers a library scan and returns task info", async () => {
            const mockTask: ScanTask = {
                id: "task-123",
                library_id: 1,
                priority: "High",
                status: "Pending",
                created_at: "2024-01-01T00:00:00Z",
                started_at: null,
                completed_at: null,
                progress: null,
                result: null,
                error: null,
            };
            const mockResponse: SubmitScanResponse = {
                task_id: "task-123",
                task: mockTask,
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResponse));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const contentApi = createContentApi(client);

            const result = await contentApi.triggerScan(1);

            expect(result.task_id).toBe("task-123");
            expect(result.task).toEqual(mockTask);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries/1/scan");
            expect(init.method).toBe("POST");
        });
    });
});
