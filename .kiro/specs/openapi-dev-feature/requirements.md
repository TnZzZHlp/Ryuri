# Requirements Document

## Introduction

本功能为后端 API 添加自动生成 OpenAPI 文档的能力。OpenAPI 文档仅在编译时启用 `dev` feature 时才会附加到应用中，生产环境不会包含此功能，以减少二进制体积和潜在的安全风险。

## Glossary

- **OpenAPI**: 一种用于描述 RESTful API 的规范格式
- **Swagger UI**: 用于可视化和交互式测试 OpenAPI 文档的 Web 界面
- **utoipa**: Rust 生态中用于自动生成 OpenAPI 文档的库
- **dev feature**: Cargo 的条件编译特性，用于区分开发和生产构建
- **Handler**: Axum 框架中处理 HTTP 请求的函数

## Requirements

### Requirement 1

**User Story:** As a developer, I want OpenAPI documentation to be automatically generated from my API handlers, so that I can have up-to-date API documentation without manual maintenance.

#### Acceptance Criteria

1. WHEN the `dev` feature is enabled THEN the system SHALL generate OpenAPI documentation from all API handlers
2. WHEN the `dev` feature is disabled THEN the system SHALL exclude all OpenAPI-related code from the compiled binary
3. WHEN OpenAPI documentation is generated THEN the system SHALL include all request/response schemas derived from handler types
4. WHEN OpenAPI documentation is generated THEN the system SHALL include path parameters, query parameters, and request bodies for each endpoint

### Requirement 2

**User Story:** As a developer, I want to access Swagger UI at a dedicated endpoint, so that I can interactively explore and test the API.

#### Acceptance Criteria

1. WHEN the `dev` feature is enabled THEN the system SHALL serve Swagger UI at the `/swagger-ui` path
2. WHEN a user accesses the Swagger UI endpoint THEN the system SHALL display all documented API endpoints
3. WHEN the `dev` feature is disabled THEN the system SHALL not expose any Swagger UI routes

### Requirement 3

**User Story:** As a developer, I want the OpenAPI schema to be accessible as a JSON endpoint, so that I can use it with external tools.

#### Acceptance Criteria

1. WHEN the `dev` feature is enabled THEN the system SHALL serve the OpenAPI JSON schema at `/api-docs/openapi.json`
2. WHEN the `dev` feature is disabled THEN the system SHALL not expose the OpenAPI JSON endpoint

### Requirement 4

**User Story:** As a developer, I want all existing API handlers to be documented with OpenAPI annotations, so that the documentation is complete.

#### Acceptance Criteria

1. WHEN documenting handlers THEN the system SHALL annotate all auth handlers (`/api/auth/*`) with OpenAPI metadata
2. WHEN documenting handlers THEN the system SHALL annotate all library handlers (`/api/libraries/*`) with OpenAPI metadata
3. WHEN documenting handlers THEN the system SHALL annotate all content handlers (`/api/contents/*`) with OpenAPI metadata
4. WHEN documenting handlers THEN the system SHALL annotate all progress handlers (`/api/chapters/*/progress`) with OpenAPI metadata
5. WHEN documenting handlers THEN the system SHALL annotate the Bangumi search handler (`/api/bangumi/search`) with OpenAPI metadata

### Requirement 5

**User Story:** As a developer, I want all data models used in API requests and responses to have OpenAPI schema definitions, so that the documentation accurately describes the data structures.

#### Acceptance Criteria

1. WHEN generating schemas THEN the system SHALL derive `ToSchema` for all request DTOs
2. WHEN generating schemas THEN the system SHALL derive `ToSchema` for all response DTOs
3. WHEN generating schemas THEN the system SHALL include field descriptions where applicable
4. WHEN serializing and deserializing schemas THEN the system SHALL produce equivalent values through round-trip conversion
