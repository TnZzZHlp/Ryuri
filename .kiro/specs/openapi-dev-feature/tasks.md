# Implementation Plan

- [x] 1. 为数据模型添加条件 ToSchema 派生





  - [x] 1.1 为 content 模型添加 ToSchema 派生


    - 修改 `backend/src/models/content.rs`
    - 为 `ContentType`, `ContentResponse`, `Chapter` 添加 `#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]`
    - _Requirements: 5.1, 5.2_

  - [x] 1.2 为 library 模型添加 ToSchema 派生

    - 修改 `backend/src/models/library.rs`
    - 为 `Library`, `LibraryWithStats`, `ScanPath`, `CreateLibraryRequest`, `UpdateLibraryRequest` 添加派生
    - _Requirements: 5.1, 5.2_

  - [x] 1.3 为 user 模型添加 ToSchema 派生

    - 修改 `backend/src/models/user.rs`
    - 为 `UserResponse`, `LoginRequest`, `LoginResponse`, `UpdateUserRequest`, `UpdatePasswordRequest` 添加派生
    - _Requirements: 5.1, 5.2_

  - [x] 1.4 为 progress 模型添加 ToSchema 派生

    - 修改 `backend/src/models/progress.rs`
    - 为 `ProgressResponse`, `ContentProgressResponse`, `UpdateProgressRequest` 添加派生
    - _Requirements: 5.1, 5.2_
  - [x] 1.5 编写属性测试验证 DTO 序列化往返


    - **Property 2: DTO Serialization Round-Trip**
    - **Validates: Requirements 5.4**


- [x] 2. 为 handlers 中的类型添加 ToSchema 派生




  - [x] 2.1 为 content handler 类型添加 ToSchema 派生


    - 修改 `backend/src/handlers/content.rs`
    - 为 `ScanResponse`, `SearchQuery`, `ChapterTextResponse`, `UpdateMetadataRequest`, `PageParams`, `ChapterTextParams` 添加派生
    - _Requirements: 5.1, 5.2_

  - [x] 2.2 为 library handler 类型添加 ToSchema 派生

    - 修改 `backend/src/handlers/library.rs`
    - 为 `AddScanPathRequest`, `ScanPathParams` 添加派生
    - _Requirements: 5.1, 5.2_

  - [x] 2.3 为 progress handler 类型添加 ToSchema 派生

    - 修改 `backend/src/handlers/progress.rs`
    - 为 `UpdateProgressWithPercentageRequest` 添加派生
    - _Requirements: 5.1, 5.2_

  - [x] 2.4 为 bangumi service 类型添加 ToSchema 派生

    - 修改 `backend/src/services/bangumi.rs`
    - 为 `BangumiSearchResult` 添加派生
    - _Requirements: 5.1, 5.2_

  - [x] 2.5 为 router 中的类型添加 ToSchema 派生

    - 修改 `backend/src/router.rs`
    - 为 `ScanResponse`, `BangumiSearchQuery` 添加派生
    - _Requirements: 5.1, 5.2_

- [x] 3. 创建 OpenAPI 文档模块





  - [x] 3.1 创建 openapi.rs 模块


    - 创建 `backend/src/openapi.rs`
    - 定义 `ApiDoc` 结构体，使用 `#[derive(OpenApi)]` 宏
    - 配置所有 paths 和 schemas
    - 使用 `#[cfg(feature = "dev")]` 条件编译
    - _Requirements: 1.1, 1.3_

  - [x] 3.2 在 lib.rs 中导出 openapi 模块






    - 修改 `backend/src/lib.rs`
    - 添加条件编译的模块导出
    - _Requirements: 1.1_


- [x] 4. 集成 OpenAPI 路由





  - [x] 4.1 在 router.rs 中添加 OpenAPI 路由

    - 修改 `backend/src/router.rs`
    - 添加条件编译的 Swagger UI 和 OpenAPI JSON 端点
    - Swagger UI 路径: `/swagger-ui`
    - OpenAPI JSON 路径: `/api-docs/openapi.json`
    - _Requirements: 2.1, 3.1_

  - [x] 4.2 编写属性测试验证 Schema 完整性

    - **Property 1: Schema Completeness**


    - **Validates: Requirements 1.3, 5.1, 5.2**

- [x] 5. Checkpoint - 确保所有测试通过



  - Ensure all tests pass, ask the user if questions arise.
