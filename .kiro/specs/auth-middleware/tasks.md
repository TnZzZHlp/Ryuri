# Implementation Plan

- [x] 1. Create auth middleware module and implement core middleware function





  - Create `backend/src/middlewares/auth.rs` file
  - Implement `auth_middleware` function using `from_fn_with_state`
  - Extract JWT token from Authorization header
  - Verify token using AuthService
  - Store AuthUser in request extensions on success
  - Return 401 error responses on authentication failures
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.2, 5.3_

- [x] 1.1 Write property test for valid token extraction and storage


  - **Property 1: Valid token extraction and storage**
  - **Validates: Requirements 1.1, 1.3, 3.1**

- [x] 1.2 Write property test for invalid token rejection


  - **Property 2: Invalid token rejection**
  - **Validates: Requirements 1.4**

- [x] 1.3 Write property test for error response format


  - **Property 4: Error response format**
  - **Validates: Requirements 5.1**

- [x] 1.4 Write example tests for specific error cases


  - Test missing Authorization header returns 401
  - Test expired token returns specific error message
  - Test invalid token format returns specific error message
  - **Validates: Requirements 1.5, 5.2, 5.3**

- [x] 2. Implement new AuthUser extractor based on request extensions





  - Define AuthUser struct in middlewares module
  - Implement FromRequestParts trait for AuthUser
  - Extract AuthUser from request extensions (not from State)
  - Return appropriate error if AuthUser is missing from extensions
  - _Requirements: 3.1, 3.2, 3.4_

- [x] 2.1 Write property test for extractor success after middleware


  - **Property 3: Extractor success after middleware**
  - **Validates: Requirements 3.3**


- [x] 3. Update middlewares module exports




  - Create `backend/src/middlewares/mod.rs` if it doesn't exist
  - Export `auth_middleware` function
  - Export `AuthUser` struct
  - Update `backend/src/lib.rs` to include middlewares module
  - _Requirements: 2.1_


- [x] 4. Refactor router configuration to separate public and protected routes




  - Identify all public routes (e.g., /api/auth/login)
  - Identify all protected routes (all others)
  - Create separate Router for public routes
  - Create separate Router for protected routes
  - Apply auth middleware layer to protected routes using `from_fn_with_state`
  - Merge public and protected routers
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 4.1 Write example test for public routes bypass authentication

  - **Example 2: Public routes bypass authentication**
  - **Validates: Requirements 4.2**


- [x] 5. Update all handler functions to use new AuthUser extractor




  - Update `backend/src/handlers/auth.rs` handlers (get_me, update_me, update_password)
  - Update `backend/src/handlers/library.rs` handlers
  - Update `backend/src/handlers/content.rs` handlers
  - Update `backend/src/handlers/progress.rs` handlers
  - Update `backend/src/handlers/scan_queue.rs` handlers
  - Ensure handlers import AuthUser from middlewares module
  - _Requirements: 6.3_



- [x] 6. Remove old authentication implementation



  - Remove `middleware` module from `backend/src/services/auth.rs`
  - Remove `HasAuthService` trait definition
  - Remove `HasAuthService` implementation from AppState
  - Remove old AuthUser FromRequestParts implementation
  - Clean up any unused imports
  - _Requirements: 6.1, 6.2, 6.4_

- [x] 7. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Update integration tests for authentication flow





  - Test that protected routes require authentication
  - Test that public routes work without authentication
  - Test end-to-end authentication flow with valid token
  - Test end-to-end authentication flow with invalid token
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 4.2_
