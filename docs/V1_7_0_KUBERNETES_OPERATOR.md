# Kubernetes Native Operator (v1.7.0)

**Feature Version**: v1.7.0
**Implementation Status**: Design Complete
**Timeline**: Q1-Q2 2026
**Last Updated**: 2025-11-18

---

## Overview

The Kubernetes Operator enables clnrm to run as a native Kubernetes application, automating deployment, scaling, and lifecycle management of container pools across Kubernetes clusters.

## Architecture

### Operator Components

```
┌──────────────────────────────────────────────────────┐
│ clnrm-operator (Kubernetes Operator)                 │
│                                                      │
│ ├── Controller (CRDS)                               │
│ │   ├── TestSuite CRD                              │
│ │   ├── TestRun CRD                                │
│ │   └── ContainerPool CRD                          │
│ │                                                   │
│ ├── Reconciler                                      │
│ │   ├── TestSuite → Deployment                    │
│ │   ├── TestRun → Pod                             │
│ │   └── ContainerPool → StatefulSet               │
│ │                                                   │
│ ├── Webhook                                         │
│ │   ├── Validation                                │
│ │   └── Mutation                                  │
│ │                                                   │
│ └── Metrics                                         │
│     └── Prometheus integration                    │
└──────────────────────────────────────────────────────┘
```

### Custom Resource Definitions (CRDs)

#### TestSuite CRD

```yaml
apiVersion: clnrm.ai/v1alpha1
kind: TestSuite
metadata:
  name: my-test-suite
  namespace: default
spec:
  image: myrepo/test-suite:latest
  services:
    - name: api
      image: myapi:latest
      replicas: 3
    - name: postgres
      image: postgres:15
      replicas: 1
  schedule: "0 * * * *"  # Hourly
  retention: 7d  # Keep results 7 days
status:
  phase: Running
  lastRun: "2025-11-18T10:00:00Z"
  successCount: 145
  failureCount: 2
```

#### TestRun CRD

```yaml
apiVersion: clnrm.ai/v1alpha1
kind: TestRun
metadata:
  name: test-suite-run-001
  namespace: default
spec:
  testSuite: my-test-suite
  parallelism: 4  # 4 concurrent tests
  timeout: 30m
status:
  phase: Running
  progress: "45/100"
  containers:
    active: 12
    completed: 45
  results:
    passed: 45
    failed: 0
    skipped: 55
```

#### ContainerPool CRD

```yaml
apiVersion: clnrm.ai/v1alpha1
kind: ContainerPool
metadata:
  name: postgres-pool
  namespace: default
spec:
  image: postgres:15
  maxSize: 20
  minIdle: 5
  nodeSelector:
    pool-type: database
status:
  active: 15
  idle: 5
  created: 100
  destroyed: 80
```

### Reconciliation Flow

```
User creates TestSuite CRD
        ↓
Operator detects change
        ↓
Validate: Image exists, resources available
        ↓
Create StatefulSet for container pools
        ↓
Create ConfigMap for test configuration
        ↓
Wait for pool readiness
        ↓
Update TestSuite.status → Ready
```

## Implementation

### Project Structure

```
crates/clnrm-k8s/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── operator.rs
│   ├── crd/
│   │   ├── test_suite.rs
│   │   ├── test_run.rs
│   │   └── container_pool.rs
│   ├── reconciler/
│   │   ├── test_suite_reconciler.rs
│   │   ├── test_run_reconciler.rs
│   │   └── pool_reconciler.rs
│   ├── webhook/
│   │   ├── validation.rs
│   │   └── mutation.rs
│   └── metrics.rs
└── manifests/
    ├── rbac.yaml
    ├── crd.yaml
    ├── operator-deployment.yaml
    └── helm/
        └── Chart.yaml
```

### Core Implementation

```rust
// crates/clnrm-k8s/src/operator.rs

use kube::{Api, Client, CustomResourceExt};
use kube_runtime::controller::Controller;

pub struct ClnrmOperator {
    client: Client,
    test_suite_controller: Controller<TestSuite>,
    test_run_controller: Controller<TestRun>,
    pool_controller: Controller<ContainerPool>,
}

impl ClnrmOperator {
    pub async fn run(&self) -> Result<()> {
        // Run all controllers concurrently
        tokio::join!(
            self.reconcile_test_suites(),
            self.reconcile_test_runs(),
            self.reconcile_pools(),
        );

        Ok(())
    }

    async fn reconcile_test_suites(&self) -> Result<()> {
        Controller::new(
            Api::<TestSuite>::all(self.client.clone()),
            ListParams::default(),
        )
        .run(Self::reconcile_test_suite, Self::error_policy, ())
        .boxed()
        .await
    }

    async fn reconcile_test_suite(
        suite: Arc<TestSuite>,
        ctx: Arc<Self>,
    ) -> Result<Action> {
        // Get current spec
        let ns = suite.namespace();
        let name = &suite.name();

        // Create deployment for container pools
        let deployment = ctx.create_pool_deployment(&suite).await?;
        ctx.client.create_with_defaults(&deployment).await?;

        // Update status
        let mut status = suite.status.clone().unwrap_or_default();
        status.phase = "Ready".to_string();
        ctx.patch_status(&suite, status).await?;

        Ok(Action::requeue(Duration::from_secs(300)))
    }
}
```

## Helm Chart

### Chart Structure

```yaml
# charts/clnrm-operator/Chart.yaml
apiVersion: v2
name: clnrm-operator
description: clnrm Kubernetes Operator
version: 1.7.0
appVersion: "1.7.0"
```

### Values

```yaml
# charts/clnrm-operator/values.yaml
operator:
  replicas: 3  # HA setup
  image:
    repository: clnrm/operator
    tag: 1.7.0
  resources:
    requests:
      cpu: 1
      memory: 1Gi
    limits:
      cpu: 2
      memory: 2Gi

pools:
  default:
    maxSize: 50
    minIdle: 10
    adaptiveSizing: true

webhook:
  enabled: true
  port: 8443
```

## Features

### 1. Automatic Pool Lifecycle

```
Create TestSuite
    ↓
Auto-create ContainerPool for each service
    ↓
Monitor pool health
    ↓
Scale pools based on test load
    ↓
Clean up pools on deletion
```

### 2. Scheduling

```yaml
spec:
  schedule: "0 * * * *"     # Hourly
  concurrencyPolicy: Allow  # Allow overlapping
  successfulRunsToKeep: 5   # Retention
  failedRunsToKeep: 3
```

### 3. Service Mesh Integration

```yaml
spec:
  serviceMesh:
    enabled: true
    provider: istio
    virtualServiceTemplate:
      timeout: 30s
      retries: 3
```

### 4. Monitoring & Observability

```yaml
spec:
  monitoring:
    enabled: true
    prometheus:
      scrape: true
      port: 8080
    traces:
      enabled: true
      sampler: probabilistic
      ratio: 0.1
```

## Installation

### kubectl

```bash
# Add CRDs
kubectl apply -f manifests/crd.yaml

# Install RBAC
kubectl apply -f manifests/rbac.yaml

# Deploy operator
kubectl apply -f manifests/operator-deployment.yaml

# Verify
kubectl get deployment -n clnrm-system clnrm-operator
kubectl get crd | grep clnrm
```

### Helm

```bash
# Add Helm repo
helm repo add clnrm https://charts.clnrm.ai
helm repo update

# Install
helm install clnrm-operator clnrm/clnrm-operator \
  --namespace clnrm-system \
  --create-namespace \
  --values custom-values.yaml

# Verify
helm status clnrm-operator -n clnrm-system
```

## Usage Examples

### Simple Test Suite

```yaml
apiVersion: clnrm.ai/v1alpha1
kind: TestSuite
metadata:
  name: integration-tests
spec:
  image: myrepo/tests:latest
  parallelism: 4
```

### Multi-Service Test Suite

```yaml
apiVersion: clnrm.ai/v1alpha1
kind: TestSuite
metadata:
  name: e2e-tests
spec:
  image: myrepo/e2e-tests:latest
  services:
    - name: api
      image: myrepo/api:latest
      port: 8080
    - name: database
      image: postgres:15
      env:
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: password
    - name: redis
      image: redis:7-alpine
  parallelism: 8
  timeout: 1h
  retryPolicy:
    maxRetries: 3
    backoff: exponential
```

### Scheduled Test Runs

```yaml
apiVersion: clnrm.ai/v1alpha1
kind: ScheduledRun
metadata:
  name: nightly-tests
spec:
  testSuite: integration-tests
  schedule: "0 2 * * *"  # 2 AM daily
  timezone: America/New_York
  concurrencyPolicy: Forbid  # Don't overlap
  successfulRunsToKeep: 7
  failedRunsToKeep: 10
```

## Testing

### Unit Tests

- CRD validation
- Reconciliation logic
- Status updates

### Integration Tests

- Full operator lifecycle
- Multi-pod deployment
- Failure scenarios

### E2E Tests

- Real Kubernetes cluster
- Multiple namespaces
- Cross-node scheduling

## Success Criteria

- ✅ CRDs fully functional
- ✅ Operator HA (3 replicas)
- ✅ 99.9% uptime
- ✅ <5s reconciliation latency
- ✅ Multi-tenant support
- ✅ Helm chart production-ready

## Deployment Recommendations

### High Availability

```yaml
operator:
  replicas: 3  # Quorum for leader election
  affinity:
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchLabels:
            app: clnrm-operator
        topologyKey: kubernetes.io/hostname
```

### Resource Limits

```yaml
resources:
  requests:
    cpu: 1000m
    memory: 1Gi
  limits:
    cpu: 2000m
    memory: 2Gi
```

### Monitoring

```yaml
metrics:
  enabled: true
  serviceMonitor:
    enabled: true  # Prometheus scraping
```

## Troubleshooting

### Operator Pod Crashes

1. Check logs: `kubectl logs -n clnrm-system deployment/clnrm-operator`
2. Verify CRDs: `kubectl get crd | grep clnrm`
3. Check RBAC: `kubectl auth can-i create testsuite --namespace default`

### TestSuite Stuck in Pending

1. Check events: `kubectl describe testsuite <name>`
2. Verify image availability: `kubectl run test --image=<image>`
3. Check node resources: `kubectl top nodes`

### Pool Scaling Issues

1. Monitor pool status: `kubectl get containerpool`
2. Check resource metrics: `kubectl get resourcequota`
3. Review operator logs for scale decisions

## References

- [Kubernetes Operator Pattern](https://kubernetes.io/docs/concepts/extend-kubernetes/operator/)
- [kube-rs Documentation](https://docs.rs/kube/)
- [Helm Chart Best Practices](https://helm.sh/docs/chart_best_practices/)

---

**Version History**

| Version | Status | Notes |
|---------|--------|-------|
| **v1.7.0** | Design Complete | Implementation Q1-Q2 2026 |
| **v1.8.0** | Planned | Multi-cluster support |

**Last Updated**: 2025-11-18
