# Implementation Plan

- [x] 1. 扩展Content API添加缩略图获取方法





  - 在`frontend/src/api/content.ts`中添加`getThumbnail`方法
  - 实现Blob响应处理逻辑
  - 确保使用统一的认证机制
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 1.1 为getThumbnail方法编写属性测试


  - **Property 5: Authenticated API calls**
  - **Validates: Requirements 2.1**






- [x] 1.2 为getThumbnail方法编写属性测试





  - **Property 6: Blob response type**
  - **Validates: Requirements 2.2**




- [x] 2. 扩展Content Store添加缩略图状态管理





  - 添加`thumbnailUrls`和`thumbnailLoading`状态
  - 实现`getThumbnailUrl`和`isThumbnailLoading` getters
  - 添加缩略图相关的响应式引用
  - _Requirements: 3.4, 3.5_



- [ ] 3. 实现缩略图加载核心逻辑











  - 实现`loadThumbnail`方法处理单个缩略图加载
  - 实现`preloadThumbnails`方法批量预加载


  - 添加加载状态跟踪防止重复请求
  - 实现错误处理和静默失败逻辑


  - _Requirements: 1.2, 1.4, 1.5, 3.1_



- [ ] 3.1 为loadThumbnail编写属性测试
  - **Property 7: Object URL creation and caching**
  - **Validates: Requirements 3.1**

- [ ] 3.2 为loadThumbnail编写属性测试


  - **Property 8: Cache hit prevents redundant requests**
  - **Validates: Requirements 3.2**



- [ ] 3.3 为getThumbnailUrl编写属性测试
  - **Property 2: Cache retrieval consistency**
  - **Validates: Requirements 1.3**

- [ ] 3.4 为getThumbnailUrl编写属性测试
  - **Property 3: Cache miss returns null immediately**
  - **Validates: Requirements 1.4**

- [ ] 3.5 为preloadThumbnails编写属性测试
  - **Property 4: Error isolation in batch loading**
  - **Validates: Requirements 1.5**


- [x] 4. 实现缩略图缓存管理





  - 实现`invalidateThumbnailCache`方法
  - 确保正确释放Object URLs防止内存泄漏


  - 支持单个和批量清除操作
  - _Requirements: 3.3, 3.5_

- [x] 4.1 为invalidateThumbnailCache编写属性测试


  - **Property 9: Object URL cleanup on cache invalidation**
  - **Validates: Requirements 3.3**

- [-] 5. 集成缩略图预加载到fetchContents












  - 修改`fetchContents`方法自动触发预加载
  - 确保预加载不阻塞内容列表返回
  - 实现异步预加载逻辑
  - _Requirements: 1.1, 4.1, 4.3, 5.5_

- [ ] 5.1 为fetchContents集成编写属性测试






  - **Property 1: Automatic thumbnail preload trigger**
  - **Validates: Requirements 1.1**

- [ ] 5.2 为fetchContents集成编写属性测试
  - **Property 10: Non-blocking asynchronous execution**
  - **Validates: Requirements 4.3**

- [ ] 5.3 为fetchContents集成编写属性测试
  - **Property 11: Content list availability despite thumbnail failures**
  - **Validates: Requirements 4.5**

- [x] 6. 集成缩略图清理到内容删除






  - 修改`deleteContent`方法清除对应的缩略图缓存
  - 确保Object URL被正确释放
  - _Requirements: 5.3_

- [x] 6.1 为deleteContent集成编写属性测试


  - **Property 12: Thumbnail cache cleanup on content deletion**
  - **Validates: Requirements 5.3**


- [x] 7. 集成缩略图清理到缓存失效





  - 修改`invalidateCache`方法同时清除缩略图缓存
  - 支持按库ID和全局清除
  - 确保所有Object URLs被释放
  - _Requirements: 5.4_

- [x] 7.1 为invalidateCache集成编写属性测试


  - **Property 13: Coordinated cache invalidation**
  - **Validates: Requirements 5.4**


- [x] 8. 更新Library.vue组件使用新的缩略图功能




  - 移除组件内的缩略图加载逻辑
  - 使用Content Store的`getThumbnailUrl` getter
  - 简化组件代码，依赖Store管理
  - _Requirements: 1.3, 4.2_



- [ ] 9. Checkpoint - 确保所有测试通过






  - 确保所有测试通过，如有问题请询问用户
