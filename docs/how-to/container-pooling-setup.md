# How-To: Enable and Configure Container Pooling

**Problem**: Tests are slow due to 2-5s container startup overhead
**Solution**: Pre-warm pool of containers, reduce startup to 0.5ms

## Quick Answer

```bash
# Enable pooling (one environment variable!)
CLNRM_ENABLE_POOLING=1 clnrm run

# Expected improvement: 5-10x faster
```

## Enable Pooling

### Option 1: Environment Variable (Temporary)

```bash
CLNRM_ENABLE_POOLING=1 clnrm run
```

### Option 2: Persistent Configuration

Add to `.bashrc`, `.zshrc`, or CI/CD configuration:

```bash
export CLNRM_ENABLE_POOLING=1
export CLNRM_POOL_SIZE=5
export CLNRM_POOL_IDLE_TIMEOUT_MS=60000

clnrm run
```

### Option 3: TOML Configuration

In `.clnrm.toml`:
```toml
[pool]
enabled = true
size = 5
idle_timeout_ms = 60000
```

## Configure Pool Size

Pool size controls how many containers are pre-warmed:

```bash
# Small (2 containers) - low memory, slower
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=2 clnrm run

# Medium (5 containers) - balanced
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=5 clnrm run

# Large (20 containers) - fast, high memory
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=20 clnrm run
```

## Tune for Your Workload

### Light Tests (Quick Tests, Few Tests)
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=3 clnrm run
```

### Heavy Tests (Long Tests, Many Tests)
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=10 clnrm run --parallel --jobs 8
```

### Tight Memory (Containers, Kubernetes)
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=1 clnrm run
```

### High Concurrency (100+ Tests)
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=20 clnrm run --parallel --jobs 16
```

## Monitor Pool Performance

After running with pooling:

```bash
CLNRM_ENABLE_POOLING=1 clnrm run --verbose

# Output shows:
# Pool Statistics:
#   Hit Rate: 94%
#   Avg Acquisition: 0.3ms
#   Pool Size: 5
```

### Understanding Metrics

- **Hit Rate > 90%** ✅ Good (containers available when needed)
- **Hit Rate < 70%** ⚠️ Increase pool size
- **Avg Acquisition < 1ms** ✅ Good
- **Avg Acquisition > 100ms** ⚠️ Docker issue

## Benchmark Improvement

```bash
# Before pooling
time clnrm run
# real 12.345s

# After pooling
CLNRM_ENABLE_POOLING=1 time clnrm run
# real 2.150s

# Improvement: 5.7x faster!
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run tests with pooling
  run: |
    CLNRM_ENABLE_POOLING=1 \
    CLNRM_POOL_SIZE=10 \
    clnrm run --parallel --jobs 8
```

### GitLab CI

```yaml
test:
  script:
    - CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=10 clnrm run
```

### Jenkins

```groovy
stage('Test') {
  environment {
    CLNRM_ENABLE_POOLING = '1'
    CLNRM_POOL_SIZE = '10'
  }
  steps {
    sh 'clnrm run --parallel --jobs 8'
  }
}
```

## Troubleshooting

### Pooling Not Enabled (No Speedup)

Check if enabled:
```bash
CLNRM_ENABLE_POOLING=1 clnrm run --verbose | grep "Pool"

# Should show: "Pool Statistics:"
```

### Out of Memory with Large Pool

Reduce pool size:
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=2 clnrm run
```

### Low Hit Rate (Pooling Ineffective)

Increase idle timeout:
```bash
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_SIZE=10 \
CLNRM_POOL_IDLE_TIMEOUT_MS=300000 \
clnrm run
```

## See Also

- [Tutorial 2: Container Pooling](../tutorials/02-container-pooling/)
- [Explanation: Container Pooling](../explanation/container-pooling.md)
- [How-To: Performance Tuning](./performance-tuning.md)
- [How-To: Parallel Execution](./parallel-execution.md)
