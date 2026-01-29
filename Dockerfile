# Build stage
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY assets ./assets

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/qrlink /app/qrlink

# Copy migrations (needed for embedded migrations)
COPY --from=builder /app/migrations /app/migrations

# Copy branding assets
COPY --from=builder /app/assets /app/assets

# Create data directory with proper permissions
RUN mkdir -p /data && chmod 777 /data

# Set environment variables
ENV DATABASE_URL=sqlite:/data/shortener.db
ENV BASE_URL=http://localhost:8080
ENV HOST=0.0.0.0
ENV PORT=8080
ENV RATE_LIMIT_PER_MINUTE=60
ENV QR_BRANDING_LOGO=/app/assets/logo.svg
ENV QR_SIZE=512
ENV CLEANUP_INTERVAL_MINUTES=60
ENV RUST_LOG=qrlink=info,tower_http=info

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/qrlink"]
