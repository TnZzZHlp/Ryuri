# Stage 1: Build Frontend
FROM node:20-slim AS frontend-builder
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

WORKDIR /app

# Copy VERSION file for frontend build
COPY VERSION .

WORKDIR /app/frontend

COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY frontend/ .
RUN pnpm build

# Stage 2: Build Backend
FROM rust:1-bookworm AS backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev libfontconfig1-dev libclang-dev

# Copy VERSION file first
COPY VERSION .

# Copy source code
COPY backend/ ./backend/

# Copy frontend dist to where rust-embed expects it (../frontend/dist relative to backend/)
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Build
WORKDIR /app/backend
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libfontconfig1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/backend/target/release/backend /app/ryuri

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

CMD ["/app/ryuri"]
