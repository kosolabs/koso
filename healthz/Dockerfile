FROM rust:1.92.0@sha256:bed2d7f8140d73c26f16c298c91ae8487a09f40d3840c0d8d139537e1b51e148 AS build
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
FROM gcr.io/distroless/cc-debian13@sha256:05d26fe67a875592cd65f26b2bcfadb8830eae53e68945784e39b23e62c382e0 AS runtime
WORKDIR /app

COPY --from=build /app/target/release/healthz ./

ENV PORT=8000
ENV RUST_BACKTRACE=1

CMD ["./healthz"]
