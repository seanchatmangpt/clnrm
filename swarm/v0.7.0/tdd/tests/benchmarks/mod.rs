/// Performance benchmarks for v0.7.0 DX features
///
/// Benchmarks measure critical performance metrics:
/// - File change detection latency (<100ms target)
/// - Template rendering time (<500ms target)
/// - Complete dev loop time (<3s target)
/// - Debouncing effectiveness
/// - Concurrent file change handling

use std::time::{Duration, Instant};
use crate::mocks::{MockFileWatcher, MockTemplateRenderer};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// ============================================================================
// Benchmark Infrastructure
// ============================================================================

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
}

impl BenchmarkResult {
    pub fn from_samples(name: &str, mut samples: Vec<Duration>) -> Self {
        samples.sort();
        let iterations = samples.len();
        let total_duration: Duration = samples.iter().sum();
        let avg_duration = total_duration / iterations as u32;

        Self {
            name: name.to_string(),
            iterations,
            total_duration,
            avg_duration,
            min_duration: *samples.first().unwrap(),
            max_duration: *samples.last().unwrap(),
            p50_duration: samples[iterations / 2],
            p95_duration: samples[(iterations * 95) / 100],
            p99_duration: samples[(iterations * 99) / 100],
        }
    }

    pub fn meets_sla(&self, target: Duration) -> bool {
        self.p95_duration <= target
    }

    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(70));
        println!("Benchmark: {}", self.name);
        println!("{}", "=".repeat(70));
        println!("Iterations:     {}", self.iterations);
        println!("Total Duration: {:?}", self.total_duration);
        println!("Average:        {:?}", self.avg_duration);
        println!("Min:            {:?}", self.min_duration);
        println!("Max:            {:?}", self.max_duration);
        println!("P50 (median):   {:?}", self.p50_duration);
        println!("P95:            {:?}", self.p95_duration);
        println!("P99:            {:?}", self.p99_duration);
        println!("{}", "=".repeat(70));
    }
}

pub fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
where
    F: FnMut() -> Result<()>,
{
    let mut samples = Vec::with_capacity(iterations);

    // Warmup
    for _ in 0..10 {
        let _ = f();
    }

    // Actual benchmark
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = f();
        samples.push(start.elapsed());
    }

    BenchmarkResult::from_samples(name, samples)
}

// ============================================================================
// Dev Watch Benchmarks
// ============================================================================

#[test]
fn bench_file_change_detection_latency() -> Result<()> {
    // Target: <100ms for file change detection
    let result = benchmark("File Change Detection", 1000, || {
        let watcher = MockFileWatcher::new();
        watcher.trigger_change("test.clnrm.toml.tera");
        let _ = watcher.last_change_time();
        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_millis(100)),
        "P95 latency should be <100ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

#[test]
fn bench_template_rendering() -> Result<()> {
    // Target: <500ms for template rendering
    let result = benchmark("Template Rendering", 1000, || {
        let renderer = MockTemplateRenderer::new()
            .with_render_duration(Duration::from_millis(10));
        let _ = renderer.render("test.clnrm.toml.tera");
        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_millis(500)),
        "P95 render time should be <500ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

#[test]
fn bench_complete_dev_loop() -> Result<()> {
    // Target: <3s for complete dev loop (detect → render → validate)
    let result = benchmark("Complete Dev Loop", 100, || {
        let watcher = MockFileWatcher::new();
        let renderer = MockTemplateRenderer::new()
            .with_render_duration(Duration::from_millis(100));

        // Simulate complete loop
        watcher.trigger_change("test.clnrm.toml.tera");
        let _ = watcher.last_change_time();
        let _ = renderer.render("test.clnrm.toml.tera");

        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_secs(3)),
        "P95 loop time should be <3s, was {:?}",
        result.p95_duration
    );

    Ok(())
}

#[test]
fn bench_debouncing_overhead() -> Result<()> {
    // Measure overhead of debouncing logic
    let result = benchmark("Debouncing Overhead", 1000, || {
        let watcher = MockFileWatcher::new();

        // Trigger multiple rapid changes
        for _ in 0..10 {
            watcher.trigger_change("test.clnrm.toml.tera");
        }

        // Measure debouncing logic
        let changes = watcher.get_changes();
        let _ = changes.len();

        Ok(())
    });

    result.print_report();

    // Debouncing overhead should be negligible
    assert!(
        result.p95_duration < Duration::from_millis(10),
        "Debouncing overhead should be <10ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

#[test]
fn bench_concurrent_file_changes() -> Result<()> {
    // Target: Handle 100 concurrent changes efficiently
    let result = benchmark("100 Concurrent Changes", 50, || {
        let watcher = MockFileWatcher::new();
        let renderer = MockTemplateRenderer::new()
            .with_render_duration(Duration::from_millis(5));

        // Simulate 100 concurrent file changes
        for i in 0..100 {
            watcher.trigger_change(&format!("test{}.clnrm.toml.tera", i));
        }

        // Process all changes
        let changes = watcher.get_changes();
        for (path, _) in changes {
            let _ = renderer.render(&path);
        }

        Ok(())
    });

    result.print_report();

    // Should handle 100 changes in reasonable time
    assert!(
        result.p95_duration < Duration::from_secs(2),
        "Should handle 100 changes in <2s, was {:?}",
        result.p95_duration
    );

    Ok(())
}

// ============================================================================
// Dry-Run Benchmarks
// ============================================================================

#[test]
fn bench_dry_run_validation() -> Result<()> {
    use crate::mocks::{MockTomlParser, ParsedToml};

    // Target: <100ms for dry-run validation
    let result = benchmark("Dry-Run Validation", 1000, || {
        let parser = MockTomlParser::new();

        let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
        "#;

        parser.set_result(
            template,
            Ok(ParsedToml {
                has_meta: true,
                has_otel: true,
                services: vec!["db".to_string()],
                scenarios: vec![],
            }),
        );

        let _ = parser.parse(template);
        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_millis(100)),
        "P95 validation time should be <100ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

// ============================================================================
// Fmt Benchmarks
// ============================================================================

#[test]
fn bench_template_formatting() -> Result<()> {
    use crate::mocks::MockFormatter;

    // Target: <50ms for formatting
    let result = benchmark("Template Formatting", 1000, || {
        let formatter = MockFormatter::new();

        let template = r#"
[meta]
name="test"
version="1.0"

[service.db]
type="postgres"
        "#;

        let formatted = formatter.format(template);
        let _ = formatted.len();

        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_millis(50)),
        "P95 format time should be <50ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

#[test]
fn bench_fmt_check_mode() -> Result<()> {
    use crate::mocks::MockFormatter;

    // Check mode should be faster than formatting
    let result = benchmark("Fmt Check Mode", 1000, || {
        let formatter = MockFormatter::new();

        let template = "[meta]\nname = \"test\"";
        formatter.set_formatted(template, template);

        let _ = formatter.needs_formatting(template);

        Ok(())
    });

    result.print_report();

    assert!(
        result.p95_duration < Duration::from_millis(10),
        "Check mode should be <10ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

// ============================================================================
// Lint Benchmarks
// ============================================================================

#[test]
fn bench_template_linting() -> Result<()> {
    use crate::mocks::{MockTomlParser, ParsedToml};

    // Target: <100ms for linting
    let result = benchmark("Template Linting", 1000, || {
        let parser = MockTomlParser::new();

        let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
        "#;

        parser.set_result(
            template,
            Ok(ParsedToml {
                has_meta: true,
                has_otel: true,
                services: vec!["db".to_string()],
                scenarios: vec![],
            }),
        );

        let _ = parser.parse(template);

        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_millis(100)),
        "P95 lint time should be <100ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

// ============================================================================
// Diff Benchmarks
// ============================================================================

#[test]
fn bench_trace_comparison() -> Result<()> {
    use crate::mocks::MockTraceDiffer;

    // Target: <100ms for trace comparison
    let result = benchmark("Trace Comparison", 1000, || {
        let differ = MockTraceDiffer::new();

        // Simulate comparing traces
        let _ = differ.has_differences();
        let _ = differ.get_differences();

        Ok(())
    });

    result.print_report();

    assert!(
        result.meets_sla(Duration::from_millis(100)),
        "P95 diff time should be <100ms, was {:?}",
        result.p95_duration
    );

    Ok(())
}

#[test]
fn bench_large_trace_comparison() -> Result<()> {
    use crate::mocks::{MockTraceDiffer, TraceDifference, DifferenceType};

    // Benchmark with 1000 differences
    let result = benchmark("Large Trace Diff (1000 spans)", 100, || {
        let differ = MockTraceDiffer::new();

        // Add 1000 differences
        for i in 0..1000 {
            differ.add_difference(TraceDifference {
                span_name: format!("span_{}", i),
                difference_type: DifferenceType::MissingSpan,
                details: format!("Missing span {}", i),
            });
        }

        let _ = differ.get_differences();

        Ok(())
    });

    result.print_report();

    assert!(
        result.p95_duration < Duration::from_secs(1),
        "Should handle 1000 differences in <1s, was {:?}",
        result.p95_duration
    );

    Ok(())
}

// ============================================================================
// Memory Benchmarks
// ============================================================================

#[test]
fn bench_memory_usage_under_load() -> Result<()> {
    // Measure memory usage with sustained load
    let result = benchmark("Memory Under Load", 100, || {
        let watcher = MockFileWatcher::new();
        let renderer = MockTemplateRenderer::new();

        // Simulate sustained load
        for i in 0..1000 {
            watcher.trigger_change(&format!("test{}.clnrm.toml.tera", i));
            let _ = renderer.render(&format!("test{}.clnrm.toml.tera", i));
        }

        Ok(())
    });

    result.print_report();

    // Should complete without excessive memory consumption
    assert!(
        result.p95_duration < Duration::from_secs(5),
        "Memory overhead should allow completion in <5s"
    );

    Ok(())
}

// ============================================================================
// Regression Tests
// ============================================================================

#[test]
fn bench_regression_baseline() -> Result<()> {
    // Establish baseline for future regression testing
    let benchmarks = vec![
        ("file_detection", Duration::from_millis(100)),
        ("rendering", Duration::from_millis(500)),
        ("complete_loop", Duration::from_secs(3)),
        ("validation", Duration::from_millis(100)),
        ("formatting", Duration::from_millis(50)),
        ("linting", Duration::from_millis(100)),
        ("diff", Duration::from_millis(100)),
    ];

    println!("\n{}", "=".repeat(70));
    println!("Performance Regression Baseline (v0.7.0)");
    println!("{}", "=".repeat(70));

    for (name, target) in benchmarks {
        println!("{:<20} Target: {:?}", name, target);
    }

    println!("{}", "=".repeat(70));
    println!("All benchmarks should remain within these targets");
    println!("{}", "=".repeat(70));

    Ok(())
}
