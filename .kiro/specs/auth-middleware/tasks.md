# Implementation Plan

- [x] 1. Create auth middleware module structure





  - Create `backend/src/middlewares/` folder and `mod.rs`
  - Create `backend/src/middlewares/auth.rs` file for auth middleware
  - Register middlewares module in `lib.rs`
  - Define `AuthMiddlewareLayer` struct with `auth_service` and `public_routes` fields
  - Define `AuthMiddlewareService<S>` struct
  - _Requirements: 1.1, 2.1_







- [x] 2. Implement middleware core logic




  - [ ] 2.1 Implement `is_public_route` function for path matching
    - Accept path string and public routes slice
    - Return true if path matches any public route


    - _Requirements: 2.2, 2.3_
  - [ ] 2.2 Write property test for public route matching
    - **Property 3: Public routes bypass authentication**
    - **Validates: Requirements 2.2, 2.3**
  - [x] 2.3 Implement `tower::Layer` trait for `AuthMiddlewareLayer`



    - Create `AuthMiddlewareService` wrapping inner service


    - Pass auth_service and public_routes to service
    - _Requirements: 1.1_

  - [ ] 2.4 Implement `tower::Service` trait for `AuthMiddlewareService`
    - Check if request path is public route, bypass auth if true
    - Extract Authorization header from request
    - Validate Bearer token format





    - Verify JWT token using auth_service



    - Inject AuthUser into request extensions on success


    - Return 401 error response on failure






    - _Requirements: 1.1, 1.2, 1.3, 1.4, 3.1_









- [ ] 3. Implement error response handling

  - [ ] 3.1 Create `AuthErrorResponse` struct for JSON error responses
    - Include `error`, `code`, and `message` fields
    - Implement serialization
    - _Requirements: 4.1, 4.2, 4.3_
  - [ ] 3.2 Implement error response generation for each failure case
    - Missing authorization header
    - Invalid header format
    - Token verification failure
    - _Requirements: 4.1, 4.2, 4.3_
  - [ ] 3.3 Write property test for error response format
    - **Property 5: Error responses have correct format**
    - **Validates: Requirements 4.1, 4.2, 4.3**

- [ ] 4. Integrate middleware into router

  - [ ] 4.1 Update `create_router_with_layers` in `router.rs`
    - Add `AuthMiddlewareLayer` to the layer stack
    - Configure public routes list (at minimum `/api/auth/login`)
    - _Requirements: 1.5, 2.1_
  - [ ] 4.2 Verify existing `AuthUser` extractor compatibility
    - Ensure handlers using AuthUser extractor still work
    - AuthUser can now also read from extensions set by middleware
    - _Requirements: 3.3_

- [ ] 5. Write property-based tests for middleware

  - [ ] 5.1 Write property test for unauthenticated request rejection
    - **Property 1: Protected routes reject unauthenticated requests**
    - **Validates: Requirements 1.1, 1.2, 1.3**
  - [ ] 5.2 Write property test for valid token acceptance
    - **Property 2: Valid tokens allow access to protected routes**
    - **Validates: Requirements 1.4**
  - [ ] 5.3 Write property test for user info injection
    - **Property 4: User info is available after successful auth**
    - **Validates: Requirements 3.1, 3.2**

- [ ] 6. Checkpoint - Ensure all tests pass

  - Ensure all tests pass, ask the user if questions arise.
