# Stress Test Configurations

This directory contains example stress test configurations for the clnrm framework.

## Overview

Stress tests validate the framework's behavior under load by generating permutations of:
- **Container types**: Different Docker images
- **Test iterations**: Multiple test runs per container
- **OTEL span depth**: Varying levels of telemetry nesting

## Configuration Files

### basic_stress.toml
- **Use case**: Development and quick validation
- **Resources**: Minimal (1GB RAM, 5 containers max)
- **Duration**: ~2-5 minutes
- **Total tests**: ~50 permutations

```bash
clnrm stress --config tests/stress/basic_stress.toml
```

### medium_stress.toml
- **Use case**: CI/CD pipelines and integration testing
- **Resources**: Moderate (3GB RAM, 15 containers max)
- **Duration**: ~10-15 minutes
- **Total tests**: ~180 permutations

```bash
clnrm stress --config tests/stress/medium_stress.toml
```

### heavy_stress.toml
- **Use case**: Production validation and performance benchmarking
- **Resources**: Heavy (8GB RAM, 30 containers max)
- **Duration**: ~30-45 minutes
- **Total tests**: ~900 permutations

```bash
clnrm stress --config tests/stress/heavy_stress.toml
```

## Running Stress Tests

### Using Configuration Files

```bash
# Run with a predefined configuration
clnrm stress --config tests/stress/basic_stress.toml

# Run with output directory
clnrm stress --config tests/stress/medium_stress.toml --output results/
```

### Using CLI Arguments

```bash
# Basic stress test
clnrm stress \
  --containers alpine:latest ubuntu:latest \
  --test-count 10 \
  --span-depth 5 \
  --max-containers 10 \
  --concurrency 2

# Heavy stress test
clnrm stress \
  --containers alpine:latest ubuntu:latest debian:stable-slim \
  --test-count 50 \
  --span-depth 20 \
  --max-containers 30 \
  --concurrency 8 \
  --max-memory-mb 8192 \
  --timeout-secs 300
```

## Understanding Permutations

The stress test generates permutations across three dimensions:

1. **Containers**: Each specified container image
2. **Iterations**: Number of test runs per container
3. **Span Depths**: OTEL span nesting levels (powers of 2)

**Example calculation:**
- Containers: 3 images
- Test count: 20 iterations
- Span depth: 10 levels → generates depths [1, 2, 4, 8, 10]

Total permutations: `3 × 20 × 5 = 300 tests`

## Resource Requirements

### Basic Configuration
- RAM: 1-2 GB
- CPU: 2 cores
- Disk: 500 MB (for container images)
- Time: 2-5 minutes

### Medium Configuration
- RAM: 3-4 GB
- CPU: 4 cores
- Disk: 1 GB
- Time: 10-15 minutes

### Heavy Configuration
- RAM: 8-12 GB
- CPU: 8+ cores
- Disk: 2 GB
- Time: 30-45 minutes

## Metrics Collected

Stress tests collect comprehensive metrics:

- **Test execution times**: Min, max, average
- **Container pool utilization**: Peak and average
- **OTEL span generation**: Total spans created
- **Success rate**: Percentage of passing tests
- **Resource consumption**: Memory and CPU usage
- **Error analysis**: Categorized failure reasons

## Output

Results are written to JSON format:

```json
{
  "total_tests": 300,
  "passed_tests": 295,
  "failed_tests": 5,
  "skipped_tests": 0,
  "total_duration_ms": 125430,
  "avg_test_duration_ms": 418.1,
  "peak_pool_utilization": 87.5,
  "total_spans_generated": 45000,
  "executions": [...],
  "errors": [...]
}
```

## Interpreting Results

### Success Criteria

✅ **Pass**: All tests complete with 100% success rate
⚠️ **Warning**: >95% success rate (some degradation acceptable)
❌ **Fail**: <95% success rate (indicates issues)

### Performance Benchmarks

- **Test duration**: Should be <500ms average for basic containers
- **Pool utilization**: Should stay below 90% (allows headroom)
- **Span generation**: Should complete without errors

### Common Issues

1. **Resource exhaustion**: Increase `max_containers` or reduce `concurrency`
2. **Timeout errors**: Increase `test_timeout` or reduce `span_depth`
3. **Container startup failures**: Check Docker daemon and increase `container_startup_timeout`

## Best Practices

1. **Start small**: Begin with `basic_stress.toml` before scaling up
2. **Monitor resources**: Watch system resources during execution
3. **Isolate environments**: Run heavy stress tests in dedicated environments
4. **Analyze failures**: Review error messages and adjust configuration
5. **Iterate**: Use results to tune resource limits and timeouts

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Run stress tests
  run: |
    clnrm stress --config tests/stress/medium_stress.toml
  timeout-minutes: 20
```

### GitLab CI Example

```yaml
stress_test:
  script:
    - clnrm stress --config tests/stress/medium_stress.toml
  timeout: 20m
  artifacts:
    paths:
      - stress_test_results.json
```

## Customization

Create custom configurations by copying and modifying existing examples:

```bash
cp tests/stress/basic_stress.toml tests/stress/custom_stress.toml
# Edit custom_stress.toml
clnrm stress --config tests/stress/custom_stress.toml
```

## Troubleshooting

### Out of Memory (OOM)

- Reduce `max_containers`
- Reduce `max_memory_mb`
- Reduce `test_count`

### Container Startup Timeouts

- Increase `container_startup_timeout`
- Pre-pull images: `docker pull alpine:latest`
- Check Docker daemon status

### Slow Execution

- Increase `concurrency` (if resources available)
- Reduce `span_depth`
- Use smaller container images

## Support

For issues or questions:
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: docs/STRESS_TESTING.md
