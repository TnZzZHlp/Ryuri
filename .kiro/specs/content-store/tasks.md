# Implementation Plan

- [x] 1. Create useContentStore with core state and actions



  - [x] 1.1 Create `src/stores/useContentStore.ts` with state structure


    - Implement state: contents (Map), currentContent, chapters (Map), loading, error
    - Implement getters: contentsByLibrary, currentChapters
    - _Requirements: 1.1, 1.2, 1.3_


  - [x] 1.2 Implement fetchContents action with caching
    - Fetch contents from API and cache by libraryId
    - Support force refresh parameter
    - Return cached data when available and force is false
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 1.3 Write property test for content cache consistency


    - **Property 1: Content cache consistency**
    - **Validates: Requirements 2.1**

  - [x] 1.4 Write property test for cache hit behavior

    - **Property 2: Cache hit avoids API call**
    - **Validates: Requirements 2.2**


  - [x] 1.5 Write property test for force refresh

    - **Property 3: Force refresh updates cache**
    - **Validates: Requirements 2.3**


- [x] 2. Implement content selection and chapter management



  - [x] 2.1 Implement selectContent action

    - Set currentContent to selected content
    - Fetch and cache chapters for the content
    - Use cached chapters if available
    - _Requirements: 4.1, 4.2, 4.3_


  - [x] 2.2 Implement clearCurrentContent action
    - Clear currentContent state
    - _Requirements: 4.1_


  - [x] 2.3 Write property test for content selection

    - **Property 4: Content selection updates state atomically**
    - **Validates: Requirements 4.1, 4.2**


  - [x] 2.4 Write property test for chapter cache hit

    - **Property 5: Chapter cache hit avoids API call**
    - **Validates: Requirements 4.3**

- [x] 3. Implement search and delete functionality





  - [x] 3.1 Implement searchContents action


    - Call search API with query
    - Return cached contents for empty query
    - _Requirements: 3.1, 3.2_


  - [x] 3.2 Implement deleteContent action

    - Delete content via API
    - Remove from cache
    - Clear currentContent if it was the deleted content
    - _Requirements: 5.1, 5.2_


  - [x] 3.3 Implement invalidateCache action

    - Clear cache for specific libraryId or all caches
    - _Requirements: 2.3_


  - [x] 3.4 Write property test for content deletion

    - **Property 6: Content deletion removes from cache**
    - **Validates: Requirements 5.1, 5.2**

- [x] 4. Implement error handling





  - [x] 4.1 Add error clearing at start of each action


    - Clear error state before API calls
    - Set error state on failures
    - _Requirements: 6.1, 6.2_

  - [x] 4.2 Write property test for error clearing


    - **Property 7: Error cleared on new operation**
    - **Validates: Requirements 6.2**


- [x] 5. Checkpoint - Ensure all tests pass




  - Ensure all tests pass, ask the user if questions arise.
