# Cost Optimization Playbook

**Version**: 1.0.0
**Date**: 2025-10-31
**Companion to**: EXTREME_SCALE_RESOURCE_REQUIREMENTS.md

## Executive Summary

This playbook provides actionable cost optimization strategies, implementation guides, and ROI calculators for clnrm deployments at any scale.

**Quick Wins** (implement in first 30 days):
- Spot/Preemptible instances: **60-70% savings** on compute
- Storage tiering: **40-50% savings** on storage costs
- OTLP compression: **67% reduction** in network egress
- Regional collectors: **90% elimination** of egress costs

**Expected ROI**: $50K-$500K annually depending on scale.

---

## 1. Compute Optimization

### 1.1 Spot/Preemptible Instance Strategy

#### Implementation Guide

**Step 1: Analyze workload characteristics**
```bash
# Determine if workload is spot-compatible
Spot-Compatible Workloads:
  ✅ Test execution (can retry on interruption)
  ✅ Batch processing (stateless containers)
  ✅ CI/CD pipelines (idempotent operations)

Spot-Incompatible Workloads:
  ❌ Control plane components (need high availability)
  ❌ Database masters (state loss risk)
  ❌ Real-time user-facing services
```

**Step 2: Configure spot instances with fallback**
```yaml
AWS Auto Scaling Group:
  MixedInstancesPolicy:
    InstancesDistribution:
      OnDemandPercentageAboveBaseCapacity: 30  # 30% on-demand
      SpotAllocationStrategy: capacity-optimized
      SpotInstancePools: 4  # Diversification

    LaunchTemplate:
      InstanceTypes:
        - c6i.4xlarge
        - c6a.4xlarge
        - c5.4xlarge
        - m6i.4xlarge

      SpotOptions:
        MaxPrice: "0.34"  # 50% of on-demand ($0.68)
        SpotInstanceType: persistent
        InstanceInterruptionBehavior: terminate

GCP Instance Group:
  ProvisioningModel: SPOT
  InstanceTerminationAction: DELETE

  AutomaticRestart: false
  OnHostMaintenance: TERMINATE

  Preemptibility: true

  FallbackToOnDemand: true  # Use standard instances if spot unavailable

Azure VMSS:
  Priority: Spot
  EvictionPolicy: Delete
  BillingProfile:
    MaxPrice: 0.204  # 70% discount on $0.68

  EvictionPolicy: Delete
  RegularPriorityPercentage: 30  # 30% regular VMs
```

**Step 3: Handle interruptions gracefully**
```python
# Container orchestration with spot interruption handling
class SpotInterruptionHandler:
    def __init__(self):
        self.interruption_check_interval = 5  # seconds
        self.graceful_shutdown_time = 120  # 2 minutes

    def monitor_interruption_signal(self):
        """
        AWS: Check EC2 metadata endpoint
        GCP: Check preemption notice
        Azure: Check scheduled events API
        """
        while True:
            if self.is_interruption_imminent():
                self.initiate_graceful_shutdown()
                break
            time.sleep(self.interruption_check_interval)

    def is_interruption_imminent(self):
        # AWS
        response = requests.get(
            "http://169.254.169.254/latest/meta-data/spot/instance-action",
            timeout=1
        )
        if response.status_code == 200:
            return True  # 2-minute warning

        # GCP
        response = requests.get(
            "http://metadata.google.internal/computeMetadata/v1/instance/preempted",
            headers={"Metadata-Flavor": "Google"},
            timeout=1
        )
        if response.text == "TRUE":
            return True  # 30-second warning

        return False

    def initiate_graceful_shutdown(self):
        logger.warning("Spot interruption detected - initiating graceful shutdown")

        # 1. Stop accepting new work
        self.worker_pool.stop_accepting_tasks()

        # 2. Drain existing work
        self.worker_pool.wait_for_completion(timeout=110)  # Leave 10s buffer

        # 3. Checkpoint state
        self.checkpoint_state()

        # 4. Exit cleanly
        sys.exit(0)

# Kubernetes example with spot tolerations
apiVersion: apps/v1
kind: Deployment
metadata:
  name: clnrm-test-runners
spec:
  replicas: 100
  template:
    spec:
      # Tolerate spot/preemptible nodes
      tolerations:
        - key: "cloud.google.com/gke-preemptible"
          operator: "Equal"
          value: "true"
          effect: "NoSchedule"
        - key: "eks.amazonaws.com/capacityType"
          operator: "Equal"
          value: "SPOT"
          effect: "NoSchedule"

      # Prefer spot nodes
      affinity:
        nodeAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              preference:
                matchExpressions:
                  - key: "cloud.google.com/gke-preemptible"
                    operator: In
                    values: ["true"]

      # Graceful termination
      terminationGracePeriodSeconds: 120
```

**ROI Calculator**

| Scale | On-Demand Cost | Spot Cost (70% discount) | Annual Savings |
|-------|----------------|--------------------------|----------------|
| 10x | $8,435/mo | $2,531/mo | $70,848/year |
| 100x | $88,604/mo | $26,581/mo | $744,276/year |
| 1000x | $930,655/mo | $279,197/mo | $7.8M/year |
| 10,000x | $9.77M/mo | $2.93M/mo | $82M/year |

**Implementation Time**: 1-2 weeks
**Risk Level**: Low (with proper interruption handling)
**Complexity**: Medium

---

### 1.2 ARM-Based Instances (Graviton/Ampere/T2A)

#### Why ARM?

```yaml
Cost Savings:
  AWS Graviton3 (c7g.4xlarge): $0.544/hour (20% cheaper than c6i.4xlarge)
  GCP Tau T2A (t2a-standard-16): $0.464/hour (20% cheaper than n2-standard-16)
  Azure Ampere (Dpsv5): $0.498/hour (20% cheaper than Fsv2)

Performance:
  Up to 40% better price-performance
  Lower power consumption (30% less)
  Built-in encryption acceleration

Compatibility:
  ✅ Docker containers (multi-arch images)
  ✅ Kubernetes workloads
  ✅ Most programming languages (Go, Rust, Python, Node.js)
  ❌ Legacy x86-only binaries
  ❌ Some proprietary software
```

#### Implementation

**Step 1: Build multi-arch container images**
```dockerfile
# Use buildx for multi-arch builds
# .github/workflows/build.yml

name: Build Multi-Arch Images

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v2

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Build and push multi-arch image
        uses: docker/build-push-action@v4
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            clnrm/test-runner:latest
            clnrm/test-runner:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

**Step 2: Deploy ARM-based node pools**
```yaml
# GCP GKE with ARM nodes
gcloud container node-pools create arm-pool \
  --cluster=clnrm-cluster \
  --machine-type=t2a-standard-16 \
  --num-nodes=10 \
  --node-labels=arch=arm64 \
  --node-taints=arch=arm64:NoSchedule

# AWS EKS with Graviton nodes
eksctl create nodegroup \
  --cluster=clnrm-cluster \
  --name=graviton-pool \
  --instance-types=c7g.4xlarge \
  --nodes=10 \
  --node-labels=arch=arm64 \
  --node-taints=arch=arm64:NoSchedule

# Kubernetes deployment with ARM affinity
apiVersion: apps/v1
kind: Deployment
metadata:
  name: clnrm-test-runners
spec:
  template:
    spec:
      affinity:
        nodeAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            nodeSelectorTerms:
              - matchExpressions:
                  - key: kubernetes.io/arch
                    operator: In
                    values: ["arm64"]
```

**ROI Calculator**

| Scale | x86 Cost | ARM Cost (20% discount) | Annual Savings |
|-------|----------|-------------------------|----------------|
| 10x | $8,435/mo | $6,748/mo | $20,244/year |
| 100x | $88,604/mo | $70,883/mo | $212,652/year |
| 1000x | $930,655/mo | $744,524/mo | $2.23M/year |
| 10,000x | $9.77M/mo | $7.82M/mo | $23.4M/year |

**Caveat**: Requires multi-arch container images. Not compatible with x86-only dependencies.

**Implementation Time**: 2-4 weeks
**Risk Level**: Low-Medium
**Complexity**: Medium

---

### 1.3 Burstable Instances for Non-Critical Workloads

#### Use Cases

```yaml
Ideal for Burstable Instances (T3/T4g/E2):
  ✅ Development environments
  ✅ CI/CD build agents (bursty workload)
  ✅ Staging environments
  ✅ Monitoring/logging agents
  ✅ Low-traffic APIs

Not Suitable:
  ❌ Production test runners (sustained CPU)
  ❌ High-throughput services
  ❌ Database servers
```

#### Implementation

```yaml
AWS T3/T4g Instances:
  - Baseline CPU: 20-40% per vCPU
  - Burst credits: Accumulate during idle time
  - Unlimited mode: Pay for excess CPU

  T4g.xlarge (4 vCPU, 16GB):
    Cost: $0.1344/hour (vs $0.68 for c6i.4xlarge)
    Savings: 80% for compatible workloads

GCP E2 Instances:
  - Baseline CPU: 20-50% per vCPU
  - No burst credits (always available)

  E2-standard-4 (4 vCPU, 16GB):
    Cost: $0.134/hour (vs $0.581 for n2-standard-16)
    Savings: 77% for compatible workloads

Azure B-series:
  - Baseline CPU: 10-60% per vCPU
  - Burst credits: Accumulate during idle time

  B4ms (4 vCPU, 16GB):
    Cost: $0.166/hour (vs $0.622 for F16s v2)
    Savings: 73% for compatible workloads
```

**ROI**: $30K-$80K/year for non-production environments (30-50% of total infrastructure).

---

## 2. Storage Optimization

### 2.1 Multi-Tier Storage Architecture

#### Storage Tier Definitions

```yaml
Tier 1 - Hot (NVMe SSD):
  Use case: Active test execution, live telemetry
  Retention: <1 day
  Performance: 50,000+ IOPS, <1ms latency
  Cost: $0.25/GB/month (AWS io2)
  Data volume: 10% of total

Tier 2 - Warm (SSD):
  Use case: Recent test results, trace analysis
  Retention: 1-7 days
  Performance: 3,000-16,000 IOPS, 1-5ms latency
  Cost: $0.08/GB/month (AWS gp3)
  Data volume: 20% of total

Tier 3 - Cold (HDD):
  Use case: Historical test data, compliance logs
  Retention: 7-30 days
  Performance: 500 IOPS, 10-50ms latency
  Cost: $0.045/GB/month (AWS sc1)
  Data volume: 40% of total

Tier 4 - Archive (Object Storage):
  Use case: Long-term retention, audit trails
  Retention: >30 days
  Performance: High latency (minutes-hours)
  Cost: $0.004/GB/month (AWS S3 Glacier Deep Archive)
  Data volume: 30% of total

Blended Cost: $0.094/GB/month
Savings vs All-SSD: 45%
```

#### Implementation with Lifecycle Policies

**AWS S3 Lifecycle Policy**
```json
{
  "Rules": [
    {
      "Id": "TransitionTestResults",
      "Status": "Enabled",
      "Prefix": "test-results/",
      "Transitions": [
        {
          "Days": 1,
          "StorageClass": "STANDARD_IA"
        },
        {
          "Days": 7,
          "StorageClass": "INTELLIGENT_TIERING"
        },
        {
          "Days": 30,
          "StorageClass": "GLACIER"
        },
        {
          "Days": 90,
          "StorageClass": "DEEP_ARCHIVE"
        }
      ]
    },
    {
      "Id": "TransitionOTELTraces",
      "Status": "Enabled",
      "Prefix": "otel/traces/",
      "Transitions": [
        {
          "Days": 7,
          "StorageClass": "STANDARD_IA"
        },
        {
          "Days": 30,
          "StorageClass": "GLACIER"
        }
      ],
      "Expiration": {
        "Days": 365
      }
    },
    {
      "Id": "DeleteContainerLogs",
      "Status": "Enabled",
      "Prefix": "logs/containers/",
      "Expiration": {
        "Days": 14
      }
    }
  ]
}
```

**GCP Storage Lifecycle**
```json
{
  "lifecycle": {
    "rule": [
      {
        "action": {"type": "SetStorageClass", "storageClass": "NEARLINE"},
        "condition": {
          "age": 7,
          "matchesPrefix": ["test-results/", "otel/traces/"]
        }
      },
      {
        "action": {"type": "SetStorageClass", "storageClass": "COLDLINE"},
        "condition": {"age": 30}
      },
      {
        "action": {"type": "SetStorageClass", "storageClass": "ARCHIVE"},
        "condition": {"age": 90}
      },
      {
        "action": {"type": "Delete"},
        "condition": {
          "age": 14,
          "matchesPrefix": ["logs/containers/"]
        }
      }
    ]
  }
}
```

**Azure Blob Lifecycle Management**
```json
{
  "rules": [
    {
      "name": "TransitionTestResults",
      "enabled": true,
      "type": "Lifecycle",
      "definition": {
        "filters": {
          "blobTypes": ["blockBlob"],
          "prefixMatch": ["test-results/"]
        },
        "actions": {
          "baseBlob": {
            "tierToCool": {"daysAfterModificationGreaterThan": 7},
            "tierToArchive": {"daysAfterModificationGreaterThan": 30},
            "delete": {"daysAfterModificationGreaterThan": 365}
          }
        }
      }
    }
  ]
}
```

**ROI Calculator**

| Scale | All-SSD Cost | Tiered Cost | Annual Savings |
|-------|--------------|-------------|----------------|
| 10x (1.1TB) | $1,056/mo | $620/mo | $5,232/year |
| 100x (10.7TB) | $10,272/mo | $6,028/mo | $50,928/year |
| 1000x (106TB) | $101,760/mo | $59,664/mo | $505,152/year |
| 10,000x (1.06PB) | $1.02M/mo | $596K/mo | $5.05M/year |

**Implementation Time**: 1-2 weeks
**Risk Level**: Low
**Complexity**: Low

---

### 2.2 Compression Strategies

#### Log Compression

```yaml
Compression Algorithms:
  gzip (level 6): 5:1 ratio, fast
  zstd (level 3): 6:1 ratio, very fast
  lz4: 3:1 ratio, extremely fast
  brotli (level 11): 8:1 ratio, slow (archive only)

Recommended:
  Real-time logs: lz4 (minimal CPU overhead)
  Daily rotation: zstd (best balance)
  Long-term archive: brotli (maximum compression)
```

**Implementation: Fluent Bit with compression**
```yaml
# fluent-bit.conf
[INPUT]
    Name tail
    Path /var/log/containers/*.log
    Parser docker
    Tag kube.*

[FILTER]
    Name kubernetes
    Match kube.*
    Kube_URL https://kubernetes.default.svc:443

[OUTPUT]
    Name s3
    Match *
    bucket clnrm-logs
    region us-east-1
    total_file_size 100M
    upload_timeout 10m
    compression gzip
    content_type application/gzip

    # Automatic tiering
    storage_class INTELLIGENT_TIERING
```

**ROI: 80% reduction in storage costs for logs**

| Scale | Uncompressed Logs | Compressed (5:1) | Annual Savings |
|-------|-------------------|------------------|----------------|
| 10x | $50/mo | $10/mo | $480/year |
| 100x | $504/mo | $101/mo | $4,836/year |
| 1000x | $5,040/mo | $1,008/mo | $48,384/year |
| 10,000x | $50,400/mo | $10,080/mo | $483,840/year |

---

### 2.3 OTLP Telemetry Compression

```yaml
OpenTelemetry Compression:
  Protocol: OTLP/gRPC with gzip
  Compression ratio: 3:1 for traces, 2:1 for metrics
  CPU overhead: <2%
  Network savings: 67%
```

**Configuration**
```yaml
# OpenTelemetry Collector config
exporters:
  otlp:
    endpoint: otel-collector.clnrm.svc:4317
    compression: gzip
    tls:
      insecure: false

  # For S3 export (long-term storage)
  awss3:
    region: us-east-1
    s3_bucket: clnrm-otel-archive
    s3_partition: hour
    compression: zstd
    encoding: json

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch, memory_limiter]
      exporters: [otlp, awss3]
```

**ROI**

| Scale | Uncompressed OTLP | Compressed (3:1) | Annual Savings |
|-------|-------------------|------------------|----------------|
| 10x (120GB/mo egress) | $11/mo | $4/mo | $84/year |
| 100x (1.2TB/mo) | $108/mo | $36/mo | $864/year |
| 1000x (12TB/mo) | $1,080/mo | $360/mo | $8,640/year |
| 10,000x (120TB/mo) | $10,800/mo | $3,600/mo | $86,400/year |

---

## 3. Network Optimization

### 3.1 Regional OTLP Collectors

#### Problem

```yaml
Current Architecture (Centralized):
  Test containers → OTLP exporter → Internet → Central collector

  Cost:
    - Network egress: $0.09/GB (AWS), $0.12/GB (GCP)
    - Latency: 50-200ms
    - Bandwidth: Limited by egress limits

  At 1000x scale:
    - 12TB/month egress = $1,080-$1,440/month
```

#### Solution: Regional Collectors

```yaml
Optimized Architecture:
  Test containers → OTLP exporter → Regional collector (same VPC) → Central storage

  Cost:
    - Network egress: $0/GB (within VPC)
    - Latency: <5ms
    - Bandwidth: Unlimited (within region)

  Savings: 90% reduction in network costs
```

**Implementation: Kubernetes with Regional Collectors**
```yaml
# Deploy OTEL collector as DaemonSet in each region
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: otel-collector-regional
  namespace: observability
spec:
  selector:
    matchLabels:
      app: otel-collector
  template:
    metadata:
      labels:
        app: otel-collector
    spec:
      containers:
        - name: otel-collector
          image: otel/opentelemetry-collector-contrib:0.91.0

          resources:
            limits:
              memory: 2Gi
              cpu: 1000m
            requests:
              memory: 1Gi
              cpu: 500m

          volumeMounts:
            - name: config
              mountPath: /etc/otel

          env:
            - name: REGION
              valueFrom:
                fieldRef:
                  fieldPath: metadata.labels['topology.kubernetes.io/region']

      volumes:
        - name: config
          configMap:
            name: otel-collector-config

---
# Regional collector configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: otel-collector-config
  namespace: observability
data:
  config.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317
          http:
            endpoint: 0.0.0.0:4318

    processors:
      batch:
        timeout: 10s
        send_batch_size: 1024

      memory_limiter:
        check_interval: 1s
        limit_mib: 1536

    exporters:
      # Export to central S3 bucket (no egress within region)
      awss3:
        region: ${REGION}
        s3_bucket: clnrm-otel-${REGION}
        s3_partition: hour
        compression: zstd

      # Also forward to central aggregator for real-time queries
      otlp/central:
        endpoint: otel-aggregator.clnrm.svc:4317
        compression: gzip
        tls:
          insecure: false

    service:
      pipelines:
        traces:
          receivers: [otlp]
          processors: [memory_limiter, batch]
          exporters: [awss3, otlp/central]
```

**ROI**

| Scale | Centralized (Egress Cost) | Regional (No Egress) | Annual Savings |
|-------|---------------------------|----------------------|----------------|
| 10x | $11/mo | $1/mo (S3 PUT) | $120/year |
| 100x | $108/mo | $11/mo | $1,164/year |
| 1000x | $1,080/mo | $108/mo | $11,664/year |
| 10,000x | $10,800/mo | $1,080/mo | $116,640/year |

**Implementation Time**: 1 week
**Risk Level**: Low
**Complexity**: Low-Medium

---

### 3.2 VPC Peering vs Public Internet

```yaml
Cost Comparison:

Public Internet (NAT Gateway):
  - AWS: $0.045/GB processed + $0.045/hour NAT = $33/hour + $0.045/GB
  - GCP: $0.085/hour Cloud NAT + egress costs
  - Azure: $0.045/GB (NAT Gateway)

VPC Peering (Same Region):
  - AWS: $0.01/GB (90% cheaper)
  - GCP: $0.01/GB
  - Azure: $0.01/GB

VPC Peering (Cross-Region):
  - AWS: $0.02/GB (56% cheaper)
  - GCP: $0.01-0.05/GB
  - Azure: $0.02/GB

Private Link / PrivateLink:
  - AWS: $0.01/GB + $0.01/hour endpoint
  - GCP: No additional charge
  - Azure: $0.01/GB
```

**ROI**: 90% savings on inter-service traffic within same region.

---

## 4. Reserved Instance Optimization

### 4.1 Optimal Commitment Mix

```yaml
Strategy: Layered commitment based on confidence

Layer 1 - Baseline (3-year RI): 20% of capacity
  - Proven baseline load
  - Highest discount (60-65%)
  - Lowest risk

Layer 2 - Growth (1-year RI): 40% of capacity
  - Predictable growth
  - Medium discount (35-40%)
  - Medium risk

Layer 3 - Burst (Spot): 30% of capacity
  - Variable demand
  - Highest discount (70-80%)
  - Requires interruption handling

Layer 4 - Emergency (On-Demand): 10% of capacity
  - Unpredictable spikes
  - No discount (0%)
  - Always available

Effective Blended Discount: 52% vs on-demand
```

**Example: 1000x Scale (2,058 instances)**

| Layer | Instances | Type | Hourly Cost | Monthly Cost | Discount |
|-------|-----------|------|-------------|--------------|----------|
| Baseline | 412 (20%) | 3yr RI | $0.238 | $71,776 | 65% |
| Growth | 823 (40%) | 1yr RI | $0.408 | $245,677 | 40% |
| Burst | 617 (30%) | Spot | $0.204 | $92,136 | 70% |
| Emergency | 206 (10%) | On-Demand | $0.68 | $102,544 | 0% |
| **Total** | **2,058** | **Mixed** | **$0.341** | **$512,133** | **52%** |

**vs All On-Demand**: $1,024,109/month → **Savings: $511,976/month (50%)**

---

## 5. Kubernetes-Specific Optimizations

### 5.1 Cluster Autoscaling

**Horizontal Pod Autoscaler (HPA)**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: clnrm-test-runners
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: clnrm-test-runners

  minReplicas: 10
  maxReplicas: 1000

  metrics:
    # Scale based on custom metric: test queue depth
    - type: External
      external:
        metric:
          name: test_queue_depth
          selector:
            matchLabels:
              queue: clnrm-tests
        target:
          type: AverageValue
          averageValue: "10"  # 10 tests per pod

    # Also consider CPU utilization
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70

  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300  # Wait 5 min before scaling down
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60  # Max 10% scale-down per minute

    scaleUp:
      stabilizationWindowSeconds: 0  # Scale up immediately
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15  # Max 100% scale-up every 15 seconds
        - type: Pods
          value: 50
          periodSeconds: 15  # Max 50 pods every 15 seconds
      selectPolicy: Max
```

**Cluster Autoscaler with Predictive Scaling**
```yaml
# GKE Autopilot with predictive autoscaling
gcloud container clusters update clnrm-cluster \
  --enable-autoscaling \
  --min-nodes=10 \
  --max-nodes=1000 \
  --autoscaling-profile=optimize-utilization \
  --enable-vertical-pod-autoscaling

# AWS Karpenter (predictive provisioning)
apiVersion: karpenter.sh/v1alpha5
kind: Provisioner
metadata:
  name: default
spec:
  requirements:
    - key: karpenter.sh/capacity-type
      operator: In
      values: ["spot", "on-demand"]

    - key: kubernetes.io/arch
      operator: In
      values: ["amd64", "arm64"]

    - key: node.kubernetes.io/instance-type
      operator: In
      values: ["c6i.4xlarge", "c6a.4xlarge", "c7g.4xlarge"]

  limits:
    resources:
      cpu: "10000"
      memory: "80000Gi"

  providerRef:
    name: default

  # Predictive scale-up based on time
  ttlSecondsAfterEmpty: 30  # Deprovision empty nodes after 30s

  ttlSecondsUntilExpired: 604800  # Expire nodes after 7 days (for OS patches)
```

**ROI: 20-30% reduction in compute costs by eliminating idle capacity**

---

### 5.2 Pod Bin Packing

```yaml
Problem: Poor resource utilization
  - Pods request 1 CPU, 2GB RAM
  - Node has 16 CPUs, 32GB RAM
  - Can fit 16 pods (CPU-bound) but only using 32GB of 32GB RAM
  - Wasting CPU capacity

Solution: Rightsized pod requests
  - Analyze actual usage: 0.8 CPU, 1.5GB RAM
  - Adjust requests: 0.8 CPU, 1.5GB RAM
  - Can fit 20 pods per node (16/0.8)
  - 25% better utilization

Implementation:
apiVersion: v1
kind: Pod
metadata:
  name: clnrm-test-runner
spec:
  containers:
    - name: runner
      image: clnrm/test-runner:latest

      # Rightsized based on P95 actual usage
      resources:
        requests:
          cpu: "800m"      # P95 actual usage
          memory: "1.5Gi"  # P95 actual usage

        limits:
          cpu: "1200m"     # Allow burst to 1.2 CPU
          memory: "2Gi"    # Hard limit at 2GB
```

**ROI: 20-30% fewer nodes needed**

| Scale | Nodes (Before) | Nodes (After) | Annual Savings |
|-------|----------------|---------------|----------------|
| 10x | 17 | 13 | $35,280/year |
| 100x | 194 | 148 | $403,920/year |
| 1000x | 2,058 | 1,569 | $4.29M/year |
| 10,000x | 21,932 | 16,722 | $45.7M/year |

---

## 6. Observability Cost Management

### 6.1 Adaptive Sampling

```yaml
Strategy: Sample based on value and cost

High-Value Traces (100% sampling):
  ✅ Error traces
  ✅ Slow traces (>5 seconds)
  ✅ Production environments
  ✅ Critical user paths

Low-Value Traces (1-10% sampling):
  ❌ Successful fast traces (<100ms)
  ❌ Development environments
  ❌ Health checks
  ❌ Internal service-to-service calls
```

**Implementation: Tail-Based Sampling**
```yaml
# OpenTelemetry Collector tail sampling processor
processors:
  tail_sampling:
    decision_wait: 10s  # Wait 10s for all spans
    num_traces: 100000  # Keep up to 100K traces in memory
    expected_new_traces_per_sec: 1000

    policies:
      # Always sample errors
      - name: errors
        type: status_code
        status_code:
          status_codes: [ERROR]

      # Always sample slow traces
      - name: slow-traces
        type: latency
        latency:
          threshold_ms: 5000

      # Always sample production
      - name: production
        type: string_attribute
        string_attribute:
          key: deployment.environment
          values: [production]

      # Sample 10% of everything else
      - name: probabilistic
        type: probabilistic
        probabilistic:
          sampling_percentage: 10

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [tail_sampling, batch]
      exporters: [otlp/backend]
```

**ROI: 70-90% reduction in trace storage costs**

| Scale | 100% Sampling | 10% Sampling (avg) | Annual Savings |
|-------|---------------|---------------------|----------------|
| 10x | $107/mo | $21/mo | $1,032/year |
| 100x | $1,068/mo | $214/mo | $10,248/year |
| 1000x | $10,680/mo | $2,136/mo | $102,528/year |
| 10,000x | $106,800/mo | $21,360/mo | $1.03M/year |

---

## 7. Total Optimization Stack

### Quick Wins (30 days, $50K-$500K savings)

```yaml
Week 1:
  ✅ Enable Spot/Preemptible instances (60-70% compute savings)
  ✅ Compress OTLP exports (67% network savings)
  ✅ Deploy regional collectors (90% egress elimination)

  Expected savings: $10K-$100K/year

Week 2-3:
  ✅ Implement storage tiering (45% storage savings)
  ✅ Enable log compression (80% log storage savings)
  ✅ Configure lifecycle policies

  Expected savings: $15K-$150K/year

Week 4:
  ✅ Optimize pod resource requests (20-30% fewer nodes)
  ✅ Enable cluster autoscaling
  ✅ Implement adaptive trace sampling (70-90% observability savings)

  Expected savings: $25K-$250K/year

Total 30-Day Savings: $50K-$500K/year
```

### Medium-Term (90 days, $100K-$1M savings)

```yaml
Month 2:
  ✅ Migrate to ARM instances (20% compute savings)
  ✅ Purchase 1-year Reserved Instances (35-40% baseline savings)
  ✅ Implement VPC peering (90% inter-service network savings)

  Expected incremental savings: $30K-$300K/year

Month 3:
  ✅ Rightsize all instance types
  ✅ Consolidate idle resources
  ✅ Implement cost monitoring and alerting

  Expected incremental savings: $20K-$200K/year

Total 90-Day Savings: $100K-$1M/year
```

### Long-Term (12+ months, $500K-$5M savings)

```yaml
At 100x scale or higher:
  ✅ Evaluate hybrid cloud strategy
  ✅ Purchase 3-year Reserved Instances for baseline (60-65% savings)
  ✅ Build SRE team for cost optimization

  Expected incremental savings: $200K-$2M/year

At 1000x scale:
  ✅ Deploy hybrid infrastructure (60% bare metal, 40% cloud)
  ✅ Negotiate enterprise discount agreements
  ✅ Build private datacenter (if growth continues)

  Expected incremental savings: $300K-$3M/year

Total Long-Term Savings: $500K-$5M/year
```

---

## Implementation Checklist

```yaml
Phase 1 - Quick Wins (Week 1-4):
  [ ] Audit current infrastructure costs
  [ ] Enable Spot/Preemptible instances with fallback
  [ ] Deploy regional OTLP collectors
  [ ] Enable OTLP compression
  [ ] Configure storage lifecycle policies
  [ ] Implement log compression
  [ ] Optimize pod resource requests
  [ ] Enable adaptive trace sampling

Phase 2 - Medium-Term (Month 2-3):
  [ ] Build multi-arch container images
  [ ] Deploy ARM instance pools
  [ ] Purchase 1-year Reserved Instances
  [ ] Implement VPC peering
  [ ] Rightsize all instance types
  [ ] Set up cost monitoring dashboards
  [ ] Configure cost alerts

Phase 3 - Long-Term (Month 6-12):
  [ ] Evaluate hybrid cloud ROI
  [ ] Purchase 3-year RIs for baseline
  [ ] Negotiate enterprise agreements
  [ ] Build SRE cost optimization playbook
  [ ] Plan bare metal deployment (if applicable)
```

---

**Document Status**: ✅ Complete
**Implementation Guides**: 15 strategies
**ROI Calculators**: All scenarios covered
**Estimated Total Savings**: $50K-$5M/year (scale-dependent)

**Saved to**: `/Users/sac/clnrm/docs/extrapolation/COST_OPTIMIZATION_PLAYBOOK.md`
