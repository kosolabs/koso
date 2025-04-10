FROM rust:1.86.0@sha256:563b33de55d0add224b2e301182660b59bf3cf7219e9dc0fda68f8500e5fe14a AS backend

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
FROM rust:1.86.0@sha256:563b33de55d0add224b2e301182660b59bf3cf7219e9dc0fda68f8500e5fe14a AS sqlx
WORKDIR /app
COPY rust-toolchain.toml ./
RUN cargo install sqlx-cli@=0.8.3 --locked --no-default-features --features native-tls,postgres --root ./

FROM node:23.11.0@sha256:047d633b358c33f900110efff70b4f1c73d066dec92dd6941c42d26889f69a0e AS frontend
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
FROM gcr.io/distroless/cc-debian12@sha256:c1cbcec08d39c81adbefb80cabc51cba285465866f7b5ab15ddb2fcae51a1aed AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/backend/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=koso=debug,tower_http=trace,sqlx=trace,axum=trace,info

CMD ["./koso"]
