FROM rust AS backend
WORKDIR /app

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
# Mount caches to preserve the build cache build over build.
# Copy the build cache out to expose it to the build below.
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/build/dummy.rs build/dummy.rs
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release --lib \
    && cp --preserve -r /app/target ./cache-target \
    && cp --preserve -r /usr/local/cargo/git/db ./cargo-git-db \
    && cp --preserve -r /usr/local/cargo/registry ./cargo-registry

# Move cargo build cache files to their proper places
# in order to speed along the subsequent build.
RUN mv ./cache-target /app/target  \
    && mkdir /usr/local/cargo/git \
    && mv  ./cargo-git-db /usr/local/cargo/git/db \
    && mv ./cargo-registry /usr/local/cargo/registry

# Build the backend.
COPY backend/src ./src
RUN cargo build --release

# Build the sqlx binary, used to apply database migrations.
FROM rust AS sqlx
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node AS frontend
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
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app

COPY --from=sqlx /app/bin/sqlx ./
COPY backend/migrations ./migrations
COPY --from=backend /app/target/release/koso ./
COPY --from=frontend /app/build ./static

ENV DATABASE_URL=postgresql://koso:koso@localhost/koso
CMD ["./koso"]