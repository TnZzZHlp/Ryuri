# Design Document: Auto Thumbnail Fetch

## Overview

本设计文档描述了在Content Store中实现自动缩略图获取功能的技术方案。该功能将缩略图管理完全集成到Pinia状态管理中，通过扩展现有的Content API和Content Store，实现缩略图的自动预加载、缓存和管理。

核心设计原则：
- 最小化对现有代码的改动
- 保持与现有架构的一致性
- 提供非阻塞的异步加载体验
- 自动化缩略图生命周期管理

## Architecture

### 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                        Vue Component                         │
│                      (Library.vue)                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ 使用 computed/ref
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                      Content Store                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  State:                                              │   │
│  │  - contents: Map<number, ContentResponse[]>         │   │
│  │  - thumbnailUrls: Map<number, string>               │   │
│  │  - thumbnailLoading: Set<number>                    │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Actions:                                            │   │
│  │  - fetchContents() → 自动触发预加载                  │   │
│  │  - getThumbnailUrl(id) → 返回缓存的URL              │   │
│  │  - loadThumbnail(id) → 加载单个缩略图                │   │
│  │  - invalidateThumbnailCache() → 清除缩略图缓存       │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ 调用 API
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                       Content API                            │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  getThumbnail(id): Promise<Blob>                     │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ HTTP GET
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                    Backend API                               │
│              GET /api/contents/{id}/thumbnail                │
└─────────────────────────────────────────────────────────────┘
```

### 数据流

1. **内容列表加载流程**:
   ```
   Component.onMounted()
     → ContentStore.fetchContents(libraryId)
       → ContentAPI.list(libraryId)
       → 存储到 contents Map
       → 自动触发 preloadThumbnails()
         → 遍历 has_thumbnail=true 的内容
         → 并发调用 loadThumbnail(id)
   ```

2. **缩略图加载流程**:
   ```
   ContentStore.loadThumbnail(id)
     → 检查缓存 (thumbnailUrls.has(id))
     → 如果未缓存:
       → 标记为加载中 (thumbnailLoading.add(id))
       → ContentAPI.getThumbnail(id)
       → 创建 Object URL
       → 存储到缓存 (thumbnailUrls.set(id, url))
       → 移除加载标记 (thumbnailLoading.delete(id))
   ```

3. **缩略图访问流程**:
   ```
   Component 访问 getThumbnailUrl(id)
     → 返回 thumbnailUrls.get(id) ?? null
     → Component 使用 v-if 条件渲染
   ```

## Components and Interfaces

### Content API 扩展

在 `frontend/src/api/content.ts` 中添加新方法：

```typescript
export interface ContentApi {
    // ... 现有方法 ...
    
    /**
     * Gets the thumbnail image for a content.
     * 
     * @param id - The content ID
     * @returns Promise resolving to the thumbnail image Blob
     */
    getThumbnail(id: number): Promise<Blob>;
}
```

实现：

```typescript
async getThumbnail(id: number): Promise<Blob> {
    // 使用 ApiClient 但需要特殊处理 Blob 响应
    const url = buildUrl(client.baseUrl, `/api/contents/${id}/thumbnail`);
    const token = client.getToken();
    
    const response = await fetch(url, {
        headers: {
            'Authorization': token ? buildAuthHeader(token) : '',
        },
    });
    
    if (!response.ok) {
        const error = await parseErrorResponse(response);
        throw error;
    }
    
    return response.blob();
}
```

### Content Store 扩展

在 `frontend/src/stores/useContentStore.ts` 中添加：

#### 新增 State

```typescript
// 缩略图URL缓存: content_id -> Object URL
const thumbnailUrls = ref<Map<number, string>>(new Map());

// 正在加载的缩略图ID集合，用于防止重复加载
const thumbnailLoading = ref<Set<number>>(new Set());
```

#### 新增 Getters

```typescript
/**
 * 获取指定内容的缩略图URL
 * 如果缩略图未加载，返回null
 */
const getThumbnailUrl = computed(() => {
    return (contentId: number): string | null => {
        return thumbnailUrls.value.get(contentId) ?? null;
    };
});

/**
 * 检查缩略图是否正在加载
 */
const isThumbnailLoading = computed(() => {
    return (contentId: number): boolean => {
        return thumbnailLoading.value.has(contentId);
    };
});
```

#### 新增 Actions

```typescript
/**
 * 加载单个缩略图
 * 如果已缓存或正在加载，直接返回
 */
async function loadThumbnail(contentId: number): Promise<void> {
    // 已缓存，直接返回
    if (thumbnailUrls.value.has(contentId)) {
        return;
    }
    
    // 正在加载，避免重复请求
    if (thumbnailLoading.value.has(contentId)) {
        return;
    }
    
    thumbnailLoading.value.add(contentId);
    
    try {
        const blob = await getContentApi(getToken).getThumbnail(contentId);
        const url = URL.createObjectURL(blob);
        thumbnailUrls.value.set(contentId, url);
    } catch (error) {
        // 静默处理错误，不影响其他缩略图加载
        console.warn(`Failed to load thumbnail for content ${contentId}:`, error);
    } finally {
        thumbnailLoading.value.delete(contentId);
    }
}

/**
 * 预加载内容列表的所有缩略图
 */
async function preloadThumbnails(contents: ContentResponse[]): Promise<void> {
    const loadPromises = contents
        .filter(content => content.has_thumbnail)
        .map(content => loadThumbnail(content.id));
    
    // 并发加载所有缩略图，不等待全部完成
    // 使用 Promise.allSettled 确保单个失败不影响其他
    await Promise.allSettled(loadPromises);
}

/**
 * 清除缩略图缓存
 * 释放所有 Object URLs 以防止内存泄漏
 */
function invalidateThumbnailCache(contentId?: number): void {
    if (contentId !== undefined) {
        const url = thumbnailUrls.value.get(contentId);
        if (url) {
            URL.revokeObjectURL(url);
            thumbnailUrls.value.delete(contentId);
        }
    } else {
        // 清除所有缓存
        for (const url of thumbnailUrls.value.values()) {
            URL.revokeObjectURL(url);
        }
        thumbnailUrls.value.clear();
    }
}
```

#### 修改现有 Actions

修改 `fetchContents` 方法，在获取内容后自动预加载缩略图：

```typescript
async function fetchContents(libraryId: number, force = false): Promise<ContentResponse[]> {
    error.value = null;

    if (!force && contents.value.has(libraryId)) {
        return contents.value.get(libraryId)!;
    }

    loading.value = true;
    try {
        const response = await getContentApi(getToken).list(libraryId);
        contents.value.set(libraryId, response);
        
        // 自动预加载缩略图（不阻塞返回）
        preloadThumbnails(response).catch(err => {
            console.warn('Failed to preload thumbnails:', err);
        });
        
        return response;
    } catch (e) {
        error.value = e instanceof Error ? e.message : '获取内容列表失败';
        throw e;
    } finally {
        loading.value = false;
    }
}
```

修改 `deleteContent` 方法，删除内容时清除对应的缩略图缓存：

```typescript
async function deleteContent(id: number): Promise<void> {
    error.value = null;
    loading.value = true;
    
    try {
        await getContentApi(getToken).delete(id);

        // 从缓存中移除
        for (const [libraryId, contentList] of contents.value.entries()) {
            const filtered = contentList.filter(c => c.id !== id);
            if (filtered.length !== contentList.length) {
                contents.value.set(libraryId, filtered);
            }
        }

        chapters.value.delete(id);
        
        // 清除缩略图缓存
        invalidateThumbnailCache(id);

        if (currentContent.value?.id === id) {
            currentContent.value = null;
        }
    } catch (e) {
        error.value = e instanceof Error ? e.message : '删除内容失败';
        throw e;
    } finally {
        loading.value = false;
    }
}
```

修改 `invalidateCache` 方法，同时清除缩略图缓存：

```typescript
function invalidateCache(libraryId?: number): void {
    if (libraryId !== undefined) {
        contents.value.delete(libraryId);
        // 清除该库的所有缩略图
        const libraryContents = contents.value.get(libraryId) ?? [];
        libraryContents.forEach(content => {
            invalidateThumbnailCache(content.id);
        });
    } else {
        contents.value.clear();
        chapters.value.clear();
        // 清除所有缩略图
        invalidateThumbnailCache();
    }
}
```

## Data Models

### 现有类型（无需修改）

```typescript
// frontend/src/api/types.ts
export interface ContentResponse {
    id: number;
    library_id: number;
    content_type: ContentType;
    title: string;
    chapter_count: number;
    has_thumbnail: boolean;  // 已存在，用于判断是否需要加载缩略图
    metadata: unknown | null;
    created_at: string;
}
```

### Store State 类型

```typescript
// Content Store 内部状态
interface ContentStoreState {
    // 现有状态
    contents: Map<number, ContentResponse[]>;
    currentContent: ContentResponse | null;
    chapters: Map<number, Chapter[]>;
    loading: boolean;
    error: string | null;
    
    // 新增状态
    thumbnailUrls: Map<number, string>;      // content_id -> Object URL
    thumbnailLoading: Set<number>;           // 正在加载的 content_id 集合
}
```

## 
Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

在编写correctness properties之前，让我先进行prework分析：


### Property Reflection

在编写属性之前，让我识别并消除冗余：

**冗余分析：**

1. **属性 1.1 和 5.5 重复** - 都测试fetchContents自动触发预加载。保留1.1，移除5.5。

2. **属性 1.2 和 3.1 部分重叠** - 1.2测试URL存储到缓存，3.1测试Object URL创建和存储。3.1更全面，包含了1.2的内容。保留3.1，移除1.2。

3. **属性 1.3 和 3.2 相关但不重复** - 1.3测试从缓存返回URL，3.2测试缓存命中时不发起新请求。两者互补，都保留。

4. **属性 2.3, 2.4, 2.5 是实现细节** - 这些是接口存在性检查，不是行为属性。可以合并为一个example测试。

5. **属性 3.4 和 3.5 是实现细节** - 数据结构和方法存在性检查，可以合并为一个example测试。

6. **属性 4.1 和 4.3 相关** - 4.1测试立即返回，4.3测试异步执行。4.3更全面，包含了4.1的含义。保留4.3，移除4.1。

**保留的属性：**
- 1.1: 自动触发预加载
- 1.3: 从缓存返回URL
- 1.4: 缓存未命中返回null
- 1.5: 错误处理不影响其他加载
- 2.1: 使用统一认证
- 2.2: 返回Blob对象
- 3.1: 创建Object URL并缓存
- 3.2: 缓存命中不发起新请求
- 3.3: 清除时释放Object URLs
- 4.3: 异步执行不阻塞
- 4.5: 加载失败不影响内容列表
- 5.3: 删除内容清除缩略图缓存
- 5.4: invalidate同时清除两种缓存

**合并的example测试：**
- API接口完整性（2.3, 2.4, 2.5）
- Store结构完整性（3.4, 3.5）

现在编写correctness properties：

### Property 1: Automatic thumbnail preload trigger

*For any* library ID and content list returned by fetchContents, when the content list contains items with has_thumbnail=true, the Content Store should automatically initiate thumbnail loading for those items.

**Validates: Requirements 1.1**

### Property 2: Cache retrieval consistency

*For any* content ID with a loaded thumbnail, calling getThumbnailUrl should return the same URL that was stored in the cache.

**Validates: Requirements 1.3**

### Property 3: Cache miss returns null immediately

*For any* content ID that has not been loaded, calling getThumbnailUrl should immediately return null without blocking or throwing errors.

**Validates: Requirements 1.4**

### Property 4: Error isolation in batch loading

*For any* set of content IDs where some API calls fail, the successful thumbnail loads should complete and be cached, while failed loads should not prevent other loads from completing.

**Validates: Requirements 1.5**

### Property 5: Authenticated API calls

*For any* thumbnail API request, the request should include an Authorization header with the Bearer token format.

**Validates: Requirements 2.1**

### Property 6: Blob response type

*For any* successful getThumbnail API call, the returned value should be a Blob object.

**Validates: Requirements 2.2**

### Property 7: Object URL creation and caching

*For any* content ID loaded for the first time, the Content Store should create an Object URL from the Blob and store it in the thumbnailUrls Map.

**Validates: Requirements 3.1**

### Property 8: Cache hit prevents redundant requests

*For any* content ID that has already been loaded, subsequent calls to loadThumbnail should not trigger new API requests and should return immediately.

**Validates: Requirements 3.2**

### Property 9: Object URL cleanup on cache invalidation

*For any* cached thumbnail URLs, when invalidateThumbnailCache is called, all Object URLs should be revoked using URL.revokeObjectURL before being removed from the cache.

**Validates: Requirements 3.3**

### Property 10: Non-blocking asynchronous execution

*For any* content list fetch operation, the fetchContents method should return the content data before all thumbnails have finished loading.

**Validates: Requirements 4.3**

### Property 11: Content list availability despite thumbnail failures

*For any* content list where thumbnail loading fails, the content list data should remain accessible and usable through the Content Store.

**Validates: Requirements 4.5**

### Property 12: Thumbnail cache cleanup on content deletion

*For any* content ID that has a cached thumbnail, when deleteContent is called for that ID, the thumbnail cache entry should be removed and its Object URL should be revoked.

**Validates: Requirements 5.3**

### Property 13: Coordinated cache invalidation

*For any* library ID or global cache invalidation, calling invalidateCache should clear both the contents Map and the thumbnailUrls Map, revoking all Object URLs.

**Validates: Requirements 5.4**

## Error Handling

### API错误处理

1. **网络错误**
   - 缩略图加载失败时，静默记录错误到console.warn
   - 不抛出异常，不影响其他缩略图加载
   - 不影响内容列表的正常显示

2. **认证错误**
   - 如果token无效或过期，getThumbnail会抛出ApiError
   - loadThumbnail捕获错误并静默处理
   - 用户可以通过重新登录解决

3. **404错误**
   - 如果内容没有缩略图但has_thumbnail=true，返回404
   - 静默处理，不在UI显示错误
   - 保持占位符显示

### 资源管理错误

1. **内存泄漏防护**
   - 所有Object URL在不再需要时必须被revoke
   - invalidateThumbnailCache确保清理所有URL
   - Store销毁时应清理所有资源

2. **并发控制**
   - thumbnailLoading Set防止同一ID的重复加载
   - 如果已在加载中，后续请求直接返回
   - 避免竞态条件

### 降级策略

1. **缩略图不可用时**
   - 显示默认占位符图标（📚）
   - 不阻塞用户浏览内容
   - 不显示错误消息

2. **API完全不可用时**
   - 内容列表正常显示
   - 所有缩略图显示占位符
   - 不影响其他功能

## Testing Strategy

### Unit Testing

使用Vitest进行单元测试，覆盖以下场景：

1. **Content API测试**
   - getThumbnail方法存在性和签名
   - 成功返回Blob对象
   - 正确处理认证header
   - 错误响应处理

2. **Content Store测试**
   - State初始化正确
   - Getters返回正确值
   - Actions正确更新state
   - 错误处理逻辑

3. **集成测试**
   - fetchContents触发预加载
   - 缓存机制工作正常
   - 删除操作清理缓存
   - invalidateCache清理所有资源

### Property-Based Testing

使用fast-check库进行属性测试，每个测试运行至少100次迭代：

1. **Property 1-13测试**
   - 每个correctness property对应一个PBT测试
   - 使用fast-check生成随机输入
   - 验证属性在所有输入下都成立

2. **生成器策略**
   - Content ID: fc.integer({ min: 1, max: 10000 })
   - Content list: fc.array(fc.record({ id, has_thumbnail, ... }))
   - API responses: fc.oneof(success, error)
   - Blob: 使用mock Blob对象

3. **测试标记**
   - 每个PBT测试使用注释标记对应的property
   - 格式: `// **Feature: auto-thumbnail-fetch, Property N: [property text]**`
   - 便于追踪和维护

### 测试文件组织

```
frontend/tests/
  stores/
    content.thumbnail.props.test.ts  # Property-based tests
  unit/
    api/
      content.thumbnail.test.ts      # Unit tests for API
    stores/
      content.thumbnail.test.ts      # Unit tests for Store
```

### Mock策略

1. **API Mocking**
   - 使用vi.mock模拟fetch调用
   - 模拟成功和失败响应
   - 模拟不同的HTTP状态码

2. **Blob Mocking**
   - 创建简单的Blob对象用于测试
   - 不需要真实的图片数据

3. **URL Mocking**
   - Mock URL.createObjectURL和URL.revokeObjectURL
   - 追踪调用以验证资源管理

## Implementation Notes

### 性能考虑

1. **并发控制**
   - 使用Promise.allSettled并发加载多个缩略图
   - 不限制并发数，依赖浏览器的连接池管理
   - 单个失败不影响其他加载

2. **缓存策略**
   - 使用Map结构提供O(1)查找性能
   - Object URL在内存中，访问速度快
   - 不设置过期时间，依赖手动invalidate

3. **内存管理**
   - 及时revoke不再使用的Object URL
   - 考虑在大型库中实现LRU缓存（未来优化）

### 兼容性

1. **浏览器API**
   - URL.createObjectURL: 所有现代浏览器支持
   - Blob: 所有现代浏览器支持
   - fetch: 所有现代浏览器支持

2. **TypeScript**
   - 使用严格类型检查
   - 所有API都有完整的类型定义

### 未来扩展

1. **LRU缓存**
   - 当缓存数量超过阈值时，清除最久未使用的缩略图
   - 防止大型库占用过多内存

2. **预加载优先级**
   - 优先加载可见区域的缩略图
   - 使用Intersection Observer API

3. **缩略图尺寸优化**
   - 后端支持多种尺寸的缩略图
   - 根据显示尺寸请求合适的版本

4. **离线支持**
   - 使用Service Worker缓存缩略图
   - 支持离线浏览
