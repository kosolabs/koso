FROM rust:1.88.0@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0 AS build
WORKDIR /app/healthz

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ../
COPY ./healthz/Cargo.toml ./
COPY ./backend/Cargo.toml ../backend/
COPY ./healthz/build/dummy.rs ./build/dummy.rs
RUN cargo build --release --lib

# Build the binary.
COPY ./healthz/src/ ./src/
RUN cargo build --release

# Assemble the app.
#
# Use the :debug image to debug
# https://github.com/GoogleContainerTools/distroless?tab=readme-ov-file#debug-images
FROM gcr.io/distroless/cc-debian12@sha256:bf0494952368db47e9e38eecc325c33f5ee299b1b1ccc5d9e30bdf1e5e4e3a58 AS runtime
WORKDIR /app

COPY --from=build /app/target/release/healthz ./

ENV PORT=8000
ENV RUST_BACKTRACE=1

CMD ["./healthz"]
