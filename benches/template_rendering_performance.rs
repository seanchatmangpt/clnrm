// Performance benchmarks for template rendering and variable substitution
// Tests rendering of complex templates with nested variables

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;

// Mock template renderer (simulating Handlebars/Tera)
struct MockTemplateRenderer {
    templates: HashMap<String, String>,
    context: HashMap<String, String>,
}

impl MockTemplateRenderer {
    fn new() -> Self {
        Self {
            templates: HashMap::new(),
            context: HashMap::new(),
        }
    }

    fn register_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }

    fn set_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }

    fn render(&self, template_name: &str) -> Result<String, String> {
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| "Template not found".to_string())?;

        let mut result = template.clone();

        // Simple variable substitution: {{var_name}}
        for (key, value) in &self.context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    fn render_with_context(
        &self,
        template_name: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, String> {
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| "Template not found".to_string())?;

        let mut result = template.clone();

        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

// Generate complex template
fn generate_complex_template(var_count: usize) -> String {
    let mut template = String::from(r#"
# Test Report: {{test_name}}

## Test Metadata
- **Test ID**: {{test_id}}
- **Environment**: {{environment}}
- **Start Time**: {{start_time}}
- **End Time**: {{end_time}}
- **Duration**: {{duration_ms}}ms
- **Status**: {{status}}

## Services
"#);

    for i in 0..std::cmp::min(var_count / 10, 20) {
        template.push_str(&format!(
            r#"
### Service {}
- **Name**: {{{{service_{}_name}}}}
- **Image**: {{{{service_{}_image}}}}
- **Status**: {{{{service_{}_status}}}}
- **Port**: {{{{service_{}_port}}}}
"#,
            i, i, i, i, i
        ));
    }

    template.push_str("\n## Test Steps\n");

    for i in 0..std::cmp::min(var_count / 5, 50) {
        template.push_str(&format!(
            r#"
### Step {}: {{{{step_{}_name}}}}
- **Command**: {{{{step_{}_command}}}}
- **Exit Code**: {{{{step_{}_exit_code}}}}
- **Duration**: {{{{step_{}_duration}}}}ms
- **Output**: {{{{step_{}_output}}}}
"#,
            i, i, i, i, i, i
        ));
    }

    template.push_str(
        r#"
## Assertions
- **Total Steps**: {{total_steps}}
- **Passed Steps**: {{passed_steps}}
- **Failed Steps**: {{failed_steps}}
- **Success Rate**: {{success_rate}}%

## OpenTelemetry Metrics
- **Total Spans**: {{total_spans}}
- **Trace ID**: {{trace_id}}
- **OTEL Overhead**: {{otel_overhead_ms}}ms

## Resource Usage
- **Peak Memory**: {{peak_memory_mb}}MB
- **CPU Usage**: {{cpu_usage_percent}}%
- **Network I/O**: {{network_io_mb}}MB

## Conclusion
{{conclusion}}
"#,
    );

    template
}

// Generate template context
fn generate_template_context(var_count: usize) -> HashMap<String, String> {
    let mut context = HashMap::new();

    context.insert("test_name".to_string(), "Performance Test".to_string());
    context.insert("test_id".to_string(), "test_12345".to_string());
    context.insert("environment".to_string(), "production".to_string());
    context.insert(
        "start_time".to_string(),
        "2025-10-30T10:00:00Z".to_string(),
    );
    context.insert("end_time".to_string(), "2025-10-30T10:05:00Z".to_string());
    context.insert("duration_ms".to_string(), "300000".to_string());
    context.insert("status".to_string(), "PASSED".to_string());

    for i in 0..std::cmp::min(var_count / 10, 20) {
        context.insert(format!("service_{}_name", i), format!("service_{}", i));
        context.insert(format!("service_{}_image", i), format!("alpine:latest"));
        context.insert(format!("service_{}_status", i), format!("running"));
        context.insert(format!("service_{}_port", i), format!("{}", 8000 + i));
    }

    for i in 0..std::cmp::min(var_count / 5, 50) {
        context.insert(format!("step_{}_name", i), format!("Step {}", i));
        context.insert(
            format!("step_{}_command", i),
            format!("echo 'step {}'", i),
        );
        context.insert(format!("step_{}_exit_code", i), format!("0"));
        context.insert(format!("step_{}_duration", i), format!("{}", 100 + i * 10));
        context.insert(format!("step_{}_output", i), format!("Step {} output", i));
    }

    context.insert("total_steps".to_string(), "50".to_string());
    context.insert("passed_steps".to_string(), "48".to_string());
    context.insert("failed_steps".to_string(), "2".to_string());
    context.insert("success_rate".to_string(), "96.0".to_string());
    context.insert("total_spans".to_string(), "500".to_string());
    context.insert(
        "trace_id".to_string(),
        "abc123def456".to_string(),
    );
    context.insert("otel_overhead_ms".to_string(), "45".to_string());
    context.insert("peak_memory_mb".to_string(), "384".to_string());
    context.insert("cpu_usage_percent".to_string(), "45.2".to_string());
    context.insert("network_io_mb".to_string(), "12.5".to_string());
    context.insert(
        "conclusion".to_string(),
        "Test completed successfully with 96% success rate.".to_string(),
    );

    context
}

// Benchmark: Simple template rendering
fn bench_template_render_simple(c: &mut Criterion) {
    let mut renderer = MockTemplateRenderer::new();
    renderer.register_template(
        "simple".to_string(),
        "Hello {{name}}, your test {{test_id}} {{status}}!".to_string(),
    );

    let mut context = HashMap::new();
    context.insert("name".to_string(), "User".to_string());
    context.insert("test_id".to_string(), "12345".to_string());
    context.insert("status".to_string(), "PASSED".to_string());

    c.bench_function("template_render_simple", |b| {
        b.iter(|| {
            let result = renderer.render_with_context(black_box("simple"), black_box(&context));
            black_box(result)
        })
    });
}

// Benchmark: Complex template rendering with varying variable counts
fn bench_template_render_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_render_complex");
    group.measurement_time(Duration::from_secs(15));

    for var_count in [10, 50, 100, 200, 500].iter() {
        let template = generate_complex_template(*var_count);
        let context = generate_template_context(*var_count);

        let mut renderer = MockTemplateRenderer::new();
        renderer.register_template("complex".to_string(), template);

        group.throughput(Throughput::Elements(*var_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(var_count),
            var_count,
            |b, _| {
                b.iter(|| {
                    let result =
                        renderer.render_with_context(black_box("complex"), black_box(&context));
                    black_box(result)
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Template caching and reuse
fn bench_template_caching(c: &mut Criterion) {
    let template = generate_complex_template(100);
    let context = generate_template_context(100);

    let mut renderer = MockTemplateRenderer::new();
    renderer.register_template("cached".to_string(), template);

    c.bench_function("template_render_cached_100_renders", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let result =
                    renderer.render_with_context(black_box("cached"), black_box(&context));
                black_box(result).unwrap();
            }
        })
    });
}

// Benchmark: Multiple template registration
fn bench_template_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_registration");

    for template_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(template_count),
            template_count,
            |b, &count| {
                b.iter(|| {
                    let mut renderer = MockTemplateRenderer::new();
                    for i in 0..count {
                        let template = format!("Template {{{{var_{}}}}} content", i);
                        renderer.register_template(format!("template_{}", i), template);
                    }
                    black_box(renderer.templates.len())
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Nested variable substitution
fn bench_template_nested_vars(c: &mut Criterion) {
    let template = r#"
{{parent_1}}: {{child_1_1}}, {{child_1_2}}, {{child_1_3}}
{{parent_2}}: {{child_2_1}}, {{child_2_2}}, {{child_2_3}}
{{parent_3}}: {{child_3_1}}, {{child_3_2}}, {{child_3_3}}
"#;

    let mut context = HashMap::new();
    for parent in 1..=3 {
        context.insert(format!("parent_{}", parent), format!("Parent {}", parent));
        for child in 1..=3 {
            context.insert(
                format!("child_{}_{}", parent, child),
                format!("Child {}.{}", parent, child),
            );
        }
    }

    let mut renderer = MockTemplateRenderer::new();
    renderer.register_template("nested".to_string(), template.to_string());

    c.bench_function("template_render_nested_vars", |b| {
        b.iter(|| {
            let result = renderer.render_with_context(black_box("nested"), black_box(&context));
            black_box(result)
        })
    });
}

// Benchmark: Template rendering with missing variables
fn bench_template_missing_vars(c: &mut Criterion) {
    let template = generate_complex_template(100);
    let mut context = generate_template_context(50); // Only half the variables

    let mut renderer = MockTemplateRenderer::new();
    renderer.register_template("incomplete".to_string(), template);

    c.bench_function("template_render_missing_vars", |b| {
        b.iter(|| {
            let result =
                renderer.render_with_context(black_box("incomplete"), black_box(&context));
            black_box(result)
        })
    });
}

// Benchmark: Concurrent template rendering
fn bench_template_concurrent(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("template_render_concurrent_10_threads", |b| {
        b.to_async(&runtime).iter(|| async move {
            use std::sync::Arc;

            let template = generate_complex_template(100);
            let context = Arc::new(generate_template_context(100));

            let mut renderer = MockTemplateRenderer::new();
            renderer.register_template("concurrent".to_string(), template);
            let renderer = Arc::new(renderer);

            let mut handles = Vec::new();

            for _ in 0..10 {
                let renderer_clone = Arc::clone(&renderer);
                let context_clone = Arc::clone(&context);

                let handle = tokio::spawn(async move {
                    for _ in 0..10 {
                        let _ = renderer_clone.render_with_context("concurrent", &context_clone);
                    }
                });
                handles.push(handle);
            }

            futures::future::join_all(handles).await;
        })
    });
}

// Benchmark: Template rendering memory usage
fn bench_template_memory(c: &mut Criterion) {
    c.bench_function("template_render_memory_large", |b| {
        b.iter(|| {
            let template = generate_complex_template(1000);
            let context = generate_template_context(1000);

            let mut renderer = MockTemplateRenderer::new();
            renderer.register_template("large".to_string(), template);

            let result = renderer.render_with_context("large", &context);
            black_box(result).unwrap();

            // Explicit cleanup
            drop(renderer);
        })
    });
}

criterion_group!(
    template_benches,
    bench_template_render_simple,
    bench_template_render_complex,
    bench_template_caching,
    bench_template_registration,
    bench_template_nested_vars,
    bench_template_missing_vars,
    bench_template_concurrent,
    bench_template_memory,
);

criterion_main!(template_benches);
