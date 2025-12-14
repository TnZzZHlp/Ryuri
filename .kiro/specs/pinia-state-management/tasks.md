# Implementation Plan

- [x] 1. Install and configure Pinia





  - [x] 1.1 Install Pinia package


    - Run `pnpm add pinia` in frontend directory
    - _Requirements: 1.1_
  - [x] 1.2 Configure Pinia in main.ts


    - Import createPinia and register as Vue plugin before router
    - _Requirements: 1.1, 1.2_

- [x] 2. Implement Auth Store






  - [x] 2.1 Create useAuthStore with state and actions

    - Create `src/stores/useAuthStore.ts`
    - Implement state: token, user, loading, error
    - Implement actions: login, logout, fetchUser, updateUser, updatePassword
    - Implement getter: isAuthenticated
    - Restore token from localStorage on initialization
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - [x] 2.2 Write property test for login atomicity


    - **Property 1: Login updates token and user atomically**
    - **Validates: Requirements 2.2**
  - [x] 2.3 Write property test for logout clearing state

    - **Property 2: Logout clears all auth state**
    - **Validates: Requirements 2.3**
  - [x] 2.4 Write property test for token persistence round-trip

    - **Property 3: Token persistence round-trip**
    - **Validates: Requirements 2.4, 2.5**
  - [x] 2.5 Write property test for isAuthenticated getter

    - **Property 4: isAuthenticated reflects token presence**
    - **Validates: Requirements 2.6**


- [x] 3. Implement Library Store




  - [x] 3.1 Create useLibraryStore with state and actions


    - Create `src/stores/useLibraryStore.ts`
    - Implement state: libraries, currentLibrary, loading, error
    - Implement actions: fetchLibraries, selectLibrary, createLibrary, updateLibrary, deleteLibrary
    - Implement getter: libraryById
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - [x] 3.2 Write property test for library cache consistency


    - **Property 5: Library cache consistency**
    - **Validates: Requirements 3.2**
  - [x] 3.3 Write property test for library selection


    - **Property 6: Library selection updates current library**
    - **Validates: Requirements 3.3**
  - [x] 3.4 Write property test for library deletion


    - **Property 7: Library deletion removes from cache**
    - **Validates: Requirements 3.5**

- [x] 4. Implement UI Store





  - [x] 4.1 Create useUIStore with state and actions


    - Create `src/stores/useUIStore.ts`
    - Implement state: sidebarCollapsed
    - Implement actions: toggleSidebar, setSidebarCollapsed
    - _Requirements: 4.1, 4.2, 4.3_
  - [x] 4.2 Write property test for sidebar toggle


    - **Property 8: Sidebar toggle is self-inverse**
    - **Validates: Requirements 4.2**


- [x] 5. Checkpoint - Ensure all tests pass




  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Refactor components to use Pinia stores





  - [x] 6.1 Refactor AppSidebar.vue to use Auth Store


    - Replace useAuth() with useAuthStore()
    - Fix the `data.user` bug by using store.user directly
    - Remove unused icon imports
    - _Requirements: 5.1_


  - [x] 6.2 Update NavUser.vue props to work with store user type


    - Adjust User interface to match UserResponse from store






    - _Requirements: 5.2_




  - [ ] 6.3 Refactor Login.vue to use Auth Store
    - Replace useAuth() with useAuthStore()
    - Use store.login action and store.loading state
    - _Requirements: 5.3_

- [ ] 7. Clean up deprecated code

  - [ ] 7.1 Deprecate useAuth composable
    - Add deprecation comment to useAuth.ts
    - Keep for backward compatibility during transition
    - _Requirements: 5.4_

- [ ] 8. Final Checkpoint - Ensure all tests pass

  - Ensure all tests pass, ask the user if questions arise.
