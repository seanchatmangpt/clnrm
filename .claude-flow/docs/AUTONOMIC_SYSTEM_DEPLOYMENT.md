# Autonomic System Deployment Guide

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Service Setup](#service-setup)
6. [AI Model Selection](#ai-model-selection)
7. [Performance Tuning](#performance-tuning)
8. [Monitoring & Observability](#monitoring--observability)
9. [Security Best Practices](#security-best-practices)
10. [Troubleshooting](#troubleshooting)
11. [FAQ](#faq)

---

## Quick Start

Get up and running with autonomic features in 5 minutes:

```bash
# 1. Install Cleanroom CLI
cargo install clnrm

# 2. Install Ollama (macOS/Linux)
curl -fsSL https://ollama.ai/install.sh | sh

# 3. Pull a recommended model
ollama pull qwen2.5-coder:7b

# 4. Start Ollama service
ollama serve

# 5. Run your first AI-powered test orchestration
clnrm ai-orchestrate --predict-failures --auto-optimize
```

**That's it!** You now have a fully functional autonomic testing platform.

---

## Prerequisites

### System Requirements

#### Minimum (Development)
- **CPU**: 4 cores
- **RAM**: 8 GB
- **Disk**: 20 GB free space
- **OS**: Linux, macOS, or Windows with WSL2

#### Recommended (Production)
- **CPU**: 8+ cores
- **RAM**: 16+ GB
- **Disk**: 50+ GB SSD
- **OS**: Linux (Ubuntu 22.04+ or RHEL 8+)

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| **Rust** | 1.70+ | Build and run Cleanroom CLI |
| **Docker** | 20.10+ | Container orchestration |
| **Ollama** | 0.1.0+ | Local AI model inference |
| **SurrealDB** | 2.0+ | Real-time database (optional) |

### Optional Dependencies

- **Git**: For version control and CI/CD integration
- **Node.js**: For web dashboard (coming soon)
- **Prometheus**: For metrics collection
- **Grafana**: For visualization

---

## Installation

### 1. Install Rust

```bash
# Install rustup (Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Cleanroom CLI

#### Option A: Install from crates.io (Recommended)

```bash
cargo install clnrm
```

#### Option B: Build from source

```bash
# Clone repository
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm

# Build and install
cargo build --release
cargo install --path crates/clnrm
```

#### Verify Installation

```bash
clnrm --version
# Expected output: clnrm 0.4.0
```

### 3. Install Docker

#### Linux (Ubuntu/Debian)

```bash
# Update package index
sudo apt-get update

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Verify installation
docker --version
docker run hello-world
```

#### macOS

```bash
# Install Docker Desktop
brew install --cask docker

# Start Docker Desktop from Applications
# Or use command line:
open -a Docker

# Verify installation
docker --version
```

#### Windows (WSL2)

```powershell
# Install Docker Desktop for Windows
# Download from: https://www.docker.com/products/docker-desktop

# Enable WSL2 integration
# Configure in Docker Desktop settings

# Verify in WSL2
docker --version
```

### 4. Install Ollama

#### Linux

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Verify installation
ollama --version

# Start Ollama service (runs in background)
ollama serve &
```

#### macOS

```bash
# Option 1: Download and install from website
# https://ollama.ai/download

# Option 2: Use Homebrew
brew install ollama

# Start Ollama service
ollama serve
```

#### Windows

```powershell
# Download installer from https://ollama.ai/download
# Run the installer

# Start Ollama from Start Menu
# Or use command line:
ollama serve
```

### 5. Install SurrealDB (Optional)

SurrealDB is optional but recommended for advanced features like distributed testing and real-time analytics.

#### Using Docker (Recommended)

```bash
# Pull SurrealDB image
docker pull surrealdb/surrealdb:latest

# Run SurrealDB
docker run --name surrealdb \
  -p 8000:8000 \
  -v surrealdb-data:/data \
  surrealdb/surrealdb:latest start \
  --user root \
  --pass root \
  file:/data/database.db

# Verify installation
curl http://localhost:8000/health
```

#### Native Installation

```bash
# Linux/macOS
curl -sSf https://install.surrealdb.com | sh

# Start SurrealDB
surreal start --user root --pass root file://data.db
```

---

## Configuration

### Environment Variables

Create a `.env` file in your project root:

```env
# Ollama Configuration
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=qwen2.5-coder:7b
OLLAMA_TIMEOUT_SECONDS=300

# SurrealDB Configuration (optional)
SURREALDB_HOST=127.0.0.1
SURREALDB_PORT=8000
SURREALDB_USERNAME=root
SURREALDB_PASSWORD=root

# Cleanroom Configuration
CLNRM_LOG_LEVEL=info
CLNRM_PARALLEL_WORKERS=4
CLNRM_MAX_RETRIES=3
CLNRM_TIMEOUT=300

# AI Features
CLNRM_AI_ENABLED=true
CLNRM_AI_CONFIDENCE_THRESHOLD=0.8
CLNRM_AI_PREDICT_FAILURES=true
CLNRM_AI_AUTO_OPTIMIZE=false

# Performance Tuning
CLNRM_CONTAINER_REUSE=true
CLNRM_CACHE_ENABLED=true
CLNRM_MEMORY_LIMIT_MB=2048
```

### Configuration File

Create `cleanroom.toml` in your project root:

```toml
[framework]
name = "my-test-suite"
version = "1.0.0"
timeout = 300
parallel_workers = 4

[ai]
enabled = true
provider = "ollama"
model = "qwen2.5-coder:7b"
confidence_threshold = 0.8
predict_failures = true
auto_optimize = false

[ollama]
host = "http://localhost:11434"
timeout_seconds = 300
default_model = "qwen2.5-coder:7b"

[surrealdb]
enabled = false
host = "127.0.0.1"
port = 8000
username = "root"
password = "root"
strict = false

[performance]
container_reuse = true
cache_enabled = true
memory_limit_mb = 2048
max_retries = 3

[logging]
level = "info"
format = "json"
output = "stdout"

[telemetry]
enabled = true
export_format = "otlp"
endpoint = "http://localhost:4317"
```

---

## Service Setup

### Ollama Service Configuration

#### 1. Configure as systemd Service (Linux)

Create `/etc/systemd/system/ollama.service`:

```ini
[Unit]
Description=Ollama AI Service
After=network.target

[Service]
Type=simple
User=ollama
Group=ollama
ExecStart=/usr/local/bin/ollama serve
Restart=always
RestartSec=10
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_MODELS=/var/lib/ollama/models"

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable ollama
sudo systemctl start ollama
sudo systemctl status ollama
```

#### 2. Configure as launchd Service (macOS)

Create `~/Library/LaunchAgents/com.ollama.service.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.ollama.service</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/ollama</string>
        <string>serve</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load and start:

```bash
launchctl load ~/Library/LaunchAgents/com.ollama.service.plist
launchctl start com.ollama.service
```

### SurrealDB Service Configuration

#### 1. Docker Compose Setup (Recommended)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  surrealdb:
    image: surrealdb/surrealdb:latest
    container_name: surrealdb
    ports:
      - "8000:8000"
    volumes:
      - surrealdb-data:/data
    command: >
      start
      --user root
      --pass root
      --auth
      --log trace
      file:/data/database.db
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

volumes:
  surrealdb-data:
    driver: local
```

Start services:

```bash
docker-compose up -d
docker-compose logs -f surrealdb
```

#### 2. Kubernetes Deployment (Production)

Create `surrealdb-deployment.yaml`:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: surrealdb
spec:
  selector:
    app: surrealdb
  ports:
    - protocol: TCP
      port: 8000
      targetPort: 8000
  type: ClusterIP
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: surrealdb
spec:
  replicas: 1
  selector:
    matchLabels:
      app: surrealdb
  template:
    metadata:
      labels:
        app: surrealdb
    spec:
      containers:
      - name: surrealdb
        image: surrealdb/surrealdb:latest
        ports:
        - containerPort: 8000
        args:
          - "start"
          - "--user=root"
          - "--pass=root"
          - "file:/data/database.db"
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: surrealdb-pvc
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: surrealdb-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

Deploy:

```bash
kubectl apply -f surrealdb-deployment.yaml
kubectl get pods -l app=surrealdb
```

### Health Checks

#### Ollama Health Check

```bash
# Check if Ollama is running
curl http://localhost:11434/api/version

# Expected output:
# {"version":"0.1.0"}

# Test model inference
curl http://localhost:11434/api/generate -d '{
  "model": "qwen2.5-coder:7b",
  "prompt": "Write hello world in Rust",
  "stream": false
}'
```

#### SurrealDB Health Check

```bash
# Check health endpoint
curl http://localhost:8000/health

# Expected output: OK

# Test connection
curl -X POST http://localhost:8000/sql \
  -H "Content-Type: application/json" \
  -u root:root \
  -d '{"query": "INFO FOR DB;"}'
```

---

## AI Model Selection

### Recommended Models for Production

| Model | Size | Use Case | Performance | Memory |
|-------|------|----------|-------------|--------|
| **qwen2.5-coder:7b** | 7B params | General coding, balanced | Fast (2-3s) | 8 GB |
| **qwen2.5-coder:14b** | 14B params | Complex analysis | Medium (4-6s) | 16 GB |
| **codellama:13b** | 13B params | Code generation | Medium (4-5s) | 14 GB |
| **deepseek-coder:6.7b** | 6.7B params | Fast inference | Very fast (1-2s) | 6 GB |
| **phi-3:mini** | 3.8B params | Resource constrained | Very fast (<1s) | 4 GB |

### Pull and Configure Models

```bash
# Pull recommended models
ollama pull qwen2.5-coder:7b
ollama pull deepseek-coder:6.7b
ollama pull phi-3:mini

# List available models
ollama list

# Test a model
ollama run qwen2.5-coder:7b "Explain test-driven development"

# Remove unused models
ollama rm old-model-name
```

### Model Selection Guide

#### Development Environment
- **Best choice**: `phi-3:mini` or `deepseek-coder:6.7b`
- **Why**: Fast inference, low memory footprint
- **Trade-off**: Slightly lower accuracy for complex tasks

#### Production Environment
- **Best choice**: `qwen2.5-coder:7b`
- **Why**: Excellent balance of speed, accuracy, and resource usage
- **Trade-off**: None for most use cases

#### High-Accuracy Requirements
- **Best choice**: `qwen2.5-coder:14b` or `codellama:13b`
- **Why**: Superior accuracy for complex analysis
- **Trade-off**: Higher latency and memory requirements

### Custom Model Configuration

```toml
# cleanroom.toml

[ai.models.fast]
name = "phi-3:mini"
timeout = 10
max_tokens = 500
temperature = 0.1

[ai.models.balanced]
name = "qwen2.5-coder:7b"
timeout = 30
max_tokens = 1000
temperature = 0.3

[ai.models.accurate]
name = "qwen2.5-coder:14b"
timeout = 60
max_tokens = 2000
temperature = 0.5
```

### Benchmark Your Models

```bash
# Run AI orchestration benchmark
clnrm ai-orchestrate --benchmark --output benchmark-results.json

# Compare models
clnrm ai-predict --benchmark --models "phi-3:mini,qwen2.5-coder:7b,codellama:13b"
```

---

## Performance Tuning

### CPU and Memory Optimization

#### 1. Configure Parallel Workers

```bash
# Automatic (recommended)
export CLNRM_PARALLEL_WORKERS=auto

# Manual based on CPU cores
export CLNRM_PARALLEL_WORKERS=8

# Run with specific worker count
clnrm ai-orchestrate --max-workers 8
```

**Rule of thumb**: `workers = CPU_cores - 1` for CPU-bound tasks

#### 2. Memory Limits

```toml
[performance]
memory_limit_mb = 2048
container_memory_limit_mb = 1024
ai_model_memory_mb = 8192
```

```bash
# Set memory limits via environment
export CLNRM_MEMORY_LIMIT_MB=2048

# Docker memory limits
docker run --memory=2g --memory-swap=4g ...
```

#### 3. Container Reuse Strategy

```toml
[performance]
container_reuse = true
container_max_lifetime_minutes = 30
container_max_uses = 100
```

**Benefits**:
- 20-30% faster test startup
- Reduced Docker API calls
- Lower resource churn

### Network Optimization

#### 1. Local Model Storage

```bash
# Configure Ollama model directory
export OLLAMA_MODELS=/mnt/fast-ssd/ollama-models

# Pre-pull models to cache
ollama pull qwen2.5-coder:7b
ollama pull deepseek-coder:6.7b
```

#### 2. Connection Pooling

```toml
[surrealdb]
connection_pool_size = 20
connection_timeout_seconds = 5
idle_timeout_minutes = 10

[ollama]
max_concurrent_requests = 10
request_timeout_seconds = 300
```

### Disk I/O Optimization

#### 1. Use SSD for Container Storage

```bash
# Configure Docker to use SSD
sudo systemctl stop docker
sudo vim /etc/docker/daemon.json
```

```json
{
  "data-root": "/mnt/ssd/docker",
  "storage-driver": "overlay2"
}
```

```bash
sudo systemctl start docker
```

#### 2. Enable Caching

```toml
[performance]
cache_enabled = true
cache_directory = "/var/cache/clnrm"
cache_max_size_mb = 5000
cache_ttl_hours = 24
```

### Ollama Performance Tuning

#### 1. GPU Acceleration (NVIDIA)

```bash
# Install NVIDIA Docker runtime
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | \
  sudo tee /etc/apt/sources.list.d/nvidia-docker.list

sudo apt-get update
sudo apt-get install -y nvidia-docker2
sudo systemctl restart docker

# Enable GPU for Ollama
export OLLAMA_GPU=1
ollama serve
```

#### 2. CPU Optimization

```bash
# Set thread count based on CPU cores
export OLLAMA_NUM_THREADS=8

# Enable AVX2 instructions (if supported)
export OLLAMA_AVX2=1

# Restart Ollama
ollama serve
```

### Performance Benchmarking

```bash
# Run comprehensive benchmark
clnrm ai-orchestrate --benchmark \
  --iterations 10 \
  --output performance-report.json

# Analyze results
cat performance-report.json | jq '.metrics'
```

**Expected Performance Targets**:

| Metric | Target | Good | Excellent |
|--------|--------|------|-----------|
| **AI Inference Time** | < 5s | < 3s | < 1s |
| **Test Execution Time** | < 120s | < 60s | < 30s |
| **Container Startup** | < 10s | < 5s | < 2s |
| **Memory Usage** | < 70% | < 50% | < 30% |
| **CPU Utilization** | < 80% | < 60% | < 40% |

---

## Monitoring & Observability

### OpenTelemetry Integration

#### 1. Enable Telemetry

```toml
[telemetry]
enabled = true
export_format = "otlp"
endpoint = "http://localhost:4317"
sample_rate = 1.0

[telemetry.traces]
enabled = true
exporter = "otlp"

[telemetry.metrics]
enabled = true
exporter = "prometheus"
port = 9090

[telemetry.logs]
enabled = true
exporter = "stdout"
level = "info"
```

#### 2. Run with OpenTelemetry

```bash
# Start Jaeger for traces
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Start Prometheus for metrics
docker run -d --name prometheus \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# Run tests with telemetry
clnrm ai-orchestrate --telemetry
```

#### 3. View Traces

```bash
# Open Jaeger UI
open http://localhost:16686

# Query traces
curl http://localhost:16686/api/traces?service=clnrm
```

### Metrics Collection

#### Prometheus Configuration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'clnrm'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'ollama'
    static_configs:
      - targets: ['localhost:11434']

  - job_name: 'surrealdb'
    static_configs:
      - targets: ['localhost:8000']
```

#### Key Metrics to Monitor

```promql
# Test execution duration
histogram_quantile(0.95, clnrm_test_duration_seconds_bucket)

# AI inference latency
histogram_quantile(0.95, clnrm_ai_inference_duration_seconds_bucket)

# Test success rate
rate(clnrm_test_success_total[5m]) / rate(clnrm_test_total[5m])

# Container startup time
histogram_quantile(0.95, clnrm_container_startup_seconds_bucket)

# Memory usage
clnrm_memory_usage_bytes / clnrm_memory_limit_bytes

# CPU utilization
rate(clnrm_cpu_usage_seconds_total[5m])
```

### Grafana Dashboard

#### 1. Install Grafana

```bash
docker run -d --name grafana \
  -p 3000:3000 \
  -v grafana-storage:/var/lib/grafana \
  grafana/grafana-oss
```

#### 2. Import Dashboard

```bash
# Access Grafana
open http://localhost:3000
# Default credentials: admin/admin

# Add Prometheus data source
# Configuration → Data Sources → Add Prometheus
# URL: http://prometheus:9090

# Import dashboard from docs/grafana-dashboard.json
```

#### 3. Key Dashboard Panels

- **Test Execution Overview**: Success rate, duration, failure trends
- **AI Performance**: Inference time, model usage, prediction accuracy
- **Resource Utilization**: CPU, memory, disk, network
- **Service Health**: Ollama, SurrealDB, container status
- **Performance Trends**: Historical analysis, optimization impact

### Logging

#### 1. Structured Logging

```bash
# Enable JSON logging
export CLNRM_LOG_FORMAT=json
export CLNRM_LOG_LEVEL=info

# Run with structured logs
clnrm ai-orchestrate 2>&1 | tee clnrm.log
```

#### 2. Log Aggregation with Loki

```yaml
# docker-compose.yml
version: '3.8'

services:
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - loki-data:/loki

  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./clnrm.log:/var/log/clnrm.log
      - ./promtail-config.yml:/etc/promtail/config.yml
    command: -config.file=/etc/promtail/config.yml

volumes:
  loki-data:
```

#### 3. Alert Configuration

Create `alerts.yml`:

```yaml
groups:
  - name: clnrm_alerts
    interval: 30s
    rules:
      - alert: HighTestFailureRate
        expr: rate(clnrm_test_failures_total[5m]) > 0.2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High test failure rate detected"
          description: "Test failure rate is {{ $value }} (>20%)"

      - alert: AIInferenceSlowdown
        expr: histogram_quantile(0.95, clnrm_ai_inference_duration_seconds_bucket) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "AI inference is slow"
          description: "P95 inference time is {{ $value }}s (>10s)"

      - alert: HighMemoryUsage
        expr: clnrm_memory_usage_bytes / clnrm_memory_limit_bytes > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanizePercentage }}"
```

---

## Security Best Practices

### 1. Secrets Management

#### Environment Variables (Development Only)

```bash
# Never commit .env files
echo ".env" >> .gitignore

# Use separate .env files per environment
.env.development
.env.staging
.env.production
```

#### HashiCorp Vault (Recommended for Production)

```bash
# Install Vault
brew install vault  # macOS
# or
wget https://releases.hashicorp.com/vault/1.15.0/vault_1.15.0_linux_amd64.zip

# Start Vault dev server
vault server -dev

# Set secrets
vault kv put secret/clnrm \
  surrealdb_password="secure-password" \
  ollama_api_key="api-key-here"

# Fetch secrets in application
export VAULT_ADDR=http://127.0.0.1:8200
vault kv get -field=surrealdb_password secret/clnrm
```

#### Kubernetes Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: clnrm-secrets
type: Opaque
stringData:
  surrealdb-username: root
  surrealdb-password: secure-password-here
  ollama-api-key: api-key-here
---
apiVersion: v1
kind: Pod
metadata:
  name: clnrm-runner
spec:
  containers:
  - name: clnrm
    image: clnrm:latest
    env:
    - name: SURREALDB_USERNAME
      valueFrom:
        secretKeyRef:
          name: clnrm-secrets
          key: surrealdb-username
    - name: SURREALDB_PASSWORD
      valueFrom:
        secretKeyRef:
          name: clnrm-secrets
          key: surrealdb-password
```

### 2. Network Security

#### Firewall Configuration

```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 11434/tcp # Ollama (internal only)
sudo ufw deny 8000/tcp   # SurrealDB (internal only)
sudo ufw enable

# Restrict to specific IPs
sudo ufw allow from 10.0.0.0/8 to any port 11434
```

#### Service Isolation

```yaml
# docker-compose.yml with network isolation
version: '3.8'

networks:
  internal:
    driver: bridge
    internal: true
  external:
    driver: bridge

services:
  surrealdb:
    networks:
      - internal
    # Not exposed to external network

  ollama:
    networks:
      - internal
    # Not exposed to external network

  clnrm-runner:
    networks:
      - internal
      - external
    # Can access both networks
```

### 3. Access Control

#### SurrealDB Authentication

```toml
[surrealdb]
username = "${SURREALDB_USERNAME}"
password = "${SURREALDB_PASSWORD}"
namespace = "production"
database = "clnrm"

[surrealdb.tls]
enabled = true
cert_file = "/etc/ssl/certs/surrealdb.crt"
key_file = "/etc/ssl/private/surrealdb.key"
ca_file = "/etc/ssl/certs/ca.crt"
```

#### Role-Based Access Control (RBAC)

```sql
-- Define roles in SurrealDB
DEFINE SCOPE test_runner SESSION 24h
  SIGNUP ( CREATE user SET email = $email, pass = crypto::argon2::generate($pass) )
  SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) );

-- Grant permissions
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD pass ON user TYPE string;

GRANT SELECT, CREATE, UPDATE ON test_results TO test_runner;
DENY DELETE ON test_results FROM test_runner;
```

### 4. Container Security

#### Run as Non-Root User

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd -m -u 1000 clnrm
USER clnrm
WORKDIR /home/clnrm
COPY --from=builder /app/target/release/clnrm /usr/local/bin/
ENTRYPOINT ["clnrm"]
```

#### Security Scanning

```bash
# Scan Docker image for vulnerabilities
docker scan clnrm:latest

# Scan dependencies
cargo audit

# Static analysis
cargo clippy -- -D warnings
```

### 5. Data Protection

#### Encryption at Rest

```bash
# Encrypt SurrealDB data volume
cryptsetup luksFormat /dev/sdb1
cryptsetup luksOpen /dev/sdb1 surrealdb-encrypted
mkfs.ext4 /dev/mapper/surrealdb-encrypted
mount /dev/mapper/surrealdb-encrypted /var/lib/surrealdb
```

#### Encryption in Transit

```toml
[ollama]
host = "https://ollama.internal:11434"
tls_verify = true
tls_cert = "/etc/ssl/certs/ollama.crt"

[surrealdb]
host = "wss://surrealdb.internal:8000"
tls_verify = true
```

### 6. Audit Logging

```toml
[security]
audit_log_enabled = true
audit_log_file = "/var/log/clnrm/audit.log"
audit_log_level = "info"
audit_events = ["auth", "config_change", "test_execution", "ai_inference"]
```

```bash
# Monitor audit logs
tail -f /var/log/clnrm/audit.log | jq '.event,.user,.timestamp'
```

---

## Troubleshooting

### Common Issues

#### 1. Ollama Connection Refused

**Symptoms**:
```
Error: Failed to connect to Ollama: Connection refused (os error 111)
```

**Solutions**:

```bash
# Check if Ollama is running
ps aux | grep ollama

# Start Ollama service
ollama serve

# Check port binding
netstat -tulpn | grep 11434

# Test connection
curl http://localhost:11434/api/version

# Check firewall
sudo ufw status
sudo ufw allow 11434/tcp
```

#### 2. SurrealDB Authentication Failed

**Symptoms**:
```
Error: Failed to authenticate: Invalid credentials
```

**Solutions**:

```bash
# Verify credentials
curl -X POST http://localhost:8000/sql \
  -H "Content-Type: application/json" \
  -u root:root \
  -d '{"query": "INFO FOR DB;"}'

# Reset SurrealDB
docker stop surrealdb
docker rm surrealdb
docker run --name surrealdb \
  -p 8000:8000 \
  surrealdb/surrealdb:latest start \
  --user root --pass root \
  file:/data/database.db

# Check environment variables
echo $SURREALDB_USERNAME
echo $SURREALDB_PASSWORD
```

#### 3. Out of Memory Errors

**Symptoms**:
```
Error: Container failed: OOMKilled
```

**Solutions**:

```bash
# Check memory usage
free -h
docker stats

# Increase memory limits
export CLNRM_MEMORY_LIMIT_MB=4096

# Configure Docker memory
sudo vim /etc/docker/daemon.json
```

```json
{
  "default-ulimits": {
    "memlock": {
      "Hard": -1,
      "Name": "memlock",
      "Soft": -1
    }
  }
}
```

```bash
sudo systemctl restart docker

# Use smaller AI model
ollama pull phi-3:mini
export OLLAMA_MODEL=phi-3:mini
```

#### 4. Model Download Timeout

**Symptoms**:
```
Error: Failed to pull model: request timeout
```

**Solutions**:

```bash
# Increase timeout
export OLLAMA_TIMEOUT_SECONDS=600

# Download manually
ollama pull qwen2.5-coder:7b --verbose

# Check network
ping ollama.ai
curl -I https://ollama.ai

# Use proxy if needed
export HTTP_PROXY=http://proxy.internal:8080
export HTTPS_PROXY=http://proxy.internal:8080
ollama pull qwen2.5-coder:7b
```

#### 5. AI Prediction Failures

**Symptoms**:
```
Warning: AI prediction confidence below threshold (0.45 < 0.8)
```

**Solutions**:

```bash
# Lower confidence threshold
clnrm ai-predict --confidence-threshold 0.5

# Use more accurate model
export OLLAMA_MODEL=qwen2.5-coder:14b

# Provide more historical data
clnrm ai-predict --analyze-history --days 60

# Increase context
export OLLAMA_CONTEXT_LENGTH=4096
```

#### 6. Container Startup Slow

**Symptoms**:
```
Container startup took 45s (>10s threshold)
```

**Solutions**:

```bash
# Enable container reuse
export CLNRM_CONTAINER_REUSE=true

# Use SSD for Docker storage
# See "Disk I/O Optimization" section

# Pre-pull images
docker pull surrealdb/surrealdb:latest
docker pull postgres:16
docker pull redis:7

# Clean up unused containers
docker system prune -a
```

### Debug Mode

```bash
# Enable debug logging
export CLNRM_LOG_LEVEL=debug
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run with verbose output
clnrm ai-orchestrate --verbose --debug

# Save debug logs
clnrm ai-orchestrate 2>&1 | tee debug.log

# Analyze logs
grep ERROR debug.log
grep WARN debug.log
```

### Performance Profiling

```bash
# Profile CPU usage
cargo flamegraph --bin clnrm -- ai-orchestrate

# Profile memory
valgrind --tool=massif --massif-out-file=massif.out clnrm ai-orchestrate
ms_print massif.out

# Profile system calls
strace -c clnrm ai-orchestrate

# Profile I/O
iotop -p $(pgrep clnrm)
```

### Getting Help

#### Community Support

- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Discord**: Join our community server
- **Stack Overflow**: Tag `clnrm` or `cleanroom-testing`

#### Enterprise Support

For enterprise support, security audits, and custom deployments:
- **Email**: support@cleanroom.dev
- **Slack**: Request access to enterprise channel

#### Reporting Bugs

```bash
# Generate diagnostic report
clnrm diagnostics --output clnrm-diagnostics.json

# Include in bug report:
# 1. clnrm-diagnostics.json
# 2. Reproduction steps
# 3. Expected vs actual behavior
# 4. Environment details
```

---

## FAQ

### General Questions

#### Q: Do I need to use both Ollama and SurrealDB?

**A**: Only Ollama is required for AI features. SurrealDB is optional and recommended for:
- Distributed testing across multiple machines
- Real-time analytics and historical data
- Advanced features like multi-team coordination

#### Q: Can I use cloud AI services instead of Ollama?

**A**: Not currently. The autonomic system is designed for local, privacy-preserving AI. Cloud integration is planned for future releases.

#### Q: What's the minimum hardware for production?

**A**:
- **8 CPU cores** (16 recommended)
- **16 GB RAM** (32 GB recommended)
- **50 GB SSD** (100 GB recommended)
- **Linux server** (Ubuntu 22.04+ or RHEL 8+)

#### Q: How much does it cost to run?

**A**: The software is open-source and free. You only pay for:
- Infrastructure (servers, cloud instances)
- Electricity for local hardware
- Optional enterprise support

### AI & Models

#### Q: Which AI model should I use?

**A**:
- **Development**: `phi-3:mini` (fast, 4 GB RAM)
- **Production**: `qwen2.5-coder:7b` (balanced, 8 GB RAM)
- **High accuracy**: `qwen2.5-coder:14b` (accurate, 16 GB RAM)

#### Q: Can I use multiple models simultaneously?

**A**: Not currently. You can switch models between runs:
```bash
export OLLAMA_MODEL=qwen2.5-coder:7b
clnrm ai-orchestrate
```

#### Q: How accurate are failure predictions?

**A**: The system achieves 85% confidence on average, based on:
- Historical test data (30+ days)
- Code complexity analysis
- Resource usage patterns
- Failure trend detection

#### Q: Do AI models improve over time?

**A**: Yes, through:
- Learning from test execution history
- Pattern recognition from failures
- Optimization feedback loops
- (Future) Federated learning from community

### Performance

#### Q: How fast are AI-powered tests?

**A**: Typical performance:
- **AI inference**: 1-3s per query
- **Test orchestration**: 2-3x faster than manual
- **Optimization**: 40-60% time reduction
- **Total overhead**: <10% for AI features

#### Q: Can I run tests in parallel?

**A**: Yes, highly optimized for parallelism:
```bash
# Automatic worker count
clnrm ai-orchestrate --max-workers auto

# Manual worker count
clnrm ai-orchestrate --max-workers 16
```

#### Q: Does container reuse really help?

**A**: Yes, significantly:
- **20-30% faster startup** for containers
- **Reduced Docker API calls** (less overhead)
- **Lower resource churn** (better stability)

### Security

#### Q: Is my test data shared with AI providers?

**A**: No. Ollama runs 100% locally. No data leaves your infrastructure.

#### Q: How do I secure SurrealDB in production?

**A**:
1. Use strong passwords (not `root:root`)
2. Enable TLS encryption
3. Configure firewall rules
4. Use RBAC for access control
5. Enable audit logging

#### Q: Can I run in an air-gapped environment?

**A**: Yes:
1. Pre-download Ollama models
2. Use local Docker registry
3. Bundle dependencies offline
4. See "Air-Gapped Deployment" guide (coming soon)

### Troubleshooting

#### Q: Tests fail with "connection refused"?

**A**: Check services are running:
```bash
# Ollama
curl http://localhost:11434/api/version

# SurrealDB (if used)
curl http://localhost:8000/health

# Docker
docker ps
```

#### Q: AI predictions are inaccurate?

**A**:
1. Ensure 30+ days of historical data
2. Use appropriate model for complexity
3. Lower confidence threshold if needed
4. Check resource constraints

#### Q: Out of memory errors?

**A**:
1. Use smaller AI model (`phi-3:mini`)
2. Increase system memory
3. Reduce parallel workers
4. Enable container memory limits

### Integration

#### Q: Does it work with GitHub Actions?

**A**: Yes, example workflow:
```yaml
- name: Run AI-powered tests
  run: |
    ollama pull qwen2.5-coder:7b
    ollama serve &
    clnrm ai-orchestrate --predict-failures
```

#### Q: Can I integrate with Jenkins?

**A**: Yes, example pipeline:
```groovy
stage('AI Testing') {
  steps {
    sh 'ollama serve &'
    sh 'clnrm ai-orchestrate --auto-optimize'
  }
}
```

#### Q: Does it support Kubernetes?

**A**: Yes, see "Kubernetes Deployment" section for manifests.

### Licensing

#### Q: What license is Cleanroom under?

**A**: MIT License - free for commercial and personal use.

#### Q: Can I use it in my commercial product?

**A**: Yes, MIT License allows commercial use without restrictions.

#### Q: Do I need to open-source my tests?

**A**: No, you only need to include the MIT license notice.

---

## Next Steps

### Quick Wins (Day 1)

1. ✅ Run your first AI-orchestrated test
2. ✅ Enable failure prediction
3. ✅ Generate optimization report
4. ✅ Set up basic monitoring

### Week 1 Goals

1. Configure production services (Ollama, SurrealDB)
2. Tune performance settings
3. Implement security best practices
4. Set up monitoring dashboards

### Month 1 Goals

1. Integrate with CI/CD pipeline
2. Train team on autonomic features
3. Establish performance baselines
4. Implement production alerting

### Continuous Improvement

1. Monitor AI prediction accuracy
2. Optimize based on metrics
3. Expand test coverage
4. Share insights with team

---

## Additional Resources

- **Official Documentation**: https://cleanroom.dev/docs
- **API Reference**: https://cleanroom.dev/api
- **Examples**: https://github.com/seanchatmangpt/clnrm/tree/master/examples
- **Blog**: https://cleanroom.dev/blog
- **YouTube**: Video tutorials and demos

---

## Feedback

We value your feedback! Help us improve this guide:

- **GitHub**: https://github.com/seanchatmangpt/clnrm/issues
- **Email**: docs@cleanroom.dev
- **Survey**: https://cleanroom.dev/feedback

---

**Version**: 1.0.0
**Last Updated**: 2025-10-16
**License**: MIT
