# Design Document: OpenAPI Dev Feature

## Overview

本设计为后端 API 添加自动生成 OpenAPI 文档的能力，使用 `utoipa` 库实现。OpenAPI 文档和 Swagger UI 仅在编译时启用 `dev` feature 时才会包含在应用中，生产构建不会包含这些功能。

## Architecture

```mermaid
graph TB
    subgraph "Conditional Compilation (dev feature)"
        OpenAPI[OpenAPI Spec Generation]
        SwaggerUI[Swagger UI Handler]
        JsonEndpoint[OpenAPI JSON Endpoint]
    end
    
    subgraph "Core Application"
        Router[Router]
        Handlers[API Handlers]
        Models[Data Models]
    end
    
    Router -->|dev feature| OpenAPI
    Router -->|dev feature| SwaggerUI
    Router -->|dev feature| JsonEndpoint
    
    Handlers -->|#[utoipa::path]| OpenAPI
    Models -->|ToSchema derive| OpenAPI
```

### 条件编译策略

使用 Rust 的 `#[cfg(feature = "dev")]` 属性实现条件编译：

1. **模型层**: 为所有 DTO 类型添加条件派生 `ToSchema`
2. **处理器层**: 为所有 handler 函数添加条件 `#[utoipa::path]` 注解
3. **路由层**: 条件性地添加 Swagger UI 和 OpenAPI JSON 端点

## Components and Interfaces

### 1. OpenAPI 文档模块 (`src/openapi.rs`)

新建模块，仅在 `dev` feature 启用时编译：

```rust
#[cfg(feature = "dev")]
pub mod openapi {
    use utoipa::OpenApi;
    
    #[derive(OpenApi)]
    #[openapi(
        paths(/* all handler paths */),
        components(schemas(/* all DTOs */)),
        tags(/* API tags */),
        info(title = "Comic Reader API", version = "1.0.0")
    )]
    pub struct ApiDoc;
}
```

### 2. 路由集成

在 `router.rs` 中条件性添加 OpenAPI 路由：

```rust
#[cfg(feature = "dev")]
fn add_openapi_routes(router: Router<AppState>) -> Router<AppState> {
    use utoipa_swagger_ui::SwaggerUi;
    use crate::openapi::ApiDoc;
    
    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

#[cfg(not(feature = "dev"))]
fn add_openapi_routes(router: Router<AppState>) -> Router<AppState> {
    router
}
```

### 3. 模型注解

为所有 API DTO 添加条件派生：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ContentResponse {
    // fields...
}
```

## Data Models

### 需要添加 ToSchema 派生的类型

#### Request DTOs
- `LoginRequest`
- `RegisterRequest`
- `UpdateUserRequest`
- `UpdatePasswordRequest`
- `CreateLibraryRequest`
- `UpdateLibraryRequest`
- `AddScanPathRequest`
- `UpdateMetadataRequest`
- `UpdateProgressRequest`
- `UpdateProgressWithPercentageRequest`
- `SearchQuery`
- `BangumiSearchQuery`

#### Response DTOs
- `UserResponse`
- `LoginResponse`
- `Library`
- `LibraryWithStats`
- `ScanPath`
- `ContentResponse`
- `Chapter`
- `ScanResponse`
- `ChapterTextResponse`
- `ProgressResponse`
- `ContentProgressResponse`
- `BangumiSearchResult`

#### Enum Types
- `ContentType`

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Schema Completeness
*For any* type annotated with `ToSchema`, the generated OpenAPI specification SHALL contain a corresponding schema definition in the `components/schemas` section.
**Validates: Requirements 1.3, 5.1, 5.2**

### Property 2: DTO Serialization Round-Trip
*For any* valid DTO instance (request or response type), serializing to JSON and deserializing back SHALL produce a value equivalent to the original.
**Validates: Requirements 5.4**

## Error Handling

### 编译时错误
- 如果 `ToSchema` 派生失败（例如类型不支持），编译器会报错
- 如果 `#[utoipa::path]` 注解与实际 handler 签名不匹配，编译器会报错

### 运行时错误
- Swagger UI 和 OpenAPI JSON 端点不会产生业务逻辑错误
- 这些端点仅在开发环境可用，不影响生产环境

## Testing Strategy

### 测试框架
- 使用 `proptest` 进行属性测试（已在项目中配置）

### 单元测试
1. 验证 OpenAPI spec 包含所有预期的路径
2. 验证所有 DTO 类型都有对应的 schema 定义
3. 验证 Swagger UI 路由在 dev feature 启用时存在

### 属性测试
1. **DTO 序列化往返测试**: 对所有 DTO 类型进行 JSON 序列化/反序列化往返测试
2. **Schema 完整性测试**: 验证所有标注的类型都出现在生成的 OpenAPI spec 中

### 测试注解格式
每个属性测试必须包含以下注释格式：
```rust
// **Feature: openapi-dev-feature, Property {number}: {property_text}**
// **Validates: Requirements X.Y**
```
