# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/lib.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/governance-agent /usr/local/bin/
COPY --from=builder /app/target/release/voting-aggregator /usr/local/bin/
COPY --from=builder /app/target/release/reward-distributor /usr/local/bin/

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Default command
CMD ["governance-agent"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep -f governance-agent || exit 1

# Expose any ports if needed (adjust as necessary)
# EXPOSE 8080

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1