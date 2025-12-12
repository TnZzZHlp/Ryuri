# Requirements Document

## Introduction

本文档定义了使用 `axum::middleware::from_fn_with_state` 实现鉴权中间件的需求。当前系统通过 `AuthUser` extractor 实现了基于 JWT token 的鉴权，但这种方式需要在每个 handler 中手动添加参数。新的实现将删除原有的 extractor 鉴权方式，改用中间件在路由层面提供统一的身份验证，验证 JWT token 并将用户信息注入到请求扩展中，使得受保护的路由组可以统一应用鉴权逻辑。

## Glossary

- **AuthMiddleware**: 使用 `from_fn_with_state` 实现的鉴权中间件函数
- **AppState**: 应用程序状态，包含 AuthService 等服务
- **JWT Token**: JSON Web Token，用于用户身份验证
- **AuthUser**: 已认证用户的数据结构，包含 user_id 和 username
- **Request Extensions**: Axum 请求扩展机制，用于在中间件和 handler 之间传递数据
- **Protected Routes**: 需要身份验证的路由
- **Public Routes**: 不需要身份验证的路由
- **Extractor**: Axum 的参数提取器，用于从请求中提取数据

## Requirements

### Requirement 1

**User Story:** 作为开发者，我希望创建一个可复用的鉴权中间件，以便在路由层面统一处理身份验证逻辑，而不需要在每个 handler 中手动添加 AuthUser 参数。

#### Acceptance Criteria

1. WHEN the middleware is applied to a route THEN the system SHALL extract the JWT token from the Authorization header
2. WHEN a valid Bearer token is present THEN the system SHALL verify the token using AuthService
3. WHEN token verification succeeds THEN the system SHALL insert the authenticated user information into request extensions
4. WHEN token verification fails THEN the system SHALL return a 401 Unauthorized response
5. WHEN the Authorization header is missing THEN the system SHALL return a 401 Unauthorized response

### Requirement 2

**User Story:** 作为开发者，我希望中间件能够访问 AppState，以便使用 AuthService 进行 token 验证。

#### Acceptance Criteria

1. WHEN the middleware is created THEN the system SHALL accept AppState as a parameter via `from_fn_with_state`
2. WHEN verifying tokens THEN the system SHALL use the AuthService from AppState
3. WHEN the middleware processes a request THEN the system SHALL maintain access to all AppState services

### Requirement 3

**User Story:** 作为开发者，我希望在 handler 中能够方便地获取已认证的用户信息，以便实现业务逻辑。

#### Acceptance Criteria

1. WHEN a request passes through the auth middleware THEN the system SHALL store AuthUser in request extensions
2. WHEN a handler needs user information THEN the system SHALL provide a new extractor to retrieve AuthUser from request extensions
3. WHEN extracting AuthUser in a protected route THEN the system SHALL always succeed because the middleware has validated it
4. WHEN the new extractor is used THEN the system SHALL not require access to AppState or AuthService

### Requirement 4

**User Story:** 作为开发者，我希望能够灵活地将中间件应用到特定路由或路由组，以便区分公开路由和受保护路由。

#### Acceptance Criteria

1. WHEN configuring routes THEN the system SHALL allow applying the middleware to specific route groups
2. WHEN a route does not have the middleware THEN the system SHALL process the request without authentication
3. WHEN organizing routes THEN the system SHALL support nesting routers with and without authentication

### Requirement 5

**User Story:** 作为开发者，我希望中间件能够提供清晰的错误信息，以便调试和用户反馈。

#### Acceptance Criteria

1. WHEN authentication fails THEN the system SHALL return a JSON error response with appropriate status code
2. WHEN the token is expired THEN the system SHALL return a specific error message indicating token expiration
3. WHEN the token format is invalid THEN the system SHALL return a specific error message indicating invalid format
4. WHEN logging authentication failures THEN the system SHALL include relevant context for debugging


### Requirement 6

**User Story:** 作为开发者，我希望移除原有的基于 FromRequestParts 的 AuthUser extractor 实现，以便统一使用中间件方式进行鉴权。

#### Acceptance Criteria

1. WHEN refactoring the authentication system THEN the system SHALL remove the FromRequestParts implementation for AuthUser
2. WHEN the old extractor is removed THEN the system SHALL ensure no handler depends on the state-based extraction
3. WHEN migrating to middleware THEN the system SHALL update all existing handlers to use the new extension-based extractor
4. WHEN the migration is complete THEN the system SHALL have a single, consistent authentication approach across all routes
