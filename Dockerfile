# syntax=docker/dockerfile:1
#
# Rust Öğren (TR): one image with the axum server, the static frontend, the Rust
# toolchain + rust-analyzer, and every crate the exercises need (offline builds).
#
#   docker compose up --build        # http://localhost:3000
#
# Stages: web (Next.js static export) -> server-build (release binary) -> runtime.

ARG RUST_IMAGE=rust:1.96.1-slim-trixie
ARG NODE_IMAGE=node:22.23.1-trixie-slim

# ---------------------------------------------------------------------------------------
# 1. Frontend: `next build` with `output: 'export'` -> app/web/out
#    generateStaticParams reads ../../book/src/SUMMARY.md and ../../exercises at build time.
# ---------------------------------------------------------------------------------------
FROM ${NODE_IMAGE} AS web
WORKDIR /src
COPY app/web/package.json app/web/package-lock.json app/web/
RUN cd app/web && npm ci --no-audit --no-fund
COPY book/src/SUMMARY.md book/src/SUMMARY.md
COPY exercises exercises
COPY tr tr
COPY app/web app/web
ENV NEXT_TELEMETRY_DISABLED=1
RUN cd app/web && npm run build && test -f out/index.html

# ---------------------------------------------------------------------------------------
# 2. Server binary. All root workspace members must be present for cargo to load the
#    workspace, hence exercises/, helpers/ and app/tools/ are copied too.
# ---------------------------------------------------------------------------------------
FROM ${RUST_IMAGE} AS server-build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY exercises exercises
COPY helpers helpers
COPY app/server app/server
COPY app/tools app/tools
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p server \
    && cp target/release/server /usr/local/bin/server

# ---------------------------------------------------------------------------------------
# 3. Runtime: toolchain (cargo/rustc compile the user's code), rust-analyzer, content.
# ---------------------------------------------------------------------------------------
FROM ${RUST_IMAGE} AS runtime
# rust-src is needed by rust-analyzer for std completions/hover.
RUN rustup component add rust-analyzer rust-src \
    && apt-get update \
    && apt-get install -y --no-install-recommends tini \
    && rm -rf /var/lib/apt/lists/*

# User code runs as an unprivileged user. Named volumes inherit the ownership of these
# directories on first mount.
RUN useradd --create-home --uid 1000 learner \
    && mkdir -p /app/app/workspace /app/app/data \
    && chown -R learner:learner /app/app

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY book book
COPY exercises exercises
COPY helpers helpers
COPY tr tr

USER learner
# Per-user CARGO_HOME for the registry cache; the toolchain stays in /usr/local (read-only).
ENV CARGO_HOME=/home/learner/.cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Pre-fetch every crate the exercises depend on (tokio, thiserror, anyhow, ...) using the
# same workspace layout the server generates, so builds work offline.
RUN mkdir /tmp/prefetch \
    && cp -r exercises helpers Cargo.lock /tmp/prefetch/ \
    && printf '[workspace]\nmembers = ["exercises/*/*", "helpers/*"]\nresolver = "2"\n' > /tmp/prefetch/Cargo.toml \
    && cd /tmp/prefetch && cargo fetch \
    && rm -rf /tmp/prefetch

COPY --from=server-build /usr/local/bin/server /usr/local/bin/server
COPY --from=web /src/app/web/out /app/app/web/out

ENV RUST_LEARN_ROOT=/app \
    WEB_DIST=/app/app/web/out \
    WORKSPACE_DIR=/app/app/workspace \
    DATA_DIR=/app/app/data \
    HOST=0.0.0.0 \
    PORT=3000 \
    CARGO_NET_OFFLINE=true \
    RUST_ANALYZER=rust-analyzer

EXPOSE 3000
# tini reaps orphaned grandchildren (rustc, test threads' subprocesses) after group kills.
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["server"]
