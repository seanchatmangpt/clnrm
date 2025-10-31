# Kubernetes Deployment Runbook

## Overview

Deploy clnrm with Weaver validation in Kubernetes clusters.

## Prerequisites

- Kubernetes 1.21+
- kubectl configured
- Registry accessible from cluster
- OTLP collector deployed (optional)

## Basic Deployment

### 1. ConfigMap for Registry

```yaml
# registry-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: weaver-registry
  namespace: clnrm
data:
  registry_manifest.yaml: |
    registry:
      url: https://example.com/registry
      description: clnrm telemetry registry
    groups:
      - id: clnrm.core
        prefix: clnrm
        brief: Core clnrm telemetry
```

### 2. Deployment

```yaml
# clnrm-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: clnrm-weaver
  namespace: clnrm
  labels:
    app: clnrm
    component: validator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: clnrm
      component: validator
  template:
    metadata:
      labels:
        app: clnrm
        component: validator
    spec:
      containers:
      - name: weaver
        image: clnrm:v1.2.0
        command:
          - weaver
          - registry
          - live-check
          - --registry=/registry
          - --otlp-grpc-port=4317
          - --admin-port=8080
          - --output=/output
          - --format=json
        ports:
        - name: otlp-grpc
          containerPort: 4317
          protocol: TCP
        - name: admin
          containerPort: 8080
          protocol: TCP
        volumeMounts:
        - name: registry
          mountPath: /registry
          readOnly: true
        - name: output
          mountPath: /output
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10

      volumes:
      - name: registry
        configMap:
          name: weaver-registry
      - name: output
        emptyDir: {}
```

### 3. Service

```yaml
# clnrm-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: clnrm-weaver
  namespace: clnrm
  labels:
    app: clnrm
    component: validator
spec:
  type: ClusterIP
  ports:
  - name: otlp-grpc
    port: 4317
    targetPort: 4317
    protocol: TCP
  - name: admin
    port: 8080
    targetPort: 8080
    protocol: TCP
  selector:
    app: clnrm
    component: validator
```

### 4. Deploy

```bash
# Create namespace
kubectl create namespace clnrm

# Apply manifests
kubectl apply -f registry-configmap.yaml
kubectl apply -f clnrm-deployment.yaml
kubectl apply -f clnrm-service.yaml

# Verify
kubectl get pods -n clnrm
kubectl get svc -n clnrm
```

## Test Job

```yaml
# test-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: clnrm-tests
  namespace: clnrm
spec:
  template:
    spec:
      containers:
      - name: test-runner
        image: clnrm:v1.2.0
        command:
          - cargo
          - test
          - --features
          - otel
          - --workspace
        env:
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://clnrm-weaver:4317"
        - name: RUST_LOG
          value: "info"
        - name: RUST_BACKTRACE
          value: "1"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
      restartPolicy: Never
  backoffLimit: 3
```

## StatefulSet for Persistence

```yaml
# weaver-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: weaver
  namespace: clnrm
spec:
  serviceName: weaver
  replicas: 1
  selector:
    matchLabels:
      app: weaver
  template:
    metadata:
      labels:
        app: weaver
    spec:
      containers:
      - name: weaver
        image: clnrm:v1.2.0
        command: ["weaver", "registry", "live-check", "--registry=/registry"]
        volumeMounts:
        - name: registry
          mountPath: /registry
        - name: validation-data
          mountPath: /output
      volumes:
      - name: registry
        configMap:
          name: weaver-registry
  volumeClaimTemplates:
  - metadata:
      name: validation-data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

## OTLP Collector Integration

```yaml
# otel-collector.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: otel-collector-config
  namespace: clnrm
data:
  config.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4318

    processors:
      batch:
        timeout: 1s

    exporters:
      otlp:
        endpoint: clnrm-weaver:4317
        tls:
          insecure: true
      logging:
        loglevel: info

    service:
      pipelines:
        traces:
          receivers: [otlp]
          processors: [batch]
          exporters: [otlp, logging]

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: otel-collector
  namespace: clnrm
spec:
  replicas: 2
  selector:
    matchLabels:
      app: otel-collector
  template:
    metadata:
      labels:
        app: otel-collector
    spec:
      containers:
      - name: collector
        image: otel/opentelemetry-collector-contrib:latest
        command:
          - /otelcol-contrib
          - --config=/conf/config.yaml
        ports:
        - name: otlp-grpc
          containerPort: 4318
          protocol: TCP
        volumeMounts:
        - name: config
          mountPath: /conf
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: config
        configMap:
          name: otel-collector-config

---
apiVersion: v1
kind: Service
metadata:
  name: otel-collector
  namespace: clnrm
spec:
  type: ClusterIP
  ports:
  - name: otlp-grpc
    port: 4318
    targetPort: 4318
  selector:
    app: otel-collector
```

## HorizontalPodAutoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: clnrm-weaver-hpa
  namespace: clnrm
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: clnrm-weaver
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## NetworkPolicy

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: clnrm-network-policy
  namespace: clnrm
spec:
  podSelector:
    matchLabels:
      app: clnrm
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: otel-collector
    - podSelector:
        matchLabels:
          app: clnrm-tests
    ports:
    - protocol: TCP
      port: 4317
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: jaeger
    ports:
    - protocol: TCP
      port: 14250
```

## Helm Chart

### 1. Chart Structure

```
clnrm-chart/
├── Chart.yaml
├── values.yaml
└── templates/
    ├── deployment.yaml
    ├── service.yaml
    ├── configmap.yaml
    ├── job.yaml
    └── hpa.yaml
```

### 2. values.yaml

```yaml
# values.yaml
image:
  repository: clnrm
  tag: v1.2.0
  pullPolicy: IfNotPresent

weaver:
  replicaCount: 1
  resources:
    requests:
      memory: 256Mi
      cpu: 250m
    limits:
      memory: 1Gi
      cpu: 1000m

  service:
    type: ClusterIP
    otlpPort: 4317
    adminPort: 8080

  registry:
    path: /registry
    configMap: weaver-registry

  output:
    path: /output
    persistence:
      enabled: true
      size: 10Gi

otelCollector:
  enabled: true
  replicaCount: 2
  endpoint: otel-collector:4318

autoscaling:
  enabled: true
  minReplicas: 1
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

tests:
  runOnDeploy: false
  image: clnrm:v1.2.0
  backoffLimit: 3
```

### 3. Install

```bash
# Install chart
helm install clnrm ./clnrm-chart -n clnrm --create-namespace

# Upgrade
helm upgrade clnrm ./clnrm-chart -n clnrm

# Uninstall
helm uninstall clnrm -n clnrm
```

## CI/CD Integration

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - validate
  - deploy

variables:
  KUBERNETES_NAMESPACE: clnrm
  DOCKER_REGISTRY: registry.example.com

build:
  stage: build
  script:
    - docker build -t $DOCKER_REGISTRY/clnrm:$CI_COMMIT_SHA .
    - docker push $DOCKER_REGISTRY/clnrm:$CI_COMMIT_SHA

test:
  stage: test
  script:
    - kubectl apply -f test-job.yaml
    - kubectl wait --for=condition=complete --timeout=600s job/clnrm-tests -n $KUBERNETES_NAMESPACE

validate:
  stage: validate
  script:
    - kubectl exec -n $KUBERNETES_NAMESPACE deploy/clnrm-weaver -- cat /output/validation_report.json > report.json
    - |
      violations=$(jq '.violations' report.json)
      if [ "$violations" -gt 0 ]; then
        echo "Validation failed with $violations violations"
        exit 1
      fi

deploy:
  stage: deploy
  only:
    - main
  script:
    - helm upgrade --install clnrm ./clnrm-chart -n $KUBERNETES_NAMESPACE
```

## Monitoring

### Prometheus ServiceMonitor

```yaml
# servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: clnrm-weaver
  namespace: clnrm
spec:
  selector:
    matchLabels:
      app: clnrm
      component: validator
  endpoints:
  - port: admin
    path: /metrics
    interval: 30s
```

### Grafana Dashboard

```yaml
# grafana-dashboard-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: clnrm-dashboard
  namespace: monitoring
data:
  clnrm-weaver.json: |
    {
      "dashboard": {
        "title": "clnrm Weaver Validation",
        "panels": [
          {
            "title": "Validation Status",
            "targets": [
              {
                "expr": "weaver_validation_status"
              }
            ]
          },
          {
            "title": "Violations",
            "targets": [
              {
                "expr": "weaver_violations_total"
              }
            ]
          }
        ]
      }
    }
```

## Troubleshooting

### Pod Won't Start

```bash
# Check events
kubectl describe pod -n clnrm <pod-name>

# Check logs
kubectl logs -n clnrm <pod-name>

# Check resource constraints
kubectl top pod -n clnrm
```

### Service Not Reachable

```bash
# Check service
kubectl get svc -n clnrm

# Test connectivity
kubectl run -it --rm debug --image=alpine --restart=Never -- sh
  apk add curl
  curl http://clnrm-weaver:4317

# Check endpoints
kubectl get endpoints -n clnrm
```

### Volume Issues

```bash
# Check PVC
kubectl get pvc -n clnrm

# Describe PVC
kubectl describe pvc -n clnrm validation-data

# Check permissions
kubectl exec -n clnrm <pod-name> -- ls -la /output
```

---

**Last Updated:** 2025-10-30
