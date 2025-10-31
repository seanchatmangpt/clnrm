# OTLP Quick Start - clnrm v1.2.0

## 🚀 3-Step Setup (Recommended)

### 1. Use Existing Infrastructure
```bash
source ./scripts/use_existing_collector.sh
```
**Result:** Environment configured, OTLP endpoint ready at `http://localhost:4317`

### 2. Run Tests with Telemetry
```bash
clnrm self-test --suite quick
```
**Result:** Telemetry automatically exported to OTLP collector

### 3. Validate OTLP Chain
```bash
./scripts/validate_otlp_export.sh
```
**Result:** Verification that traces reached Jaeger backend

## 🔬 Weaver Validation

```bash
# After running tests, validate schemas
weaver registry live-check --registry registry/
```

## 🐳 Alternative: Standalone Infrastructure

If you need isolated OTLP infrastructure:

```bash
# Start
./scripts/start_weaver_collector.sh

# Configure
source ./scripts/otlp_config.sh

# Run tests
clnrm self-test --suite quick

# Validate
./scripts/validate_otlp_export.sh

# Stop
./scripts/stop_weaver_collector.sh
```

## 📊 View Traces

```bash
# Open Jaeger UI
open http://localhost:16686

# Or check collector metrics
curl http://localhost:8888/metrics | grep otelcol
```

## 🏥 Health Check

```bash
# Quick health status
./scripts/health_check_collector.sh
```

## 📚 Full Documentation

See `/Users/sac/clnrm/docs/backend/OTLP_INFRASTRUCTURE.md` for complete guide.

## ⚡ Key Endpoints

- **OTLP gRPC:** `http://localhost:4317`
- **OTLP HTTP:** `http://localhost:4318`
- **Jaeger UI:** `http://localhost:16686`
- **Collector Health:** `http://localhost:13133`
- **Metrics:** `http://localhost:8888/metrics`
