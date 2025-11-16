# Tutorial 2: Container Pooling (10 minutes)

**⏱ Estimated Time**: 10 minutes
**📋 Prerequisites**: Completed Tutorial 1
**🎯 Learning Objectives**: Enable pooling and achieve 80% faster test startup

## What You'll Learn

By the end of this tutorial, you'll:
- ✅ Understand why tests are slow (2-5s per startup)
- ✅ Enable pooling with one environment variable
- ✅ See 80% speedup in practice
- ✅ Configure pool size and timeout
- ✅ Monitor pool performance

---

## The Problem: Slow Test Startup

Without pooling, each test creates a new Docker container:

```
Test 1: Create container (2-5s) → Run test → Destroy
Test 2: Create container (2-5s) → Run test → Destroy
Test 3: Create container (2-5s) → Run test → Destroy
...
Test 100: Create container (2-5s) → Run test → Destroy
```

**Total startup time for 100 tests: 200-500 seconds just for container creation!**

Even if each test only takes 100ms to run, 80% of time is wasted on startup.

---

## The Solution: Container Pooling

Instead of creating new containers, **pre-warm a pool of containers** and reuse them:

```
Pre-warm pool: [Container 1, Container 2, Container 3, ...]
                      ↓
Test 1: Grab container (0.1-0.5ms) → Run → Return to pool
Test 2: Grab container (0.1-0.5ms) → Run → Return to pool
Test 3: Grab container (0.1-0.5ms) → Run → Return to pool
...
Test 100: Grab container (0.1-0.5ms) → Run → Return to pool
```

**Total startup time: 0.1-0.5 seconds (80% faster!)** 🚀

---

## Step 1: Benchmark Without Pooling (2 minutes)

Using the test from Tutorial 1, let's measure speed:

```bash
# Create 5 copies of the test to measure
cd tests/
for i in {1..5}; do
  cp my-first-test.clnrm.toml test-${i}.clnrm.toml
done
cd ..

# Run WITHOUT pooling and time it
time clnrm run

# Output example:
# real    12.345s
# user    2.134s
# sys     1.456s
```

Note the total time. This is our baseline.

---

## Step 2: Enable Pooling (1 minute)

Enable with a single environment variable:

```bash
# Run WITH pooling enabled
CLNRM_ENABLE_POOLING=1 time clnrm run

# Output example:
# real    2.450s    ← 5x faster!
# user    1.234s
# sys     0.567s
```

### What Changed?

```bash
# Without pooling
time clnrm run         # ~12.3 seconds

# With pooling
CLNRM_ENABLE_POOLING=1 time clnrm run  # ~2.4 seconds

# Speed improvement
12.3 / 2.4 = 5.1x faster
```

**That's the power of pooling!** The difference is even more dramatic with many tests.

---

## Step 3: Understand the Pool (2 minutes)

When you enable pooling, clnrm:

1. **Pre-warms containers** — Creates 5 Alpine containers at startup (default)
2. **Stores in FIFO queue** — Keeps them ready to use
3. **Assigns on demand** — Gives a container to each test
4. **Returns after use** — Container goes back to pool for reuse
5. **Background cleanup** — Removes idle containers to save memory

### Pool Architecture

```
                        clnrm run (with pooling)
                               ↓
                    Pre-warm 5 containers
                    ↓    ↓    ↓    ↓    ↓
Idle Pool:     [C1]  [C2]  [C3]  [C4]  [C5]
                ↓
Test 1: Grab C1 (0.5ms) → Run → Return
                ↓
Test 2: Grab C2 (0.5ms) → Run → Return
                ↓
Test 3: Grab C3 (0.5ms) → Run → Return
```

---

## Step 4: Configure Pooling (3 minutes)

You can customize pooling behavior via environment variables:

```bash
# Pool with 10 containers (default: 5)
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_SIZE=10 \
clnrm run

# Pool with longer idle timeout (default: 60s)
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_IDLE_TIMEOUT_MS=120000 \
clnrm run

# All together
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_SIZE=10 \
CLNRM_POOL_IDLE_TIMEOUT_MS=120000 \
clnrm run
```

### Configuration Options

| Variable | Default | Range | Meaning |
|----------|---------|-------|---------|
| `CLNRM_ENABLE_POOLING` | false | true/false | Enable pooling |
| `CLNRM_POOL_SIZE` | 5 | 1-100 | Pre-warmed containers |
| `CLNRM_POOL_IDLE_TIMEOUT_MS` | 60000 | 1000-600000 | Idle timeout (ms) |

### Tuning Guidelines

**For Fast Feedback (CI/CD)**:
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=10 clnrm run
```

**For Resource-Limited Environments**:
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=2 clnrm run
```

**For High Concurrency** (100+ parallel tests):
```bash
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=20 clnrm run --parallel --jobs 16
```

---

## Step 5: Monitor Pool Performance (2 minutes)

After running tests with pooling, you see metrics:

```bash
CLNRM_ENABLE_POOLING=1 clnrm run

# Output includes:
Pool Statistics:
  Hit Rate: 94%              # Percent of times container was available
  Miss Rate: 6%              # Percent of times had to create new
  Avg Acquisition Time: 0.3ms
  Slowest Acquisition: 2.1s  # First test (pre-warming)
  Pool Size: 5
  Active Containers: 3
```

### Understanding the Metrics

- **Hit Rate (94%)** — Great! Most tests grabbed pre-warmed containers
- **Avg Acquisition (0.3ms)** — Very fast (vs 2-5s without pooling)
- **Slowest Acquisition (2.1s)** — First test (container pre-warming, expected)

### Target Metrics

- ✅ **Hit Rate > 90%** — Pool is effective
- ✅ **Avg Acquisition < 1ms** — Pool working well
- ⚠️ **Hit Rate < 70%** — Increase pool size or idle timeout
- ⚠️ **Avg Acquisition > 100ms** — Issue with pool, check Docker

---

## Parallel Testing + Pooling (Advanced)

Pooling works great with parallel testing:

```bash
# Without pooling (50 sequential tests)
time clnrm run              # ~25 seconds (0.5s each)

# With pooling (50 sequential tests)
CLNRM_ENABLE_POOLING=1 time clnrm run   # ~5 seconds (5x faster)

# With pooling + parallel (50 concurrent tests with 10 jobs)
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_SIZE=10 \
clnrm run --parallel --jobs 10  # ~2 seconds (12x faster!)
```

---

## Key Concepts

### Why Pooling Works
- **Elimination of startup overhead** — 2-5 seconds → 0.5 milliseconds
- **Container reuse** — No need to recreate
- **FIFO fairness** — Each test gets a container in order
- **Background cleanup** — Old containers automatically removed

### Trade-offs

| Aspect | Benefit | Cost |
|--------|---------|------|
| **Speed** | 5-10x faster ✅ | |
| **Memory** | | Uses more (per container in pool) |
| **Startup** | Faster ✅ | Longer initial warm-up |
| **Simplicity** | One env var ✅ | One more thing to tune |

---

## Troubleshooting

### Pooling not working (Hit Rate = 0%)
**Problem**: Tests always create new containers

**Check**:
```bash
# Verify pooling is enabled
echo $CLNRM_ENABLE_POOLING   # Should be "1"

# Try increasing timeout
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_IDLE_TIMEOUT_MS=300000 clnrm run

# Check Docker is working
docker ps
```

### Memory usage high
**Problem**: Pool is too large

**Solution**:
```bash
# Reduce pool size
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_SIZE=2 clnrm run
```

### Tests timing out
**Problem**: Container doesn't acquire in time

**Solution**:
```bash
# Check Docker performance
docker stats

# Or disable pooling to debug
clnrm run  # Run without pooling to see if it's pool-related
```

---

## Summary

You now know:
- ✅ **Why tests are slow** — Container startup (2-5s each)
- ✅ **How pooling helps** — Pre-warm and reuse containers (0.5ms each)
- ✅ **How to enable** — One environment variable
- ✅ **How to configure** — Pool size, idle timeout
- ✅ **How to monitor** — Hit rate, acquisition time

---

## Next Steps

### Want to make sure tests are correct?
→ [Tutorial 3: Weaver Validation](../03-weaver-validation/)

### Want to use pooling with parallel tests?
→ [How-To: Parallel Execution](../../how-to/parallel-execution.md)

### Want to understand how pooling works?
→ [Explanation: Container Pooling](../../explanation/container-pooling.md)

### Want to optimize further?
→ [How-To: Performance Tuning](../../how-to/performance-tuning.md)

---

**Congratulations!** You've achieved 5-10x speedup with pooling! 🚀

Next: [Tutorial 3: Weaver Validation](../03-weaver-validation/)
