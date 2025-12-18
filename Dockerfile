# Stage 1: Build Frontend
FROM node:20-slim AS frontend-builder
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

WORKDIR /app/frontend

COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY frontend/ .
RUN pnpm build

# Stage 2: Build Backend
FROM rust:1-bookworm AS backend-builder

WORKDIR /app/backend

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy source code
COPY backend/ .

# Copy frontend dist to where rust-embed expects it (../frontend/dist relative to backend/)
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# Build
RUN cargo build --release --features dev

# Stage 3: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/backend/target/release/backend /app/ryuri

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

CMD ["/app/ryuri"]
