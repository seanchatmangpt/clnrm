# OTLP Infrastructure for clnrm Weaver Validation

## Overview

Production-grade OTLP (OpenTelemetry Protocol) collector infrastructure for clnrm v1.2.0 Weaver live-check validation.

**Architecture:**
```
clnrm → OTLP Collector (4317/4318) → Jaeger Backend → Weaver Validation
```

## Components

### 1. OTLP Collector
- **Image:** `otel/opentelemetry-collector-contrib:0.112.0`
- **Purpose:** Receive telemetry from clnrm, process, and export to Jaeger
- **Ports:**
  - `4317` - OTLP gRPC receiver
  - `4318` - OTLP HTTP receiver
  - `13133` - Health check endpoint
  - `8888` - Prometheus metrics
  - `55679` - zpages (debug UI)

### 2. Jaeger Backend
- **Image:** `jaegertracing/all-in-one:1.63`
- **Purpose:** Trace storage and visualization
- **Ports:**
  - `16686` - Jaeger UI
  - `4317` - OTLP receiver (internal)
  - `14268` - Jaeger native receiver
  - `14269` - Health check

## Quick Start

### Start Infrastructure

```bash
# Start OTLP collector + Jaeger
./scripts/start_weaver_collector.sh

# Output will show:
#   ✅ Services started
#   ✅ Health checks passed
#   📡 Access information
```

### Configure clnrm

```bash
# Export OTLP endpoint
source ./scripts/otlp_config.sh

# Or manually
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"
export OTEL_SERVICE_NAME="clnrm"
```

### Run Tests with Telemetry

```bash
# Run clnrm tests (telemetry will export to collector)
clnrm self-test --suite quick

# Validate OTLP export chain
./scripts/validate_otlp_export.sh
```

### View Traces

Open Jaeger UI: http://localhost:16686

### Stop Infrastructure

```bash
# Stop services (preserve data)
./scripts/stop_weaver_collector.sh

# Stop and remove data
./scripts/stop_weaver_collector.sh --clean
```

## Scripts

### `start_weaver_collector.sh`
**Purpose:** Start OTLP infrastructure with health checks

**Features:**
- Prerequisites validation (Docker, Compose, daemon)
- Service startup with Docker Compose
- Health check monitoring
- Endpoint testing
- Status reporting
- Access information display

**Usage:**
```bash
./scripts/start_weaver_collector.sh
```

### `stop_weaver_collector.sh`
**Purpose:** Clean shutdown of infrastructure

**Options:**
- Normal: `./scripts/stop_weaver_collector.sh` (preserve volumes)
- Clean: `./scripts/stop_weaver_collector.sh --clean` (remove all data)

### `validate_otlp_export.sh`
**Purpose:** End-to-end validation of OTLP export chain

**Tests:**
1. Infrastructure health
2. OTLP endpoint connectivity
3. Telemetry export from clnrm
4. Trace visibility in Jaeger
5. Collector metrics
6. Sample trace retrieval

**Output:** Validation report in `/tmp/otlp_validation_report_*.txt`

### `health_check_collector.sh`
**Purpose:** Quick health status check

**Checks:**
- Container running status
- Container health status
- Network port listening
- HTTP endpoint responses
- Trace count
- Resource usage

**Usage:**
```bash
./scripts/health_check_collector.sh
```

## Configuration Files

### `docker-compose.weaver.yml`
Docker Compose definition for OTLP infrastructure.

**Services:**
- `otel-collector` - OpenTelemetry Collector
- `jaeger` - Jaeger all-in-one backend

**Networks:**
- `clnrm-weaver-network` - Bridge network for service communication

**Volumes:**
- `otel-logs` - Persistent storage for collector logs

### `config/otel-collector-config.yaml`
OpenTelemetry Collector configuration.

**Receivers:**
- OTLP gRPC (port 4317)
- OTLP HTTP (port 4318)

**Processors:**
- `batch` - Batch processing (1s timeout, 512 batch size)
- `memory_limiter` - OOM prevention (512 MiB limit)
- `resource` - Resource attribute enrichment
- `attributes` - Custom attribute injection

**Exporters:**
- `otlp/jaeger` - Export to Jaeger
- `logging` - Console output (debugging)
- `file` - Persistent file storage

**Pipelines:**
- Traces: otlp → processors → jaeger + logging + file
- Metrics: otlp → processors → logging + file
- Logs: otlp → processors → logging + file

## Endpoints Reference

### OTLP Endpoints
```
gRPC:  http://localhost:4317
HTTP:  http://localhost:4318/v1/traces
       http://localhost:4318/v1/metrics
       http://localhost:4318/v1/logs
```

### Jaeger UI
```
UI:    http://localhost:16686
API:   http://localhost:16686/api
```

### Health Checks
```
Collector: http://localhost:13133
Jaeger:    http://localhost:14269
```

### Debug Tools
```
Metrics:   http://localhost:8888/metrics
zpages:    http://localhost:55679/debug/tracez
pprof:     http://localhost:1777/debug/pprof
```

## Integration with clnrm

### Telemetry Export

clnrm uses OpenTelemetry SDK to export telemetry:

```rust
use clnrm_core::telemetry::{init_otel, OtelConfig, Export};

let config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "testing",
    sample_ratio: 1.0,
    export: Export::OtlpGrpc {
        endpoint: "http://localhost:4317"
    },
    enable_fmt_layer: false,
    headers: None,
};

let _guard = init_otel(config)?;
```

### Environment Variables

clnrm respects standard OTel environment variables:

```bash
OTEL_EXPORTER_OTLP_ENDPOINT       # OTLP endpoint
OTEL_EXPORTER_OTLP_PROTOCOL       # grpc or http
OTEL_SERVICE_NAME                 # Service name
OTEL_SERVICE_VERSION              # Service version
OTEL_RESOURCE_ATTRIBUTES          # Resource attributes
OTEL_TRACES_SAMPLER               # Sampling strategy
```

## Weaver Validation

### Prerequisites

1. **Start infrastructure:**
   ```bash
   ./scripts/start_weaver_collector.sh
   ```

2. **Configure clnrm:**
   ```bash
   source ./scripts/otlp_config.sh
   ```

3. **Run tests to generate telemetry:**
   ```bash
   clnrm self-test --suite quick
   ```

### Run Weaver Live Check

```bash
# Validate telemetry against schemas
weaver registry live-check --registry registry/

# Expected output:
#   ✅ All schemas validated
#   ✅ Runtime telemetry matches schemas
```

### Validation Workflow

1. **Schema Validation:** `weaver registry check -r registry/`
   - Validates schema definitions
   - Checks for schema errors

2. **Live Validation:** `weaver registry live-check --registry registry/`
   - Captures runtime telemetry from OTLP collector
   - Validates telemetry against schemas
   - Detects schema violations

3. **Manual Verification:**
   - View traces in Jaeger UI
   - Check spans match schema definitions
   - Verify attributes are present and correct

## Troubleshooting

### Infrastructure Won't Start

**Check Docker daemon:**
```bash
./scripts/docker_startup.sh
```

**Check prerequisites:**
```bash
docker --version
docker compose version
docker ps
```

**View logs:**
```bash
docker compose -f docker-compose.weaver.yml logs
docker logs clnrm-otel-collector
docker logs clnrm-jaeger
```

### No Telemetry in Jaeger

**Verify OTLP endpoint:**
```bash
echo $OTEL_EXPORTER_OTLP_ENDPOINT  # Should be http://localhost:4317
```

**Check port connectivity:**
```bash
nc -z localhost 4317  # gRPC
nc -z localhost 4318  # HTTP
```

**Test collector health:**
```bash
curl http://localhost:13133
```

**Run validation script:**
```bash
./scripts/validate_otlp_export.sh
```

### Collector Not Healthy

**Check logs:**
```bash
docker logs clnrm-otel-collector --tail 100
```

**Common issues:**
- Configuration file syntax error
- Port already in use (4317, 4318)
- Jaeger not running
- Insufficient memory

**Restart infrastructure:**
```bash
./scripts/stop_weaver_collector.sh --clean
./scripts/start_weaver_collector.sh
```

### Weaver Validation Fails

**Verify telemetry export:**
```bash
./scripts/validate_otlp_export.sh
```

**Check trace in Jaeger:**
1. Open http://localhost:16686
2. Select service "clnrm"
3. Click "Find Traces"
4. Verify spans match schema

**Common issues:**
- No telemetry exported (check OTEL_EXPORTER_OTLP_ENDPOINT)
- Schema mismatch (check span names, attributes)
- Missing attributes (check resource configuration)

## Performance Tuning

### Batch Processing

Default: 1s timeout, 512 batch size

**Faster export (testing):**
```yaml
processors:
  batch:
    timeout: 100ms      # Export every 100ms
    send_batch_size: 64  # Smaller batches
```

**Efficient export (production):**
```yaml
processors:
  batch:
    timeout: 5s          # Export every 5s
    send_batch_size: 1024 # Larger batches
```

### Memory Limits

Default: 512 MiB limit, 128 MiB spike

**High-volume testing:**
```yaml
processors:
  memory_limiter:
    limit_mib: 1024
    spike_limit_mib: 256
```

### Sampling

Current: 100% sampling (all traces)

**Production sampling:**
```bash
export OTEL_TRACES_SAMPLER=traceidratio
export OTEL_TRACES_SAMPLER_ARG=0.1  # 10% sampling
```

## Monitoring

### Collector Metrics

```bash
# Prometheus metrics
curl http://localhost:8888/metrics

# Key metrics
otelcol_receiver_accepted_spans    # Spans received
otelcol_exporter_sent_spans        # Spans exported
otelcol_processor_batch_batch_send_size  # Batch sizes
```

### Jaeger Metrics

```bash
# Jaeger health
curl http://localhost:14269

# Service list
curl http://localhost:16686/api/services

# Trace count
curl 'http://localhost:16686/api/traces?service=clnrm&limit=100'
```

### Resource Usage

```bash
# Container stats
docker stats clnrm-otel-collector clnrm-jaeger

# Logs
docker compose -f docker-compose.weaver.yml logs -f --tail=100
```

## Production Deployment

### Security

**Add authentication:**
```yaml
# config/otel-collector-config.yaml
exporters:
  otlp/jaeger:
    endpoint: jaeger:4317
    headers:
      authorization: "Bearer ${API_TOKEN}"
```

**Use TLS:**
```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
        tls:
          cert_file: /etc/certs/server.crt
          key_file: /etc/certs/server.key
```

### High Availability

**Multiple collectors:**
```yaml
services:
  otel-collector-1:
    # ... config ...
  otel-collector-2:
    # ... config ...

  load-balancer:
    image: nginx
    # ... load balancer config ...
```

**Persistent storage:**
```yaml
exporters:
  otlp/jaeger:
    endpoint: cassandra-backend:4317
```

### Scaling

**Horizontal scaling:**
- Deploy multiple collector instances
- Load balance OTLP traffic
- Use persistent Jaeger backend (Cassandra, Elasticsearch)

**Vertical scaling:**
- Increase memory limits
- Adjust batch sizes
- Optimize processor pipeline

## References

- [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [clnrm Telemetry Guide](../WEAVER_USER_GUIDE.md)
