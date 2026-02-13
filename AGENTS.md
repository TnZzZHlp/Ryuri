# AGENTS.md - Coding Guidelines for Ryuri

This document provides guidelines for AI agents working on the Ryuri comic reader project.

## Project Overview

Ryuri is a self-hosted comic/manga reader with:
- **Frontend**: Vue 3 + TypeScript + Vite + Tailwind CSS + shadcn-vue
- **Backend**: Rust + Axum + SQLx + SQLite

## Build/Lint/Test Commands

### Frontend (`cd frontend`)
```bash
# Install dependencies
pnpm install

# Development server (proxies /api to localhost:3000)
pnpm dev

# Build for production
pnpm build

# Type check only
pnpm vue-tsc -b --noEmit

# Run tests (Vitest configured but no test script in package.json)
npx vitest run
npx vitest run --reporter=verbose    # Verbose output
npx vitest run --reporter=basic      # Basic output
npx vitest run <path-to-test-file>   # Single test file
```

### Backend (`cd backend`)
```bash
# Run development server
cargo run

# Build release binary
cargo build --release

# Run all tests
cargo test

# Run single test
cargo test <test_name>
cargo test <test_name> -- --nocapture  # With output

# Run property tests (proptest)
cargo test --test <props_file>  # e.g., cargo test --test auth_props

# Format code
cargo fmt

# Lint
cargo clippy
cargo clippy -- -D warnings  # Treat warnings as errors
```

### Full Build (Production)
```bash
cd frontend && pnpm install && pnpm build
cd ../backend && cargo build --release
# Binary: backend/target/release/backend
```

## Code Style Guidelines

### TypeScript/Vue (Frontend)

- **Indentation**: 4 spaces
- **Quotes**: Single quotes for strings
- **Semicolons**: No semicolons (enforced by default in Vue ecosystem)
- **Vue Components**:
  - Use `<script setup lang="ts">` (Composition API)
  - Props interface named `Props`
  - Components use PascalCase (e.g., `Button.vue`)
  - One component per file
- **Imports**:
  - Group imports: Vue/libs first, then `@/` aliases, then relative
  - Use `@/` alias for src directory imports
- **Types**: Explicit return types on exported functions
- **Naming**:
  - Components: PascalCase
  - Files: PascalCase for components, camelCase for utilities
  - Stores: `useXxxStore` pattern (e.g., `useAuthStore`)

### Rust (Backend)

- **Indentation**: 4 spaces (standard rustfmt)
- **Formatting**: Use `cargo fmt`
- **Documentation**:
  - Module-level: `//!` comments at top of file
  - Item-level: `///` comments with doc attributes
  - Include examples in doc comments where helpful
- **Imports** (grouped and ordered):
  1. Standard library (`std`, `core`)
  2. External crates
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
- **Modules**: Each major feature gets its own module with mod.rs or lib.rs entry

### Testing

#### Frontend (Vitest)
- Configured in `vitest.config.ts`
- Uses Node environment
- Globals enabled
- Tests live alongside source files or in `__tests__` directories
- Run single test: `npx vitest run path/to/test.ts`

#### Backend (Built-in + Proptest)
- Unit tests: `#[cfg(test)]` modules in source files
- Integration tests: `tests/` directory
- Property tests: Use `proptest!` macro with strategies
- Run specific test file: `cargo test --test <filename>`

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
    router.ts         # Vue Router configuration
    main.ts           # Entry point

backend/
  src/
    handlers/         # HTTP route handlers
    services/         # Business logic
    models/           # Data structures (DTOs, entities)
    repository/       # Database access layer
    middlewares/      # Axum middleware
    extractors/       # Custom Axum extractors
    db.rs             # Database initialization
    error.rs          # Error types and handling
    router.rs         # Route configuration
    state.rs          # AppState and configuration
    main.rs           # Entry point
  tests/              # Integration and property tests
```

## Environment Variables

Backend requires:
- `DATABASE_URL` - SQLite connection string (e.g., `sqlite:ryuri.db?mode=rwc`)
- `JWT_SECRET` - Secret for JWT signing (generate random hex string)
- `HOST` - Bind address (default: 0.0.0.0)
- `PORT` - Port (default: 3000)
- `JWT_EXPIRATION_HOURS` - Token expiry (default: 24)

## Key Dependencies

### Frontend
- Vue 3.5, Vue Router 4, Pinia 3
- Tailwind CSS 4, shadcn-vue, reka-ui
- Zod 4 for validation
- @vueuse/core for utilities

### Backend
- Axum 0.8 (web framework)
- SQLx 0.8 (async SQLite)
- Tokio 1.49 (async runtime)
- Argon2 (password hashing)
- jsonwebtoken 10 (JWT)
- thiserror 2 (error handling)
- proptest (property testing)

## Notes

- Frontend proxies `/api` to `http://127.0.0.1:3000` during development
- Backend embeds frontend assets at compile time (using `rust-embed`)
- Internationalization supported via `rust-i18n` (backend) and `vue-i18n` (frontend)
- Follow existing patterns in the codebase for consistency
- Run `cargo fmt` and `cargo clippy` before committing Rust code
- Build frontend before building backend binary for embedded assets
