FROM rust:1.83.0@sha256:a45bf1f5d9af0a23b26703b3500d70af1abff7f984a7abef5a104b42c02a292b AS backend

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
FROM rust:1.83.0@sha256:a45bf1f5d9af0a23b26703b3500d70af1abff7f984a7abef5a104b42c02a292b AS sqlx
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node:23.5.0@sha256:1745a99b66da41b5ccd6f7be3810f74ddab16eb4579de10de378adb50d2e6e6f AS frontend
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
FROM gcr.io/distroless/cc-debian12@sha256:f913198471738d9eedcd00c0ca812bf663e8959eebff3a3cbadb027ed9da0c38 AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/backend/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV DATABASE_URL=postgresql://koso:koso@localhost/koso
CMD ["./koso"]
