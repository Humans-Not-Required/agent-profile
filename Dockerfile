# Stage 1: Build React frontend
FROM node:22-slim AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci --prefer-offline

COPY frontend/ .
RUN npm run build

# Stage 2: Build Rust binary (with embedded frontend assets)
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
COPY tests/ tests/
COPY openapi.json ./
COPY SKILL.md ./

# Copy the compiled frontend so rust-embed can include it
COPY --from=frontend-builder /app/frontend/dist/ frontend/dist/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/agent-profile /usr/local/bin/agent-profile

# Stage 3: Runtime
FROM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/Humans-Not-Required/agent-profile"
LABEL org.opencontainers.image.description="Agent 'About Me' Profile Pages — canonical identity pages for AI agents"
LABEL org.opencontainers.image.licenses="MIT"

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash appuser
WORKDIR /app

COPY --from=builder /usr/local/bin/agent-profile /app/agent-profile

RUN mkdir -p /data && chown appuser:appuser /data

USER appuser

ENV DATABASE_URL=/data/agent-profile.db
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8003

EXPOSE 8003

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:8003/api/v1/health || exit 1

CMD ["/app/agent-profile"]
