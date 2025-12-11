# Requirements Document

## Introduction

本功能将 Pinia 状态管理库引入 Wyuri 前端项目，替换现有的模块级 ref 状态管理方式。通过 Pinia 提供统一的状态管理模式，支持 Vue DevTools 调试、状态持久化、以及更清晰的数据流管理，降低多组件间状态共享的心智负担。

## Glossary

- **Pinia**: Vue 官方推荐的状态管理库，提供类型安全、DevTools 支持和模块化 store
- **Store**: Pinia 中的状态容器，包含 state、getters 和 actions
- **Auth Store**: 管理用户认证状态的 store，包括 token、用户信息、登录/登出操作
- **Library Store**: 管理内容库列表和当前选中库的 store
- **UI Store**: 管理全局 UI 状态的 store，如侧边栏折叠状态、主题等
- **Composable**: Vue 3 组合式函数，用于封装可复用的逻辑

## Requirements

### Requirement 1

**User Story:** As a developer, I want to install and configure Pinia in the project, so that I can use centralized state management.

#### Acceptance Criteria

1. WHEN the application starts THEN the System SHALL initialize Pinia as a Vue plugin before mounting the app
2. WHEN Pinia is configured THEN the System SHALL enable Vue DevTools integration for state inspection
3. WHEN stores are created THEN the System SHALL place them in the `src/stores` directory following naming convention `use{Name}Store.ts`

### Requirement 2

**User Story:** As a developer, I want to migrate authentication state to Pinia, so that I can debug auth state changes and have cleaner component code.

#### Acceptance Criteria

1. WHEN the Auth Store is created THEN the System SHALL contain state for token, user, loading, and error
2. WHEN a user logs in successfully THEN the Auth Store SHALL update token and user state atomically
3. WHEN a user logs out THEN the Auth Store SHALL clear token and user state and remove token from localStorage
4. WHEN the application initializes with an existing token THEN the Auth Store SHALL restore the token from localStorage
5. WHEN the Auth Store token changes THEN the System SHALL persist the token to localStorage immediately
6. WHEN components access auth state THEN the System SHALL provide reactive getters for isAuthenticated and current user

### Requirement 3

**User Story:** As a developer, I want a Library Store to cache library data, so that I can avoid redundant API calls and share library state across components.

#### Acceptance Criteria

1. WHEN the Library Store is created THEN the System SHALL contain state for libraries list, current library, and loading status
2. WHEN libraries are fetched THEN the Library Store SHALL cache the results and provide them to all consuming components
3. WHEN a library is selected THEN the Library Store SHALL update the current library state
4. WHEN a library is created or updated THEN the Library Store SHALL refresh the cached libraries list
5. WHEN a library is deleted THEN the Library Store SHALL remove it from the cached list and clear current library if it was selected

### Requirement 4

**User Story:** As a developer, I want a UI Store for global UI state, so that I can manage sidebar state and other UI preferences consistently.

#### Acceptance Criteria

1. WHEN the UI Store is created THEN the System SHALL contain state for sidebar collapsed status
2. WHEN the sidebar toggle is clicked THEN the UI Store SHALL update the collapsed state
3. WHEN components need sidebar state THEN the System SHALL provide reactive access through the UI Store

### Requirement 5

**User Story:** As a developer, I want to refactor existing components to use Pinia stores, so that the codebase has consistent state management patterns.

#### Acceptance Criteria

1. WHEN AppSidebar component renders THEN the component SHALL access user data from Auth Store instead of calling fetchUser directly
2. WHEN NavUser component renders THEN the component SHALL receive user data from Auth Store
3. WHEN Login view handles authentication THEN the view SHALL use Auth Store actions for login
4. WHEN components need auth state THEN the components SHALL import and use useAuthStore instead of useAuth composable
