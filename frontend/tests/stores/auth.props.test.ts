/**
 * Property-based tests for Auth Store.
 *
 * Tests the core functionality of the auth store including login atomicity,
 * logout state clearing, token persistence, and isAuthenticated getter.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import * as fc from 'fast-check'
import { setActivePinia, createPinia } from 'pinia'
import { useAuthStore, TOKEN_KEY } from '@/stores/useAuthStore'
import type { LoginResponse, UserResponse } from '@/api/types'

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// User ID generator
const userIdArb = fc.integer({ min: 1, max: 1000000 })

// Username generator - valid usernames
const usernameArb = fc.stringMatching(/^[a-zA-Z][a-zA-Z0-9_]{2,19}$/)

// Token generator - JWT-like format
const tokenArb = fc.stringMatching(/^[A-Za-z0-9_-]{10,50}\.[A-Za-z0-9_-]{10,50}\.[A-Za-z0-9_-]{10,50}$/)

// Non-empty token generator for isAuthenticated tests
const nonEmptyTokenArb = fc.string({ minLength: 1, maxLength: 100 })

// Bangumi API key generator (nullable)
const bangumiApiKeyArb = fc.option(fc.string({ minLength: 10, maxLength: 50 }), { nil: null })

// ISO date string generator - use integer timestamps to avoid invalid date issues
const dateStringArb = fc.integer({ min: 1577836800000, max: 1924905600000 }) // 2020-01-01 to 2030-12-31
  .map(ts => new Date(ts).toISOString())

// UserResponse generator
const userResponseArb = fc.record({
  id: userIdArb,
  username: usernameArb,
  bangumi_api_key: bangumiApiKeyArb,
  created_at: dateStringArb,
})

// LoginResponse generator
const loginResponseArb = fc.record({
  user: userResponseArb,
  token: tokenArb,
})

// ============================================================================
// Test Setup
// ============================================================================

// Mock localStorage
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

describe('Property 1: Login updates token and user atomically', () => {
  /**
   * **Feature: pinia-state-management, Property 1: Login updates token and user atomically**
   * **Validates: Requirements 2.2**
   *
   * For any valid login response containing token and user data, after calling
   * the login action, both the store's token and user state should match the response values.
   */

  beforeEach(() => {
    vi.stubGlobal('localStorage', localStorageMock)
    localStorageMock.clear()
    setActivePinia(createPinia())
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('login action updates both token and user to match response', async () => {
    await fc.assert(
      fc.asyncProperty(loginResponseArb, usernameArb, fc.string({ minLength: 1 }), async (loginResponse, username, password) => {
        // Mock the auth API
        vi.doMock('@/api/auth', () => ({
          createAuthApi: () => ({
            login: vi.fn().mockResolvedValue(loginResponse),
          }),
        }))

        // Create fresh store
        setActivePinia(createPinia())
        const store = useAuthStore()

        // Mock fetch to return the login response
        const mockFetch = vi.fn().mockResolvedValue({
          ok: true,
          status: 200,
          headers: new Headers({ 'content-length': '100' }),
          json: async () => loginResponse,
        })
        vi.stubGlobal('fetch', mockFetch)

        // Perform login
        const result = await store.login(username, password)

        // Verify token and user are updated atomically
        expect(store.token).toBe(loginResponse.token)
        expect(store.user).toEqual(loginResponse.user)
        expect(result.token).toBe(loginResponse.token)
        expect(result.user).toEqual(loginResponse.user)
      }),
      { numRuns: 100 }
    )
  })

  it('login action persists token to localStorage', async () => {
    await fc.assert(
      fc.asyncProperty(loginResponseArb, usernameArb, fc.string({ minLength: 1 }), async (loginResponse, username, password) => {
        setActivePinia(createPinia())
        const store = useAuthStore()

        // Mock fetch
        const mockFetch = vi.fn().mockResolvedValue({
          ok: true,
          status: 200,
          headers: new Headers({ 'content-length': '100' }),
          json: async () => loginResponse,
        })
        vi.stubGlobal('fetch', mockFetch)

        await store.login(username, password)

        // Verify token is persisted
        expect(localStorageMock.setItem).toHaveBeenCalledWith(TOKEN_KEY, loginResponse.token)
      }),
      { numRuns: 100 }
    )
  })
})


describe('Property 2: Logout clears all auth state', () => {
  /**
   * **Feature: pinia-state-management, Property 2: Logout clears all auth state**
   * **Validates: Requirements 2.3**
   *
   * For any auth store state with token and user set, calling logout should result
   * in token being null, user being null, and localStorage not containing the auth token key.
   */

  beforeEach(() => {
    vi.stubGlobal('localStorage', localStorageMock)
    localStorageMock.clear()
    setActivePinia(createPinia())
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('logout clears token, user, and localStorage', async () => {
    await fc.assert(
      fc.asyncProperty(loginResponseArb, usernameArb, fc.string({ minLength: 1 }), async (loginResponse, username, password) => {
        setActivePinia(createPinia())
        const store = useAuthStore()

        // Mock fetch for login
        const mockFetch = vi.fn().mockResolvedValue({
          ok: true,
          status: 200,
          headers: new Headers({ 'content-length': '100' }),
          json: async () => loginResponse,
        })
        vi.stubGlobal('fetch', mockFetch)

        // First login to set state
        await store.login(username, password)

        // Verify state is set
        expect(store.token).toBe(loginResponse.token)
        expect(store.user).toEqual(loginResponse.user)

        // Now logout
        store.logout()

        // Verify all state is cleared
        expect(store.token).toBeNull()
        expect(store.user).toBeNull()
        expect(localStorageMock.removeItem).toHaveBeenCalledWith(TOKEN_KEY)
      }),
      { numRuns: 100 }
    )
  })

  it('logout is idempotent - calling multiple times has same effect', () => {
    fc.assert(
      fc.property(fc.integer({ min: 1, max: 10 }), (callCount) => {
        setActivePinia(createPinia())
        const store = useAuthStore()

        // Call logout multiple times
        for (let i = 0; i < callCount; i++) {
          store.logout()
        }

        // State should always be cleared
        expect(store.token).toBeNull()
        expect(store.user).toBeNull()
      }),
      { numRuns: 100 }
    )
  })
})

describe('Property 3: Token persistence round-trip', () => {
  /**
   * **Feature: pinia-state-management, Property 3: Token persistence round-trip**
   * **Validates: Requirements 2.4, 2.5**
   *
   * For any token string stored via the auth store, creating a new store instance
   * should restore that same token from localStorage.
   */

  beforeEach(() => {
    vi.stubGlobal('localStorage', localStorageMock)
    localStorageMock.clear()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('token stored in localStorage is restored on store initialization', async () => {
    await fc.assert(
      fc.asyncProperty(loginResponseArb, usernameArb, fc.string({ minLength: 1 }), async (loginResponse, username, password) => {
        // First store instance - login and store token
        setActivePinia(createPinia())
        const store1 = useAuthStore()

        // Mock fetch for login
        const mockFetch = vi.fn().mockResolvedValue({
          ok: true,
          status: 200,
          headers: new Headers({ 'content-length': '100' }),
          json: async () => loginResponse,
        })
        vi.stubGlobal('fetch', mockFetch)

        await store1.login(username, password)

        // Verify token is stored
        expect(localStorageMock.store[TOKEN_KEY]).toBe(loginResponse.token)

        // Create new Pinia instance (simulating app restart)
        setActivePinia(createPinia())
        const store2 = useAuthStore()

        // Token should be restored from localStorage
        expect(store2.token).toBe(loginResponse.token)
      }),
      { numRuns: 100 }
    )
  })

  it('token set directly in localStorage is read on initialization', () => {
    fc.assert(
      fc.property(tokenArb, (token) => {
        // Set token directly in localStorage
        localStorageMock.store[TOKEN_KEY] = token

        // Create store - should read from localStorage
        setActivePinia(createPinia())
        const store = useAuthStore()

        expect(store.token).toBe(token)

        // Clean up
        localStorageMock.clear()
      }),
      { numRuns: 100 }
    )
  })
})

describe('Property 4: isAuthenticated reflects token presence', () => {
  /**
   * **Feature: pinia-state-management, Property 4: isAuthenticated reflects token presence**
   * **Validates: Requirements 2.6**
   *
   * For any token state (null or non-null string), the isAuthenticated getter
   * should return true if and only if token is a non-empty string.
   */

  beforeEach(() => {
    vi.stubGlobal('localStorage', localStorageMock)
    localStorageMock.clear()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('isAuthenticated is true when token is non-empty string', () => {
    fc.assert(
      fc.property(nonEmptyTokenArb, (token) => {
        // Set token in localStorage before creating store
        localStorageMock.store[TOKEN_KEY] = token

        setActivePinia(createPinia())
        const store = useAuthStore()

        expect(store.isAuthenticated).toBe(true)

        // Clean up
        localStorageMock.clear()
      }),
      { numRuns: 100 }
    )
  })

  it('isAuthenticated is false when token is null', () => {
    fc.assert(
      fc.property(fc.constant(null), () => {
        // Ensure no token in localStorage
        localStorageMock.clear()

        setActivePinia(createPinia())
        const store = useAuthStore()

        expect(store.isAuthenticated).toBe(false)
      }),
      { numRuns: 100 }
    )
  })

  it('isAuthenticated changes correctly after login and logout', async () => {
    await fc.assert(
      fc.asyncProperty(loginResponseArb, usernameArb, fc.string({ minLength: 1 }), async (loginResponse, username, password) => {
        setActivePinia(createPinia())
        const store = useAuthStore()

        // Initially not authenticated
        expect(store.isAuthenticated).toBe(false)

        // Mock fetch for login
        const mockFetch = vi.fn().mockResolvedValue({
          ok: true,
          status: 200,
          headers: new Headers({ 'content-length': '100' }),
          json: async () => loginResponse,
        })
        vi.stubGlobal('fetch', mockFetch)

        // After login - authenticated
        await store.login(username, password)
        expect(store.isAuthenticated).toBe(true)

        // After logout - not authenticated
        store.logout()
        expect(store.isAuthenticated).toBe(false)
      }),
      { numRuns: 100 }
    )
  })
})
