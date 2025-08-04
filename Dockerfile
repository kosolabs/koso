FROM rust:1.88.0@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0 AS backend

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
FROM rust:1.88.0@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0 AS sqlx
WORKDIR /app
COPY rust-toolchain.toml ./
RUN cargo install sqlx-cli@=0.8.6 --locked --no-default-features --features native-tls,postgres --root ./

FROM node:24.5.0@sha256:dd5c5e4d0a67471a683116483409d1e46605a79521b000c668cff29df06efd51 AS frontend
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
FROM gcr.io/distroless/cc-debian12@sha256:bf0494952368db47e9e38eecc325c33f5ee299b1b1ccc5d9e30bdf1e5e4e3a58 AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=koso=debug,tower_http=trace,sqlx=trace,axum=trace,info

CMD ["./koso"]
