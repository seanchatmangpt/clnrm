# OTEL Span Batching Optimization (v1.6.0)

**Feature Version**: v1.6.0
**Target Performance**: <300ms for 10K spans (-16% improvement)
**Implementation Status**: Design Complete
**Last Updated**: 2025-11-18

---

## Executive Summary

This document outlines the optimization of OpenTelemetry span batching to reduce export latency by 16% (356ms → <300ms for 10K spans) at scale. The optimization targets high-throughput scenarios (100K+ spans) while maintaining zero span loss on critical paths.

## Current Performance Analysis

### Baseline (v1.5.0)

```
Span Volume | Export Time | Throughput | Status
─────────────────────────────────────────────────
1K spans    | 35ms        | 28K spans/s | ✓ Acceptable
10K spans   | 356ms       | 28K spans/s | ⚠ 10-16% regression
100K spans  | 3.5s        | 28K spans/s | ✗ Unacceptable
```

### Bottleneck Analysis

**Root Cause: Synchronous Batch Export**

```rust
// v1.5.0 implementation
pub async fn export_spans(&self, spans: Vec<SpanData>) -> Result<()> {
    // Lock batch writer
    let mut batch = self.batch_lock.lock().await;

    // Append spans serially
    for span in spans {
        batch.push(span);
        if batch.len() >= MAX_BATCH {
            // Blocking export
            self.http_client.post(endpoint, batch).await?;
            batch.clear();
        }
    }

    Ok(())
}
```

**Issues**:
1. Synchronous HTTP export blocks further appends
2. Network latency (100-200ms) serializes batch operations
3. No parallelism in batch handling
4. Memory spikes with large batches

## Optimization Strategy

### Solution Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ Fast Path: Span Appending (Lock-Free)                        │
│ ┌────────────────────────────────────────────────────────┐  │
│ │ span_channel: mpsc<Vec<SpanData>>                     │  │
│ │ (Non-blocking send to queue)                          │  │
│ └────────────────────────────────────────────────────────┘  │
│                          ↓                                    │
│ Slow Path: Async Export (Non-Blocking)                      │
│ ┌────────────────────────────────────────────────────────┐  │
│ │ background_exporter task                              │  │
│ │ • Collect spans from channel                          │  │
│ │ • Wait for batch_size OR flush_interval                │  │
│ │ • Export asynchronously                               │  │
│ │ • Update metrics                                      │  │
│ └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Key Components

#### 1. **AsyncSpanExporter**

```rust
pub struct AsyncSpanExporter {
    /// Channel to queue spans for export
    sender: mpsc::Sender<Vec<SpanData>>,
    /// Batch size configuration (default: 512)
    batch_size: usize,
    /// Flush interval for incomplete batches
    flush_interval: Duration,
    /// Metrics tracking
    metrics: Arc<ExportMetrics>,
}

pub struct ExportMetrics {
    batches_exported: Arc<AtomicU64>,
    spans_exported: Arc<AtomicU64>,
    export_duration_ms: Arc<AtomicU64>,
    dropped_spans: Arc<AtomicU64>,
}
```

#### 2. **AdaptiveSampler**

```rust
pub struct AdaptiveSampler {
    /// Base sampling rate (0.0 - 1.0)
    base_rate: f64,
    /// Current adaptive rate
    current_rate: Arc<AtomicU32>,
    /// Span counter for rate adjustment
    span_counter: Arc<AtomicU64>,
    /// Critical span patterns (never sampled)
    critical_patterns: Vec<Regex>,
}

impl AdaptiveSampler {
    /// Decide whether to sample a span
    pub fn should_sample(&self, span: &SpanData) -> bool {
        // Never drop error/slow spans
        if span.status == SpanStatus::Error {
            return true;
        }
        if span.duration_ms > SLOW_THRESHOLD {
            return true;
        }

        // Probabilistic sampling for others
        rand::random::<f64>() < self.current_rate as f64 / 100.0
    }
}
```

#### 3. **Batch Management**

```rust
pub struct SpanBatcher {
    /// Current batch accumulator
    current_batch: Arc<Mutex<Vec<SpanData>>>,
    /// Batch size limits
    min_batch: usize,
    max_batch: usize,
    /// Last flush time
    last_flush: Arc<Mutex<Instant>>,
}

impl SpanBatcher {
    /// Try to flush batch if conditions met
    async fn try_flush(&self) -> Option<Vec<SpanData>> {
        let batch = self.current_batch.lock().await;

        // Flush if batch size threshold or timeout reached
        if batch.len() >= self.max_batch
            || (batch.len() > 0 && last_flush.elapsed() > FLUSH_INTERVAL) {
            return Some(batch.clone());
        }

        None
    }
}
```

## Configuration

### Default Configuration

```rust
pub struct OtelConfig {
    /// Batch size tuning
    batch_size: usize,      // Default: 512 spans
    max_batch: usize,       // Default: 2048 spans
    min_batch: usize,       // Default: 128 spans

    /// Export settings
    flush_interval: Duration, // Default: 5 seconds
    export_timeout: Duration, // Default: 30 seconds

    /// Sampling
    base_sampling_rate: f64, // Default: 1.0 (no sampling)
    adaptive_sampling: bool, // Default: true
    max_sampling_rate: f64,  // Default: 0.1 (drop 90% at saturation)

    /// Queue limits
    queue_capacity: usize,  // Default: 100K spans
    drop_policy: DropPolicy, // Default: DropOldest
}

pub enum DropPolicy {
    DropOldest,    // FIFO when queue full
    DropNewest,    // Reject new spans when full
    DropRandom,    // Random drop (for unbiased sampling)
}
```

### Environment Variable Configuration

```bash
# Batch tuning
export OTEL_BSP_MAX_BATCH_SIZE=2048
export OTEL_BSP_SCHEDULED_DELAY_MS=5000
export OTEL_BSP_MAX_QUEUE_SIZE=100000

# Sampling
export OTEL_TRACES_SAMPLER=adaptive
export OTEL_TRACES_SAMPLER_ARG=0.5

# Export timeout
export OTEL_BSP_MAX_EXPORT_BATCH_TIMEOUT_MS=30000
```

## Performance Targets

### Latency Improvements

| Metric | v1.5.0 | v1.6.0 | Improvement |
|--------|--------|--------|------------|
| **1K spans** | 35ms | <50ms | No regression |
| **10K spans** | 356ms | <300ms | **-16%** ✓ |
| **100K spans** | 3.5s | <1.0s | **-71%** ✓ |
| **P99 latency** | 200ms | <100ms | **-50%** ✓ |

### Throughput Characteristics

| Scenario | Throughput | Latency | Notes |
|----------|-----------|---------|-------|
| **Steady state (1K spans/sec)** | 1K/s | <5ms | <1% resource |
| **High load (10K spans/sec)** | 10K/s | <10ms | ~5% resource |
| **Peak load (100K spans/sec)** | 100K/s | <50ms | ~20% resource |
| **Overload (1M spans/sec)** | Sampled | ~100ms | Adaptive sampling |

### Memory Bounds

| Metric | Target | Mechanism |
|--------|--------|-----------|
| **Queue memory** | <100MB | Bounded queue capacity |
| **Batch memory** | <50MB | Max batch size limit |
| **Total export memory** | <200MB | Multiple batch buffers |

## Implementation Phases

### Phase 1: Async Export Pipeline (Week 1)

**Deliverables**:
- AsyncSpanExporter implementation
- Non-blocking channel for span queuing
- Background export task

**Code**:
```rust
// crates/clnrm-core/src/telemetry/async_export.rs

pub struct AsyncSpanExporter {
    sender: mpsc::UnboundedSender<Vec<SpanData>>,
    background_task: JoinHandle<()>,
    metrics: Arc<ExportMetrics>,
}

impl AsyncSpanExporter {
    pub async fn new(config: ExportConfig) -> Result<Self> {
        let (sender, mut receiver) = mpsc::unbounded_channel();

        let background_task = tokio::spawn(async move {
            Self::export_loop(receiver, config).await;
        });

        Ok(Self {
            sender,
            background_task,
            metrics: Arc::new(ExportMetrics::new()),
        })
    }

    async fn export_loop(
        mut receiver: mpsc::UnboundedReceiver<Vec<SpanData>>,
        config: ExportConfig,
    ) {
        while let Some(batch) = receiver.recv().await {
            if let Err(e) = Self::export_batch(&batch, &config).await {
                warn!("Export failed: {}", e);
            }
        }
    }

    async fn export_batch(batch: &[SpanData], config: &ExportConfig) -> Result<()> {
        let start = Instant::now();

        // Send to OTLP endpoint
        let request = ExportTraceServiceRequest { spans: batch.to_vec() };
        config.http_client.post(&config.endpoint, request).await?;

        let elapsed = start.elapsed();
        debug!("Exported {} spans in {:.2}ms", batch.len(), elapsed.as_secs_f64() * 1000.0);

        Ok(())
    }
}
```

**Testing**:
- Async channel behavior
- Non-blocking exports
- Background task lifecycle

### Phase 2: Batch Tuning (Week 1)

**Deliverables**:
- Configurable batch sizes
- Environment variable support
- Metrics tracking

**Configuration**:
```rust
pub struct BatchConfig {
    size: usize,        // Per-batch span count
    timeout: Duration,  // Max time before flush
    queue_capacity: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            size: 512,
            timeout: Duration::from_secs(5),
            queue_capacity: 100_000,
        }
    }
}
```

### Phase 3: Adaptive Sampling (Week 2)

**Deliverables**:
- Adaptive sampler implementation
- Critical span pattern recognition
- Sampling rate adjustments

**Implementation**:
```rust
pub struct AdaptiveSampler {
    base_rate: f64,
    current_rate: Arc<AtomicU32>,
    span_counter: Arc<AtomicU64>,
    critical_patterns: Vec<String>,
}

impl AdaptiveSampler {
    pub fn should_sample(&self, span: &SpanData) -> bool {
        // Always preserve errors
        if is_error_span(span) || is_slow_span(span) {
            return true;
        }

        // Probabilistic sampling
        let random = rand::random::<f64>();
        let rate = self.current_rate.load(Ordering::Relaxed) as f64 / 100.0;
        random < rate
    }

    async fn adjust_rate(&self, queue_depth: usize, capacity: usize) {
        let utilization = queue_depth as f64 / capacity as f64;

        let new_rate = if utilization > 0.9 {
            // Overload: aggressive sampling
            0.1 // Drop 90%
        } else if utilization > 0.7 {
            // High load: moderate sampling
            0.5 // Drop 50%
        } else {
            // Normal load: no sampling
            1.0 // Keep all
        };

        self.current_rate.store((new_rate * 100.0) as u32, Ordering::Relaxed);
    }
}
```

### Phase 4: Benchmarking & Tuning (Week 2)

**Deliverables**:
- Performance benchmarks
- Tuning recommendations
- Documentation

**Benchmarks**:
```bash
# Benchmark span export throughput
cargo bench --bench otel_export -- --span-count 10000

# Profile memory usage
cargo bench --bench otel_memory -- --span-count 100000

# Measure latency percentiles
cargo bench --bench otel_latency -- --percentiles "p50,p95,p99"
```

## Success Criteria

### Performance
- ✅ <300ms for 10K spans (16% improvement)
- ✅ <1s for 100K spans
- ✅ P99 latency <100ms under 10K spans/sec load
- ✅ Memory bounded to <200MB regardless of load

### Functionality
- ✅ Zero span loss on error/slow paths
- ✅ Graceful degradation under overload
- ✅ Backward compatible with v1.5.0 APIs
- ✅ Configurable via environment variables

### Quality
- ✅ 100% test coverage (unit + integration)
- ✅ Zero clippy warnings
- ✅ Weaver validation passing
- ✅ No performance regressions on low-load scenarios

## Weaver Validation

### Telemetry Schema

```yaml
spans:
  - name: otel.export
    attributes:
      span_count: int          # Spans in batch
      batch_size: int          # Configured batch size
      export_duration_ms: float # Export latency
      dropped_spans: int       # Spans sampled out

metrics:
  - name: otel.spans.total
    type: counter
    labels:
      status: (success|dropped|error)

  - name: otel.batch.export_duration_ms
    type: histogram
    buckets: [1, 5, 10, 25, 50, 100, 250, 500, 1000]
```

### Live Validation

```bash
# Export 10K spans and validate telemetry
clnrm run tests/ --span-count 10000 --otel-exporter stdout

# Check: All spans accounted for (none lost except via sampling)
# Check: Export latency <300ms
# Check: Telemetry schema valid per Weaver
```

## Migration Guide (v1.5.0 → v1.6.0)

### No Changes Required

The optimization is transparent to users:

```rust
// v1.5.0 code works unchanged
#[cfg(feature = "otel")]
{
    let otel_config = OtelConfig::default();
    let _guard = init_otel(otel_config)?;

    // Just works with v1.6.0 optimizations
}
```

### Optional: Tuning for Peak Load

```rust
// For high-throughput scenarios
let otel_config = OtelConfig {
    batch_size: 2048,
    adaptive_sampling: true,
    max_sampling_rate: 0.05, // More aggressive at overload
    ..Default::default()
};
```

## Troubleshooting

### Symptom: High Memory Usage

**Cause**: Queue backing up (export slower than ingestion)

**Solution**:
1. Increase batch size: `OTEL_BSP_MAX_BATCH_SIZE=2048`
2. Enable sampling: `OTEL_TRACES_SAMPLER=adaptive`
3. Check endpoint connectivity

### Symptom: Missing Spans

**Cause**: Exceeded queue capacity

**Solution**:
1. Increase queue: `OTEL_BSP_MAX_QUEUE_SIZE=200000`
2. Use drop policy: `OTEL_DROP_POLICY=DropOldest`
3. Enable sampling to reduce ingestion rate

### Symptom: High Latency (>500ms)

**Cause**: Network issues or endpoint overload

**Solution**:
1. Check network: `curl -v <OTEL_EXPORTER_OTLP_ENDPOINT>`
2. Reduce batch size: `OTEL_BSP_MAX_BATCH_SIZE=256`
3. Increase export timeout: `OTEL_BSP_MAX_EXPORT_BATCH_TIMEOUT_MS=60000`

## References

- [OpenTelemetry Protocol Specification](https://opentelemetry.io/docs/specs/otel/protocol/)
- [Batch Processor Best Practices](https://opentelemetry.io/docs/specs/otel/protocol/exporter/)
- [Performance Testing Guide](./PERFORMANCE_TESTING.md)

---

**Version History**

| Version | Status | Notes |
|---------|--------|-------|
| **v1.6.0** | Design Complete | Implementation pending |
| v1.5.0 | Released | Baseline performance: 356ms for 10K spans |

**Last Updated**: 2025-11-18
**Target Release**: December 2025
