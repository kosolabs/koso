FROM rust:1.82.0@sha256:d9c3c6f1264a547d84560e06ffd79ed7a799ce0bff0980b26cf10d29af888377 AS backend

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
WORKDIR /app
COPY Cargo.lock Cargo.lock
COPY kosolib/Cargo.toml kosolib/rust-toolchain.toml ./kosolib/
COPY backend/build/dummy.rs kosolib/src/lib.rs
COPY backend/Cargo.toml backend/rust-toolchain.toml ./backend/
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
FROM rust:1.82.0@sha256:d9c3c6f1264a547d84560e06ffd79ed7a799ce0bff0980b26cf10d29af888377 AS sqlx
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node:23.3.0@sha256:a2fea8b0b74b6e828caa6d83f4b2a0dcb2eb1ff90f30205c32f7bd36ddf976c4 AS frontend
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
FROM gcr.io/distroless/cc-debian12@sha256:2fb69596e692931f909c4c69ab09e50608959eaf8898c44fa64db741a23588b0 AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/backend/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV DATABASE_URL=postgresql://koso:koso@localhost/koso
CMD ["./koso"]
