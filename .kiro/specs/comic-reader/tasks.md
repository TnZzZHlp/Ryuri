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

- [ ] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: User Authentication

- [ ] 4. Implement authentication service
  - [ ] 4.1 Implement password hashing with argon2/bcrypt
    - _Requirements: 9.1, 9.5_
  - [ ] 4.2 Implement JWT token generation and verification
    - _Requirements: 9.2, 9.4_
  - [ ] 4.3 Implement AuthService (register, login, verify_token, update_password, update_user)
    - _Requirements: 9.1, 9.2, 9.3, 9.5_
  - [ ] 4.4 Implement auth middleware for protected routes
    - _Requirements: 9.4_
  - [ ] 4.5 Write property test for password hashing security
    - **Property 23: Password Hashing Security**
    - **Validates: Requirements 9.1, 9.2**
  - [ ] 4.6 Write property test for JWT token validity
    - **Property 24: JWT Token Validity**
    - **Validates: Requirements 9.2, 9.4**
  - [ ] 4.7 Write property test for user registration uniqueness
    - **Property 22: User Registration Uniqueness**
    - **Validates: Requirements 9.1**

- [ ] 5. Implement auth API handlers
  - [ ] 5.1 Implement POST /api/auth/login handler
    - _Requirements: 9.2, 9.3_
  - [ ] 5.2 Implement GET /api/auth/me and PUT /api/auth/me handlers
    - _Requirements: 9.4_
  - [ ] 5.3 Implement PUT /api/auth/password handler
    - _Requirements: 9.5_

- [ ] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Library Management

- [ ] 7. Implement library service and repository
  - [ ] 7.1 Implement LibraryService (create, get, list, update, delete)
    - _Requirements: 1.1, 1.4, 1.6, 1.7_
  - [ ] 7.2 Implement ScanPath management (add, remove, list)
    - _Requirements: 1.2, 1.3_
  - [ ] 7.3 Write property test for library CRUD round-trip
    - **Property 1: Library CRUD Round-Trip**
    - **Validates: Requirements 1.1, 1.7**
  - [ ] 7.4 Write property test for scan path association integrity
    - **Property 2: Scan Path Association Integrity**
    - **Validates: Requirements 1.2**
  - [ ] 7.5 Write property test for cascade deletion - scan path removal
    - **Property 3: Cascade Deletion - Scan Path Removal**
    - **Validates: Requirements 1.3**
  - [ ] 7.6 Write property test for library statistics accuracy
    - **Property 4: Library Statistics Accuracy**
    - **Validates: Requirements 1.4**
  - [ ] 7.7 Write property test for cascade deletion - library removal
    - **Property 6: Cascade Deletion - Library Removal**
    - **Validates: Requirements 1.6**

- [ ] 8. Implement library API handlers
  - [ ] 8.1 Implement GET/POST /api/libraries handlers
    - _Requirements: 1.1, 1.4_
  - [ ] 8.2 Implement GET/PUT/DELETE /api/libraries/{id} handlers
    - _Requirements: 1.6, 1.7_
  - [ ] 8.3 Implement scan path management endpoints
    - _Requirements: 1.2, 1.3_

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Content Scanning and Import

- [ ] 10. Implement archive extractors
  - [ ] 10.1 Implement ComicArchiveExtractor (ZIP, CBZ, CBR, RAR support)
    - _Requirements: 2.2, 3.2_
  - [ ] 10.2 Implement NovelArchiveExtractor (ZIP, EPUB, TXT support)
    - _Requirements: 2.3, 4.2_
  - [ ] 10.3 Write property test for chapter sorting consistency
    - **Property 7: Chapter Sorting Consistency**
    - **Validates: Requirements 2.2, 2.3**

- [ ] 11. Implement scan service
  - [ ] 11.1 Implement ScanService for scanning library paths
    - _Requirements: 2.1_
  - [ ] 11.2 Implement content folder detection and chapter identification
    - _Requirements: 2.2, 2.3, 2.4_
  - [ ] 11.3 Implement thumbnail generation for comics (first page of first chapter)
    - _Requirements: 2.5_
  - [ ] 11.4 Implement default thumbnail handling for novels
    - _Requirements: 2.6_
  - [ ] 11.5 Write property test for content title derivation
    - **Property 8: Content Title Derivation**
    - **Validates: Requirements 2.4**
  - [ ] 11.6 Write property test for content-scanpath association
    - **Property 9: Content-ScanPath Association**
    - **Validates: Requirements 2.7**

- [ ] 12. Implement content service
  - [ ] 12.1 Implement ContentService (get, list, search, delete, list_chapters)
    - _Requirements: 2.8, 2.9, 2.10_
  - [ ] 12.2 Implement get_page for comic image extraction
    - _Requirements: 3.2, 7.2_
  - [ ] 12.3 Implement get_chapter_text for novel text extraction
    - _Requirements: 4.2, 7.3_
  - [ ] 12.4 Write property test for content retrieval completeness
    - **Property 5: Content Retrieval Completeness**
    - **Validates: Requirements 1.5**
  - [ ] 12.5 Write property test for content deletion cascade
    - **Property 10: Content Deletion Cascade**
    - **Validates: Requirements 2.9**
  - [ ] 12.6 Write property test for search result relevance
    - **Property 11: Search Result Relevance**
    - **Validates: Requirements 2.10**
  - [ ] 12.7 Write property test for image ordering consistency
    - **Property 12: Image Ordering Consistency**
    - **Validates: Requirements 3.2**

- [ ] 13. Implement content API handlers
  - [ ] 13.1 Implement GET /api/libraries/{id}/contents handler
    - _Requirements: 1.5, 2.8_
  - [ ] 13.2 Implement POST /api/libraries/{id}/scan handler
    - _Requirements: 2.1_
  - [ ] 13.3 Implement GET /api/libraries/{id}/search handler
    - _Requirements: 2.10_
  - [ ] 13.4 Implement GET/DELETE /api/contents/{id} handlers
    - _Requirements: 2.9_
  - [ ] 13.5 Implement GET /api/contents/{id}/chapters handler
    - _Requirements: 3.1, 4.1_
  - [ ] 13.6 Implement GET /api/contents/{id}/chapters/{chapter}/pages/{page} handler
    - _Requirements: 3.2, 7.2_
  - [ ] 13.7 Implement GET /api/contents/{id}/chapters/{chapter}/text handler
    - _Requirements: 4.2, 7.3_
  - [ ] 13.8 Write property test for API response completeness
    - **Property 15: API Response Completeness**
    - **Validates: Requirements 7.1**

- [ ] 14. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Reading Progress

- [ ] 15. Implement progress service
  - [ ] 15.1 Implement ProgressService (get_progress, update_progress)
    - _Requirements: 5.1, 5.2, 5.3_
  - [ ] 15.2 Implement progress percentage calculation
    - _Requirements: 5.4_
  - [ ] 15.3 Write property test for progress persistence round-trip
    - **Property 13: Progress Persistence Round-Trip**
    - **Validates: Requirements 3.5, 4.5, 5.1**
  - [ ] 15.4 Write property test for progress percentage accuracy
    - **Property 14: Progress Percentage Accuracy**
    - **Validates: Requirements 5.4**
  - [ ] 15.5 Write property test for progress user isolation
    - **Property 25: Progress User Isolation**
    - **Validates: Requirements 9.7**
  - [ ] 15.6 Write property test for progress validation
    - **Property 16: Progress Validation**
    - **Validates: Requirements 7.4**

- [ ] 16. Implement progress API handlers
  - [ ] 16.1 Implement GET /api/contents/{id}/progress handler
    - _Requirements: 5.1_
  - [ ] 16.2 Implement PUT /api/contents/{id}/progress handler
    - _Requirements: 5.2, 5.3, 7.4_

- [ ] 17. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Bangumi Metadata Integration

- [ ] 18. Implement Bangumi service
  - [ ] 18.1 Implement BangumiService (search, get_subject, auto_scrape)
    - _Requirements: 8.1, 8.4_
  - [ ] 18.2 Integrate auto-scraping into scan service
    - _Requirements: 8.1, 8.2, 8.3_
  - [ ] 18.3 Implement metadata update for content
    - _Requirements: 8.5, 8.7_
  - [ ] 18.4 Write property test for metadata JSON blob storage
    - **Property 21: Metadata JSON Blob Storage**
    - **Validates: Requirements 8.4, 8.6**

- [ ] 19. Implement Bangumi API handlers
  - [ ] 19.1 Implement GET /api/bangumi/search handler
    - _Requirements: 8.4_
  - [ ] 19.2 Implement PUT /api/contents/{id}/metadata handler
    - _Requirements: 8.5, 8.7_

- [ ] 20. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 7: Watch Mode and Scheduled Scanning

- [ ] 21. Implement watch and scheduler services
  - [ ] 21.1 Implement WatchService for file system monitoring
    - _Requirements: 1.9, 1.10, 1.11_
  - [ ] 21.2 Implement SchedulerService for periodic scanning
    - _Requirements: 1.8_
  - [ ] 21.3 Integrate watch/scheduler with library lifecycle
    - _Requirements: 1.8, 1.9_
  - [ ] 21.4 Write property test for scan interval configuration
    - **Property 19: Scan Interval Configuration**
    - **Validates: Requirements 1.8**
  - [ ] 21.5 Write property test for watch mode state consistency
    - **Property 20: Watch Mode State Consistency**
    - **Validates: Requirements 1.9, 1.10, 1.11**

- [ ] 22. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 8: Backend Integration and Router Setup

- [ ] 23. Wire up Axum router and application state
  - [ ] 23.1 Create AppState with all services
    - _Requirements: 7.1_
  - [ ] 23.2 Configure complete router with all routes
    - _Requirements: 7.1_
  - [ ] 23.3 Add CORS and middleware configuration
    - _Requirements: 7.1_
  - [ ] 23.4 Implement main.rs with server startup
    - _Requirements: 10.1_

- [ ] 24. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 9: Frontend Foundation

- [ ] 25. Set up frontend project structure and dependencies
  - [ ] 25.1 Install required dependencies (vue-router, pinia, axios, fast-check)
    - _Requirements: 7.1_
  - [ ] 25.2 Create directory structure (views, components, stores, api)
    - _Requirements: 7.1_
  - [ ] 25.3 Configure Vue Router with routes
    - _Requirements: 7.1_
  - [ ] 25.4 Configure Pinia store
    - _Requirements: 7.1_

- [ ] 26. Implement API client layer
  - [ ] 26.1 Create base API client with axios and error handling
    - _Requirements: 7.1, 7.5_
  - [ ] 26.2 Implement auth API module
    - _Requirements: 9.2, 9.3_
  - [ ] 26.3 Implement library API module
    - _Requirements: 1.1, 1.4_
  - [ ] 26.4 Implement content API module
    - _Requirements: 2.8, 7.1_
  - [ ] 26.5 Implement reader API module (progress, pages, text)
    - _Requirements: 5.1, 7.2, 7.3_

- [ ] 27. Implement TypeScript interfaces
  - [ ] 27.1 Define all TypeScript interfaces matching backend models
    - _Requirements: 7.1_

## Phase 10: Frontend Authentication

- [ ] 28. Implement auth store and views
  - [ ] 28.1 Implement auth Pinia store (login, logout, token management)
    - _Requirements: 9.2, 9.4_
  - [ ] 28.2 Implement LoginView.vue
    - _Requirements: 9.2, 9.3_
  - [ ] 28.3 Implement RegisterView.vue
    - _Requirements: 9.1_
  - [ ] 28.4 Add route guards for protected routes
    - _Requirements: 9.4_

## Phase 11: Frontend Library Management

- [ ] 29. Implement library components and views
  - [ ] 29.1 Implement library Pinia store
    - _Requirements: 1.1, 1.4_
  - [ ] 29.2 Implement LibraryList.vue component
    - _Requirements: 1.4_
  - [ ] 29.3 Implement LibraryCard.vue component
    - _Requirements: 1.4_
  - [ ] 29.4 Implement LibraryForm.vue component (create/edit)
    - _Requirements: 1.1, 1.7, 1.8, 1.9_
  - [ ] 29.5 Implement HomeView.vue (library list page)
    - _Requirements: 1.4_

## Phase 12: Frontend Content Management

- [ ] 30. Implement content components and views
  - [ ] 30.1 Implement content Pinia store
    - _Requirements: 2.8_
  - [ ] 30.2 Implement ContentGrid.vue component
    - _Requirements: 2.8_
  - [ ] 30.3 Implement ContentCard.vue component (with thumbnail and progress)
    - _Requirements: 2.8, 5.4_
  - [ ] 30.4 Implement ContentSearch.vue component
    - _Requirements: 2.10_
  - [ ] 30.5 Implement Thumbnail.vue component
    - _Requirements: 2.5, 2.6_
  - [ ] 30.6 Implement LibraryView.vue (content list page)
    - _Requirements: 1.5, 2.8_

## Phase 13: Frontend Comic Reader

- [ ] 31. Implement comic reader components and views
  - [ ] 31.1 Implement reader Pinia store
    - _Requirements: 3.2, 5.2_
  - [ ] 31.2 Implement ChapterList.vue component
    - _Requirements: 3.1_
  - [ ] 31.3 Implement ComicReader.vue component (single-page and scroll modes)
    - _Requirements: 3.2, 3.3, 6.1_
  - [ ] 31.4 Implement ReaderSettings.vue component (comic settings)
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - [ ] 31.5 Implement ComicReaderView.vue
    - _Requirements: 3.1, 3.2, 3.4, 3.5_
  - [ ] 31.6 Implement next chapter prompt
    - _Requirements: 3.4_

## Phase 14: Frontend Novel Reader

- [ ] 32. Implement novel reader components and views
  - [ ] 32.1 Implement NovelReader.vue component
    - _Requirements: 4.2, 4.3_
  - [ ] 32.2 Implement novel-specific ReaderSettings (font size, theme)
    - _Requirements: 6.5, 6.6_
  - [ ] 32.3 Implement NovelReaderView.vue
    - _Requirements: 4.1, 4.2, 4.4, 4.5_
  - [ ] 32.4 Implement next chapter prompt for novels
    - _Requirements: 4.4_

## Phase 15: Frontend Settings and Progress

- [ ] 33. Implement settings and progress management
  - [ ] 33.1 Implement settings Pinia store (reader preferences)
    - _Requirements: 6.1, 6.5_
  - [ ] 33.2 Implement ProgressBar.vue component
    - _Requirements: 5.4_
  - [ ] 33.3 Implement progress auto-save with debouncing (5 second persistence)
    - _Requirements: 5.2, 5.3_
  - [ ] 33.4 Implement resume reading prompt
    - _Requirements: 5.1_

## Phase 16: Frontend Metadata Display

- [ ] 34. Implement metadata display
  - [ ] 34.1 Implement metadata display in ContentCard
    - _Requirements: 8.6_
  - [ ] 34.2 Implement manual metadata scraping UI
    - _Requirements: 8.4, 8.5_
  - [ ] 34.3 Implement metadata editing UI
    - _Requirements: 8.7_

## Phase 17: Final Integration

- [ ] 35. Final integration and polish
  - [ ] 35.1 Wire up all frontend components with backend API
    - _Requirements: 7.1_
  - [ ] 35.2 Implement global error handling and notifications
    - _Requirements: 7.5_
  - [ ] 35.3 Add loading states and UI feedback
    - _Requirements: 3.3, 4.3_

- [ ] 36. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
