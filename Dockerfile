# syntax=docker/dockerfile:1.7
# Single-stage build for crm-quality-inspector (Rust + axum).
# Single-stage because multi-stage layered caches were producing stale
# binary MD5s on rebuild (cargo incremental cache reuse). This one liner
# always rebuilds fresh and works reliably.
FROM rust:1.88-bookworm
WORKDIR /app

# Install libpq-dev for sqlx build, plus runtime deps (libpq5 + tini + curl
# for HEALTHCHECK). We build AND run in the same image — it's larger
# (~2.5GB) but reliable and matches the "systemd service" workflow.
RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq-dev libpq5 ca-certificates tini curl \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static

# Build fresh
RUN cargo clean 2>/dev/null; \
    cargo build --release --bin crm-quality-inspector

# ENV defaults intentionally NOT set. Operators MUST provide:
#   - DATABASE_URL  (postgres connection string)
#   - ADMIN_USERNAME
#   - ADMIN_PASSWORD (≥12 chars, no weak substrings — see src/lib.rs)
#   - RUST_LOG      (defaults to "info" if unset; "debug" leaks details)
#   - SERVER_ADDR   (defaults to 0.0.0.0:3000 in the binary)
# The app refuses to start if DATABASE_URL or ADMIN_PASSWORD are missing/weak.
EXPOSE 3000

# Healthcheck (tini + curl + /api/health must all be present)
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -fsS http://127.0.0.1:3000/api/health || exit 1

# tini reaps zombies and forwards signals
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/target/release/crm-quality-inspector"]
