/**
 * Unit tests for Library API module.
 *
 * Tests library management API functions including CRUD operations
 * and scan path management.
 *
 * **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8**
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiClient } from "../client";
import { createLibraryApi } from "../library";
import { ApiError } from "../types";
import type {
    Library,
    LibraryWithStats,
    CreateLibraryRequest,
    UpdateLibraryRequest,
    ScanPath,
} from "../types";

describe("Library API", () => {
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

    function mockNoContentResponse() {
        return {
            ok: true,
            status: 204,
            headers: new Headers({ "content-length": "0" }),
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
         * **Validates: Requirement 3.1**
         * WHEN a user requests the library list, THE Library_API SHALL return
         * all Library objects with their statistics
         */

        it("retrieves all libraries with statistics", async () => {
            const mockLibraries: LibraryWithStats[] = [
                {
                    id: 1,
                    name: "Comics",
                    scan_interval: 3600,
                    watch_mode: true,
                    created_at: "2024-01-01T00:00:00Z",
                    updated_at: "2024-01-01T00:00:00Z",
                    path_count: 2,
                    content_count: 50,
                },
                {
                    id: 2,
                    name: "Novels",
                    scan_interval: 0,
                    watch_mode: false,
                    created_at: "2024-01-02T00:00:00Z",
                    updated_at: "2024-01-02T00:00:00Z",
                    path_count: 1,
                    content_count: 30,
                },
            ];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockLibraries));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.list();

            expect(result).toEqual(mockLibraries);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries");
            expect(init.method).toBe("GET");
        });
    });

    describe("create", () => {
        /**
         * **Validates: Requirement 3.2**
         * WHEN a user creates a new library, THE Library_API SHALL send
         * the creation request and return the new Library object
         */

        it("creates a new library with all options", async () => {
            const createRequest: CreateLibraryRequest = {
                name: "New Library",
                scan_interval: 7200,
                watch_mode: true,
            };
            const mockLibrary: Library = {
                id: 3,
                name: "New Library",
                scan_interval: 7200,
                watch_mode: true,
                created_at: "2024-01-03T00:00:00Z",
                updated_at: "2024-01-03T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockLibrary));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.create(createRequest);

            expect(result).toEqual(mockLibrary);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries");
            expect(init.method).toBe("POST");
            expect(JSON.parse(init.body as string)).toEqual(createRequest);
        });

        it("creates a library with minimal options", async () => {
            const createRequest: CreateLibraryRequest = {
                name: "Minimal Library",
            };
            const mockLibrary: Library = {
                id: 4,
                name: "Minimal Library",
                scan_interval: 0,
                watch_mode: false,
                created_at: "2024-01-04T00:00:00Z",
                updated_at: "2024-01-04T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockLibrary));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.create(createRequest);

            expect(result.name).toBe("Minimal Library");
        });
    });

    describe("get", () => {
        /**
         * **Validates: Requirement 3.3**
         * WHEN a user requests a specific library by ID, THE Library_API SHALL
         * return the Library details with statistics
         */

        it("retrieves a library by ID with statistics", async () => {
            const mockLibrary: LibraryWithStats = {
                id: 1,
                name: "Comics",
                scan_interval: 3600,
                watch_mode: true,
                created_at: "2024-01-01T00:00:00Z",
                updated_at: "2024-01-01T00:00:00Z",
                path_count: 2,
                content_count: 50,
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockLibrary));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.get(1);

            expect(result).toEqual(mockLibrary);

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toBe("http://localhost:3000/api/libraries/1");
        });

        it("throws ApiError when library not found", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(404, "Library not found")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            await expect(libraryApi.get(999)).rejects.toThrow(ApiError);

            try {
                await libraryApi.get(999);
            } catch (error) {
                expect((error as ApiError).isNotFound()).toBe(true);
            }
        });
    });

    describe("update", () => {
        /**
         * **Validates: Requirement 3.4**
         * WHEN a user updates a library, THE Library_API SHALL send
         * the update request and return the updated Library object
         */

        it("updates a library with new values", async () => {
            const updateRequest: UpdateLibraryRequest = {
                name: "Updated Comics",
                scan_interval: 1800,
            };
            const mockLibrary: Library = {
                id: 1,
                name: "Updated Comics",
                scan_interval: 1800,
                watch_mode: true,
                created_at: "2024-01-01T00:00:00Z",
                updated_at: "2024-01-05T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockLibrary));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.update(1, updateRequest);

            expect(result).toEqual(mockLibrary);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries/1");
            expect(init.method).toBe("PUT");
            expect(JSON.parse(init.body as string)).toEqual(updateRequest);
        });
    });

    describe("delete", () => {
        /**
         * **Validates: Requirement 3.5**
         * WHEN a user deletes a library, THE Library_API SHALL send
         * the deletion request and return success status
         */

        it("deletes a library", async () => {
            mockFetch.mockResolvedValue(mockNoContentResponse());

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            await libraryApi.delete(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries/1");
            expect(init.method).toBe("DELETE");
        });
    });

    describe("listScanPaths", () => {
        /**
         * **Validates: Requirement 3.6**
         * WHEN a user requests scan paths for a library, THE Library_API SHALL
         * return all ScanPath objects for that library
         */

        it("retrieves all scan paths for a library", async () => {
            const mockPaths: ScanPath[] = [
                {
                    id: 1,
                    library_id: 1,
                    path: "/media/comics/manga",
                    created_at: "2024-01-01T00:00:00Z",
                },
                {
                    id: 2,
                    library_id: 1,
                    path: "/media/comics/western",
                    created_at: "2024-01-02T00:00:00Z",
                },
            ];
            mockFetch.mockResolvedValue(mockSuccessResponse(mockPaths));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.listScanPaths(1);

            expect(result).toEqual(mockPaths);

            const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
            expect(url).toBe("http://localhost:3000/api/libraries/1/paths");
        });
    });

    describe("addScanPath", () => {
        /**
         * **Validates: Requirement 3.7**
         * WHEN a user adds a scan path to a library, THE Library_API SHALL
         * send the add request and return the new ScanPath object
         */

        it("adds a scan path to a library", async () => {
            const mockPath: ScanPath = {
                id: 3,
                library_id: 1,
                path: "/media/comics/new",
                created_at: "2024-01-03T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockPath));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            const result = await libraryApi.addScanPath(1, "/media/comics/new");

            expect(result).toEqual(mockPath);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries/1/paths");
            expect(init.method).toBe("POST");
            expect(JSON.parse(init.body as string)).toEqual({
                path: "/media/comics/new",
            });
        });
    });

    describe("removeScanPath", () => {
        /**
         * **Validates: Requirement 3.8**
         * WHEN a user removes a scan path from a library, THE Library_API SHALL
         * send the removal request and return success status
         */

        it("removes a scan path from a library", async () => {
            mockFetch.mockResolvedValue(mockNoContentResponse());

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.token",
            });
            const libraryApi = createLibraryApi(client);

            await libraryApi.removeScanPath(1, 2);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/libraries/1/paths/2");
            expect(init.method).toBe("DELETE");
        });
    });
});
