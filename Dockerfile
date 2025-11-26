FROM rust:1.91.1@sha256:4a29b0db5c961cd530f39276ece3eb6e66925b59599324c8c19723b72a423615 AS backend

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY backend/Cargo.toml ./backend/
COPY ./healthz/Cargo.toml ./healthz/
COPY backend/build/dummy.rs backend/build/dummy.rs
WORKDIR /app/backend
RUN cargo build --release --lib

# Build the backend.
WORKDIR /app
COPY backend/src/ ./backend/src/
WORKDIR /app/backend
RUN cargo build --release

# Build the sqlx binary, used to apply database migrations.
FROM rust:1.91.1@sha256:4a29b0db5c961cd530f39276ece3eb6e66925b59599324c8c19723b72a423615 AS sqlx
WORKDIR /app

# The debian 13 image doesn't seem to include libzstd1. Install it.
# See https://github.com/GoogleContainerTools/distroless/issues/1887
# Error: ./sqlx: error while loading shared libraries: libzstd.so.1: cannot open shared object file: No such file or directory
RUN apt-get update && apt-get install -y --no-install-recommends libzstd1

COPY rust-toolchain.toml ./
RUN cargo install sqlx-cli@=0.8.6 --locked --no-default-features --features native-tls,postgres --root ./

FROM node:24.11.1@sha256:aa648b387728c25f81ff811799bbf8de39df66d7e2d9b3ab55cc6300cb9175d9 AS frontend
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app

# Setup dependencies
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/.npmrc ./
RUN pnpm install --frozen-lockfile

# Build the frontend.
COPY frontend/*.json frontend/*.js frontend/*.cjs frontend/*.ts ./
COPY frontend/src ./src
COPY frontend/static ./static
RUN pnpm run build

# Assemble the app.
#
# Use the :debug image to debug
# https://github.com/GoogleContainerTools/distroless?tab=readme-ov-file#debug-images
FROM gcr.io/distroless/cc-debian13@sha256:0e90484916aa263753c9885ee4a9b4d6fa756b8242600e2e37fe74c7ec5574f4 AS runtime
WORKDIR /app

# From zlib1g
COPY --from=sqlx /lib/*/libzstd.so.1 /lib/
COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=koso=debug,tower_http=trace,sqlx=trace,axum=trace,info

CMD ["./koso"]
