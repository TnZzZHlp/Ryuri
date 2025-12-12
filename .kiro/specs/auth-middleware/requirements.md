# Requirements Document

## Introduction

本功能为后端 API 实现全局认证中间件，自动对除登录接口以外的所有 API 端点进行 JWT 鉴权。当前系统使用 `AuthUser` extractor 在每个需要认证的 handler 中手动添加认证，这种方式容易遗漏且不够统一。通过中间件方式可以集中管理认证逻辑，确保所有受保护的端点都经过认证检查。

## Glossary

- **Auth_Middleware**: 认证中间件，在请求到达 handler 之前验证 JWT token 的有效性
- **JWT_Token**: JSON Web Token，用于用户身份验证的令牌
- **Public_Route**: 公开路由，不需要认证即可访问的端点（如登录接口）
- **Protected_Route**: 受保护路由，需要有效 JWT token 才能访问的端点
- **Authorization_Header**: HTTP 请求头中的 Authorization 字段，格式为 "Bearer {token}"

## Requirements

### Requirement 1

**User Story:** As a backend developer, I want to apply authentication middleware globally, so that all protected routes are automatically secured without manual extractor injection.

#### Acceptance Criteria

1. WHEN a request arrives at a Protected_Route without an Authorization_Header THEN the Auth_Middleware SHALL return a 401 Unauthorized response with an error message
2. WHEN a request arrives at a Protected_Route with an invalid JWT_Token THEN the Auth_Middleware SHALL return a 401 Unauthorized response with an error message
3. WHEN a request arrives at a Protected_Route with an expired JWT_Token THEN the Auth_Middleware SHALL return a 401 Unauthorized response with an error message
4. WHEN a request arrives at a Protected_Route with a valid JWT_Token THEN the Auth_Middleware SHALL allow the request to proceed to the handler
5. WHEN a request arrives at the login endpoint (/api/auth/login) THEN the Auth_Middleware SHALL allow the request to proceed without authentication

### Requirement 2

**User Story:** As a system architect, I want the middleware to be configurable, so that I can easily specify which routes should be public.

#### Acceptance Criteria

1. WHEN configuring the Auth_Middleware THEN the system SHALL accept a list of Public_Route patterns to exclude from authentication
2. WHEN a request path matches a Public_Route pattern THEN the Auth_Middleware SHALL bypass authentication checks
3. WHEN a request path does not match any Public_Route pattern THEN the Auth_Middleware SHALL require authentication

### Requirement 3

**User Story:** As a handler developer, I want to access authenticated user information in handlers, so that I can use user context for business logic.

#### Acceptance Criteria

1. WHEN the Auth_Middleware successfully validates a JWT_Token THEN the system SHALL inject user information into request extensions
2. WHEN a handler needs user information THEN the handler SHALL be able to extract user_id and username from request extensions
3. WHEN the existing AuthUser extractor is used THEN the extractor SHALL continue to work alongside the middleware

### Requirement 4

**User Story:** As a security engineer, I want consistent error responses for authentication failures, so that clients can handle errors uniformly.

#### Acceptance Criteria

1. WHEN authentication fails due to missing token THEN the Auth_Middleware SHALL return a JSON response with error code "UNAUTHORIZED" and message "Missing authorization header"
2. WHEN authentication fails due to invalid token format THEN the Auth_Middleware SHALL return a JSON response with error code "UNAUTHORIZED" and message "Invalid authorization header format"
3. WHEN authentication fails due to token verification failure THEN the Auth_Middleware SHALL return a JSON response with error code "UNAUTHORIZED" and the verification error message
