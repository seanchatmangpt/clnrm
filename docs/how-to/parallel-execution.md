# How-To: Run Tests in Parallel

**Problem**: Sequential tests are slow. 100 tests taking 0.5s each = 50 seconds total
**Solution**: Run multiple tests concurrently, divide time by job count

## Quick Answer

```bash
# Run with 4 concurrent jobs (default)
clnrm run --parallel

# Run with 16 concurrent jobs
clnrm run --parallel --jobs 16

# Run with container pooling + parallelism (10x faster!)
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16
```

## Step-by-Step

### 1. Enable Parallel Mode

```bash
clnrm run --parallel
```

This runs tests concurrently with default settings (usually 4-8 jobs depending on CPU).

### 2. Configure Job Count

```bash
# Set specific job count
clnrm run --parallel --jobs 16

# Relationship: Tests ÷ Jobs = Total Time
# 100 tests, 4 jobs = ~25 seconds
# 100 tests, 16 jobs = ~6 seconds
```

### 3. Combine with Pooling (Recommended)

```bash
# 10x speedup!
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16
```

Performance breakdown:
- Sequential, no pooling: ~50 seconds
- Parallel (16 jobs), no pooling: ~6 seconds (8x faster)
- Parallel (16 jobs) + pooling: ~3 seconds (16x faster!)

## Choosing Job Count

| Scenario | Jobs | Reasoning |
|----------|------|-----------|
| Few tests (< 10) | 2-4 | Overhead not worth it |
| Many tests (10-50) | 4-8 | Good balance |
| Many tests (50+) | 16+ | Maximum parallelism |
| High resource tests | 2-4 | Each test uses lots of memory/CPU |
| Light tests | 16+ | Tests use minimal resources |

## Real-World Example

```bash
# Measure baseline (sequential)
time clnrm run
# Output: real 45.234s

# Measure parallel
time clnrm run --parallel --jobs 8
# Output: real 6.789s (6.6x faster)

# Measure parallel + pooling
CLNRM_ENABLE_POOLING=1 time clnrm run --parallel --jobs 8
# Output: real 2.123s (21x faster!)
```

## In CI/CD (GitHub Actions Example)

```yaml
- name: Run tests (parallel)
  run: |
    CLNRM_ENABLE_POOLING=1 \
    clnrm run --parallel --jobs 16
```

## Troubleshooting

### Error: Out of Memory
**Problem**: Too many concurrent tests exhaust RAM

**Solution**:
```bash
# Reduce job count
clnrm run --parallel --jobs 4

# Or reduce pool size
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_SIZE=2 \
clnrm run --parallel --jobs 4
```

### Error: Docker Connection Issues
**Problem**: Docker daemon overwhelmed by concurrent container operations

**Solution**:
```bash
# Reduce parallelism
clnrm run --parallel --jobs 2

# Or enable pooling to reuse containers
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 4
```

## See Also

- [Explanation: Concurrency Model](../explanation/concurrency.md)
- [How-To: Container Pooling Setup](./container-pooling-setup.md)
- [How-To: Performance Tuning](./performance-tuning.md)
