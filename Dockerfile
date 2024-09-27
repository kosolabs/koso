FROM rust AS backend

WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock ./
# Speed up re-builds triggered by changes to src/
# by building a dummy library and thus dependencies
# in a separate layer.
COPY backend/build/dummy.rs build/dummy.rs
RUN cargo build --release --lib

# Now run the real build!
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release
RUN mv target/release/koso .

FROM rust AS sqlx

WORKDIR /app

RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres --root ./

FROM node AS frontend

WORKDIR /app

COPY frontend/*.json frontend/*.js frontend/*.cjs frontend/*.ts ./
RUN npm ci

COPY frontend/src ./src
COPY frontend/static ./static
RUN npm run build

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

COPY --from=backend /app/koso ./
COPY --from=backend /app/migrations ./migrations
COPY --from=sqlx /app/bin/sqlx ./
COPY --from=frontend /app/build ./static

ENV DATABASE_URL=postgresql://koso:koso@host.docker.internal/koso

CMD ["./koso"]