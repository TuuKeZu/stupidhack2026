# Multi-stage Dockerfile for building and running the Rust warp websocket server

# Builder stage: compile the binary
FROM rust:latest as builder
WORKDIR /usr/src/app

# Copy manifests and fetch dependencies to maximize cache usage
COPY Cargo.toml Cargo.lock ./
# Provide a minimal src so `cargo fetch` can run and cache dependencies
RUN mkdir -p src && echo "fn main() { println!(\"placeholder\"); }" > src/main.rs && \
    cargo fetch --locked || true

# Copy full source and build release binary
COPY . .
RUN cargo build --release --locked

# Runtime stage: small image with only the binary
FROM debian:bookworm-slim
# Install CA certs for TLS/HTTPS if needed
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder
COPY --from=builder /usr/src/app/target/release/stupidhack-2026 /usr/local/bin/stupidhack-2026

# Expose the server port
EXPOSE 3030

# Run as non-root user for better security (optional)
USER 1000

ENTRYPOINT ["/usr/local/bin/stupidhack-2026"]
