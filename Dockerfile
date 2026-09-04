# syntax=docker/dockerfile:1.7
# Two-stage build for crm-quality-inspector (Rust + axum)
# Stage 1: build static binary
FROM rust:1.81-bookworm AS builder
WORKDIR /app

# Cache deps separately from source for faster incremental builds
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Now copy real source and rebuild
COPY . .
# Use a dummy main binary target name to avoid the build script collision
RUN cargo build --release --bin crm-quality-inspector

# Stage 2: minimal runtime
FROM debian:bookworm-slim AS runtime

# Install runtime deps: PostgreSQL client, ca-certificates, tini for signal handling
RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq5 ca-certificates tini curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r crm && useradd -r -g crm -d /app -s /sbin/nologin crm

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/crm-quality-inspector /app/crm-quality-inspector

# Copy static assets (the Rust binary includes them via include_str!, but we
# keep them in case you want to mount a custom static dir)
COPY --from=builder /app/static /app/static

# Data directory for SQLite (if used) or any local state
RUN mkdir -p /app/data && chown -R crm:crm /app

USER crm

# Defaults — override with -e or compose
ENV RUST_LOG=info,crm_qi=debug,sqlx=warn \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=3000 \
    ADMIN_USERNAME=admin \
    ADMIN_PASSWORD="" \
    DATABASE_URL=""

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -fsS http://127.0.0.1:3000/api/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/crm-quality-inspector"]
