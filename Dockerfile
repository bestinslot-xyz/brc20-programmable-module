FROM rust:1.88.0-bookworm AS builder

# Install additional build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl-dev \
        libclang-dev && \
    rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY test_utils/Cargo.toml ./test_utils/

# Copy source code
COPY src ./src
COPY test_utils/src ./test_utils/src
COPY benches ./benches

# Build the application with server features
RUN cargo build --release --features server --bin brc20-prog

FROM gcr.io/distroless/cc-debian12

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/brc20-prog /usr/local/bin/brc20-prog

# Set the entrypoint
ENTRYPOINT ["brc20-prog"]