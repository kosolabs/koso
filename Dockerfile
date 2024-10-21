FROM rust:1.82.0@sha256:81584ce20ac0fc77ac45384c28f356cb76489e8c71998962fed0008dbe496987 AS backend
WORKDIR /app

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/build/dummy.rs build/dummy.rs
RUN cargo build --release --lib

# Build the backend.
COPY backend/src ./src
RUN cargo build --release

# Build the sqlx binary, used to apply database migrations.
FROM rust:1.82.0@sha256:81584ce20ac0fc77ac45384c28f356cb76489e8c71998962fed0008dbe496987 AS sqlx
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node:23.0.0@sha256:9d09fa506f5b8465c5221cbd6f980e29ae0ce9a3119e2b9bc0842e6a3f37bb59 AS frontend
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
FROM gcr.io/distroless/cc-debian12@sha256:3310655aac0d85eb9d579792387af1ff3eb7a1667823478be58020ab0e0d97a8 AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV DATABASE_URL=postgresql://koso:koso@localhost/koso
CMD ["./koso"]