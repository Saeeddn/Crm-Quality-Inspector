# syntax=docker/dockerfile:1.7
# Multi-stage build for crm-quality-inspector.
#
# Why multi-stage (not single-stage): produces a ~60MB runtime image instead
# of the ~2.5GB image you get when the rust toolchain stays in the runtime
# layer. The previous single-stage was a workaround for "stale binary MD5 on
# rebuild" — that was actually a cargo incremental-cache reuse bug, which
# proper layer caching (the dummy main.rs trick below) prevents cleanly.
#
# Layering strategy:
#   1. Copy only Cargo.toml + Cargo.lock → cache dependencies as one layer
#   2. Copy a dummy src/main.rs that imports nothing → cache "deps" build
#   3. Copy real source → incremental rebuild only changes the binary layer
#
# The runtime image only contains: libpq5, tini, curl, the binary, and the
# static/ directory (which is embedded via include_str! at compile time).

# ── Stage 1: build ────────────────────────────────────────────────
FROM rust:1.88-bookworm AS builder

WORKDIR /app

# Install build-time deps for sqlx (libpq-dev for headers)
RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies in a separate layer: copy manifests, build a dummy crate
# that depends on the same crates, then throw the dummy away. This makes
# subsequent rebuilds (when only src/ changes) reuse the cached deps layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 && echo 'fn main() { println!("dummy"); }' > src/main.rs \
 && echo '' > src/lib.rs \
 && cargo build --release --bin crm-quality-inspector \
 && rm -rf src target/release/deps/crm_qi-* target/release/deps/crm_quality_inspector-*

# Now copy the real source and build the actual binary
COPY src ./src
COPY static ./static
# Touch a file to force cargo to see the new source mtime (defensive against
# Windows-style copy not preserving mtimes correctly)
RUN find src -type f -exec touch {} +
RUN cargo build --release --bin crm-quality-inspector

# ── Stage 2: runtime ──────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Runtime dependencies: libpq5 (sqlx runtime), tini (signal handling),
# curl (HEALTHCHECK), ca-certificates (for HTTPS in app code if needed).
RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq5 ca-certificates tini curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r crm && useradd -r -g crm -d /app -s /sbin/nologin crm

WORKDIR /app

# Copy only the binary and static assets. Static is included so future
# runtime code could serve it (the current code embeds via include_str!,
# so static/ is not strictly needed at runtime — but keep it for ops).
COPY --from=builder /app/target/release/crm-quality-inspector /app/crm-quality-inspector
COPY --from=builder /app/static /app/static

# Non-root user for the runtime
RUN chown -R crm:crm /app
USER crm

# ENV defaults intentionally NOT set. Operators MUST provide:
#   - DATABASE_URL  (postgres connection string)
#   - ADMIN_USERNAME
#   - ADMIN_PASSWORD (≥12 chars, no weak substrings — see src/lib.rs)
#   - SERVER_ADDR   (defaults to 0.0.0.0:3000 in the binary)
# The app refuses to start if DATABASE_URL or ADMIN_PASSWORD are missing/weak.
EXPOSE 3000

# Healthcheck (tini + curl + /api/health must all be present)
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -fsS http://127.0.0.1:3000/api/health || exit 1

# tini reaps zombies and forwards signals
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/crm-quality-inspector"]
