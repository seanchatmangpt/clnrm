# Performance Tuning

Performance tuning optimizes clnrm execution for faster test runs, better resource utilization, and improved scalability in production environments.

## Overview

Performance tuning covers:
- **Execution optimization** - Parallel execution, resource allocation
- **Image management** - Pre-pulling, caching, layer optimization
- **Container optimization** - Resource limits, networking, storage
- **System optimization** - OS tuning, filesystem, memory management
- **Monitoring and profiling** - Performance analysis, bottleneck identification

## Execution Optimization

### Parallel Execution Configuration

Configure optimal parallel execution:

```toml
# Parallel execution configuration
[execution]
parallel = true
workers = 8
max_concurrent_services = 20

[execution.strategy]
type = "dependency_aware"
load_balancing = "round_robin"
resource_aware = true

[execution.timeouts]
test_timeout_minutes = 30
service_startup_seconds = 60
health_check_seconds = 10

[execution.retry]
max_retries = 3
backoff_strategy = "exponential"
backoff_base_seconds = 2
```

### Resource Allocation

Optimize resource allocation for your workload:

```toml
# Resource allocation configuration
[resources]
cpu_cores = 8
memory_gb = 16
storage_gb = 50

[resources.allocation]
strategy = "bin_packing"
cpu_per_worker = 1
memory_per_worker_gb = 2

[resources.limits]
max_containers = 50
max_concurrent_tests = 20
max_memory_usage_percent = 80

[resources.reservation]
min_cpu_cores = 0.5
min_memory_mb = 512
```

### Worker Pool Configuration

Configure worker pools for different test types:

```toml
# Worker pool configuration
[workers]
pools = [
    { name = "cpu_intensive", cpu_cores = 4, memory_gb = 8, count = 2 },
    { name = "memory_intensive", cpu_cores = 2, memory_gb = 16, count = 1 },
    { name = "network_intensive", cpu_cores = 2, memory_gb = 4, count = 4 },
    { name = "default", cpu_cores = 1, memory_gb = 2, count = 8 }
]

[workers.scheduling]
strategy = "resource_aware"
priority_classes = ["high", "medium", "low"]
preemption = true
```

## Image Management

### Image Pre-pulling

Pre-pull images for faster test execution:

```bash
#!/bin/bash
# pre-pull-images.sh

echo "🔄 Pre-pulling Docker images for faster test execution..."

# Extract unique images from test files
images=$(grep -r "image.*=" tests/ | grep -o '"[^"]*"' | sort | uniq)

echo "📦 Found $(echo "$images" | wc -l) unique images to pull"

for image in $images; do
    image=$(echo $image | tr -d '"')
    echo "📦 Pulling $image..."
    docker pull "$image" &
done

# Wait for all pulls to complete
wait

echo "✅ All images pre-pulled successfully"
```

### Image Caching Strategy

Implement intelligent image caching:

```toml
# Image caching configuration
[cache]
enabled = true
directory = ".clnrm/cache"
max_size_gb = 20

[cache.images]
ttl_hours = 168  # 1 week
cleanup_strategy = "lru"
compression = true

[cache.layers]
shared_layers = true
deduplication = true
```

### Multi-Registry Support

Configure multiple container registries:

```toml
# Multi-registry configuration
[registries]
primary = "registry.company.com"
fallback = ["docker.io", "gcr.io", "quay.io"]

[registries.auth]
registry.company.com = {
    username = "{{ env(name=\"REGISTRY_USER\") }}",
    password = "{{ env(name=\"REGISTRY_PASSWORD\") }}"
}

[registries.mirroring]
enabled = true
mirror_strategy = "primary_fallback"
sync_schedule = "0 2 * * *"  # Daily at 2 AM
```

## Container Optimization

### Container Resource Limits

Set appropriate resource limits:

```toml
# Container resource limits
[containers]
default_memory_limit = "512Mi"
default_cpu_limit = "500m"

[containers.api]
memory_limit = "1Gi"
cpu_limit = "1000m"
cpu_request = "100m"

[containers.database]
memory_limit = "2Gi"
cpu_limit = "2000m"
cpu_request = "500m"

[containers.cache]
memory_limit = "512Mi"
cpu_limit = "500m"
cpu_request = "100m"
```

### Network Optimization

Optimize container networking:

```toml
# Network optimization
[network]
dns_servers = ["8.8.8.8", "1.1.1.1"]
dns_search_domains = ["company.com"]
mtu = 1500

[network.optimization]
disable_ipv6 = true
enable_loopback_optimization = true
container_network_mode = "bridge"

[network.bandwidth]
limit_mbps = 1000
burst_mbps = 2000
```

### Storage Optimization

Optimize storage for faster I/O:

```toml
# Storage optimization
[storage]
driver = "overlay2"
root_directory = "/var/lib/docker"

[storage.optimization]
async_io = true
direct_io = true
block_device_alignment = true

[storage.cleanup]
enabled = true
interval_hours = 24
max_unused_space_gb = 10
```

## System Optimization

### OS-Level Tuning

Tune the operating system for better performance:

```bash
#!/bin/bash
# system-optimization.sh

echo "🔧 Optimizing system for clnrm performance..."

# Increase file descriptor limits
echo "fs.file-max = 1000000" >> /etc/sysctl.conf
echo "fs.nr_open = 1000000" >> /etc/sysctl.conf

# Increase network buffer sizes
echo "net.core.rmem_max = 16777216" >> /etc/sysctl.conf
echo "net.core.wmem_max = 16777216" >> /etc/sysctl.conf
echo "net.core.netdev_max_backlog = 5000" >> /etc/sysctl.conf

# Optimize virtual memory
echo "vm.swappiness = 1" >> /etc/sysctl.conf
echo "vm.vfs_cache_pressure = 50" >> /etc/sysctl.conf

# Apply changes
sysctl -p

echo "✅ System optimization complete"
```

### Filesystem Optimization

Optimize filesystem for better I/O performance:

```bash
#!/bin/bash
# filesystem-optimization.sh

echo "💾 Optimizing filesystem for clnrm..."

# Mount with appropriate options
mount -o remount,noatime,nodiratime /tmp
mount -o remount,noatime,nodiratime /var/lib/docker

# Create optimized mount points
mkdir -p /mnt/fast-ssd
mount -o noatime,nodiratime,discard /dev/nvme1n1 /mnt/fast-ssd

echo "✅ Filesystem optimization complete"
```

### Memory Management

Optimize memory usage:

```toml
# Memory management configuration
[memory]
total_memory_gb = 16
reserved_for_system_gb = 2

[memory.allocation]
heap_size_gb = 8
stack_size_mb = 8
page_cache_gb = 4

[memory.optimization]
huge_pages = true
transparent_huge_pages = "madvise"
memory_compaction = true
```

## Monitoring and Profiling

### Performance Monitoring

Monitor performance metrics:

```toml
# Performance monitoring configuration
[monitoring]
enabled = true
interval_seconds = 5
retention_hours = 168  # 1 week

[monitoring.metrics]
cpu_usage_percent = true
memory_usage_mb = true
disk_io_mbps = true
network_io_mbps = true
test_execution_time_seconds = true
container_startup_time_seconds = true

[monitoring.alerts]
cpu_threshold_percent = 80
memory_threshold_percent = 85
disk_threshold_percent = 90
test_timeout_minutes = 30
```

### Profiling and Analysis

Profile system performance:

```toml
# Profiling configuration
[profiling]
enabled = true
sampling_interval_ms = 1000

[profiling.cpu]
enabled = true
sample_rate_hz = 100
include_kernel = true

[profiling.memory]
enabled = true
allocation_tracking = true
leak_detection = true

[profiling.io]
enabled = true
block_io = true
network_io = true

[profiling.tools]
perf = true
valgrind = false  # Disabled for performance
flamegraph = true
```

## Load Testing Optimization

### Load Testing Configuration

Optimize for load testing scenarios:

```toml
# Load testing optimization
[load_testing]
enabled = true
max_concurrent_users = 1000
ramp_up_seconds = 60
steady_state_seconds = 300
ramp_down_seconds = 30

[load_testing.distribution]
think_time_seconds = { min = 1, max = 5 }
request_distribution = "uniform"

[load_testing.monitoring]
detailed_metrics = true
response_time_histogram = true
error_rate_tracking = true
```

### Distributed Load Testing

Configure distributed load testing:

```toml
# Distributed load testing
[distributed]
enabled = true
coordinator_host = "load-test-coordinator.company.com"
worker_count = 10

[distributed.coordination]
heartbeat_interval_seconds = 30
result_collection_timeout_minutes = 5

[distributed.workers]
worker_1 = { host = "worker-1.company.com", capacity = 100 }
worker_2 = { host = "worker-2.company.com", capacity = 100 }
# ... more workers

[distributed.load_balancing]
strategy = "round_robin"
rebalance_interval_minutes = 5
```

## Best Practices

### 1. Profile Before Optimizing

```bash
# ✅ Good: Profile before optimizing
clnrm run tests/performance/ --profiling --output profile.json
# Analyze profile.json to identify bottlenecks
```

### 2. Set Appropriate Resource Limits

```toml
# ✅ Good: Appropriate resource limits
[containers.api]
memory_limit = "1Gi"  # Not too restrictive
cpu_limit = "1000m"   # Reasonable for API workload

[containers.database]
memory_limit = "2Gi"  # Database needs more memory
cpu_limit = "2000m"   # Higher CPU allocation
```

### 3. Use Pre-pulling for Production

```bash
# ✅ Good: Pre-pull images in production
- name: Pre-pull images
  run: clnrm pull tests/ --parallel
```

### 4. Monitor Resource Usage

```toml
# ✅ Good: Monitor resource usage
[monitoring]
enabled = true
interval_seconds = 30

[monitoring.alerts]
cpu_threshold_percent = 80
memory_threshold_percent = 85
```

## Common Patterns

### High-Performance Test Environment

```toml
# High-performance test environment configuration
[performance]
high_performance_mode = true

[execution]
parallel = true
workers = 16
max_concurrent_services = 50

[containers]
default_memory_limit = "2Gi"
default_cpu_limit = "2000m"

[cache]
enabled = true
max_size_gb = 50

[monitoring]
enabled = true
interval_seconds = 10

[optimization]
image_sharing = true
container_reuse = true
parallel_downloads = true
```

### Resource-Constrained Environment

```toml
# Resource-constrained environment configuration
[performance]
constrained_mode = true

[execution]
parallel = true
workers = 2
max_concurrent_services = 5

[containers]
default_memory_limit = "512Mi"
default_cpu_limit = "500m"

[cache]
enabled = true
max_size_gb = 5

[monitoring]
enabled = true
interval_seconds = 60
```

### Development Environment

```toml
# Development environment configuration
[performance]
development_mode = true

[execution]
parallel = false
workers = 1

[containers]
default_memory_limit = "256Mi"
default_cpu_limit = "200m"

[cache]
enabled = false

[monitoring]
enabled = false
```

## Next Steps

Now that you understand performance tuning:

1. **Profile your current setup**: Run performance tests to identify bottlenecks
2. **Implement optimizations**: Apply the optimizations that provide the most benefit
3. **Learn enterprise patterns**: Move on to [Enterprise Patterns](enterprise-patterns.md)
4. **Set up monitoring**: Configure monitoring for your production environment

## Further Reading

- [Docker Performance Tuning](https://docs.docker.com/config/containers/resource_constraints/)
- [Kubernetes Performance](https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/)
- [Linux Performance Tuning](https://www.kernel.org/doc/Documentation/sysctl/)

