# Use the official Rust image as the base
FROM rust:latest AS builder

# Install libssl-dev
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application with cargo
RUN cargo build --release

# Create the runtime image
FROM debian:bookworm-slim

# Install libssl-dev
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates

# Create log directory
RUN mkdir -p /var/log/chase && \
    chmod 777 /var/log/chase

# Copy the built binary from builder
COPY --from=builder /usr/src/app/target/release/chase /usr/local/bin/app

# Set default file locations
ENV TOKENS_FILE="/app/config/tokens.json"
ENV WALLETS_FILE="/app/config/wallets.json"

# Environment variables for runtime
ENV TRIGGER_ENDPOINT=""
ENV TRIGGER_API_TOKEN=""
ENV SOL_RPC_ENDPOINT=""
ENV SOL_WSS_ENDPOINT=""
ENV IMPORT_TOKENS_ENDPOINT="https://api.example.com/api/v1/tokens?network=SOL"
ENV IMPORT_WALLETS_ENDPOINT="https://api.example.com/api/v1/wallets?network=SOL"
ENV IMPORT_INTERVAL_SECONDS="3600"

# Run the binary
CMD ["app"]
