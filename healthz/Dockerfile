FROM rust:1.82.0@sha256:d9c3c6f1264a547d84560e06ffd79ed7a799ce0bff0980b26cf10d29af888377 AS build
WORKDIR /app

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
COPY ../Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./rust-toolchain.toml ./
COPY ./build/dummy.rs ./build/dummy.rs
RUN cargo build --release --lib

# Build the backend.
COPY ./src/ ./src/
RUN cargo build --release

# Assemble the app.
#
# Use the :debug image to debug
# https://github.com/GoogleContainerTools/distroless?tab=readme-ov-file#debug-images
FROM gcr.io/distroless/cc-debian12@sha256:2fb69596e692931f909c4c69ab09e50608959eaf8898c44fa64db741a23588b0 AS runtime
WORKDIR /app

COPY --from=build /app/target/release/healthz ./
ENV RUST_BACKTRACE=1
ENTRYPOINT ["./healthz"]
