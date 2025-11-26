FROM rust:1.91.1@sha256:4a29b0db5c961cd530f39276ece3eb6e66925b59599324c8c19723b72a423615 AS build
WORKDIR /app/healthz

# The debian 13 image doesn't seem to include libzstd1. Install it.
# See https://github.com/GoogleContainerTools/distroless/issues/1887
# Error: ./sqlx: error while loading shared libraries: libzstd.so.1: cannot open shared object file: No such file or directory
RUN apt-get update && apt-get install -y --no-install-recommends libzstd1

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
FROM gcr.io/distroless/cc-debian13@sha256:0e90484916aa263753c9885ee4a9b4d6fa756b8242600e2e37fe74c7ec5574f4 AS runtime
WORKDIR /app

COPY --from=build /lib/*/libzstd.so.1 /lib/
COPY --from=build /app/target/release/healthz ./

ENV PORT=8000
ENV RUST_BACKTRACE=1

CMD ["./healthz"]
