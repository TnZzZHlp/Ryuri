# Implementation Plan

- [x] 1. Set up API layer infrastructure





  - [x] 1.1 Create API directory structure and install dependencies


    - Create `frontend/src/api/` directory
    - Install fast-check for property-based testing: `pnpm add -D fast-check`
    - _Requirements: 1.1_
  - [x] 1.2 Implement TypeScript type definitions (`types.ts`)


    - Define all request/response interfaces matching backend schemas
    - Define ContentType enum
    - Define ApiError class
    - _Requirements: 8.1, 8.2, 8.3_
  - [x] 1.3 Write property test for type serialization round-trip


    - **Property 4: Type Serialization Round-Trip**
    - **Validates: Requirements 1.4, 1.5, 1.6, 8.1, 8.4, 8.5**
  - [x] 1.4 Write property test for ContentType enum serialization


    - **Property 5: ContentType Enum Serialization**
    - **Validates: Requirements 8.2**


- [x] 2. Implement API Client




  - [x] 2.1 Implement base API client (`client.ts`)


    - Create ApiClient class with configurable base URL
    - Implement HTTP methods (get, post, put, delete)
    - Implement JSON serialization/deserialization
    - Implement authentication header injection
    - Implement error response parsing
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

  - [x] 2.2 Write property test for URL construction

    - **Property 1: Request URL Construction**
    - **Validates: Requirements 1.1**
  - [x] 2.3 Write property test for authentication header injection

    - **Property 2: Authentication Header Injection**
    - **Validates: Requirements 1.2**

  - [x] 2.4 Write property test for error response parsing
    - **Property 3: Error Response Parsing**
    - **Validates: Requirements 1.3**

- [x] 3. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement Auth API






  - [x] 4.1 Implement authentication API module (`auth.ts`)

    - Implement login function
    - Implement getMe function
    - Implement updateMe function
    - Implement updatePassword function
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [x] 4.2 Write unit tests for Auth API

    - Test login request/response handling
    - Test profile retrieval
    - Test profile update
    - Test password change
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 5. Implement Library API






  - [x] 5.1 Implement library API module (`library.ts`)

    - Implement list function
    - Implement create function
    - Implement get function
    - Implement update function
    - Implement delete function
    - Implement listScanPaths function
    - Implement addScanPath function
    - Implement removeScanPath function
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8_

  - [x] 5.2 Write unit tests for Library API

    - Test library CRUD operations
    - Test scan path operations
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8_

- [x] 6. Implement Content API






  - [x] 6.1 Implement content API module (`content.ts`)

    - Implement list function
    - Implement search function
    - Implement get function
    - Implement delete function
    - Implement updateMetadata function
    - Implement listChapters function
    - Implement triggerScan function
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

  - [x] 6.2 Write unit tests for Content API

    - Test content listing and search
    - Test content CRUD operations
    - Test chapter listing
    - Test scan triggering
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [x] 7. Implement Reader API






  - [x] 7.1 Implement reader API module (`reader.ts`)

    - Implement getPageUrl function (returns URL string, no HTTP call)
    - Implement getChapterText function
    - _Requirements: 5.1, 5.2_

  - [x] 7.2 Write unit tests for Reader API

    - Test page URL generation
    - Test chapter text retrieval
    - _Requirements: 5.1, 5.2_

- [x] 8. Implement Progress API






  - [x] 8.1 Implement progress API module (`progress.ts`)

    - Implement getContentProgress function
    - Implement getChapterProgress function
    - Implement updateChapterProgress function
    - _Requirements: 6.1, 6.2, 6.3_

  - [x] 8.2 Write unit tests for Progress API

    - Test progress retrieval
    - Test progress update
    - _Requirements: 6.1, 6.2, 6.3_


- [x] 9. Implement Bangumi API





  - [x] 9.1 Implement Bangumi API module (`bangumi.ts`)

    - Implement search function
    - _Requirements: 7.1_

  - [x] 9.2 Write unit tests for Bangumi API

    - Test search functionality
    - _Requirements: 7.1_


- [x] 10. Create unified export and finalize




  - [x] 10.1 Create index.ts with unified exports


    - Export all API modules
    - Export all types
    - Create and export default API instance
    - _Requirements: 1.1_


- [x] 11. Final Checkpoint - Ensure all tests pass




  - Ensure all tests pass, ask the user if questions arise.
