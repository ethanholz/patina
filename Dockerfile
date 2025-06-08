# Leveraging the pre-built Docker images with
# cargo-chef and the Rust toolchain
FROM clux/muslrust:stable as chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target aarch64-unknown-linux-musl --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --target aarch64-unknown-linux-musl --release

# We do not need the Rust toolchain to run the binary!
FROM alpine:latest as runtime
ENV PORT=3000
WORKDIR /app
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/byos-rust /app
COPY assets /app/assets
COPY templates /app/templates
RUN apk add --no-cache \
    chromium \
    font-liberation \
    ca-certificates
ENTRYPOINT ["/app/byos-rust"]
