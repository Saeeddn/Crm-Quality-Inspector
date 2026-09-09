# syntax=docker/dockerfile:1.7
# Production Dockerfile for crm-quality-inspector on Ollama server.
# Connects to the native PostgreSQL on host, not a container.
# Logs are written to /var/log/crm-qi/ via mounted volume.

FROM rust:1.88-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 && echo 'fn main() { println!("dummy"); }' > src/main.rs \
 && echo '' > src/lib.rs \
 && cargo build --release --bin crm-quality-inspector \
 && rm -rf src target/release/deps/crm_qi-* target/release/deps/crm_quality_inspector-*

COPY src ./src
COPY static ./static
RUN find src -type f -exec touch {} +
RUN cargo build --release --bin crm-quality-inspector

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq5 ca-certificates tini curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r crm && useradd -r -g crm -d /app -s /sbin/nologin crm \
    && mkdir -p /var/log/crm-qi && chown crm:crm /var/log/crm-qi

WORKDIR /app

COPY --from=builder /app/target/release/crm-quality-inspector /app/crm-quality-inspector
COPY --from=builder /app/static /app/static

RUN chown -R crm:crm /app
USER crm

ENV RUST_LOG=info,crm_qi=info,sqlx=warn
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
ENV LOG_FILE=/var/log/crm-qi/crm-quality-inspector.log

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -fsS http://127.0.0.1:3000/api/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/crm-quality-inspector"]
