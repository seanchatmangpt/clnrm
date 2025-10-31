# Docker Deployment Runbook

## Overview

Deploy clnrm with Weaver live-check validation in Docker containers.

## Prerequisites

- Docker 20.10+
- Docker Compose 1.29+ (optional)
- Registry available at `./registry/`

## Single Container Deployment

### 1. Build Image

```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder

WORKDIR /build
COPY . .

# Install Weaver
RUN cargo install weaver-cli

# Build clnrm with OTEL support
RUN cargo build --release --features otel

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /build/target/release/clnrm /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/weaver /usr/local/bin/

# Copy registry
COPY registry/ /app/registry/

WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
  CMD clnrm --version || exit 1

CMD ["clnrm", "run", "--validate"]
```

### 2. Build

```bash
docker build -t clnrm:latest .
```

### 3. Run

```bash
docker run -d \
  --name clnrm-test \
  -v $(pwd)/tests:/app/tests:ro \
  -v $(pwd)/validation_output:/app/validation_output \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
  -p 4317:4317 \
  -p 8080:8080 \
  clnrm:latest
```

### 4. Verify

```bash
# Check status
docker ps | grep clnrm

# View logs
docker logs clnrm-test

# Get validation report
docker exec clnrm-test cat /app/validation_output/validation_report.json
```

### 5. Cleanup

```bash
docker stop clnrm-test
docker rm clnrm-test
```

## Docker Compose Deployment

### 1. Create docker-compose.yml

```yaml
version: '3.8'

services:
  weaver:
    image: clnrm:latest
    command:
      - weaver
      - registry
      - live-check
      - --registry=/app/registry
      - --otlp-grpc-port=4317
      - --admin-port=8080
      - --output=/app/validation_output
      - --format=json
    volumes:
      - ./registry:/app/registry:ro
      - validation-output:/app/validation_output
    ports:
      - "4317:4317"
      - "8080:8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 3s
      retries: 3
    networks:
      - clnrm-network

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    command: ["--config=/etc/otel-config.yaml"]
    volumes:
      - ./otel-config.yaml:/etc/otel-config.yaml:ro
    ports:
      - "4318:4318"  # OTLP HTTP
    depends_on:
      - weaver
    networks:
      - clnrm-network

  clnrm-tests:
    image: clnrm:latest
    command: cargo test --features otel --workspace
    environment:
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://weaver:4317
      - RUST_LOG=info
      - RUST_BACKTRACE=1
    volumes:
      - .:/app
      - test-cache:/app/target
    depends_on:
      weaver:
        condition: service_healthy
    networks:
      - clnrm-network

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "4319:4317"    # OTLP gRPC
    networks:
      - clnrm-network

volumes:
  validation-output:
  test-cache:

networks:
  clnrm-network:
    driver: bridge
```

### 2. OTEL Collector Config

```yaml
# otel-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

  otlp:
    endpoint: weaver:4317
    tls:
      insecure: true

  logging:
    loglevel: info

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger, otlp, logging]
```

### 3. Deploy

```bash
# Start stack
docker-compose up -d

# View logs
docker-compose logs -f

# Check status
docker-compose ps

# View Jaeger UI
open http://localhost:16686
```

### 4. Run Tests

```bash
# Run tests in container
docker-compose run clnrm-tests

# Get validation report
docker-compose exec weaver cat /app/validation_output/validation_report.json
```

### 5. Cleanup

```bash
docker-compose down -v
```

## Multi-Stage Pipeline

### 1. Build Stage

```yaml
# docker-compose.build.yml
version: '3.8'

services:
  builder:
    image: rust:1.70
    working_dir: /build
    volumes:
      - .:/build
      - cargo-cache:/usr/local/cargo
    command: cargo build --release --features otel

volumes:
  cargo-cache:
```

### 2. Test Stage

```yaml
# docker-compose.test.yml
version: '3.8'

services:
  test-runner:
    image: clnrm:latest
    command: ./scripts/production_validation.sh quick
    environment:
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://weaver:4317
    depends_on:
      - weaver
```

### 3. Validation Stage

```yaml
# docker-compose.validate.yml
version: '3.8'

services:
  validator:
    image: clnrm:latest
    command: weaver registry live-check --registry /app/registry
    volumes:
      - ./registry:/app/registry:ro
      - validation-results:/app/output
```

### 4. Execute Pipeline

```bash
# Build
docker-compose -f docker-compose.build.yml up

# Test
docker-compose -f docker-compose.test.yml up --abort-on-container-exit

# Validate
docker-compose -f docker-compose.validate.yml up

# Check results
cat validation_output/validation_report.json
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs clnrm-test

# Inspect container
docker inspect clnrm-test

# Verify image
docker images | grep clnrm

# Rebuild
docker build --no-cache -t clnrm:latest .
```

### Network Issues

```bash
# Check network
docker network inspect clnrm-network

# Test connectivity
docker exec clnrm-test ping weaver

# DNS resolution
docker exec clnrm-test nslookup weaver
```

### Volume Permissions

```bash
# Check permissions
docker exec clnrm-test ls -la /app/validation_output

# Fix permissions
docker exec clnrm-test chown -R 1000:1000 /app/validation_output
```

## Production Deployment

### 1. Registry Configuration

```yaml
# docker-compose.prod.yml
services:
  clnrm:
    image: clnrm:v1.2.0  # Use tagged version
    restart: unless-stopped
    environment:
      - ENVIRONMENT=production
      - OTEL_EXPORTER_OTLP_ENDPOINT=${OTLP_ENDPOINT}
    secrets:
      - otlp-credentials
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M

secrets:
  otlp-credentials:
    external: true
```

### 2. Deploy

```bash
docker stack deploy -c docker-compose.prod.yml clnrm-stack
```

### 3. Monitor

```bash
# Check stack
docker stack ps clnrm-stack

# View logs
docker service logs clnrm-stack_clnrm

# Scale
docker service scale clnrm-stack_clnrm=3
```

---

**Last Updated:** 2025-10-30
