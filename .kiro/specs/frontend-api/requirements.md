# Requirements Document

## Introduction

本规范定义了前端 API 层的实现要求。该 API 层将封装所有与后端的 HTTP 通信，提供类型安全的 TypeScript 接口，使前端组件能够方便地调用后端服务。API 层不涉及任何 UI 组件，仅负责数据获取和提交。

## Glossary

- **API_Client**: 封装 HTTP 请求的客户端模块，处理请求/响应的序列化和错误处理
- **ApiError**: API 错误对象，包含状态码和错误消息
- **Auth_API**: 认证相关的 API 模块
- **Bangumi_API**: Bangumi 元数据搜索 API 模块
- **BangumiSearchResult**: Bangumi 搜索结果对象
- **Chapter**: 内容中的单个章节（漫画卷/小说章节）
- **Content**: 漫画或小说内容项，包含多个章节
- **Content_API**: 内容管理相关的 API 模块
- **ContentType**: 内容类型枚举 (Comic | Novel)
- **Library**: 内容库，包含多个扫描路径和内容项
- **Library_API**: 内容库管理相关的 API 模块
- **LoginResponse**: 登录响应对象，包含用户信息和 JWT token
- **Progress**: 用户的阅读进度记录
- **Progress_API**: 阅读进度相关的 API 模块
- **Reader_API**: 阅读器内容获取相关的 API 模块
- **ScanPath**: 文件系统扫描路径，用于发现新内容
- **Types_Module**: TypeScript 类型定义模块
- **User**: 用户信息对象

## Requirements

### Requirement 1: API 客户端基础设施

**User Story:** 作为前端开发者，我希望有一个统一的 API 客户端基础设施，以便所有 API 调用都有一致的错误处理和认证机制。

#### Acceptance Criteria

1. THE API_Client SHALL provide a base HTTP client with configurable base URL
2. WHEN a request requires authentication, THE API_Client SHALL automatically include the JWT token in the Authorization header
3. WHEN the server returns an error response, THE API_Client SHALL parse the error and throw a typed ApiError object containing status code and message
4. THE API_Client SHALL support JSON request and response serialization
5. WHEN serializing a request body, THE API_Client SHALL convert TypeScript objects to JSON format
6. WHEN deserializing a response body, THE API_Client SHALL convert JSON to typed TypeScript objects

### Requirement 2: 认证 API

**User Story:** 作为用户，我希望能够登录系统并管理我的账户信息，以便访问个人化的阅读体验。

#### Acceptance Criteria

1. WHEN a user provides valid credentials, THE Auth_API SHALL return a LoginResponse containing user info and JWT token
2. WHEN an authenticated user requests their profile, THE Auth_API SHALL return the current User information
3. WHEN an authenticated user updates their profile, THE Auth_API SHALL send the update request and return the updated User info
4. WHEN an authenticated user changes their password, THE Auth_API SHALL send the password update request and return success status

### Requirement 3: 内容库 API

**User Story:** 作为用户，我希望能够管理我的内容库，以便组织和浏览我的漫画和小说收藏。

#### Acceptance Criteria

1. WHEN a user requests the library list, THE Library_API SHALL return all Library objects with their statistics
2. WHEN a user creates a new library, THE Library_API SHALL send the creation request and return the new Library object
3. WHEN a user requests a specific library by ID, THE Library_API SHALL return the Library details with statistics
4. WHEN a user updates a library, THE Library_API SHALL send the update request and return the updated Library object
5. WHEN a user deletes a library, THE Library_API SHALL send the deletion request and return success status
6. WHEN a user requests scan paths for a library, THE Library_API SHALL return all ScanPath objects for that library
7. WHEN a user adds a scan path to a library, THE Library_API SHALL send the add request and return the new ScanPath object
8. WHEN a user removes a scan path from a library, THE Library_API SHALL send the removal request and return success status

### Requirement 4: 内容 API

**User Story:** 作为用户，我希望能够浏览和管理内容项，以便找到并阅读我想要的漫画或小说。

#### Acceptance Criteria

1. WHEN a user requests contents for a library, THE Content_API SHALL return all Content objects in that library
2. WHEN a user triggers a library scan, THE Content_API SHALL send the scan request and return the scan results
3. WHEN a user searches for content with a query string, THE Content_API SHALL return matching Content objects
4. WHEN a user requests a specific content by ID, THE Content_API SHALL return the Content details
5. WHEN a user deletes a content, THE Content_API SHALL send the deletion request and return success status
6. WHEN a user updates content metadata, THE Content_API SHALL send the update request and return the updated Content object
7. WHEN a user requests chapters for a content, THE Content_API SHALL return all Chapter objects for that content

### Requirement 5: 阅读器 API

**User Story:** 作为用户，我希望能够获取章节内容进行阅读，以便享受漫画和小说。

#### Acceptance Criteria

1. WHEN a user requests a comic page by chapter ID and page number, THE Reader_API SHALL return the page image URL
2. WHEN a user requests novel chapter text by chapter ID, THE Reader_API SHALL return the chapter text content as a string

### Requirement 6: 阅读进度 API

**User Story:** 作为用户，我希望系统能够记录和恢复我的阅读进度，以便我可以从上次离开的地方继续阅读。

#### Acceptance Criteria

1. WHEN a user requests content progress by content ID, THE Progress_API SHALL return the overall Progress for that content
2. WHEN a user requests chapter progress by chapter ID, THE Progress_API SHALL return the Progress for that chapter
3. WHEN a user updates chapter progress, THE Progress_API SHALL send the update request and return the updated Progress object

### Requirement 7: Bangumi 元数据 API

**User Story:** 作为用户，我希望能够从 Bangumi 搜索元数据，以便为我的内容添加详细信息。

#### Acceptance Criteria

1. WHEN a user searches on Bangumi with a query string, THE Bangumi_API SHALL return matching BangumiSearchResult objects with metadata

### Requirement 8: TypeScript 类型定义

**User Story:** 作为前端开发者，我希望所有 API 响应都有完整的 TypeScript 类型定义，以便获得类型安全和 IDE 支持。

#### Acceptance Criteria

1. THE Types_Module SHALL define all request and response types matching the backend API schemas
2. THE Types_Module SHALL define enum types for ContentType (Comic and Novel)
3. THE Types_Module SHALL use proper TypeScript optional types for nullable fields
4. WHEN a type is serialized to JSON, THE Types_Module SHALL ensure the serialized output matches the expected backend format
5. WHEN a JSON response is deserialized, THE Types_Module SHALL produce a correctly typed TypeScript object
