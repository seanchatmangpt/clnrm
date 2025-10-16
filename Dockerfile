# Multi-stage Dockerfile for clnrm self-testing
# Stage 1: Build clnrm with OTEL features
FROM rust:1.86-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build clnrm with OTEL features
RUN cargo build --release --features otel-traces --bin clnrm

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy clnrm binary from builder
COPY --from=builder /build/target/release/clnrm /usr/local/bin/clnrm

# Set up working directory
WORKDIR /workspace

# Configure OTEL environment variables
ENV OTEL_SERVICE_NAME=clnrm
ENV OTEL_RESOURCE_ATTRIBUTES=deployment.environment=test
ENV OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
ENV OTEL_EXPORTER_OTLP_ENDPOINT=http://otel_collector:4318
ENV OTEL_TRACES_EXPORTER=otlp
ENV OTEL_METRICS_EXPORTER=none
ENV OTEL_LOGS_EXPORTER=none

# clnrm CLI entrypoint
ENTRYPOINT ["clnrm"]
CMD ["--help"]
