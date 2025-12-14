/**
 * Property-based tests for Library Store.
 *
 * Tests the core functionality of the library store including cache consistency,
 * library selection, and library deletion.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import * as fc from 'fast-check'
import { setActivePinia, createPinia } from 'pinia'
import { useLibraryStore } from '@/stores/useLibraryStore'
import { useAuthStore } from '@/stores/useAuthStore'
import type { LibraryWithStats } from '@/api/types'

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// Library ID generator
const libraryIdArb = fc.integer({ min: 1, max: 1000000 })

// Library name generator
const libraryNameArb = fc.string({ minLength: 1, maxLength: 100 })

// Scan interval generator (in seconds)
const scanIntervalArb = fc.integer({ min: 60, max: 86400 })

// Watch mode generator
const watchModeArb = fc.boolean()

// Path count generator
const pathCountArb = fc.integer({ min: 0, max: 100 })

// Content count generator
const contentCountArb = fc.integer({ min: 0, max: 10000 })

// ISO date string generator
const dateStringArb = fc.integer({ min: 1577836800000, max: 1924905600000 })
    .map(ts => new Date(ts).toISOString())

// LibraryWithStats generator
const libraryWithStatsArb: fc.Arbitrary<LibraryWithStats> = fc.record({
    id: libraryIdArb,
    name: libraryNameArb,
    scan_interval: scanIntervalArb,
    watch_mode: watchModeArb,
    created_at: dateStringArb,
    updated_at: dateStringArb,
    path_count: pathCountArb,
    content_count: contentCountArb,
})

// Array of unique libraries (unique by ID)
const librariesArrayArb = fc.array(libraryWithStatsArb, { minLength: 0, maxLength: 20 })
    .map(libs => {
        // Ensure unique IDs
        const seen = new Set<number>()
        return libs.filter(lib => {
            if (seen.has(lib.id)) return false
            seen.add(lib.id)
            return true
        })
    })

// Non-empty array of unique libraries
const nonEmptyLibrariesArrayArb = fc.array(libraryWithStatsArb, { minLength: 1, maxLength: 20 })
    .map(libs => {
        const seen = new Set<number>()
        return libs.filter(lib => {
            if (seen.has(lib.id)) return false
            seen.add(lib.id)
            return true
        })
    })
    .filter(libs => libs.length > 0)

// ============================================================================
// Test Setup
// ============================================================================

// Mock localStorage for auth store
const localStorageMock = (() => {
    let store: Record<string, string> = {}
    return {
        getItem: vi.fn((key: string) => store[key] ?? null),
        setItem: vi.fn((key: string, value: string) => { store[key] = value }),
        removeItem: vi.fn((key: string) => { delete store[key] }),
        clear: vi.fn(() => { store = {} }),
        get store() { return store },
    }
})()

// ============================================================================
// Property Tests
// ============================================================================

describe('Property 5: Library cache consistency', () => {
    /**
     * **Feature: pinia-state-management, Property 5: Library cache consistency**
     * **Validates: Requirements 3.2**
     *
     * For any array of libraries fetched from the API, the store's libraries state
     * should contain exactly those libraries.
     */

    beforeEach(() => {
        vi.stubGlobal('localStorage', localStorageMock)
        localStorageMock.clear()
        setActivePinia(createPinia())
    })

    afterEach(() => {
        vi.unstubAllGlobals()
        vi.restoreAllMocks()
    })

    it('fetchLibraries caches exactly the libraries returned from API', async () => {
        await fc.assert(
            fc.asyncProperty(librariesArrayArb, async (apiLibraries) => {
                setActivePinia(createPinia())

                // Set up auth store with a token
                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch to return the libraries
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                // Fetch libraries
                await libraryStore.fetchLibraries()

                // Verify cache contains exactly the API response
                expect(libraryStore.libraries).toHaveLength(apiLibraries.length)
                expect(libraryStore.libraries).toEqual(apiLibraries)
            }),
            { numRuns: 100 }
        )
    })

    it('libraryById getter returns correct library from cache', async () => {
        await fc.assert(
            fc.asyncProperty(nonEmptyLibrariesArrayArb, async (apiLibraries) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                await libraryStore.fetchLibraries()

                // For each library in the cache, libraryById should return it
                for (const lib of apiLibraries) {
                    const found = libraryStore.libraryById(lib.id)
                    expect(found).toEqual(lib)
                }
            }),
            { numRuns: 100 }
        )
    })

    it('libraryById returns undefined for non-existent ID', async () => {
        await fc.assert(
            fc.asyncProperty(librariesArrayArb, libraryIdArb, async (apiLibraries, searchId) => {
                // Skip if searchId happens to be in the array
                if (apiLibraries.some(lib => lib.id === searchId)) return

                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                await libraryStore.fetchLibraries()

                // libraryById should return undefined for non-existent ID
                expect(libraryStore.libraryById(searchId)).toBeUndefined()
            }),
            { numRuns: 100 }
        )
    })
})

describe('Property 6: Library selection updates current library', () => {
    /**
     * **Feature: pinia-state-management, Property 6: Library selection updates current library**
     * **Validates: Requirements 3.3**
     *
     * For any library ID that exists in the libraries list, calling selectLibrary
     * should set currentLibrary to that library.
     */

    beforeEach(() => {
        vi.stubGlobal('localStorage', localStorageMock)
        localStorageMock.clear()
        setActivePinia(createPinia())
    })

    afterEach(() => {
        vi.unstubAllGlobals()
        vi.restoreAllMocks()
    })

    it('selectLibrary sets currentLibrary to the selected library', async () => {
        await fc.assert(
            fc.asyncProperty(nonEmptyLibrariesArrayArb, async (apiLibraries) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                await libraryStore.fetchLibraries()

                // Select each library and verify currentLibrary is updated
                for (const lib of apiLibraries) {
                    libraryStore.selectLibrary(lib.id)
                    expect(libraryStore.currentLibrary).toEqual(lib)
                }
            }),
            { numRuns: 100 }
        )
    })

    it('selectLibrary with non-existent ID does not change currentLibrary', async () => {
        await fc.assert(
            fc.asyncProperty(nonEmptyLibrariesArrayArb, libraryIdArb, async (apiLibraries, nonExistentId) => {
                // Skip if nonExistentId happens to be in the array
                if (apiLibraries.some(lib => lib.id === nonExistentId)) return

                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch
                const mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                await libraryStore.fetchLibraries()

                // First select a valid library
                const firstLib = apiLibraries[0]
                libraryStore.selectLibrary(firstLib.id)
                expect(libraryStore.currentLibrary).toEqual(firstLib)

                // Try to select non-existent ID - currentLibrary should remain unchanged
                libraryStore.selectLibrary(nonExistentId)
                expect(libraryStore.currentLibrary).toEqual(firstLib)
            }),
            { numRuns: 100 }
        )
    })
})

describe('Property 7: Library deletion removes from cache', () => {
    /**
     * **Feature: pinia-state-management, Property 7: Library deletion removes from cache**
     * **Validates: Requirements 3.5**
     *
     * For any library that is deleted, it should no longer appear in the libraries list,
     * and if it was the currentLibrary, currentLibrary should become null.
     */

    beforeEach(() => {
        vi.stubGlobal('localStorage', localStorageMock)
        localStorageMock.clear()
        setActivePinia(createPinia())
    })

    afterEach(() => {
        vi.unstubAllGlobals()
        vi.restoreAllMocks()
    })

    it('deleteLibrary removes library from cache', async () => {
        await fc.assert(
            fc.asyncProperty(nonEmptyLibrariesArrayArb, async (apiLibraries) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch for initial load
                let mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                await libraryStore.fetchLibraries()

                // Pick a library to delete
                const libToDelete = apiLibraries[0]
                const initialLength = libraryStore.libraries.length

                // Mock fetch for delete operation
                mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 204,
                    headers: new Headers({ 'content-length': '0' }),
                    json: async () => ({}),
                })
                vi.stubGlobal('fetch', mockFetch)

                // Delete the library
                await libraryStore.deleteLibrary(libToDelete.id)

                // Verify library is removed from cache
                expect(libraryStore.libraries).toHaveLength(initialLength - 1)
                expect(libraryStore.libraries.find(lib => lib.id === libToDelete.id)).toBeUndefined()
                expect(libraryStore.libraryById(libToDelete.id)).toBeUndefined()
            }),
            { numRuns: 100 }
        )
    })

    it('deleteLibrary clears currentLibrary if deleted library was selected', async () => {
        await fc.assert(
            fc.asyncProperty(nonEmptyLibrariesArrayArb, async (apiLibraries) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const libraryStore = useLibraryStore()

                // Mock fetch for initial load
                let mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 200,
                    headers: new Headers({ 'content-length': '100' }),
                    json: async () => apiLibraries,
                })
                vi.stubGlobal('fetch', mockFetch)

                await libraryStore.fetchLibraries()

                // Select a library
                const libToDelete = apiLibraries[0]
                libraryStore.selectLibrary(libToDelete.id)
                expect(libraryStore.currentLibrary).toEqual(libToDelete)

                // Mock fetch for delete operation
                mockFetch = vi.fn().mockResolvedValue({
                    ok: true,
                    status: 204,
                    headers: new Headers({ 'content-length': '0' }),
                    json: async () => ({}),
                })
                vi.stubGlobal('fetch', mockFetch)

                // Delete the selected library
                await libraryStore.deleteLibrary(libToDelete.id)

                // currentLibrary should be null
                expect(libraryStore.currentLibrary).toBeNull()
            }),
            { numRuns: 100 }
        )
    })

    it('deleteLibrary preserves currentLibrary if different library was deleted', async () => {
        await fc.assert(
            fc.asyncProperty(
                fc.array(libraryWithStatsArb, { minLength: 2, maxLength: 20 })
                    .map(libs => {
                        const seen = new Set<number>()
                        return libs.filter(lib => {
                            if (seen.has(lib.id)) return false
                            seen.add(lib.id)
                            return true
                        })
                    })
                    .filter(libs => libs.length >= 2),
                async (apiLibraries) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const libraryStore = useLibraryStore()

                    // Mock fetch for initial load
                    let mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 200,
                        headers: new Headers({ 'content-length': '100' }),
                        json: async () => apiLibraries,
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    await libraryStore.fetchLibraries()

                    // Select first library
                    const selectedLib = apiLibraries[0]
                    libraryStore.selectLibrary(selectedLib.id)
                    expect(libraryStore.currentLibrary).toEqual(selectedLib)

                    // Delete a different library
                    const libToDelete = apiLibraries[1]

                    // Mock fetch for delete operation
                    mockFetch = vi.fn().mockResolvedValue({
                        ok: true,
                        status: 204,
                        headers: new Headers({ 'content-length': '0' }),
                        json: async () => ({}),
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    await libraryStore.deleteLibrary(libToDelete.id)

                    // currentLibrary should still be the selected library
                    expect(libraryStore.currentLibrary).toEqual(selectedLib)
                }
            ),
            { numRuns: 100 }
        )
    })
})
