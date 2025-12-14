/**
 * Property-based tests for API Client.
 *
 * Tests the core functionality of the API client including URL construction,
 * authentication header injection, and error response parsing.
 */

import { describe, it, expect, vi } from "vitest";
import * as fc from "fast-check";
import {
    buildUrl,
    appendQueryParams,
    buildAuthHeader,
    ApiClient,
} from "../client";
import { ApiError } from "../types";

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// Base URL generator - valid HTTP(S) URLs without trailing content after host/port
const baseUrlArb = fc
    .record({
        protocol: fc.constantFrom("http", "https"),
        host: fc.stringMatching(/^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)*$/),
        port: fc.option(fc.integer({ min: 1, max: 65535 }), { nil: undefined }),
        trailingSlash: fc.boolean(),
    })
    .map(({ protocol, host, port, trailingSlash }) => {
        const portPart = port !== undefined ? `:${port}` : "";
        const slash = trailingSlash ? "/" : "";
        return `${protocol}://${host}${portPart}${slash}`;
    });

// Path generator - valid URL paths
const pathArb = fc
    .array(fc.stringMatching(/^[a-zA-Z0-9_-]+$/), {
        minLength: 1,
        maxLength: 5,
    })
    .chain((segments) =>
        fc.record({
            segments: fc.constant(segments),
            leadingSlash: fc.boolean(),
        })
    )
    .map(({ segments, leadingSlash }) => {
        const path = segments.join("/");
        return leadingSlash ? `/${path}` : path;
    });

// JWT token generator - simplified token format
const tokenArb = fc.stringMatching(
    /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/
);

// HTTP status code generators
const errorStatusArb = fc.integer({ min: 400, max: 599 });

// Error message generator
const errorMessageArb = fc.string({ minLength: 1, maxLength: 100 });

// Query parameter key generator
const paramKeyArb = fc.stringMatching(/^[a-zA-Z][a-zA-Z0-9_]*$/);

// Query parameter value generator
const paramValueArb = fc.oneof(
    fc.string({ minLength: 1 }),
    fc.integer(),
    fc.boolean()
);

// ============================================================================
// Property Tests
// ============================================================================

describe("Property 1: Request URL Construction", () => {
    /**
     * **Feature: frontend-api, Property 1: Request URL Construction**
     * **Validates: Requirements 1.1**
     *
     * For any API endpoint call with a configured base URL and path,
     * the resulting request URL SHALL be the concatenation of the base URL and the path.
     */

    it("buildUrl concatenates base URL and path correctly", () => {
        fc.assert(
            fc.property(baseUrlArb, pathArb, (baseUrl, path) => {
                const result = buildUrl(baseUrl, path);

                // Remove trailing slash from base for comparison
                const normalizedBase = baseUrl.endsWith("/")
                    ? baseUrl.slice(0, -1)
                    : baseUrl;

                // Ensure path starts with /
                const normalizedPath = path.startsWith("/") ? path : `/${path}`;

                // Result should be the concatenation
                expect(result).toBe(`${normalizedBase}${normalizedPath}`);
            }),
            { numRuns: 100 }
        );
    });

    it("buildUrl result always contains the base URL host", () => {
        fc.assert(
            fc.property(baseUrlArb, pathArb, (baseUrl, path) => {
                const result = buildUrl(baseUrl, path);

                // Extract host from base URL
                const urlObj = new URL(baseUrl);
                const host = urlObj.host;

                // Result should contain the host
                expect(result).toContain(host);
            }),
            { numRuns: 100 }
        );
    });

    it("buildUrl result always contains the path segments", () => {
        fc.assert(
            fc.property(baseUrlArb, pathArb, (baseUrl, path) => {
                const result = buildUrl(baseUrl, path);

                // Path segments should be present in result
                const segments = path.split("/").filter((s) => s.length > 0);
                for (const segment of segments) {
                    expect(result).toContain(segment);
                }
            }),
            { numRuns: 100 }
        );
    });

    it("buildUrl never produces double slashes in path", () => {
        fc.assert(
            fc.property(baseUrlArb, pathArb, (baseUrl, path) => {
                const result = buildUrl(baseUrl, path);

                // Remove protocol part for checking
                const withoutProtocol = result.replace(/^https?:\/\//, "");

                // Should not have double slashes
                expect(withoutProtocol).not.toContain("//");
            }),
            { numRuns: 100 }
        );
    });

    it("appendQueryParams preserves URL when no params provided", () => {
        fc.assert(
            fc.property(baseUrlArb, (url) => {
                expect(appendQueryParams(url, undefined)).toBe(url);
                expect(appendQueryParams(url, {})).toBe(url);
            }),
            { numRuns: 100 }
        );
    });

    it("appendQueryParams includes all provided parameters", () => {
        fc.assert(
            fc.property(
                baseUrlArb,
                fc.dictionary(paramKeyArb, paramValueArb, {
                    minKeys: 1,
                    maxKeys: 5,
                }),
                (url, params) => {
                    const result = appendQueryParams(url, params);

                    // All keys should be present in the query string
                    for (const key of Object.keys(params)) {
                        expect(result).toContain(key);
                    }
                }
            ),
            { numRuns: 100 }
        );
    });
});

describe("Property 2: Authentication Header Injection", () => {
    /**
     * **Feature: frontend-api, Property 2: Authentication Header Injection**
     * **Validates: Requirements 1.2**
     *
     * For any request marked as requiring authentication where a token is available,
     * the request SHALL include an Authorization header with the format "Bearer {token}".
     */

    it("buildAuthHeader produces Bearer token format", () => {
        fc.assert(
            fc.property(tokenArb, (token) => {
                const header = buildAuthHeader(token);

                // Should start with "Bearer "
                expect(header).toMatch(/^Bearer /);

                // Should contain the token
                expect(header).toBe(`Bearer ${token}`);
            }),
            { numRuns: 100 }
        );
    });

    it("buildAuthHeader preserves token exactly", () => {
        fc.assert(
            fc.property(fc.string({ minLength: 1 }), (token) => {
                const header = buildAuthHeader(token);

                // Extract token from header
                const extractedToken = header.replace("Bearer ", "");

                // Should be exactly the same
                expect(extractedToken).toBe(token);
            }),
            { numRuns: 100 }
        );
    });

    it("ApiClient includes auth header when token is available", async () => {
        await fc.assert(
            fc.asyncProperty(tokenArb, async (token) => {
                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ "content-length": "2" }),
                    json: async () => ({}),
                });
                vi.stubGlobal("fetch", mockFetch);

                const client = new ApiClient({
                    baseUrl: "http://test.local",
                    getToken: () => token,
                });

                await client.get("/test");

                // Verify Authorization header was set
                const [, init] = mockFetch.mock.calls[0] as [
                    string,
                    RequestInit
                ];
                const headers = init.headers as Record<string, string>;
                expect(headers["Authorization"]).toBe(`Bearer ${token}`);

                vi.unstubAllGlobals();
            }),
            { numRuns: 100 }
        );
    });

    it("ApiClient omits auth header when no token available", async () => {
        // Mock fetch
        const mockFetch = vi.fn().mockResolvedValue({
            ok: true,
            status: 200,
            headers: new Headers({ "content-length": "2" }),
            json: async () => ({}),
        });
        vi.stubGlobal("fetch", mockFetch);

        const client = new ApiClient({
            baseUrl: "http://test.local",
            getToken: () => null,
        });

        await client.get("/test");

        // Verify Authorization header was NOT set
        const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
        const headers = init.headers as Record<string, string>;
        expect(headers["Authorization"]).toBeUndefined();

        vi.unstubAllGlobals();
    });

    it("ApiClient omits auth header when requiresAuth is false", async () => {
        await fc.assert(
            fc.asyncProperty(tokenArb, async (token) => {
                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ "content-length": "2" }),
                    json: async () => ({}),
                });
                vi.stubGlobal("fetch", mockFetch);

                const client = new ApiClient({
                    baseUrl: "http://test.local",
                    getToken: () => token,
                });

                await client.get("/test", { requiresAuth: false });

                // Verify Authorization header was NOT set
                const [, init] = mockFetch.mock.calls[0] as [
                    string,
                    RequestInit
                ];
                const headers = init.headers as Record<string, string>;
                expect(headers["Authorization"]).toBeUndefined();

                vi.unstubAllGlobals();
            }),
            { numRuns: 100 }
        );
    });
});

describe("Property 3: Error Response Parsing", () => {
    /**
     * **Feature: frontend-api, Property 3: Error Response Parsing**
     * **Validates: Requirements 1.3**
     *
     * For any HTTP response with a non-2xx status code, the API client SHALL throw
     * an ApiError containing the status code and a message extracted from the response body.
     */

    it("ApiClient throws ApiError with correct status for error responses", async () => {
        await fc.assert(
            fc.asyncProperty(
                errorStatusArb,
                errorMessageArb,
                async (status, message) => {
                    // Mock fetch to return error response
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: false,
                        status,
                        statusText: "Error",
                        text: async () => JSON.stringify({ error: message }),
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://test.local",
                    });

                    try {
                        await client.get("/test");
                        // Should not reach here
                        expect.fail("Expected ApiError to be thrown");
                    } catch (error) {
                        expect(error).toBeInstanceOf(ApiError);
                        const apiError = error as ApiError;
                        expect(apiError.status).toBe(status);
                        expect(apiError.message).toBe(message);
                    }

                    vi.unstubAllGlobals();
                }
            ),
            { numRuns: 100 }
        );
    });

    it("ApiError status code matches HTTP response status", async () => {
        await fc.assert(
            fc.asyncProperty(errorStatusArb, async (status) => {
                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: false,
                    status,
                    statusText: "Error",
                    text: async () => JSON.stringify({ error: "Test error" }),
                });
                vi.stubGlobal("fetch", mockFetch);

                const client = new ApiClient({
                    baseUrl: "http://test.local",
                });

                try {
                    await client.get("/test");
                    expect.fail("Expected ApiError to be thrown");
                } catch (error) {
                    expect(error).toBeInstanceOf(ApiError);
                    expect((error as ApiError).status).toBe(status);
                }

                vi.unstubAllGlobals();
            }),
            { numRuns: 100 }
        );
    });

    it("ApiError helper methods work correctly based on status", () => {
        fc.assert(
            fc.property(errorStatusArb, (status) => {
                const error = new ApiError(status, "Test");

                expect(error.isUnauthorized()).toBe(status === 401);
                expect(error.isNotFound()).toBe(status === 404);
                expect(error.isBadRequest()).toBe(status === 400);
                expect(error.isServerError()).toBe(status >= 500);
            }),
            { numRuns: 100 }
        );
    });

    it("ApiClient handles non-JSON error responses gracefully", async () => {
        await fc.assert(
            fc.asyncProperty(
                errorStatusArb,
                fc.string(),
                async (status, rawText) => {
                    // Mock fetch to return non-JSON error response
                    const mockFetch = vi.fn().mockResolvedValue({
                        ok: false,
                        status,
                        statusText: "Error",
                        text: async () => rawText,
                    });
                    vi.stubGlobal("fetch", mockFetch);

                    const client = new ApiClient({
                        baseUrl: "http://test.local",
                    });

                    try {
                        await client.get("/test");
                        expect.fail("Expected ApiError to be thrown");
                    } catch (error) {
                        expect(error).toBeInstanceOf(ApiError);
                        const apiError = error as ApiError;
                        expect(apiError.status).toBe(status);
                        // Message should be the raw text or statusText for empty
                        expect(typeof apiError.message).toBe("string");
                    }

                    vi.unstubAllGlobals();
                }
            ),
            { numRuns: 100 }
        );
    });

    it("ApiClient throws ApiError with status 0 for network errors", async () => {
        // Mock fetch to throw network error
        const mockFetch = vi
            .fn()
            .mockRejectedValue(new Error("Network failure"));
        vi.stubGlobal("fetch", mockFetch);

        const client = new ApiClient({
            baseUrl: "http://test.local",
        });

        try {
            await client.get("/test");
            expect.fail("Expected ApiError to be thrown");
        } catch (error) {
            expect(error).toBeInstanceOf(ApiError);
            const apiError = error as ApiError;
            expect(apiError.status).toBe(0);
            expect(apiError.message).toBe("Network failure");
        }

        vi.unstubAllGlobals();
    });
});
