FROM rust:1.84.0@sha256:e6e40c05cfe7dd55ad13794333d31b6d0818f0c6086876e7dc65871e6c8c0b21 AS backend

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
WORKDIR /app
COPY kosolib/Cargo.toml ./kosolib/
COPY backend/build/dummy.rs kosolib/src/lib.rs
COPY backend/Cargo.toml Cargo.lock rust-toolchain.toml ./backend/
COPY backend/build/dummy.rs backend/build/dummy.rs
WORKDIR /app/backend
RUN cargo build --release --lib
# Clean up the dummy buil deps. Otherwise, below, cargo will build against the dummy kosolib.
RUN rm target/release/deps/libkoso* target/release/deps/koso*

# Build the backend.
WORKDIR /app
COPY kosolib/src/ ./kosolib/src/
COPY backend/src/ ./backend/src/
WORKDIR /app/backend
RUN cargo build --release

# Build the sqlx binary, used to apply database migrations.
FROM rust:1.84.0@sha256:e6e40c05cfe7dd55ad13794333d31b6d0818f0c6086876e7dc65871e6c8c0b21 AS sqlx
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node:23.6.1@sha256:3b73c4b366d490f76908dda253bb4516bbb3398948fd880d8682c5ef16427eca AS frontend
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app

# Setup dependencies
COPY frontend/package.json frontend/pnpm-lock.yaml ./
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

ENV DATABASE_URL=postgresql://koso:koso@localhost/koso
CMD ["./koso"]
