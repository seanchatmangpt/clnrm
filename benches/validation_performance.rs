// Performance benchmarks for Weaver validation and span expectation checking
// Tests validation of 100+ span expectations with <50ms target

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;

// Mock span expectation (simulating Weaver schema)
#[derive(Clone, Debug)]
struct SpanExpectation {
    name: String,
    attributes: HashMap<String, String>,
    status: String,
    min_occurrences: usize,
    max_occurrences: Option<usize>,
}

impl SpanExpectation {
    fn new(name: String) -> Self {
        Self {
            name,
            attributes: HashMap::new(),
            status: "ok".to_string(),
            min_occurrences: 1,
            max_occurrences: None,
        }
    }

    fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    fn with_status(mut self, status: String) -> Self {
        self.status = status;
        self
    }

    fn with_occurrences(mut self, min: usize, max: Option<usize>) -> Self {
        self.min_occurrences = min;
        self.max_occurrences = max;
        self
    }
}

// Mock collected span
#[derive(Clone, Debug)]
struct CollectedSpan {
    name: String,
    attributes: HashMap<String, String>,
    status: String,
}

impl CollectedSpan {
    fn new(name: String) -> Self {
        Self {
            name,
            attributes: HashMap::new(),
            status: "ok".to_string(),
        }
    }

    fn with_attributes(mut self, attrs: HashMap<String, String>) -> Self {
        self.attributes = attrs;
        self
    }
}

// Mock validator
struct SpanValidator {
    expectations: Vec<SpanExpectation>,
    collected_spans: Vec<CollectedSpan>,
}

impl SpanValidator {
    fn new() -> Self {
        Self {
            expectations: Vec::new(),
            collected_spans: Vec::new(),
        }
    }

    fn add_expectation(&mut self, expectation: SpanExpectation) {
        self.expectations.push(expectation);
    }

    fn add_span(&mut self, span: CollectedSpan) {
        self.collected_spans.push(span);
    }

    fn validate(&self) -> ValidationResult {
        let mut passed = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        for expectation in &self.expectations {
            let matches = self.find_matching_spans(expectation);

            let count = matches.len();
            let expected_min = expectation.min_occurrences;
            let expected_max = expectation.max_occurrences.unwrap_or(usize::MAX);

            if count >= expected_min && count <= expected_max {
                passed += 1;
            } else {
                failed += 1;
                failures.push(format!(
                    "Expected {}-{} occurrences of '{}', found {}",
                    expected_min, expected_max, expectation.name, count
                ));
            }
        }

        ValidationResult {
            total: self.expectations.len(),
            passed,
            failed,
            failures,
        }
    }

    fn find_matching_spans(&self, expectation: &SpanExpectation) -> Vec<&CollectedSpan> {
        self.collected_spans
            .iter()
            .filter(|span| self.matches_expectation(span, expectation))
            .collect()
    }

    fn matches_expectation(&self, span: &CollectedSpan, expectation: &SpanExpectation) -> bool {
        // Check name
        if span.name != expectation.name {
            return false;
        }

        // Check status
        if span.status != expectation.status {
            return false;
        }

        // Check attributes
        for (key, value) in &expectation.attributes {
            if span.attributes.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

struct ValidationResult {
    total: usize,
    passed: usize,
    failed: usize,
    failures: Vec<String>,
}

// Benchmark: Validate simple expectations
fn bench_validation_simple(c: &mut Criterion) {
    let mut validator = SpanValidator::new();

    // Add 10 expectations
    for i in 0..10 {
        let expectation =
            SpanExpectation::new(format!("clnrm.step.execute")).with_attribute(
                "step.name".to_string(),
                format!("step_{}", i),
            );
        validator.add_expectation(expectation);
    }

    // Add matching spans
    for i in 0..10 {
        let mut attrs = HashMap::new();
        attrs.insert("step.name".to_string(), format!("step_{}", i));
        let span = CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
        validator.add_span(span);
    }

    c.bench_function("validation_simple_10_expectations", |b| {
        b.iter(|| {
            let result = validator.validate();
            black_box(result.passed)
        })
    });
}

// Benchmark: Validate with varying expectation counts (TARGET: 100 expectations)
fn bench_validation_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_complex");
    group.measurement_time(Duration::from_secs(20));

    for expectation_count in [10, 50, 100, 200, 500].iter() {
        group.throughput(Throughput::Elements(*expectation_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(expectation_count),
            expectation_count,
            |b, &count| {
                let mut validator = SpanValidator::new();

                // Add expectations
                for i in 0..count {
                    let mut expectation = SpanExpectation::new(format!("clnrm.step.execute"));
                    expectation = expectation.with_attribute(
                        "step.name".to_string(),
                        format!("step_{}", i),
                    );
                    expectation = expectation.with_attribute(
                        "step.service".to_string(),
                        format!("service_{}", i % 10),
                    );
                    validator.add_expectation(expectation);
                }

                // Add matching spans (2x to test occurrence counting)
                for i in 0..count * 2 {
                    let mut attrs = HashMap::new();
                    attrs.insert("step.name".to_string(), format!("step_{}", i % count));
                    attrs.insert(
                        "step.service".to_string(),
                        format!("service_{}", (i % count) % 10),
                    );
                    let span =
                        CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
                    validator.add_span(span);
                }

                b.iter(|| {
                    let result = validator.validate();
                    black_box(result.passed)
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Validation with attribute matching
fn bench_validation_attribute_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_attribute_matching");
    group.measurement_time(Duration::from_secs(15));

    for attr_count in [1, 3, 5, 10].iter() {
        group.throughput(Throughput::Elements(100));

        group.bench_with_input(
            BenchmarkId::from_parameter(attr_count),
            attr_count,
            |b, &count| {
                let mut validator = SpanValidator::new();

                // Add expectations with multiple attributes
                for i in 0..100 {
                    let mut expectation = SpanExpectation::new(format!("clnrm.step.execute"));
                    for j in 0..count {
                        expectation = expectation.with_attribute(
                            format!("attr_{}", j),
                            format!("value_{}_{}", i, j),
                        );
                    }
                    validator.add_expectation(expectation);
                }

                // Add matching spans
                for i in 0..100 {
                    let mut attrs = HashMap::new();
                    for j in 0..count {
                        attrs.insert(format!("attr_{}", j), format!("value_{}_{}", i, j));
                    }
                    let span =
                        CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
                    validator.add_span(span);
                }

                b.iter(|| {
                    let result = validator.validate();
                    black_box(result.passed)
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Validation with large span collection
fn bench_validation_large_span_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_large_span_collection");
    group.measurement_time(Duration::from_secs(15));

    for span_count in [100, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*span_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(span_count),
            span_count,
            |b, &count| {
                let mut validator = SpanValidator::new();

                // Add 100 expectations
                for i in 0..100 {
                    let expectation =
                        SpanExpectation::new(format!("clnrm.step.execute")).with_attribute(
                            "step.name".to_string(),
                            format!("step_{}", i),
                        );
                    validator.add_expectation(expectation);
                }

                // Add large span collection
                for i in 0..count {
                    let mut attrs = HashMap::new();
                    attrs.insert("step.name".to_string(), format!("step_{}", i % 100));
                    let span =
                        CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
                    validator.add_span(span);
                }

                b.iter(|| {
                    let result = validator.validate();
                    black_box(result.passed)
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Validation with occurrence counting
fn bench_validation_occurrence_counting(c: &mut Criterion) {
    let mut validator = SpanValidator::new();

    // Expectation: step_0 should appear 5-10 times
    let expectation = SpanExpectation::new(format!("clnrm.step.execute"))
        .with_attribute("step.name".to_string(), format!("step_0"))
        .with_occurrences(5, Some(10));
    validator.add_expectation(expectation);

    // Add 8 matching spans (within range)
    for _ in 0..8 {
        let mut attrs = HashMap::new();
        attrs.insert("step.name".to_string(), format!("step_0"));
        let span = CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
        validator.add_span(span);
    }

    c.bench_function("validation_occurrence_counting", |b| {
        b.iter(|| {
            let result = validator.validate();
            black_box(result.passed)
        })
    });
}

// Benchmark: Validation with regex pattern matching (simulated)
fn bench_validation_pattern_matching(c: &mut Criterion) {
    let mut validator = SpanValidator::new();

    // Add 50 expectations with pattern-like attributes
    for i in 0..50 {
        let expectation = SpanExpectation::new(format!("clnrm.step.execute")).with_attribute(
            "step.name".to_string(),
            format!("step_{}", i),
        );
        validator.add_expectation(expectation);
    }

    // Add spans with various attribute values
    for i in 0..200 {
        let mut attrs = HashMap::new();
        attrs.insert("step.name".to_string(), format!("step_{}", i % 50));
        attrs.insert("step.extra".to_string(), format!("extra_{}", i));
        let span = CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
        validator.add_span(span);
    }

    c.bench_function("validation_pattern_matching_50_expectations", |b| {
        b.iter(|| {
            let result = validator.validate();
            black_box(result.passed)
        })
    });
}

// Benchmark: Validation failure detection
fn bench_validation_failure_detection(c: &mut Criterion) {
    let mut validator = SpanValidator::new();

    // Add expectations that WON'T match
    for i in 0..100 {
        let expectation = SpanExpectation::new(format!("clnrm.step.execute")).with_attribute(
            "step.name".to_string(),
            format!("expected_step_{}", i),
        );
        validator.add_expectation(expectation);
    }

    // Add spans that DON'T match expectations
    for i in 0..100 {
        let mut attrs = HashMap::new();
        attrs.insert("step.name".to_string(), format!("actual_step_{}", i));
        let span = CollectedSpan::new(format!("clnrm.step.execute")).with_attributes(attrs);
        validator.add_span(span);
    }

    c.bench_function("validation_failure_detection_100_failures", |b| {
        b.iter(|| {
            let result = validator.validate();
            black_box(result.failed)
        })
    });
}

// Benchmark: Concurrent validation
fn bench_validation_concurrent(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("validation_concurrent_10_validators", |b| {
        b.to_async(&runtime).iter(|| async move {
            use std::sync::Arc;

            let mut handles = Vec::new();

            for _ in 0..10 {
                let handle = tokio::spawn(async move {
                    let mut validator = SpanValidator::new();

                    // Add 100 expectations
                    for i in 0..100 {
                        let expectation = SpanExpectation::new(format!("clnrm.step.execute"))
                            .with_attribute("step.name".to_string(), format!("step_{}", i));
                        validator.add_expectation(expectation);
                    }

                    // Add matching spans
                    for i in 0..100 {
                        let mut attrs = HashMap::new();
                        attrs.insert("step.name".to_string(), format!("step_{}", i));
                        let span = CollectedSpan::new(format!("clnrm.step.execute"))
                            .with_attributes(attrs);
                        validator.add_span(span);
                    }

                    validator.validate().passed
                });
                handles.push(handle);
            }

            let results = futures::future::join_all(handles).await;
            black_box(results)
        })
    });
}

criterion_group!(
    validation_benches,
    bench_validation_simple,
    bench_validation_complex,
    bench_validation_attribute_matching,
    bench_validation_large_span_collection,
    bench_validation_occurrence_counting,
    bench_validation_pattern_matching,
    bench_validation_failure_detection,
    bench_validation_concurrent,
);

criterion_main!(validation_benches);
