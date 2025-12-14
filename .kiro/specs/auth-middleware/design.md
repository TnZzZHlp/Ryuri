# Design Document

## Overview

本设计文档描述了使用 `axum::middleware::from_fn_with_state` 实现鉴权中间件的架构设计。该实现将替换现有的基于 `FromRequestParts` 的 extractor 鉴权方式，提供一个在路由层面统一应用的中间件解决方案。

核心设计理念：
- 使用中间件在请求到达 handler 之前进行身份验证
- 将验证后的用户信息存储在 request extensions 中
- 提供简单的 extractor 从 extensions 中提取用户信息
- 支持灵活的路由组织，区分公开路由和受保护路由

## Architecture

### 组件层次结构

```
Request
  ↓
CORS Layer
  ↓
Tracing Layer
  ↓
Auth Middleware (新增)
  ↓
Router
  ↓
Handler
```

### 数据流

1. **请求进入**：HTTP 请求到达服务器
2. **中间件拦截**：Auth middleware 拦截请求
3. **提取 Token**：从 Authorization header 提取 Bearer token
4. **验证 Token**：使用 AuthService 验证 JWT token
5. **存储用户信息**：将 AuthUser 存入 request extensions
6. **继续处理**：请求继续传递给 handler
7. **Handler 提取**：Handler 使用 extractor 从 extensions 获取 AuthUser

## Components and Interfaces

### 1. Auth Middleware Function

```rust
// backend/src/middlewares/auth.rs

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError>
```

**职责**：
- 从 Authorization header 提取 JWT token
- 验证 token 的有效性
- 将 AuthUser 插入到 request extensions
- 处理认证失败的情况

**输入**：
- `State<AppState>`: 应用状态，包含 AuthService
- `Request<Body>`: HTTP 请求
- `Next`: 下一个中间件或 handler

**输出**：
- `Result<Response, AppError>`: 成功返回响应，失败返回错误

### 2. AuthUser Extractor (新版本)

```rust
// backend/src/middlewares/auth.rs

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
}
```

**职责**：
- 从 request extensions 中提取 AuthUser
- 提供类型安全的用户信息访问

**特点**：
- 不依赖 AppState
- 不需要访问 AuthService
- 假设中间件已经验证并存储了用户信息

### 3. Router Configuration

```rust
// backend/src/router.rs

pub fn create_router(state: AppState) -> Router {
    // 公开路由
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login));

    // 受保护路由
    let protected_routes = Router::new()
        .route("/api/auth/me", get(auth::get_me).put(auth::update_me))
        .route("/api/auth/password", put(auth::update_password))
        .route("/api/libraries", get(library::list).post(library::create))
        // ... 其他受保护路由
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
```

## Data Models

### AuthUser

```rust
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// 用户 ID
    pub user_id: i64,
    /// 用户名
    pub username: String,
}
```

**来源**：从 JWT claims 转换而来

**用途**：
- 存储在 request extensions 中
- 在 handler 中通过 extractor 获取
- 提供用户身份信息用于业务逻辑

### JWT Claims (已存在)

```rust
pub struct JwtClaims {
    pub sub: i64,        // user_id
    pub username: String,
    pub exp: i64,        // expiration time
    pub iat: i64,        // issued at
}
```

## Error Handling

### 错误类型

1. **Missing Authorization Header**
   - HTTP Status: 401 Unauthorized
   - Message: "Missing authorization header"

2. **Invalid Header Format**
   - HTTP Status: 401 Unauthorized
   - Message: "Invalid authorization header format"

3. **Invalid Token**
   - HTTP Status: 401 Unauthorized
   - Message: "Invalid token: {reason}"

4. **Expired Token**
   - HTTP Status: 401 Unauthorized
   - Message: "Invalid token: ExpiredSignature"

### 错误处理流程

```rust
// 中间件中的错误处理
match extract_and_verify_token(&state, &req) {
    Ok(auth_user) => {
        req.extensions_mut().insert(auth_user);
        Ok(next.run(req).await)
    }
    Err(e) => {
        // 记录错误
        tracing::warn!("Authentication failed: {:?}", e);
        // 返回 JSON 错误响应
        Err(e)
    }
}
```

## Testing Strategy

### Unit Tests

1. **Middleware Function Tests**
   - 测试有效 token 的处理
   - 测试缺失 Authorization header
   - 测试无效的 header 格式
   - 测试过期的 token
   - 测试无效的 token

2. **Extractor Tests**
   - 测试从 extensions 成功提取 AuthUser
   - 测试 extensions 中缺失 AuthUser 的情况

3. **Integration Tests**
   - 测试受保护路由需要认证
   - 测试公开路由不需要认证
   - 测试认证成功后 handler 可以访问用户信息

### Property-Based Tests

由于中间件主要是集成现有的 AuthService 功能，property-based testing 将专注于验证中间件的行为一致性。

## Implementation Notes

### 迁移步骤

1. **创建新的中间件模块**
   - 在 `backend/src/middlewares/` 创建 `auth.rs`
   - 实现 `auth_middleware` 函数
   - 实现新的 `AuthUser` extractor

2. **更新路由配置**
   - 修改 `router.rs` 以区分公开和受保护路由
   - 对受保护路由应用中间件

3. **更新 handlers**
   - 确保所有需要认证的 handler 都使用 `AuthUser` extractor
   - 移除对 `State` 的依赖（如果只是为了认证）

4. **删除旧实现**
   - 从 `services/auth.rs` 中删除 `middleware` 模块
   - 删除 `HasAuthService` trait
   - 从 `AppState` 中移除 `HasAuthService` 实现

5. **测试验证**
   - 运行所有测试确保功能正常
   - 手动测试认证流程

### 向后兼容性

由于我们要删除旧的实现，需要确保：
- 所有使用 `AuthUser` 的 handler 都已更新
- 路由配置正确区分了公开和受保护路由
- 测试代码也相应更新

### 性能考虑

- 中间件会在每个受保护路由的请求上运行
- JWT 验证是轻量级操作
- Extensions 的插入和提取开销很小
- 相比 extractor 方式，性能特征相似

### 安全考虑

- Token 验证使用现有的 `AuthService::verify_token`
- 错误信息不应泄露敏感信息
- 日志记录应包含足够的上下文用于调试，但不记录完整 token


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing the acceptance criteria, several properties were identified. Upon reflection:
- Properties 1.1 and 1.3 can be combined into a single property about successful token extraction and storage
- Property 3.3 is a consequence of 1.3 - if middleware stores AuthUser, extraction will succeed
- We focus on the core behaviors: token extraction, authentication failure handling, and response format

### Properties

**Property 1: Valid token extraction and storage**

*For any* request with a valid Bearer token in the Authorization header, the middleware should extract the token, verify it, and store the resulting AuthUser in request extensions.

**Validates: Requirements 1.1, 1.3, 3.1**

---

**Property 2: Invalid token rejection**

*For any* request with an invalid JWT token (malformed, expired, or wrong signature), the middleware should return a 401 Unauthorized response and not call the next handler.

**Validates: Requirements 1.4**

---

**Property 3: Extractor success after middleware**

*For any* request that successfully passes through the auth middleware, extracting AuthUser in the handler should always succeed and return the same user information that was stored by the middleware.

**Validates: Requirements 3.3**

---

**Property 4: Error response format**

*For any* authentication failure (missing header, invalid token, expired token), the middleware should return a JSON-formatted error response with status code 401.

**Validates: Requirements 5.1**

---

### Example-Based Tests

**Example 1: Missing Authorization header**

When a request arrives without an Authorization header, the middleware should return 401 with message "Missing authorization header".

**Validates: Requirements 1.5**

---

**Example 2: Public routes bypass authentication**

When a request is made to a public route (e.g., /api/auth/login), the middleware should not be applied and the request should succeed without a token.

**Validates: Requirements 4.2**

---

**Example 3: Expired token error message**

When a request contains an expired JWT token, the middleware should return 401 with a message indicating token expiration.

**Validates: Requirements 5.2**

---

**Example 4: Invalid token format error message**

When a request contains a malformed token (not valid JWT format), the middleware should return 401 with a message indicating invalid format.

**Validates: Requirements 5.3**
