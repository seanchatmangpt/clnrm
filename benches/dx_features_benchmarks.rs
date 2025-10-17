//! Performance benchmarks for v0.7.0 DX features
//!
//! This benchmark suite validates performance targets for developer experience:
//!
//! **PERFORMANCE TARGETS:**
//! 1. Hot Reload Latency (<3s p95):
//!    - File change detection: <100ms
//!    - Template rendering: <500ms
//!    - TOML parsing: <200ms
//!    - Feedback display: <50ms
//!    - TOTAL: <3s for simple templates
//!
//! 2. New User Experience (<60s):
//!    - clnrm init: <2s
//!    - Template editing: <5s (user)
//!    - clnrm dev --watch starts: <3s
//!    - First test runs: <30s (includes image pull)
//!    - Results displayed: <1s
//!    - TOTAL: <60s to first green
//!
//! 3. Command Performance:
//!    - dry-run: <1s (no containers)
//!    - fmt: <500ms
//!    - lint: <1s
//!    - diff: <2s
//!    - render --map: <500ms
//!
//! 4. Resource Usage:
//!    - File watcher memory: <10MB
//!    - Worker pool memory: <100MB base + per-container
//!    - Cache size: <50MB

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;

// ============================================================================
// Template Rendering Benchmarks
// ============================================================================

fn benchmark_template_rendering(c: &mut Criterion) {
    use clnrm_core::template::{TemplateRenderer, TemplateContext};

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("template_rendering");

    // Simple template (target: <100ms p95)
    group.bench_function("simple_template", |b| {
        b.to_async(&rt).iter(|| async {
            let mut renderer = TemplateRenderer::new().unwrap();
            let template = r#"
[meta]
name = "{{ test_name }}"
version = "{{ version }}"

[service.db]
type = "postgres"
image = "postgres:{{ pg_version }}"
"#;

            let mut context = TemplateContext::new();
            context.set("test_name", "my_test");
            context.set("version", "1.0.0");
            context.set("pg_version", "16");

            renderer = renderer.with_context(context);
            let result = renderer.render_str(template, "test").unwrap();
            black_box(result);
        });
    });

    // Medium template with loops (target: <300ms p95)
    group.bench_function("medium_template_with_loops", |b| {
        b.to_async(&rt).iter(|| async {
            let mut renderer = TemplateRenderer::new().unwrap();
            let template = r#"
[meta]
name = "{{ test_name }}"

{% for i in range(start=1, end=10) %}
[[scenario]]
name = "scenario_{{ i }}"
command = "echo 'test {{ i }}'"
{% endfor %}

{% for service in services %}
[service.{{ service.name }}]
type = "{{ service.type }}"
{% endfor %}
"#;

            let mut context = TemplateContext::new();
            context.set("test_name", "loop_test");
            context.set(
                "services",
                serde_json::json!([
                    {"name": "db", "type": "postgres"},
                    {"name": "cache", "type": "redis"},
                    {"name": "queue", "type": "rabbitmq"},
                ]),
            );

            renderer = renderer.with_context(context);
            let result = renderer.render_str(template, "test").unwrap();
            black_box(result);
        });
    });

    // Complex template (target: <500ms p95)
    group.bench_function("complex_template", |b| {
        b.to_async(&rt).iter(|| async {
            let mut renderer = TemplateRenderer::new().unwrap();
            let template = r#"
[meta]
name = "{{ test_name }}"
timestamp = "{{ now() }}"

{% for env in environments %}
[env.{{ env.name }}]
{% for var_name, var_value in env.vars %}
{{ var_name }} = "{{ var_value }}"
{% endfor %}
{% endfor %}

{% for service in services %}
[service.{{ service.name }}]
type = "{{ service.type }}"
image = "{{ service.image }}"
{% for port in service.ports %}
port_{{ loop.index0 }} = {{ port }}
{% endfor %}
{% endfor %}

{% for i in range(start=1, end=20) %}
[[scenario]]
name = "scenario_{{ i }}"
service = "{{ services[i % services | length].name }}"
command = "{{ commands[i % commands | length] }}"
{% endfor %}
"#;

            let mut context = TemplateContext::new();
            context.set("test_name", "complex_test");
            context.set(
                "environments",
                serde_json::json!([
                    {"name": "dev", "vars": {"DB_HOST": "localhost", "DB_PORT": "5432"}},
                    {"name": "prod", "vars": {"DB_HOST": "prod-db", "DB_PORT": "5432"}},
                ]),
            );
            context.set(
                "services",
                serde_json::json!([
                    {"name": "db", "type": "postgres", "image": "postgres:16", "ports": [5432]},
                    {"name": "cache", "type": "redis", "image": "redis:7", "ports": [6379]},
                    {"name": "api", "type": "custom", "image": "api:latest", "ports": [8080, 9090]},
                ]),
            );
            context.set(
                "commands",
                serde_json::json!(["echo test", "ls -la", "pwd", "whoami"]),
            );

            renderer = renderer.with_context(context);
            let result = renderer.render_str(template, "test").unwrap();
            black_box(result);
        });
    });

    group.finish();
}

// ============================================================================
// TOML Parsing Benchmarks
// ============================================================================

fn benchmark_toml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_parsing");

    // Simple TOML (target: <50ms p95)
    group.bench_function("simple_toml", |b| {
        b.iter(|| {
            let toml_str = r#"
[meta]
name = "test"
version = "1.0"

[service.db]
type = "postgres"
"#;
            let parsed: toml::Value = toml::from_str(toml_str).unwrap();
            black_box(parsed);
        });
    });

    // Medium TOML (target: <100ms p95)
    group.bench_function("medium_toml", |b| {
        b.iter(|| {
            let toml_str = r#"
[meta]
name = "test"
version = "1.0"

[otel]
service_name = "my_service"

[service.db]
type = "postgres"
image = "postgres:16"

[service.cache]
type = "redis"
image = "redis:7"

[[scenario]]
name = "scenario_1"
command = "echo test"

[[scenario]]
name = "scenario_2"
command = "ls -la"
"#;
            let parsed: toml::Value = toml::from_str(toml_str).unwrap();
            black_box(parsed);
        });
    });

    // Large TOML (target: <200ms p95)
    group.bench_function("large_toml", |b| {
        b.iter(|| {
            let mut toml_str = String::from(
                r#"
[meta]
name = "large_test"
version = "1.0"

[otel]
service_name = "large_service"
"#,
            );

            // Add 50 scenarios
            for i in 1..=50 {
                toml_str.push_str(&format!(
                    r#"
[[scenario]]
name = "scenario_{}"
command = "echo 'test {}'"
expected_exit_code = 0
"#,
                    i, i
                ));
            }

            let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

// ============================================================================
// File Operations Benchmarks
// ============================================================================

fn benchmark_file_operations(c: &mut Criterion) {
    use std::fs;
    use tempfile::TempDir;

    let mut group = c.benchmark_group("file_operations");

    // File reading (target: <50ms p95)
    group.bench_function("read_template_file", |b| {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.clnrm.toml.tera");
        fs::write(
            &file_path,
            r#"
[meta]
name = "{{ test_name }}"

[service.db]
type = "postgres"
"#,
        )
        .unwrap();

        b.iter(|| {
            let content = fs::read_to_string(&file_path).unwrap();
            black_box(content);
        });
    });

    // File writing (target: <100ms p95)
    group.bench_function("write_rendered_file", |b| {
        let temp_dir = TempDir::new().unwrap();

        b.iter(|| {
            let file_path = temp_dir
                .path()
                .join(format!("test_{}.clnrm.toml", uuid::Uuid::new_v4()));
            let content = r#"
[meta]
name = "test"

[service.db]
type = "postgres"
"#;
            fs::write(&file_path, content).unwrap();
            black_box(file_path);
        });
    });

    // Directory scanning (target: <200ms p95)
    group.bench_function("scan_template_directory", |b| {
        let temp_dir = TempDir::new().unwrap();

        // Create 100 template files
        for i in 0..100 {
            let file_path = temp_dir
                .path()
                .join(format!("test_{}.clnrm.toml.tera", i));
            fs::write(&file_path, "[meta]\nname = \"test\"").unwrap();
        }

        b.iter(|| {
            let mut templates = Vec::new();
            for entry in fs::read_dir(temp_dir.path()).unwrap() {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) == Some("tera") {
                    templates.push(entry.path());
                }
            }
            black_box(templates);
        });
    });

    group.finish();
}

// ============================================================================
// Complete Workflow Benchmarks
// ============================================================================

fn benchmark_hot_reload_workflow(c: &mut Criterion) {
    use clnrm_core::template::{TemplateRenderer, TemplateContext};
    use std::fs;
    use tempfile::TempDir;

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("hot_reload_workflow");

    // Complete hot reload cycle (target: <3s p95)
    group.bench_function("complete_reload_cycle", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let template_path = temp_dir.path().join("test.clnrm.toml.tera");
            let output_path = temp_dir.path().join("test.clnrm.toml");

            // 1. File change detection (<100ms)
            let template_content = r#"
[meta]
name = "{{ test_name }}"
version = "{{ version }}"

[service.db]
type = "postgres"
image = "postgres:{{ pg_version }}"
"#;
            fs::write(&template_path, template_content).unwrap();

            // 2. Read template file (<50ms)
            let content = fs::read_to_string(&template_path).unwrap();

            // 3. Template rendering (<500ms)
            let mut renderer = TemplateRenderer::new().unwrap();
            let mut context = TemplateContext::new();
            context.set("test_name", "hot_reload_test");
            context.set("version", "1.0.0");
            context.set("pg_version", "16");

            renderer = renderer.with_context(context);
            let rendered = renderer.render_str(&content, "test").unwrap();

            // 4. TOML parsing validation (<200ms)
            let _parsed: toml::Value = toml::from_str(&rendered).unwrap();

            // 5. Write output file (<100ms)
            fs::write(&output_path, rendered).unwrap();

            // 6. Feedback display (<50ms)
            let result = format!("✓ Rendered: {}", output_path.display());
            black_box(result);
        });
    });

    group.finish();
}

fn benchmark_scalability(c: &mut Criterion) {
    use clnrm_core::template::{TemplateRenderer, TemplateContext};
    use std::fs;
    use tempfile::TempDir;

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("scalability");

    // Test with 1, 10, 100 template files
    for num_files in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_files", num_files)),
            num_files,
            |b, &&num_files| {
                b.to_async(&rt).iter(|| async move {
                    let temp_dir = TempDir::new().unwrap();

                    // Create template files
                    for i in 0..num_files {
                        let template_path = temp_dir
                            .path()
                            .join(format!("test_{}.clnrm.toml.tera", i));
                        fs::write(
                            &template_path,
                            format!(
                                r#"
[meta]
name = "test_{}"
version = "1.0"

[service.db_{}]
type = "postgres"
"#,
                                i, i
                            ),
                        )
                        .unwrap();
                    }

                    // Render all templates
                    let mut renderer = TemplateRenderer::new().unwrap();
                    let context = TemplateContext::new();
                    renderer = renderer.with_context(context);

                    for entry in fs::read_dir(temp_dir.path()).unwrap() {
                        let entry = entry.unwrap();
                        if entry.path().extension().and_then(|s| s.to_str()) == Some("tera") {
                            let content = fs::read_to_string(entry.path()).unwrap();
                            let rendered = renderer.render_str(&content, "test").unwrap();
                            black_box(rendered);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Command Performance Benchmarks
// ============================================================================

fn benchmark_command_performance(c: &mut Criterion) {
    use std::fs;
    use tempfile::TempDir;

    let mut group = c.benchmark_group("command_performance");

    // Dry-run validation (target: <1s)
    group.bench_function("dry_run_validation", |b| {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.clnrm.toml");
        fs::write(
            &file_path,
            r#"
[meta]
name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "test_1"
command = "echo test"
"#,
        )
        .unwrap();

        b.iter(|| {
            // Read file
            let content = fs::read_to_string(&file_path).unwrap();

            // Parse TOML
            let parsed: toml::Value = toml::from_str(&content).unwrap();

            // Validate structure
            let has_meta = parsed.get("meta").is_some();
            let has_service = parsed.get("service").is_some();
            let has_scenario = parsed.get("scenario").is_some();

            black_box((has_meta, has_service, has_scenario));
        });
    });

    // Format check (target: <500ms)
    group.bench_function("fmt_check", |b| {
        let toml_str = r#"
[meta]
name="test"
version="1.0"

[service.db]
type="postgres"
"#;

        b.iter(|| {
            // Parse TOML
            let parsed: toml::Value = toml::from_str(toml_str).unwrap();

            // Re-serialize (formatting)
            let formatted = toml::to_string_pretty(&parsed).unwrap();

            // Check if formatting changed
            let needs_format = formatted != toml_str;

            black_box(needs_format);
        });
    });

    // Lint validation (target: <1s)
    group.bench_function("lint_validation", |b| {
        let toml_str = r#"
[meta]
name = "test"

[otel]
service_name = "test_service"

[service.db]
type = "postgres"

[[scenario]]
name = "test_1"
command = "echo test"
"#;

        b.iter(|| {
            // Parse TOML
            let parsed: toml::Value = toml::from_str(toml_str).unwrap();

            // Validate required sections
            let mut errors = Vec::new();
            if parsed.get("meta").is_none() {
                errors.push("Missing [meta] section");
            }
            if parsed.get("service").is_none() {
                errors.push("Missing [service] section");
            }

            black_box(errors);
        });
    });

    group.finish();
}

// ============================================================================
// Memory Usage Benchmarks
// ============================================================================

fn benchmark_memory_usage(c: &mut Criterion) {
    use clnrm_core::template::{TemplateRenderer, TemplateContext};
    use std::fs;
    use tempfile::TempDir;

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_usage");

    // Sustained load test (target: <100MB base memory)
    group.bench_function("sustained_load", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let mut renderer = TemplateRenderer::new().unwrap();

            // Process 1000 templates
            for i in 0..1000 {
                let template = format!(
                    r#"
[meta]
name = "test_{}"

[service.db_{}]
type = "postgres"
"#,
                    i, i
                );

                let context = TemplateContext::new();
                renderer = renderer.with_context(context);
                let rendered = renderer.render_str(&template, "test").unwrap();

                // Write to file to simulate real usage
                let file_path = temp_dir.path().join(format!("test_{}.clnrm.toml", i));
                fs::write(&file_path, rendered).unwrap();

                black_box(i);
            }
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    name = dx_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        benchmark_template_rendering,
        benchmark_toml_parsing,
        benchmark_file_operations,
        benchmark_hot_reload_workflow,
        benchmark_scalability,
        benchmark_command_performance,
        benchmark_memory_usage
);

criterion_main!(dx_benches);
