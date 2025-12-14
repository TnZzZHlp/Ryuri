# Requirements Document

## Introduction

本功能为 Wyuri 前端项目创建 `useContentStore`，用于管理库内内容（漫画/小说）的状态。该 Store 负责内容列表缓存、搜索、当前内容选择、章节管理等功能，与 `useLibraryStore` 配合使用，实现完整的内容浏览体验。

## Glossary

- **Content**: 内容项，可以是漫画（Comic）或小说（Novel）
- **Chapter**: 章节，属于某个 Content 的子项
- **ContentStore**: 管理内容状态的 Pinia store
- **Content Cache**: 按 libraryId 分组缓存的内容列表

## Requirements

### Requirement 1

**User Story:** As a developer, I want to create a Content Store to manage content state, so that I can cache content data and share it across components.

#### Acceptance Criteria

1. WHEN the Content Store is created THEN the System SHALL contain state for contents map, currentContent, chapters, loading, and error
2. WHEN the Content Store is accessed THEN the System SHALL provide a getter to retrieve contents by library ID
3. WHEN the Content Store is accessed THEN the System SHALL provide a getter to retrieve the current content's chapters

### Requirement 2

**User Story:** As a user, I want to view all contents in a library, so that I can browse available comics and novels.

#### Acceptance Criteria

1. WHEN contents are fetched for a library THEN the Content Store SHALL cache the results keyed by library ID
2. WHEN contents are fetched for a library that is already cached THEN the Content Store SHALL return cached data without making an API call
3. WHEN a force refresh is requested THEN the Content Store SHALL fetch fresh data from the API and update the cache

### Requirement 3

**User Story:** As a user, I want to search contents within a library, so that I can quickly find specific comics or novels.

#### Acceptance Criteria

1. WHEN a search query is provided THEN the Content Store SHALL call the search API and return matching contents
2. WHEN the search query is empty THEN the Content Store SHALL return all contents from the cache

### Requirement 4

**User Story:** As a user, I want to select and view a specific content, so that I can see its details and chapters.

#### Acceptance Criteria

1. WHEN a content is selected THEN the Content Store SHALL set it as the currentContent
2. WHEN a content is selected THEN the Content Store SHALL fetch and cache its chapters
3. WHEN the same content is selected again THEN the Content Store SHALL use cached chapters without making an API call

### Requirement 5

**User Story:** As a user, I want to delete a content, so that I can remove unwanted items from my library.

#### Acceptance Criteria

1. WHEN a content is deleted THEN the Content Store SHALL remove it from the cache
2. WHEN the deleted content was the currentContent THEN the Content Store SHALL clear currentContent and chapters

### Requirement 6

**User Story:** As a developer, I want the Content Store to handle errors gracefully, so that the UI can display appropriate feedback.

#### Acceptance Criteria

1. WHEN an API call fails THEN the Content Store SHALL set the error state with a descriptive message
2. WHEN a new API call starts THEN the Content Store SHALL clear the previous error state
