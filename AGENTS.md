# AGENTS.md - Coding Guidelines for Ryuri

This document provides guidelines for AI agents working on the Ryuri comic reader project.

## Project Overview

Ryuri is a self-hosted comic/manga reader with:

- **Frontend**: Vue 3 + TypeScript + Vite + Tailwind CSS 4 + shadcn-vue
- **Backend**: Rust + Axum + SQLx + SQLite

## Build/Lint/Test Commands

### Frontend (`cd frontend`)

```bash
pnpm install                    # Install dependencies
pnpm dev                        # Dev server (proxies /api to localhost:3000)
pnpm build                      # Build for production
pnpm vue-tsc -b --noEmit        # Type check only
```

### Backend (`cd backend`)

```bash
cargo run                       # Run dev server
cargo build --release           # Build release binary
cargo test                      # Run all tests
cargo test <test_name>          # Single test (e.g., cargo test auth::tests::login)
cargo test --test <props_file>  # Property tests (e.g., cargo test --test auth_props)
cargo clippy                    # Lint
cargo clippy -- -D warnings     # Lint (treat warnings as errors)
```

### Full Production Build

```bash
cd frontend && pnpm install && pnpm build
cd backend && cargo build --release
# Binary: backend/target/release/backend
```

## Code Style Guidelines

### TypeScript/Vue (Frontend)

- **Indentation**: 4 spaces
- **Quotes**: Single quotes (preferred), though some files use double quotes
- **Semicolons**: No semicolons (though occasionally appear after imports)
- **Vue Components**:
    - Use `<script setup lang="ts">` (Composition API)
    - Props interface named `Props`
    - Components use PascalCase (e.g., `Button.vue`)
    - One component per file
- **Import Order**:
    1. Vue/core libraries (e.g., `vue`, `pinia`, `vue-router`)
    2. Third-party libraries (e.g., `vue-sonner`, `zod`)
    3. `@/` aliases (e.g., `@/api/client`, `@/components/ui`)
    4. Relative imports
- **Naming**:
    - Components: PascalCase
    - Utilities: camelCase
    - Stores: `useXxxStore` pattern (e.g., `useAuthStore`)
- **Types**: Explicit return types on exported functions

### Rust (Backend)

- **Indentation**: 4 spaces (rustfmt default)
- **Documentation**:
    - Module-level: `//!` at top of file
    - Item-level: `///` with examples where helpful
- **Import Order**:
    1. Standard library (`std`, `core`)
    2. External crates (e.g., `axum`, `tokio`, `sqlx`)
    3. Internal modules (`crate::`)
- **Naming**:
    - Types/Traits: PascalCase
    - Functions/Variables: snake_case
    - Constants: UPPER_SNAKE_CASE
    - Modules: snake_case
- **Error Handling**:
    - Use `thiserror` for error enums
    - Return `Result<T, AppError>` from handlers
    - Use `?` operator for propagation
    - Custom `AppError` enum in `error.rs`

### Testing

#### Backend

- Unit tests: `#[cfg(test)]` modules
- Integration tests: `tests/` directory
- Property tests: `proptest!` macro

## Project Structure

```
frontend/
  src/
    components/ui/    # shadcn-vue components
    views/            # Page-level components
    stores/           # Pinia stores
    api/              # API client functions
    lib/              # Utility functions (cn, etc.)
    types/            # TypeScript type definitions
    router.ts         # Vue Router
    main.ts           # Entry point

backend/
  src/
    handlers/         # HTTP route handlers
    services/         # Business logic
    models/           # DTOs/entities
    repository/       # Database access layer
    middlewares/      # Axum middleware
    extractors/       # Custom extractors
    db.rs             # Database init
    error.rs          # Error types
    router.rs         # Route config
    state.rs          # AppState/config
    main.rs           # Entry point
  tests/              # Integration/property tests
```

## Environment Variables

Backend requires:

- `DATABASE_URL` - SQLite string (e.g., `sqlite:ryuri.db?mode=rwc`)
- `JWT_SECRET` - Random hex string for JWT signing
- `HOST` - Bind address (default: 0.0.0.0)
- `PORT` - Port (default: 3000)
- `JWT_EXPIRATION_HOURS` - Token expiry (default: 24)

## Key Dependencies

### Frontend

- Vue 3.5, Vue Router 4, Pinia 3
- Tailwind CSS 4, shadcn-vue, reka-ui
- Zod 4, @vueuse/core, vue-i18n

### Backend

- Axum 0.8, SQLx 0.8, Tokio 1.49
- Argon2, jsonwebtoken 10, thiserror 2
- proptest, rust-embed, rust-i18n

## Notes

- Frontend proxies `/api` to `http://127.0.0.1:3000` during development
- Backend embeds frontend assets at compile time (using `rust-embed`)
- Internationalization via `vue-i18n` (frontend) and `rust-i18n` (backend)
- Komga API compatibility for Mihon app integration
- Run `cargo fmt` and `cargo clippy` before committing Rust code
- Build frontend before building backend binary for embedded assets
