/**
 * Property-based tests for Content Store.
 *
 * Tests the core functionality of the content store including cache consistency,
 * cache hit behavior, force refresh, content selection, and deletion.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import * as fc from 'fast-check'
import { setActivePinia, createPinia } from 'pinia'
import { useContentStore } from '@/stores/useContentStore'
import { useAuthStore } from '@/stores/useAuthStore'
import type { ContentResponse, Chapter, ContentType } from '@/api/types'

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// ID generators
const idArb = fc.integer({ min: 1, max: 1000000 })
const libraryIdArb = fc.integer({ min: 1, max: 1000000 })
const contentIdArb = fc.integer({ min: 1, max: 1000000 })

// Content type generator
const contentTypeArb: fc.Arbitrary<ContentType> = fc.constantFrom('Comic', 'Novel')

// Title generator
const titleArb = fc.string({ minLength: 1, maxLength: 100 })

// ISO date string generator
const dateStringArb = fc.integer({ min: 1577836800000, max: 1924905600000 })
    .map(ts => new Date(ts).toISOString())

// ContentResponse generator
const contentResponseArb: fc.Arbitrary<ContentResponse> = fc.record({
    id: contentIdArb,
    library_id: libraryIdArb,
    content_type: contentTypeArb,
    title: titleArb,
    chapter_count: fc.integer({ min: 0, max: 1000 }),
    has_thumbnail: fc.boolean(),
    metadata: fc.constant(null),
    created_at: dateStringArb,
})

// Array of unique contents (unique by ID)
const contentsArrayArb = fc.array(contentResponseArb, { minLength: 0, maxLength: 20 })
    .map(contents => {
        const seen = new Set<number>()
        return contents.filter(c => {
            if (seen.has(c.id)) return false
            seen.add(c.id)
            return true
        })
    })

// Non-empty array of unique contents
const nonEmptyContentsArrayArb = fc.array(contentResponseArb, { minLength: 1, maxLength: 20 })
    .map(contents => {
        const seen = new Set<number>()
        return contents.filter(c => {
            if (seen.has(c.id)) return false
            seen.add(c.id)
            return true
        })
    })
    .filter(contents => contents.length > 0)

// Chapter generator
const chapterArb: fc.Arbitrary<Chapter> = fc.record({
    id: idArb,
    content_id: contentIdArb,
    title: titleArb,
    file_path: fc.string({ minLength: 1, maxLength: 200 }),
    sort_order: fc.integer({ min: 0, max: 10000 }),
})

// Array of unique chapters (unique by ID)
const chaptersArrayArb = fc.array(chapterArb, { minLength: 0, maxLength: 50 })
    .map(chapters => {
        const seen = new Set<number>()
        return chapters.filter(c => {
            if (seen.has(c.id)) return false
            seen.add(c.id)
            return true
        })
    })

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

// Helper to create mock fetch
function createMockFetch(response: unknown, status = 200) {
    return vi.fn().mockResolvedValue({
        ok: status >= 200 && status < 300,
        status,
        headers: new Headers({ 'content-length': '100' }),
        json: async () => response,
    })
}

// ============================================================================
// Property Tests
// ============================================================================

describe('Property 1: Content cache consistency', () => {
    /**
     * **Feature: content-store, Property 1: Content cache consistency**
     * **Validates: Requirements 2.1**
     *
     * For any library ID and fetched content list, after calling fetchContents,
     * the store's contents map should contain exactly those contents under that library ID key.
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

    it('fetchContents caches exactly the contents returned from API', async () => {
        await fc.assert(
            fc.asyncProperty(libraryIdArb, contentsArrayArb, async (libraryId, apiContents) => {
                setActivePinia(createPinia())

                // Set up auth store with a token
                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const contentStore = useContentStore()

                // Mock fetch to return the contents
                vi.stubGlobal('fetch', createMockFetch(apiContents))

                // Fetch contents
                const result = await contentStore.fetchContents(libraryId)

                // Verify cache contains exactly the API response
                expect(result).toEqual(apiContents)
                expect(contentStore.contentsByLibrary(libraryId)).toEqual(apiContents)
                expect(contentStore.contents.get(libraryId)).toEqual(apiContents)
            }),
            { numRuns: 100 }
        )
    })
})


describe('Property 2: Cache hit avoids API call', () => {
    /**
     * **Feature: content-store, Property 2: Cache hit avoids API call**
     * **Validates: Requirements 2.2**
     *
     * For any library ID that already has cached contents, calling fetchContents
     * without force flag should return cached data without making an API call.
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

    it('fetchContents returns cached data without API call when cache exists', async () => {
        await fc.assert(
            fc.asyncProperty(libraryIdArb, contentsArrayArb, contentsArrayArb, async (libraryId, initialContents, newContents) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const contentStore = useContentStore()

                // First fetch - populate cache
                const mockFetch = createMockFetch(initialContents)
                vi.stubGlobal('fetch', mockFetch)

                await contentStore.fetchContents(libraryId)
                expect(mockFetch).toHaveBeenCalledTimes(1)

                // Second fetch without force - should use cache
                const mockFetch2 = createMockFetch(newContents)
                vi.stubGlobal('fetch', mockFetch2)

                const result = await contentStore.fetchContents(libraryId)

                // Should return cached data, not new data
                expect(result).toEqual(initialContents)
                // Should NOT have called the API
                expect(mockFetch2).not.toHaveBeenCalled()
            }),
            { numRuns: 100 }
        )
    })
})

describe('Property 3: Force refresh updates cache', () => {
    /**
     * **Feature: content-store, Property 3: Force refresh updates cache**
     * **Validates: Requirements 2.3**
     *
     * For any library ID with existing cache, calling fetchContents with force=true
     * should call the API and update the cache with new data.
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

    it('fetchContents with force=true calls API and updates cache', async () => {
        await fc.assert(
            fc.asyncProperty(libraryIdArb, contentsArrayArb, contentsArrayArb, async (libraryId, initialContents, newContents) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const contentStore = useContentStore()

                // First fetch - populate cache
                vi.stubGlobal('fetch', createMockFetch(initialContents))
                await contentStore.fetchContents(libraryId)

                // Force refresh - should call API and update cache
                const mockFetch2 = createMockFetch(newContents)
                vi.stubGlobal('fetch', mockFetch2)

                const result = await contentStore.fetchContents(libraryId, true)

                // Should return new data
                expect(result).toEqual(newContents)
                // Should have called the API
                expect(mockFetch2).toHaveBeenCalledTimes(1)
                // Cache should be updated
                expect(contentStore.contentsByLibrary(libraryId)).toEqual(newContents)
            }),
            { numRuns: 100 }
        )
    })
})

describe('Property 4: Content selection updates state atomically', () => {
    /**
     * **Feature: content-store, Property 4: Content selection updates state atomically**
     * **Validates: Requirements 4.1, 4.2**
     *
     * For any content, calling selectContent should set currentContent to that content
     * and fetch its chapters.
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

    it('selectContent sets currentContent and fetches chapters', async () => {
        await fc.assert(
            fc.asyncProperty(contentResponseArb, chaptersArrayArb, async (content, apiChapters) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const contentStore = useContentStore()

                // Mock fetch to return chapters
                vi.stubGlobal('fetch', createMockFetch(apiChapters))

                // Select content
                await contentStore.selectContent(content)

                // Verify currentContent is set
                expect(contentStore.currentContent).toEqual(content)

                // Verify chapters are cached
                expect(contentStore.chapters.get(content.id)).toEqual(apiChapters)

                // Verify currentChapters getter returns the chapters
                expect(contentStore.currentChapters).toEqual(apiChapters)
            }),
            { numRuns: 100 }
        )
    })
})

describe('Property 5: Chapter cache hit avoids API call', () => {
    /**
     * **Feature: content-store, Property 5: Chapter cache hit avoids API call**
     * **Validates: Requirements 4.3**
     *
     * For any content whose chapters are already cached, selecting it again
     * should not make an API call for chapters.
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

    it('selectContent uses cached chapters without API call', async () => {
        await fc.assert(
            fc.asyncProperty(contentResponseArb, chaptersArrayArb, chaptersArrayArb, async (content, initialChapters, newChapters) => {
                setActivePinia(createPinia())

                const authStore = useAuthStore()
                authStore.token = 'test-token'

                const contentStore = useContentStore()

                // First selection - populate chapter cache
                const mockFetch = createMockFetch(initialChapters)
                vi.stubGlobal('fetch', mockFetch)

                await contentStore.selectContent(content)
                expect(mockFetch).toHaveBeenCalledTimes(1)

                // Clear currentContent to simulate navigating away
                contentStore.clearCurrentContent()
                expect(contentStore.currentContent).toBeNull()

                // Second selection - should use cached chapters
                const mockFetch2 = createMockFetch(newChapters)
                vi.stubGlobal('fetch', mockFetch2)

                await contentStore.selectContent(content)

                // Should have set currentContent again
                expect(contentStore.currentContent).toEqual(content)

                // Should NOT have called the API for chapters
                expect(mockFetch2).not.toHaveBeenCalled()

                // Should still have the original cached chapters
                expect(contentStore.chapters.get(content.id)).toEqual(initialChapters)
            }),
            { numRuns: 100 }
        )
    })
})


describe('Property 6: Content deletion removes from cache', () => {
    /**
     * **Feature: content-store, Property 6: Content deletion removes from cache**
     * **Validates: Requirements 5.1, 5.2**
     *
     * For any content that is deleted, it should no longer appear in the contents map,
     * and if it was currentContent, both currentContent and its chapters should be cleared.
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

    it('deleteContent removes content from cache and clears currentContent if selected', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                nonEmptyContentsArrayArb,
                chaptersArrayArb,
                async (libraryId, contents, chapters) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Pick a random content to delete
                    const contentToDelete = contents[0]

                    // Ensure the content has the correct library_id
                    const contentsWithLibrary = contents.map(c => ({ ...c, library_id: libraryId }))
                    const contentToDeleteWithLibrary = { ...contentToDelete, library_id: libraryId }

                    // First, populate the cache with contents
                    vi.stubGlobal('fetch', createMockFetch(contentsWithLibrary))
                    await contentStore.fetchContents(libraryId)

                    // Select the content to be deleted (to test currentContent clearing)
                    vi.stubGlobal('fetch', createMockFetch(chapters))
                    await contentStore.selectContent(contentToDeleteWithLibrary)

                    // Verify content is selected
                    expect(contentStore.currentContent?.id).toBe(contentToDeleteWithLibrary.id)

                    // Mock delete API call
                    vi.stubGlobal('fetch', createMockFetch(null, 204))

                    // Delete the content
                    await contentStore.deleteContent(contentToDeleteWithLibrary.id)

                    // Property 5.1: Content should no longer appear in contents map
                    const cachedContents = contentStore.contentsByLibrary(libraryId)
                    const deletedContentInCache = cachedContents.find(c => c.id === contentToDeleteWithLibrary.id)
                    expect(deletedContentInCache).toBeUndefined()

                    // Property 5.2: currentContent should be cleared since deleted content was selected
                    expect(contentStore.currentContent).toBeNull()

                    // Chapters for deleted content should also be cleared
                    expect(contentStore.chapters.get(contentToDeleteWithLibrary.id)).toBeUndefined()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('deleteContent removes content from cache but keeps currentContent if different content selected', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                // Need at least 2 contents to test this scenario
                fc.array(contentResponseArb, { minLength: 2, maxLength: 20 })
                    .map(contents => {
                        const seen = new Set<number>()
                        return contents.filter(c => {
                            if (seen.has(c.id)) return false
                            seen.add(c.id)
                            return true
                        })
                    })
                    .filter(contents => contents.length >= 2),
                chaptersArrayArb,
                async (libraryId, contents, chapters) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Ensure contents have the correct library_id
                    const contentsWithLibrary = contents.map(c => ({ ...c, library_id: libraryId }))
                    const contentToSelect = contentsWithLibrary[0]
                    const contentToDelete = contentsWithLibrary[1]

                    // Populate the cache
                    vi.stubGlobal('fetch', createMockFetch(contentsWithLibrary))
                    await contentStore.fetchContents(libraryId)

                    // Select a different content than the one we'll delete
                    vi.stubGlobal('fetch', createMockFetch(chapters))
                    await contentStore.selectContent(contentToSelect)

                    expect(contentStore.currentContent?.id).toBe(contentToSelect.id)

                    // Mock delete API call
                    vi.stubGlobal('fetch', createMockFetch(null, 204))

                    // Delete a different content
                    await contentStore.deleteContent(contentToDelete.id)

                    // Property 5.1: Deleted content should not be in cache
                    const cachedContents = contentStore.contentsByLibrary(libraryId)
                    expect(cachedContents.find(c => c.id === contentToDelete.id)).toBeUndefined()

                    // currentContent should NOT be cleared since we deleted a different content
                    expect(contentStore.currentContent?.id).toBe(contentToSelect.id)
                }
            ),
            { numRuns: 100 }
        )
    })
})


describe('Property 7: Error cleared on new operation', () => {
    /**
     * **Feature: content-store, Property 7: Error cleared on new operation**
     * **Validates: Requirements 6.2**
     *
     * For any store state with an existing error, starting a new API operation
     * should clear the error state before the operation begins.
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

    it('fetchContents clears existing error before API call', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                contentsArrayArb,
                fc.string({ minLength: 1, maxLength: 100 }),
                async (libraryId, apiContents, previousError) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Set up an existing error state
                    contentStore.error = previousError

                    // Mock successful API call
                    vi.stubGlobal('fetch', createMockFetch(apiContents))

                    // Start new operation
                    await contentStore.fetchContents(libraryId, true)

                    // Error should be cleared (either null or set to new error if failed)
                    // Since we mocked a successful response, error should be null
                    expect(contentStore.error).toBeNull()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('searchContents clears existing error before API call', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                fc.string({ minLength: 1, maxLength: 50 }),
                contentsArrayArb,
                fc.string({ minLength: 1, maxLength: 100 }),
                async (libraryId, query, apiContents, previousError) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Set up an existing error state
                    contentStore.error = previousError

                    // Mock successful API call
                    vi.stubGlobal('fetch', createMockFetch(apiContents))

                    // Start new operation
                    await contentStore.searchContents(libraryId, query)

                    // Error should be cleared
                    expect(contentStore.error).toBeNull()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('selectContent clears existing error before API call', async () => {
        await fc.assert(
            fc.asyncProperty(
                contentResponseArb,
                chaptersArrayArb,
                fc.string({ minLength: 1, maxLength: 100 }),
                async (content, apiChapters, previousError) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Set up an existing error state
                    contentStore.error = previousError

                    // Mock successful API call
                    vi.stubGlobal('fetch', createMockFetch(apiChapters))

                    // Start new operation
                    await contentStore.selectContent(content)

                    // Error should be cleared
                    expect(contentStore.error).toBeNull()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('deleteContent clears existing error before API call', async () => {
        await fc.assert(
            fc.asyncProperty(
                contentIdArb,
                fc.string({ minLength: 1, maxLength: 100 }),
                async (contentId, previousError) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Set up an existing error state
                    contentStore.error = previousError

                    // Mock successful API call (204 No Content for delete)
                    vi.stubGlobal('fetch', createMockFetch(null, 204))

                    // Start new operation
                    await contentStore.deleteContent(contentId)

                    // Error should be cleared
                    expect(contentStore.error).toBeNull()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('error is cleared even when new operation fails', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                fc.string({ minLength: 1, maxLength: 100 }),
                fc.string({ minLength: 1, maxLength: 100 }),
                async (libraryId, previousError, newErrorMessage) => {
                    // Ensure errors are different to verify clearing happened
                    fc.pre(previousError !== newErrorMessage)

                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Set up an existing error state
                    contentStore.error = previousError

                    // Mock failed API call
                    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error(newErrorMessage)))

                    // Start new operation (will fail)
                    try {
                        await contentStore.fetchContents(libraryId, true)
                    } catch {
                        // Expected to throw
                    }

                    // Error should be the new error, not the previous one
                    // This proves the error was cleared before the operation
                    expect(contentStore.error).toBe(newErrorMessage)
                    expect(contentStore.error).not.toBe(previousError)
                }
            ),
            { numRuns: 100 }
        )
    })
})


// ============================================================================
// Thumbnail Integration Property Tests
// ============================================================================

describe('Property 1: Automatic thumbnail preload trigger', () => {
    /**
     * **Feature: auto-thumbnail-fetch, Property 1: Automatic thumbnail preload trigger**
     * **Validates: Requirements 1.1**
     *
     * For any library ID and content list returned by fetchContents, when the content list
     * contains items with has_thumbnail=true, the Content Store should automatically
     * initiate thumbnail loading for those items.
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

    it('fetchContents triggers preloadThumbnails for contents with has_thumbnail=true', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                // Generate contents with at least some having has_thumbnail=true
                fc.array(
                    fc.record({
                        id: contentIdArb,
                        library_id: libraryIdArb,
                        content_type: contentTypeArb,
                        title: titleArb,
                        chapter_count: fc.integer({ min: 0, max: 1000 }),
                        has_thumbnail: fc.boolean(),
                        metadata: fc.constant(null),
                        created_at: dateStringArb,
                    }),
                    { minLength: 1, maxLength: 5 }
                ).map(contents => {
                    // Ensure unique IDs
                    const seen = new Set<number>()
                    return contents.filter(c => {
                        if (seen.has(c.id)) return false
                        seen.add(c.id)
                        return true
                    })
                }).filter(contents => contents.length > 0),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Track thumbnail API calls
                    const thumbnailCalls: number[] = []
                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            // Extract content ID from URL
                            const match = url.match(/\/contents\/(\d+)\/thumbnail/)
                            if (match) {
                                thumbnailCalls.push(parseInt(match[1], 10))
                            }
                            // Return a mock blob response immediately
                            return Promise.resolve({
                                ok: true,
                                status: 200,
                                blob: async () => new Blob(['test'], { type: 'image/jpeg' }),
                            })
                        }
                        // Return content list response
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Mock URL.createObjectURL
                    const createObjectURLMock = vi.fn().mockReturnValue('blob:test-url')
                    vi.stubGlobal('URL', {
                        ...URL,
                        createObjectURL: createObjectURLMock,
                        revokeObjectURL: vi.fn(),
                    })

                    // Fetch contents
                    await contentStore.fetchContents(libraryId)

                    // Wait for microtasks to complete (thumbnail preloading is async but immediate)
                    await Promise.resolve()
                    await Promise.resolve()

                    // Get contents that should have triggered thumbnail loading
                    const contentsWithThumbnails = apiContents.filter(c => c.has_thumbnail)

                    // Verify that thumbnail API was called for each content with has_thumbnail=true
                    for (const content of contentsWithThumbnails) {
                        expect(thumbnailCalls).toContain(content.id)
                    }

                    // Verify that thumbnail API was NOT called for contents without thumbnails
                    const contentsWithoutThumbnails = apiContents.filter(c => !c.has_thumbnail)
                    for (const content of contentsWithoutThumbnails) {
                        expect(thumbnailCalls).not.toContain(content.id)
                    }
                }
            ),
            { numRuns: 100 }
        )
    })

    it('fetchContents does not block on thumbnail preloading', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                nonEmptyContentsArrayArb.map(contents =>
                    contents.map(c => ({ ...c, has_thumbnail: true }))
                ),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Track if thumbnail promise was resolved
                    let thumbnailResolved = false
                    let thumbnailResolve: (() => void) | null = null

                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            // Return a pending promise that we control
                            return new Promise(resolve => {
                                thumbnailResolve = () => {
                                    thumbnailResolved = true
                                    resolve({
                                        ok: true,
                                        status: 200,
                                        blob: async () => new Blob(['test'], { type: 'image/jpeg' }),
                                    })
                                }
                            })
                        }
                        // Fast content list response
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Mock URL.createObjectURL
                    vi.stubGlobal('URL', {
                        ...URL,
                        createObjectURL: vi.fn().mockReturnValue('blob:test-url'),
                        revokeObjectURL: vi.fn(),
                    })

                    // Fetch contents - should return immediately without waiting for thumbnails
                    const result = await contentStore.fetchContents(libraryId)

                    // Verify contents were returned before thumbnails finished loading
                    expect(result).toEqual(apiContents)
                    expect(thumbnailResolved).toBe(false)

                    // Clean up: resolve the pending promise to avoid unhandled rejections
                    if (thumbnailResolve) thumbnailResolve()
                }
            ),
            { numRuns: 100 }
        )
    })
})

describe('Property 10: Non-blocking asynchronous execution', () => {
    /**
     * **Feature: auto-thumbnail-fetch, Property 10: Non-blocking asynchronous execution**
     * **Validates: Requirements 4.3**
     *
     * For any content list fetch operation, the fetchContents method should return
     * the content data before all thumbnails have finished loading.
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

    it('fetchContents returns content data before thumbnails finish loading', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                nonEmptyContentsArrayArb.map(contents =>
                    contents.map(c => ({ ...c, has_thumbnail: true }))
                ),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // Track if thumbnail was resolved
                    let thumbnailResolved = false
                    let thumbnailResolve: (() => void) | null = null

                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            // Return a pending promise
                            return new Promise(resolve => {
                                thumbnailResolve = () => {
                                    thumbnailResolved = true
                                    resolve({
                                        ok: true,
                                        status: 200,
                                        blob: async () => new Blob(['test'], { type: 'image/jpeg' }),
                                    })
                                }
                            })
                        }
                        // Immediate content list response
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Mock URL.createObjectURL
                    vi.stubGlobal('URL', {
                        ...URL,
                        createObjectURL: vi.fn().mockReturnValue('blob:test-url'),
                        revokeObjectURL: vi.fn(),
                    })

                    // Fetch contents
                    const result = await contentStore.fetchContents(libraryId)

                    // Verify content was returned
                    expect(result).toEqual(apiContents)

                    // Verify thumbnails haven't finished loading yet
                    expect(thumbnailResolved).toBe(false)

                    // Clean up
                    if (thumbnailResolve) thumbnailResolve()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('loading state is false after fetchContents returns, even if thumbnails still loading', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                nonEmptyContentsArrayArb.map(contents =>
                    contents.map(c => ({ ...c, has_thumbnail: true }))
                ),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    let thumbnailResolve: (() => void) | null = null

                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            // Return a pending promise
                            return new Promise(resolve => {
                                thumbnailResolve = () => {
                                    resolve({
                                        ok: true,
                                        status: 200,
                                        blob: async () => new Blob(['test'], { type: 'image/jpeg' }),
                                    })
                                }
                            })
                        }
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Mock URL.createObjectURL
                    vi.stubGlobal('URL', {
                        ...URL,
                        createObjectURL: vi.fn().mockReturnValue('blob:test-url'),
                        revokeObjectURL: vi.fn(),
                    })

                    // Fetch contents
                    await contentStore.fetchContents(libraryId)

                    // Loading should be false immediately after fetchContents returns
                    expect(contentStore.loading).toBe(false)

                    // But thumbnails should still be loading
                    const contentsWithThumbnails = apiContents.filter(c => c.has_thumbnail)
                    if (contentsWithThumbnails.length > 0) {
                        // At least some thumbnails should be in loading state
                        const anyLoading = contentsWithThumbnails.some(c =>
                            contentStore.isThumbnailLoading(c.id)
                        )
                        expect(anyLoading).toBe(true)
                    }

                    // Clean up
                    if (thumbnailResolve) thumbnailResolve()
                }
            ),
            { numRuns: 100 }
        )
    })
})

describe('Property 11: Content list availability despite thumbnail failures', () => {
    /**
     * **Feature: auto-thumbnail-fetch, Property 11: Content list availability despite thumbnail failures**
     * **Validates: Requirements 4.5**
     *
     * For any content list where thumbnail loading fails, the content list data
     * should remain accessible and usable through the Content Store.
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

    it('content list remains accessible when all thumbnail loads fail', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                nonEmptyContentsArrayArb.map(contents =>
                    contents.map(c => ({ ...c, has_thumbnail: true }))
                ),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            // All thumbnail requests fail immediately
                            return Promise.resolve({
                                ok: false,
                                status: 500,
                                statusText: 'Internal Server Error',
                                text: async () => JSON.stringify({ error: 'Server error' }),
                            })
                        }
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Fetch contents
                    const result = await contentStore.fetchContents(libraryId)

                    // Flush microtasks to let thumbnail loading complete
                    await Promise.resolve()
                    await Promise.resolve()

                    // Content list should still be accessible
                    expect(result).toEqual(apiContents)
                    expect(contentStore.contentsByLibrary(libraryId)).toEqual(apiContents)

                    // No error should be set on the store (thumbnail failures are silent)
                    expect(contentStore.error).toBeNull()
                }
            ),
            { numRuns: 100 }
        )
    })

    it('content list remains accessible when some thumbnail loads fail', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                // Generate at least 2 contents with thumbnails
                fc.array(
                    fc.record({
                        id: contentIdArb,
                        library_id: libraryIdArb,
                        content_type: contentTypeArb,
                        title: titleArb,
                        chapter_count: fc.integer({ min: 0, max: 1000 }),
                        has_thumbnail: fc.constant(true),
                        metadata: fc.constant(null),
                        created_at: dateStringArb,
                    }),
                    { minLength: 2, maxLength: 5 }
                ).map(contents => {
                    const seen = new Set<number>()
                    return contents.filter(c => {
                        if (seen.has(c.id)) return false
                        seen.add(c.id)
                        return true
                    })
                }).filter(contents => contents.length >= 2),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    // First content's thumbnail fails, rest succeed
                    const failingId = apiContents[0].id
                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            const match = url.match(/\/contents\/(\d+)\/thumbnail/)
                            const contentId = match ? parseInt(match[1], 10) : 0

                            if (contentId === failingId) {
                                // This thumbnail fails
                                return Promise.resolve({
                                    ok: false,
                                    status: 404,
                                    statusText: 'Not Found',
                                    text: async () => JSON.stringify({ error: 'Not found' }),
                                })
                            }
                            // Other thumbnails succeed
                            return Promise.resolve({
                                ok: true,
                                status: 200,
                                blob: async () => new Blob(['test'], { type: 'image/jpeg' }),
                            })
                        }
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Mock URL.createObjectURL
                    vi.stubGlobal('URL', {
                        ...URL,
                        createObjectURL: vi.fn().mockReturnValue('blob:test-url'),
                        revokeObjectURL: vi.fn(),
                    })

                    // Fetch contents
                    const result = await contentStore.fetchContents(libraryId)

                    // Flush microtasks to let thumbnail loading complete
                    await Promise.resolve()
                    await Promise.resolve()

                    // Content list should still be fully accessible
                    expect(result).toEqual(apiContents)
                    expect(contentStore.contentsByLibrary(libraryId)).toEqual(apiContents)

                    // No error should be set on the store
                    expect(contentStore.error).toBeNull()

                    // Failed thumbnail should return null
                    expect(contentStore.getThumbnailUrl(failingId)).toBeNull()

                    // Successful thumbnails should have URLs
                    for (const content of apiContents.slice(1)) {
                        expect(contentStore.getThumbnailUrl(content.id)).toBe('blob:test-url')
                    }
                }
            ),
            { numRuns: 100 }
        )
    })

    it('content list remains accessible when thumbnail API throws network error', async () => {
        await fc.assert(
            fc.asyncProperty(
                libraryIdArb,
                nonEmptyContentsArrayArb.map(contents =>
                    contents.map(c => ({ ...c, has_thumbnail: true }))
                ),
                async (libraryId, apiContents) => {
                    setActivePinia(createPinia())

                    const authStore = useAuthStore()
                    authStore.token = 'test-token'

                    const contentStore = useContentStore()

                    const mockFetch = vi.fn().mockImplementation((url: string) => {
                        if (url.includes('/thumbnail')) {
                            // Network error
                            return Promise.reject(new Error('Network error'))
                        }
                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            headers: new Headers({ 'content-length': '100' }),
                            json: async () => apiContents,
                        })
                    })
                    vi.stubGlobal('fetch', mockFetch)

                    // Fetch contents
                    const result = await contentStore.fetchContents(libraryId)

                    // Flush microtasks to let thumbnail loading complete
                    await Promise.resolve()
                    await Promise.resolve()

                    // Content list should still be accessible
                    expect(result).toEqual(apiContents)
                    expect(contentStore.contentsByLibrary(libraryId)).toEqual(apiContents)

                    // No error should be set on the store
                    expect(contentStore.error).toBeNull()
                }
            ),
            { numRuns: 100 }
        )
    })
})
