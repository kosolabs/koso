FROM rust:1.84.1@sha256:738ae99a3d75623f41e6882566b4ef37e38a9840244a47efd4a0ca22e9628b88 AS build
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
