/**
 * Library API Module
 *
 * Provides functions for library management including CRUD operations
 * and scan path management.
 *
 * **Implements: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8**
 */

import { ApiClient } from "./client";
import type {
    Library,
    LibraryWithStats,
    CreateLibraryRequest,
    UpdateLibraryRequest,
    ScanPath,
} from "./types";

/**
 * Request to add a scan path to a library.
 */
export interface AddScanPathRequest {
    path: string;
}

/**
 * Library API interface.
 */
export interface LibraryApi {
    list(): Promise<LibraryWithStats[]>;
    create(request: CreateLibraryRequest): Promise<Library>;
    get(id: number): Promise<LibraryWithStats>;
    update(id: number, request: UpdateLibraryRequest): Promise<Library>;
    delete(id: number): Promise<void>;
    listScanPaths(libraryId: number): Promise<ScanPath[]>;
    addScanPath(libraryId: number, path: string): Promise<ScanPath>;
    removeScanPath(libraryId: number, pathId: number): Promise<void>;
}

/**
 * Creates a Library API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns A LibraryApi implementation
 */
export function createLibraryApi(client: ApiClient): LibraryApi {
    return {
        /**
         * Lists all libraries with their statistics.
         *
         * **Implements: Requirement 3.1**
         *
         * @returns Array of libraries with statistics
         */
        async list(): Promise<LibraryWithStats[]> {
            return client.get<LibraryWithStats[]>("/api/libraries");
        },

        /**
         * Creates a new library.
         *
         * **Implements: Requirement 3.2**
         *
         * @param request - The library creation request
         * @returns The newly created library
         */
        async create(request: CreateLibraryRequest): Promise<Library> {
            return client.post<Library>("/api/libraries", request);
        },

        /**
         * Gets a specific library by ID with statistics.
         *
         * **Implements: Requirement 3.3**
         *
         * @param id - The library ID
         * @returns The library with statistics
         */
        async get(id: number): Promise<LibraryWithStats> {
            return client.get<LibraryWithStats>(`/api/libraries/${id}`);
        },

        /**
         * Updates an existing library.
         *
         * **Implements: Requirement 3.4**
         *
         * @param id - The library ID
         * @param request - The update request
         * @returns The updated library
         */
        async update(
            id: number,
            request: UpdateLibraryRequest
        ): Promise<Library> {
            return client.put<Library>(`/api/libraries/${id}`, request);
        },

        /**
         * Deletes a library.
         *
         * **Implements: Requirement 3.5**
         *
         * @param id - The library ID
         */
        async delete(id: number): Promise<void> {
            await client.delete<void>(`/api/libraries/${id}`);
        },

        /**
         * Lists all scan paths for a library.
         *
         * **Implements: Requirement 3.6**
         *
         * @param libraryId - The library ID
         * @returns Array of scan paths
         */
        async listScanPaths(libraryId: number): Promise<ScanPath[]> {
            return client.get<ScanPath[]>(`/api/libraries/${libraryId}/paths`);
        },

        /**
         * Adds a scan path to a library.
         *
         * **Implements: Requirement 3.7**
         *
         * @param libraryId - The library ID
         * @param path - The file system path to add
         * @returns The newly created scan path
         */
        async addScanPath(libraryId: number, path: string): Promise<ScanPath> {
            const request: AddScanPathRequest = { path };
            return client.post<ScanPath>(
                `/api/libraries/${libraryId}/paths`,
                request
            );
        },

        /**
         * Removes a scan path from a library.
         *
         * **Implements: Requirement 3.8**
         *
         * @param libraryId - The library ID
         * @param pathId - The scan path ID to remove
         */
        async removeScanPath(libraryId: number, pathId: number): Promise<void> {
            await client.delete<void>(
                `/api/libraries/${libraryId}/paths/${pathId}`
            );
        },
    };
}
