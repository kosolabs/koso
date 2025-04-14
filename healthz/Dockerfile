FROM rust:1.86.0@sha256:7b65306dd21304f48c22be08d6a3e41001eef738b3bd3a5da51119c802321883 AS build
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
FROM gcr.io/distroless/cc-debian12@sha256:c1cbcec08d39c81adbefb80cabc51cba285465866f7b5ab15ddb2fcae51a1aed AS runtime
WORKDIR /app

COPY --from=build /app/healthz/target/release/healthz ./

ENV PORT=8000
ENV RUST_BACKTRACE=1

CMD ["./healthz"]
