FROM rust:1.85.0@sha256:caa4a0e7bd1fe2e648caf3d904bc54c3bfcae9e74b4df2eb9ebe558c9e9e88c5 AS build
WORKDIR /app/healthz

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
COPY ./healthz/Cargo.toml Cargo.lock rust-toolchain.toml  ./
COPY ./healthz/build/dummy.rs ./build/dummy.rs
RUN cargo build --release --lib

# Build the backend.
COPY ./healthz/src/ ./src/
RUN cargo build --release

# Assemble the app.
#
# Use the :debug image to debug
# https://github.com/GoogleContainerTools/distroless?tab=readme-ov-file#debug-images
FROM gcr.io/distroless/cc-debian12@sha256:b7550f0b15838de14c564337eef2b804ba593ae55d81ca855421bd52f19bb480 AS runtime
WORKDIR /app

COPY --from=build /app/healthz/target/release/healthz ./

ENV PORT=8000
ENV RUST_BACKTRACE=1

CMD ["./healthz"]
