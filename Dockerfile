# Multi-stage build for messaging-service
# Stage 1: Build the Rust binary
# Use latest stable (1.83+) which supports edition2024 crates from crates.io
FROM rust:slim AS builder

WORKDIR /build

# Install required system dependencies for SQLx (OpenSSL, PostgreSQL client libraries)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest first for better layer caching
COPY Cargo.toml ./

# Copy all crate manifests
COPY crates/core/Cargo.toml ./crates/core/
COPY crates/server/Cargo.toml ./crates/server/
COPY crates/db-migrate/Cargo.toml ./crates/db-migrate/

# Copy source code
COPY crates/ ./crates/

# Copy SQLx offline query metadata for compile-time verification without database
COPY .sqlx/ ./.sqlx/

# Build the messaging-server binary in release mode with SQLx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin messaging-server

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL, PostgreSQL client libraries, ca-certificates for HTTPS)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 -s /bin/bash messaging

# Copy the binary from builder
COPY --from=builder /build/target/release/messaging-server /usr/local/bin/messaging-server

# Copy server configuration files
COPY crates/server/config/ /app/config/

# Set working directory
WORKDIR /app

# Change ownership to non-root user
RUN chown -R messaging:messaging /app

# Switch to non-root user
USER messaging

# Expose default port
EXPOSE 8080

# Set default environment variables
ENV PORT=8080 \
    HEALTH_PATH=/healthz \
    LOG_LEVEL=info

# Run the server
CMD ["messaging-server"]
