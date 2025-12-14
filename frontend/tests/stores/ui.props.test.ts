/**
 * Property-based tests for UI Store.
 *
 * Tests the core functionality of the UI store including sidebar toggle behavior.
 */

import { describe, it, expect, beforeEach } from 'vitest'
import * as fc from 'fast-check'
import { setActivePinia, createPinia } from 'pinia'
import { useUIStore } from '@/stores/useUIStore'

// ============================================================================
// Property Tests
// ============================================================================

describe('Property 8: Sidebar toggle is self-inverse', () => {
  /**
   * **Feature: pinia-state-management, Property 8: Sidebar toggle is self-inverse**
   * **Validates: Requirements 4.2**
   *
   * For any initial sidebar collapsed state, calling toggleSidebar twice
   * should return to the original state (toggle is its own inverse).
   */

  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('toggleSidebar twice returns to original state', () => {
    fc.assert(
      fc.property(fc.boolean(), (initialState) => {
        setActivePinia(createPinia())
        const store = useUIStore()

        // Set initial state
        store.setSidebarCollapsed(initialState)
        expect(store.sidebarCollapsed).toBe(initialState)

        // Toggle twice
        store.toggleSidebar()
        store.toggleSidebar()

        // Should return to original state
        expect(store.sidebarCollapsed).toBe(initialState)
      }),
      { numRuns: 100 }
    )
  })

  it('toggleSidebar inverts the current state', () => {
    fc.assert(
      fc.property(fc.boolean(), (initialState) => {
        setActivePinia(createPinia())
        const store = useUIStore()

        // Set initial state
        store.setSidebarCollapsed(initialState)

        // Toggle once
        store.toggleSidebar()

        // Should be inverted
        expect(store.sidebarCollapsed).toBe(!initialState)
      }),
      { numRuns: 100 }
    )
  })

  it('multiple toggles follow the pattern: even toggles = original, odd toggles = inverted', () => {
    fc.assert(
      fc.property(fc.boolean(), fc.integer({ min: 0, max: 20 }), (initialState, toggleCount) => {
        setActivePinia(createPinia())
        const store = useUIStore()

        // Set initial state
        store.setSidebarCollapsed(initialState)

        // Toggle n times
        for (let i = 0; i < toggleCount; i++) {
          store.toggleSidebar()
        }

        // Even number of toggles = original state, odd = inverted
        const expectedState = toggleCount % 2 === 0 ? initialState : !initialState
        expect(store.sidebarCollapsed).toBe(expectedState)
      }),
      { numRuns: 100 }
    )
  })

  it('setSidebarCollapsed sets the exact value provided', () => {
    fc.assert(
      fc.property(fc.boolean(), fc.boolean(), (initialState, newState) => {
        setActivePinia(createPinia())
        const store = useUIStore()

        // Set initial state
        store.setSidebarCollapsed(initialState)

        // Set new state
        store.setSidebarCollapsed(newState)

        // Should be exactly the new state
        expect(store.sidebarCollapsed).toBe(newState)
      }),
      { numRuns: 100 }
    )
  })
})
