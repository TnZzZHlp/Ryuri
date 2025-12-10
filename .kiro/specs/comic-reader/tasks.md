# Implementation Plan

## Phase 1: Backend Foundation

- [x] 1. Set up Rust backend project structure and dependencies





  - [x] 1.1 Configure Cargo.toml with required dependencies (axum, sqlx, tokio, serde, thiserror, jsonwebtoken, etc.)






    - _Requirements: 7.1, 10.1_
  - [x] 1.2 Create module structure (models, handlers, services, repository, extractors)


    - _Requirements: 7.1_



  - [x] 1.3 Implement AppError type and error handling infrastructure
    - _Requirements: 7.5_
  - [x] 1.4 Write property test for error response structure

    - **Property 17: Error Response Structure**
    - **Validates: Requirements 7.5**

- [x] 2. Implement database layer and models



















  - [x] 2.1 Create SQLite database initialization and migration logic





    - _Requirements: 10.1_
  - [x] 2.2 Implement data models (Library, ScanPath, Content, Chapter, User, ReadingProgress)


    - _Requirements: 1.1, 2.1, 9.1, 5.1_
  - [x] 2.3 Implement ContentType enum with serialization


    - _Requirements: 2.1_
  - [x] 2.4 Write property test for JSON serialization round-trip


    - **Property 18: JSON Serialization Round-Trip**
    - **Validates: Requirements 8.2, 8.3, 8.4**

- [x] 3. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: User Authentication

- [x] 4. Implement authentication service








  - [x] 4.1 Implement password hashing with argon2


    - _Requirements: 9.1, 9.5_

  - [x] 4.2 Implement JWT token generation and verification

    - _Requirements: 9.2, 9.4_
  - [x] 4.3 Implement AuthService (register, login, verify_token, update_password, update_user)


    - _Requirements: 9.1, 9.2, 9.3, 9.5_


  - [x] 4.4 Implement auth middleware for protected routes
    - _Requirements: 9.4_
  - [x] 4.5 Write property test for password hashing security

    - **Property 23: Password Hashing Security**
    - **Validates: Requirements 9.1, 9.2**
  - [x] 4.6 Write property test for JWT token validity

    - **Property 24: JWT Token Validity**
    - **Validates: Requirements 9.2, 9.4**
  - [x] 4.7 Write property test for user registration uniqueness

    - **Property 22: User Registration Uniqueness**
    - **Validates: Requirements 9.1**

- [x] 5. Implement auth API handlers





  - [x] 5.1 Implement POST /api/auth/login handler


    - _Requirements: 9.2, 9.3_

  - [x] 5.2 Implement GET /api/auth/me and PUT /api/auth/me handlers

    - _Requirements: 9.4_

  - [x] 5.3 Implement PUT /api/auth/password handler

    - _Requirements: 9.5_

- [x] 6. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Library Management

- [x] 7. Implement library service and repository





  - [x] 7.1 Implement LibraryService (create, get, list, update, delete)


    - _Requirements: 1.1, 1.4, 1.6, 1.7_

  - [x] 7.2 Implement ScanPath management (add, remove, list)

    - _Requirements: 1.2, 1.3_

  - [x] 7.3 Write property test for library CRUD round-trip

    - **Property 1: Library CRUD Round-Trip**
    - **Validates: Requirements 1.1, 1.7**
  - [x] 7.4 Write property test for scan path association integrity


    - **Property 2: Scan Path Association Integrity**
    - **Validates: Requirements 1.2**

  - [x] 7.5 Write property test for cascade deletion - scan path removal

    - **Property 3: Cascade Deletion - Scan Path Removal**
    - **Validates: Requirements 1.3**

  - [x] 7.6 Write property test for library statistics accuracy

    - **Property 4: Library Statistics Accuracy**
    - **Validates: Requirements 1.4**

  - [x] 7.7 Write property test for cascade deletion - library removal

    - **Property 6: Cascade Deletion - Library Removal**
    - **Validates: Requirements 1.6**

- [x] 8. Implement library API handlers






  - [x] 8.1 Implement GET/POST /api/libraries handlers

    - _Requirements: 1.1, 1.4_

  - [x] 8.2 Implement GET/PUT/DELETE /api/libraries/{id} handlers
    - _Requirements: 1.6, 1.7_

  - [x] 8.3 Implement scan path management endpoints
    - _Requirements: 1.2, 1.3_

- [x] 9. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Content Scanning and Import

- [x] 10. Implement archive extractors





  - [x] 10.1 Implement ComicArchiveExtractor (ZIP, CBZ, CBR, RAR support)


    - _Requirements: 2.2, 3.2_

  - [x] 10.2 Implement NovelArchiveExtractor (ZIP, EPUB, TXT support)

    - _Requirements: 2.3, 4.2_
  - [x] 10.3 Write property test for chapter sorting consistency


    - **Property 7: Chapter Sorting Consistency**
    - **Validates: Requirements 2.2, 2.3**

- [x] 11. Implement scan service





  - [x] 11.1 Implement ScanService for scanning library paths


    - _Requirements: 2.1_

  - [x] 11.2 Implement content folder detection and chapter identification
    - _Requirements: 2.2, 2.3, 2.4_
  - [x] 11.3 Implement thumbnail generation for comics (first page of first chapter)

    - _Requirements: 2.5_

  - [x] 11.4 Implement default thumbnail handling for novels
    - _Requirements: 2.6_
  - [x] 11.5 Write property test for content title derivation


    - **Property 8: Content Title Derivation**
    - **Validates: Requirements 2.4**
  - [x] 11.6 Write property test for content-scanpath association

    - **Property 9: Content-ScanPath Association**
    - **Validates: Requirements 2.7**

- [x] 12. Implement content service





  - [x] 12.1 Implement ContentService (get, list, search, delete, list_chapters)


    - _Requirements: 2.8, 2.9, 2.10_

  - [x] 12.2 Implement get_page for comic image extraction

    - _Requirements: 3.2, 7.2_
  - [x] 12.3 Implement get_chapter_text for novel text extraction


    - _Requirements: 4.2, 7.3_

  - [x] 12.4 Write property test for content retrieval completeness

    - **Property 5: Content Retrieval Completeness**
    - **Validates: Requirements 1.5**
  - [x] 12.5 Write property test for content deletion cascade



    - **Property 10: Content Deletion Cascade**

    - **Validates: Requirements 2.9**
  - [x] 12.6 Write property test for search result relevance

    - **Property 11: Search Result Relevance**
    - **Validates: Requirements 2.10**

  - [x] 12.7 Write property test for image ordering consistency

    - **Property 12: Image Ordering Consistency**
    - **Validates: Requirements 3.2**

- [x] 13. Implement content API handlers



  - [x] 13.1 Implement GET /api/libraries/{id}/contents handler


    - _Requirements: 1.5, 2.8_


  - [x] 13.2 Implement POST /api/libraries/{id}/scan handler

    - _Requirements: 2.1_
  - [x] 13.3 Implement GET /api/libraries/{id}/search handler

    - _Requirements: 2.10_


  - [x] 13.4 Implement GET/DELETE /api/contents/{id} handlers

    - _Requirements: 2.9_

  - [x] 13.5 Implement GET /api/contents/{id}/chapters handler
    - _Requirements: 3.1, 4.1_
  - [x] 13.6 Implement GET /api/contents/{id}/chapters/{chapter}/pages/{page} handler


    - _Requirements: 3.2, 7.2_


  - [x] 13.7 Implement GET /api/contents/{id}/chapters/{chapter}/text handler
    - _Requirements: 4.2, 7.3_
  - [x] 13.8 Write property test for API response completeness



    - **Property 15: API Response Completeness**
    - **Validates: Requirements 7.1**

- [x] 14. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Reading Progress

- [x] 15. Implement progress service (chapter-based tracking)











  - [x] 15.1 Implement ProgressService (get_chapter_progress, get_content_progress, update_progress)

    - _Requirements: 5.1, 5.2, 5.3_
  - [x] 15.2 Implement progress percentage calculation (per-chapter and overall content)

    - _Requirements: 5.4_
  - [x] 15.3 Write property test for progress persistence round-trip

    - **Property 13: Progress Persistence Round-Trip**
    - **Validates: Requirements 3.5, 4.5, 5.1**
  - [x] 15.4 Write property test for progress percentage accuracy





    - **Property 14: Progress Percentage Accuracy**
    - **Validates: Requirements 5.4**
  - [x] 15.5 Write property test for progress user isolation





    - **Property 25: Progress User Isolation**
    - **Validates: Requirements 9.7**
  - [x] 15.6 Write property test for progress validation





    - **Property 16: Progress Validation**
    - **Validates: Requirements 7.4**


- [x] 16. Implement progress API handlers





  - [x] 16.1 Implement GET /api/contents/{id}/progress handler (overall content progress)

    - _Requirements: 5.1_

  - [x] 16.2 Implement GET/PUT /api/chapters/{id}/progress handlers (chapter-level progress)

    - _Requirements: 5.2, 5.3, 7.4_

- [x] 17. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Bangumi Metadata Integration


- [x] 18. Implement Bangumi service




  - [x] 18.1 Implement BangumiService (search, get_subject, auto_scrape)


    - _Requirements: 8.1, 8.4_

  - [x] 18.2 Integrate auto-scraping into scan service

    - _Requirements: 8.1, 8.2, 8.3_

  - [x] 18.3 Implement metadata update for content

    - _Requirements: 8.5, 8.7_

  - [x] 18.4 Write property test for metadata JSON blob storage

    - **Property 21: Metadata JSON Blob Storage**
    - **Validates: Requirements 8.4, 8.6**

- [x] 19. Implement Bangumi API handlers





  - [x] 19.1 Implement GET /api/bangumi/search handler


    - _Requirements: 8.4_
  - [x] 19.2 Implement PUT /api/contents/{id}/metadata handler


    - _Requirements: 8.5, 8.7_


- [x] 20. Checkpoint - Ensure all tests pass




  - Ensure all tests pass, ask the user if questions arise.

## Phase 7: Watch Mode and Scheduled Scanning


- [x] 21. Implement watch and scheduler services




  - [x] 21.1 Implement WatchService for file system monitoring


    - _Requirements: 1.9, 1.10, 1.11_

  - [x] 21.2 Implement SchedulerService for periodic scanning

    - _Requirements: 1.8_

  - [x] 21.3 Integrate watch/scheduler with library lifecycle

    - _Requirements: 1.8, 1.9_
  - [x] 21.4 Write property test for scan interval configuration


    - **Property 19: Scan Interval Configuration**
    - **Validates: Requirements 1.8**
  - [x] 21.5 Write property test for watch mode state consistency

    - **Property 20: Watch Mode State Consistency**
    - **Validates: Requirements 1.9, 1.10, 1.11**

- [x] 22. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

## Phase 8: Backend Integration and Router Setup

- [x] 23. Wire up Axum router and application state

  - [x] 23.1 Create AppState with all services
    - _Requirements: 7.1_
  - [x] 23.2 Configure complete router with all routes
    - _Requirements: 7.1_
  - [x] 23.3 Add CORS and middleware configuration
    - _Requirements: 7.1_
  - [x] 23.4 Implement main.rs with server startup
    - _Requirements: 10.1_

- [x] 24. Integrate tracing for structured logging





  - [x] 24.1 Add tracing and tracing-subscriber dependencies to Cargo.toml


    - _Requirements: 7.1_
  - [x] 24.2 Initialize tracing subscriber in main.rs with env-filter support


    - _Requirements: 10.1_
  - [x] 24.3 Replace println! statements with tracing macros (info!, debug!, warn!, error!)


    - _Requirements: 7.1_
  - [x] 24.4 Add #[instrument] attributes to key service functions for request tracing


    - _Requirements: 7.1_

- [ ] 25. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
