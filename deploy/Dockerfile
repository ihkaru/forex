# Build Stage
FROM rust:1.78-bookworm AS builder
WORKDIR /app

# Copy root workspace manifests
COPY Cargo.toml Cargo.lock ./

# Copy all source crates
COPY crates/ crates/
COPY apps/ apps/

# Build release binaries
RUN cargo build --release --bin signal-daemon --bin api-server --bin scraper-worker

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tzdata && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/signal-daemon /usr/local/bin/signal-daemon
COPY --from=builder /app/target/release/api-server /usr/local/bin/api-server
COPY --from=builder /app/target/release/scraper-worker /usr/local/bin/scraper-worker

ENV RUST_LOG=info
CMD ["signal-daemon"]
