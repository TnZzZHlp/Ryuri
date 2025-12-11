/**
 * UI Store - Pinia store for global UI state management
 *
 * Manages global UI state including sidebar collapsed status.
 *
 * **Implements: Requirements 4.1, 4.2, 4.3**
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUIStore = defineStore('ui', () => {
  // State
  const sidebarCollapsed = ref(false)

  // Actions

  /**
   * Toggles the sidebar collapsed state.
   * **Implements: Requirement 4.2**
   */
  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  /**
   * Sets the sidebar collapsed state to a specific value.
   * **Implements: Requirement 4.2**
   */
  function setSidebarCollapsed(collapsed: boolean): void {
    sidebarCollapsed.value = collapsed
  }

  return {
    // State
    sidebarCollapsed,
    // Actions
    toggleSidebar,
    setSidebarCollapsed,
  }
})
