FROM rust AS backend

WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock ./
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

# DOCKER_DEFAULT_PLATFORM=linux/amd64 docker build --platform linux/amd64 -t gcr.io/koso/koso .
# TODO: Expose metrics server.
# docker run --env DATABASE_URL=postgresql://$USER@host.docker.internal/$USER --publish 3000:3000 --publish 3001:3001 --rm -it gcr.io/koso/koso:latest
# docker run --env DATABASE_URL=postgresql://$USER@host.docker.internal/$USER --rm -it gcr.io/koso/koso:latest "./sqlx" migrate run
ENV DATABASE_URL=postgresql://koso:koso@host.docker.internal/koso

CMD ["./koso"]