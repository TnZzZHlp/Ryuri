# Requirements Document

## Introduction

本规范定义了前端 API 层的实现要求。该 API 层将封装所有与后端的 HTTP 通信，提供类型安全的 TypeScript 接口，使前端组件能够方便地调用后端服务。API 层不涉及任何 UI 组件，仅负责数据获取和提交。

## Glossary

- **API Client**: 封装 HTTP 请求的客户端模块，处理请求/响应的序列化和错误处理
- **Content**: 漫画或小说内容项，包含多个章节
- **Chapter**: 内容中的单个章节（漫画卷/小说章节）
- **Library**: 内容库，包含多个扫描路径和内容项
- **ScanPath**: 文件系统扫描路径，用于发现新内容
- **Progress**: 用户的阅读进度记录
- **Bangumi**: 第三方元数据服务 (bangumi.tv)
- **ContentType**: 内容类型枚举 (Comic | Novel)

## Requirements

### Requirement 1: API 客户端基础设施

**User Story:** 作为前端开发者，我希望有一个统一的 API 客户端基础设施，以便所有 API 调用都有一致的错误处理和认证机制。

#### Acceptance Criteria

1. THE API Client SHALL provide a base HTTP client with configurable base URL
2. WHEN a request requires authentication THEN THE API Client SHALL automatically include the JWT token in the Authorization header
3. WHEN the server returns an error response THEN THE API Client SHALL parse the error and throw a typed error object
4. THE API Client SHALL support JSON request/response serialization

### Requirement 2: 认证 API

**User Story:** 作为用户，我希望能够登录系统并管理我的账户信息，以便访问个人化的阅读体验。

#### Acceptance Criteria

1. WHEN a user provides valid credentials THEN THE Auth API SHALL return a login response containing user info and JWT token
2. WHEN an authenticated user requests their profile THEN THE Auth API SHALL return the current user information
3. WHEN an authenticated user updates their profile THEN THE Auth API SHALL send the update request and return the updated user info
4. WHEN an authenticated user changes their password THEN THE Auth API SHALL send the password update request

### Requirement 3: 内容库 API

**User Story:** 作为用户，我希望能够管理我的内容库，以便组织和浏览我的漫画和小说收藏。

#### Acceptance Criteria

1. WHEN a user requests the library list THEN THE Library API SHALL return all libraries with their statistics
2. WHEN a user creates a new library THEN THE Library API SHALL send the creation request and return the new library
3. WHEN a user requests a specific library THEN THE Library API SHALL return the library details with statistics
4. WHEN a user updates a library THEN THE Library API SHALL send the update request and return the updated library
5. WHEN a user deletes a library THEN THE Library API SHALL send the deletion request
6. WHEN a user requests scan paths for a library THEN THE Library API SHALL return all scan paths
7. WHEN a user adds a scan path THEN THE Library API SHALL send the add request and return the new scan path
8. WHEN a user removes a scan path THEN THE Library API SHALL send the removal request

### Requirement 4: 内容 API

**User Story:** 作为用户，我希望能够浏览和管理内容项，以便找到并阅读我想要的漫画或小说。

#### Acceptance Criteria

1. WHEN a user requests contents for a library THEN THE Content API SHALL return all content items in that library
2. WHEN a user triggers a library scan THEN THE Content API SHALL send the scan request and return the scan results
3. WHEN a user searches for content THEN THE Content API SHALL return matching content items
4. WHEN a user requests a specific content THEN THE Content API SHALL return the content details
5. WHEN a user deletes a content THEN THE Content API SHALL send the deletion request
6. WHEN a user updates content metadata THEN THE Content API SHALL send the update request and return the updated content
7. WHEN a user requests chapters for a content THEN THE Content API SHALL return all chapters

### Requirement 5: 阅读器 API

**User Story:** 作为用户，我希望能够获取章节内容进行阅读，以便享受漫画和小说。

#### Acceptance Criteria

1. WHEN a user requests a comic page THEN THE Reader API SHALL return the page image URL
2. WHEN a user requests novel chapter text THEN THE Reader API SHALL return the chapter text content

### Requirement 6: 阅读进度 API

**User Story:** 作为用户，我希望系统能够记录和恢复我的阅读进度，以便我可以从上次离开的地方继续阅读。

#### Acceptance Criteria

1. WHEN a user requests content progress THEN THE Progress API SHALL return the overall reading progress for that content
2. WHEN a user requests chapter progress THEN THE Progress API SHALL return the reading progress for that chapter
3. WHEN a user updates chapter progress THEN THE Progress API SHALL send the update request and return the updated progress

### Requirement 7: Bangumi 元数据 API

**User Story:** 作为用户，我希望能够从 Bangumi 搜索元数据，以便为我的内容添加详细信息。

#### Acceptance Criteria

1. WHEN a user searches on Bangumi THEN THE Bangumi API SHALL return matching search results with metadata

### Requirement 8: TypeScript 类型定义

**User Story:** 作为前端开发者，我希望所有 API 响应都有完整的 TypeScript 类型定义，以便获得类型安全和 IDE 支持。

#### Acceptance Criteria

1. THE Types module SHALL define all request and response types matching the backend API schemas
2. THE Types module SHALL define enum types for ContentType
3. THE Types module SHALL use proper TypeScript optional types for nullable fields
