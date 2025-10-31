// Performance benchmarks for OpenTelemetry span collection
// Tests collection of 10,000+ spans with <50ms overhead target

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;

// Mock span structure (simulating OTel span)
#[derive(Clone, Debug)]
struct MockSpan {
    name: String,
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    attributes: HashMap<String, String>,
    start_time: u64,
    end_time: u64,
    status: String,
}

impl MockSpan {
    fn new(name: String, span_id: String) -> Self {
        Self {
            name,
            trace_id: format!("trace_{}", span_id),
            span_id: span_id.clone(),
            parent_span_id: None,
            attributes: HashMap::new(),
            start_time: 0,
            end_time: 0,
            status: "ok".to_string(),
        }
    }

    fn with_attributes(mut self, attrs: HashMap<String, String>) -> Self {
        self.attributes = attrs;
        self
    }

    fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_span_id = Some(parent_id);
        self
    }
}

// Mock span collector
struct MockSpanCollector {
    spans: Vec<MockSpan>,
}

impl MockSpanCollector {
    fn new() -> Self {
        Self { spans: Vec::new() }
    }

    fn collect(&mut self, span: MockSpan) {
        self.spans.push(span);
    }

    fn collect_batch(&mut self, spans: Vec<MockSpan>) {
        self.spans.extend(spans);
    }

    fn filter_by_name(&self, name: &str) -> Vec<&MockSpan> {
        self.spans.iter().filter(|s| s.name == name).collect()
    }

    fn filter_by_attribute(&self, key: &str, value: &str) -> Vec<&MockSpan> {
        self.spans
            .iter()
            .filter(|s| s.attributes.get(key).map(|v| v == value).unwrap_or(false))
            .collect()
    }

    fn count(&self) -> usize {
        self.spans.len()
    }

    fn clear(&mut self) {
        self.spans.clear();
    }
}

// Benchmark: Collect individual spans
fn bench_span_collection_individual(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_collection_individual");
    group.measurement_time(Duration::from_secs(15));

    for span_count in [100, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*span_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(span_count),
            span_count,
            |b, &count| {
                b.iter(|| {
                    let mut collector = MockSpanCollector::new();
                    for i in 0..count {
                        let span = MockSpan::new(
                            format!("clnrm.step.execute"),
                            format!("span_{}", i),
                        );
                        collector.collect(black_box(span));
                    }
                    black_box(collector.count())
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Batch span collection (TARGET: 10,000 spans)
fn bench_span_collection_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_collection_batch");
    group.measurement_time(Duration::from_secs(20));

    for span_count in [100, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*span_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(span_count),
            span_count,
            |b, &count| {
                let spans: Vec<_> = (0..count)
                    .map(|i| MockSpan::new(format!("clnrm.step.execute"), format!("span_{}", i)))
                    .collect();

                b.iter(|| {
                    let mut collector = MockSpanCollector::new();
                    collector.collect_batch(black_box(spans.clone()));
                    black_box(collector.count())
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Span collection with attributes
fn bench_span_collection_with_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_collection_with_attributes");
    group.measurement_time(Duration::from_secs(15));

    for attr_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(1000));

        group.bench_with_input(
            BenchmarkId::from_parameter(attr_count),
            attr_count,
            |b, &count| {
                b.iter(|| {
                    let mut collector = MockSpanCollector::new();
                    for i in 0..1000 {
                        let mut attrs = HashMap::new();
                        for j in 0..count {
                            attrs.insert(format!("attr_{}", j), format!("value_{}_{}", i, j));
                        }

                        let span = MockSpan::new(
                            format!("clnrm.step.execute"),
                            format!("span_{}", i),
                        )
                        .with_attributes(attrs);

                        collector.collect(black_box(span));
                    }
                    black_box(collector.count())
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Span filtering by name
fn bench_span_filter_by_name(c: &mut Criterion) {
    let mut collector = MockSpanCollector::new();

    // Populate with 10,000 spans (5 different names)
    for i in 0..10000 {
        let name = format!("clnrm.{}.execute", i % 5);
        let span = MockSpan::new(name, format!("span_{}", i));
        collector.collect(span);
    }

    c.bench_function("span_filter_by_name_10000_spans", |b| {
        b.iter(|| {
            let filtered = collector.filter_by_name(black_box("clnrm.2.execute"));
            black_box(filtered.len())
        })
    });
}

// Benchmark: Span filtering by attribute
fn bench_span_filter_by_attribute(c: &mut Criterion) {
    let mut collector = MockSpanCollector::new();

    // Populate with 10,000 spans with attributes
    for i in 0..10000 {
        let mut attrs = HashMap::new();
        attrs.insert("step.name".to_string(), format!("step_{}", i % 100));
        attrs.insert("step.service".to_string(), format!("service_{}", i % 10));

        let span = MockSpan::new(format!("clnrm.step.execute"), format!("span_{}", i))
            .with_attributes(attrs);
        collector.collect(span);
    }

    c.bench_function("span_filter_by_attribute_10000_spans", |b| {
        b.iter(|| {
            let filtered =
                collector.filter_by_attribute(black_box("step.service"), black_box("service_5"));
            black_box(filtered.len())
        })
    });
}

// Benchmark: Hierarchical span collection (parent-child relationships)
fn bench_span_collection_hierarchical(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_collection_hierarchical");
    group.measurement_time(Duration::from_secs(15));

    for depth in [2, 5, 10].iter() {
        group.throughput(Throughput::Elements(*depth as u64 * 100));

        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &d| {
            b.iter(|| {
                let mut collector = MockSpanCollector::new();

                // Create 100 parent spans, each with 'd' children
                for parent_id in 0..100 {
                    let parent = MockSpan::new(
                        format!("clnrm.test.execute"),
                        format!("parent_{}", parent_id),
                    );
                    collector.collect(parent);

                    for child_id in 0..d {
                        let child = MockSpan::new(
                            format!("clnrm.step.execute"),
                            format!("child_{}_{}", parent_id, child_id),
                        )
                        .with_parent(format!("parent_{}", parent_id));
                        collector.collect(black_box(child));
                    }
                }
                black_box(collector.count())
            })
        });
    }
    group.finish();
}

// Benchmark: Span collection memory usage
fn bench_span_collection_memory(c: &mut Criterion) {
    c.bench_function("span_collection_memory_10000_spans", |b| {
        b.iter(|| {
            let mut collector = MockSpanCollector::new();

            for i in 0..10000 {
                let mut attrs = HashMap::new();
                attrs.insert("step.name".to_string(), format!("step_{}", i));
                attrs.insert("step.service".to_string(), format!("service_{}", i % 10));
                attrs.insert("test.name".to_string(), format!("test_{}", i % 5));

                let span = MockSpan::new(format!("clnrm.step.execute"), format!("span_{}", i))
                    .with_attributes(attrs);
                collector.collect(span);
            }

            black_box(collector.count());
            // Explicit drop to measure cleanup
            drop(collector);
        })
    });
}

// Benchmark: Concurrent span collection
fn bench_span_collection_concurrent(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("span_collection_concurrent_10000_spans", |b| {
        b.to_async(&runtime).iter(|| async move {
            use std::sync::{Arc, Mutex};

            let collector = Arc::new(Mutex::new(MockSpanCollector::new()));
            let mut handles = Vec::new();

            // 10 concurrent tasks, each collecting 1000 spans
            for task_id in 0..10 {
                let collector_clone = Arc::clone(&collector);
                let handle = tokio::spawn(async move {
                    for i in 0..1000 {
                        let span = MockSpan::new(
                            format!("clnrm.step.execute"),
                            format!("span_{}_{}", task_id, i),
                        );
                        collector_clone.lock().unwrap().collect(span);
                    }
                });
                handles.push(handle);
            }

            futures::future::join_all(handles).await;
            black_box(collector.lock().unwrap().count())
        })
    });
}

// Benchmark: Span validation (checking against expectations)
fn bench_span_validation(c: &mut Criterion) {
    let mut collector = MockSpanCollector::new();

    // Populate with 1,000 spans
    for i in 0..1000 {
        let mut attrs = HashMap::new();
        attrs.insert("step.name".to_string(), format!("step_{}", i % 100));
        attrs.insert("step.status".to_string(), "ok".to_string());

        let span = MockSpan::new(format!("clnrm.step.execute"), format!("span_{}", i))
            .with_attributes(attrs);
        collector.collect(span);
    }

    c.bench_function("span_validation_100_expectations", |b| {
        b.iter(|| {
            let mut validation_count = 0;

            // Validate 100 expectations
            for i in 0..100 {
                let expected_name = "clnrm.step.execute";
                let expected_attr = format!("step_{}", i);

                let matches = collector
                    .filter_by_name(expected_name)
                    .into_iter()
                    .filter(|s| {
                        s.attributes
                            .get("step.name")
                            .map(|v| v == &expected_attr)
                            .unwrap_or(false)
                    })
                    .count();

                if matches > 0 {
                    validation_count += 1;
                }
            }

            black_box(validation_count)
        })
    });
}

criterion_group!(
    span_benches,
    bench_span_collection_individual,
    bench_span_collection_batch,
    bench_span_collection_with_attributes,
    bench_span_filter_by_name,
    bench_span_filter_by_attribute,
    bench_span_collection_hierarchical,
    bench_span_collection_memory,
    bench_span_collection_concurrent,
    bench_span_validation,
);

criterion_main!(span_benches);
