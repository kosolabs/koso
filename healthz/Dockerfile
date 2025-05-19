FROM rust:1.87.0@sha256:5e33ae75f40bf25854fa86e33487f47075016d16726355a72171f67362ad6bf7 AS build
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
FROM gcr.io/distroless/cc-debian12@sha256:c53c9416a1acdbfd6e09abba720442444a3d1a6338b8db850e5e198b59af5570 AS runtime
WORKDIR /app

COPY --from=build /app/target/release/healthz ./

ENV PORT=8000
ENV RUST_BACKTRACE=1

CMD ["./healthz"]
