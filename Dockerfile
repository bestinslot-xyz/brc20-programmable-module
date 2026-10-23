# syntax=docker/dockerfile:1

# ---- builder ----
# rocksdb (via librocksdb-sys) compiles from C++ and needs clang; pkg-config +
# libssl-dev are required for the OpenSSL-backed dependencies.
FROM rust:1.91-slim AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        clang \
        pkg-config \
        libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build the release binary from the local source tree. --locked ensures the
# build matches the committed Cargo.lock. The default feature set enables the
# `server` feature that the brc20-prog binary requires.
COPY . .
RUN cargo build --release --locked --bin brc20-prog

# ---- runtime ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/brc20-prog /usr/local/bin/brc20-prog

# Container-friendly defaults. The binary's built-in defaults bind to
# 127.0.0.1 and store the DB under target/db, neither of which is useful in a
# container; override here and let runtime env/compose override further.
ENV BRC20_PROG_RPC_SERVER_URL=0.0.0.0:18545 \
    BRC20_PROG_DB_PATH=/var/lib/brc20-prog/data

EXPOSE 18545
VOLUME ["/var/lib/brc20-prog/data"]

ENTRYPOINT ["brc20-prog"]
CMD ["-l", "info"]
