/**
 * Property-based tests for Content API thumbnail functionality.
 *
 * Tests the thumbnail retrieval functionality including authentication,
 * Blob response handling, and error scenarios.
 *
 * **Validates: Requirements 2.1, 2.2**
 */

import { describe, it, expect, vi, afterEach } from "vitest";
import * as fc from "fast-check";
import { ApiClient } from "../client";
import { createContentApi } from "../content";
import { ApiError } from "../types";

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// Content ID generator - positive integers
const contentIdArb = fc.integer({ min: 1, max: 10000 });

// JWT token generator - simplified token format
const tokenArb = fc.stringMatching(
    /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/
);

// Blob data generator - creates simple Blob objects
const blobDataArb = fc
    .uint8Array({ minLength: 100, maxLength: 1000 })
    .map((data) => new Blob([data], { type: "image/jpeg" }));

// ============================================================================
// Property Tests
// ============================================================================

describe("Property 5: Authenticated API calls", () => {
    /**
     * **Feature: auto-thumbnail-fetch, Property 5: Authenticated API calls**
     * **Validates: Requirements 2.1**
     *
     * For any thumbnail API request, the request should include an Authorization
     * header with the Bearer token format.
     */

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it("getThumbnail includes Authorization header when token is available", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                tokenArb,
                blobDataArb,
                async (contentId, token, blobData) => {
                    // Mock fetch to return successful Blob response
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 200,
                        blob: async () => blobData,
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => token,
                    });
                    const contentApi = createContentApi(client);

                    await contentApi.getThumbnail(contentId);

                    // Verify Authorization header was set with Bearer format
                    expect(mockFetch).toHaveBeenCalledTimes(1);
                    const [, init] = mockFetch.mock.calls[0] as [
                        string,
                        RequestInit
                    ];
                    const headers = init.headers as Record<string, string>;
                    expect(headers["Authorization"]).toBe(`Bearer ${token}`);
                }
            ),
            { numRuns: 100 }
        );
    });

    it("getThumbnail omits Authorization header when no token available", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                blobDataArb,
                async (contentId, blobData) => {
                    // Mock fetch to return successful Blob response
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 200,
                        blob: async () => blobData,
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => null,
                    });
                    const contentApi = createContentApi(client);

                    await contentApi.getThumbnail(contentId);

                    // Verify Authorization header was NOT set
                    expect(mockFetch).toHaveBeenCalledTimes(1);
                    const [, init] = mockFetch.mock.calls[0] as [
                        string,
                        RequestInit
                    ];
                    const headers = init.headers as Record<string, string>;
                    expect(headers["Authorization"]).toBeUndefined();
                }
            ),
            { numRuns: 100 }
        );
    });

    it("getThumbnail uses correct URL format", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                tokenArb,
                blobDataArb,
                async (contentId, token, blobData) => {
                    // Mock fetch
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 200,
                        blob: async () => blobData,
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => token,
                    });
                    const contentApi = createContentApi(client);

                    await contentApi.getThumbnail(contentId);

                    // Verify URL format
                    const [url] = mockFetch.mock.calls[0] as [
                        string,
                        RequestInit
                    ];
                    expect(url).toBe(
                        `http://localhost:3000/api/contents/${contentId}/thumbnail`
                    );
                }
            ),
            { numRuns: 100 }
        );
    });
});


describe("Property 6: Blob response type", () => {
    /**
     * **Feature: auto-thumbnail-fetch, Property 6: Blob response type**
     * **Validates: Requirements 2.2**
     *
     * For any successful getThumbnail API call, the returned value should be
     * a Blob object.
     */

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it("getThumbnail returns a Blob object on success", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                tokenArb,
                blobDataArb,
                async (contentId, token, blobData) => {
                    // Mock fetch to return Blob response
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 200,
                        blob: async () => blobData,
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => token,
                    });
                    const contentApi = createContentApi(client);

                    const result = await contentApi.getThumbnail(contentId);

                    // Verify result is a Blob
                    expect(result).toBeInstanceOf(Blob);
                    expect(result).toBe(blobData);
                }
            ),
            { numRuns: 100 }
        );
    });

    it("getThumbnail preserves Blob properties", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                tokenArb,
                async (contentId, token) => {
                    // Create a Blob with specific properties
                    const testData = new Uint8Array([1, 2, 3, 4, 5]);
                    const blobData = new Blob([testData], {
                        type: "image/png",
                    });

                    // Mock fetch
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 200,
                        blob: async () => blobData,
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => token,
                    });
                    const contentApi = createContentApi(client);

                    const result = await contentApi.getThumbnail(contentId);

                    // Verify Blob properties are preserved
                    expect(result.type).toBe("image/png");
                    expect(result.size).toBe(testData.length);
                }
            ),
            { numRuns: 100 }
        );
    });

    it("getThumbnail throws ApiError on error responses", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                tokenArb,
                fc.integer({ min: 400, max: 599 }),
                fc.string({ minLength: 1, maxLength: 100 }),
                async (contentId, token, status, errorMessage) => {
                    // Mock fetch to return error response
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: false,
                        status,
                        statusText: "Error",
                        text: async () =>
                            JSON.stringify({ error: errorMessage }),
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => token,
                    });
                    const contentApi = createContentApi(client);

                    try {
                        await contentApi.getThumbnail(contentId);
                        expect.fail("Expected ApiError to be thrown");
                    } catch (error) {
                        expect(error).toBeInstanceOf(ApiError);
                        const apiError = error as ApiError;
                        expect(apiError.status).toBe(status);
                    }
                }
            ),
            { numRuns: 100 }
        );
    });

    it("getThumbnail handles 404 errors for missing thumbnails", async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                tokenArb,
                async (contentId, token) => {
                    // Mock fetch to return 404
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: false,
                        status: 404,
                        statusText: "Not Found",
                        text: async () =>
                            JSON.stringify({ error: "Thumbnail not found" }),
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://localhost:3000",
                        getToken: () => token,
                    });
                    const contentApi = createContentApi(client);

                    try {
                        await contentApi.getThumbnail(contentId);
                        expect.fail("Expected ApiError to be thrown");
                    } catch (error) {
                        expect(error).toBeInstanceOf(ApiError);
                        const apiError = error as ApiError;
                        expect(apiError.status).toBe(404);
                        expect(apiError.isNotFound()).toBe(true);
                    }
                }
            ),
            { numRuns: 100 }
        );
    });
});
