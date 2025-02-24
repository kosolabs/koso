FROM rust:1.85.0@sha256:ad7e5fd44a71f317c88993a64d4073f9050516cd420ddacd90b7d43829f29f26 AS backend

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
WORKDIR /app
COPY backend/Cargo.toml Cargo.lock rust-toolchain.toml ./backend/
COPY backend/build/dummy.rs backend/build/dummy.rs
WORKDIR /app/backend
RUN cargo build --release --lib

# Build the backend.
WORKDIR /app
COPY backend/src/ ./backend/src/
WORKDIR /app/backend
RUN cargo build --release

# Build the sqlx binary, used to apply database migrations.
FROM rust:1.85.0@sha256:ad7e5fd44a71f317c88993a64d4073f9050516cd420ddacd90b7d43829f29f26 AS sqlx
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node:23.8.0@sha256:a182b9b37154a3e11e5c1d15145470ceb22069646d0b7390de226da2548aa2a7 AS frontend
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
FROM gcr.io/distroless/cc-debian12@sha256:b7550f0b15838de14c564337eef2b804ba593ae55d81ca855421bd52f19bb480 AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/backend/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0

CMD ["./koso"]
