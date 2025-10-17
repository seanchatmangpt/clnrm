//! V1.0 Schema Validation Tests
//!
//! Comprehensive tests for v1.0 TOML schema parsing and validation

use clnrm_core::config::*;

#[test]
fn test_v1_complete_schema() {
    let toml_content = r#"
[meta]
name = "clnrm_otel_proof"
version = "1.0.0"
description = "Complete v1.0 schema validation test"

[vars]
svc = "clnrm"
env = "ci"
endpoint = "http://localhost:4318"
exporter = "otlp"
freeze_clock = "2025-01-01T00:00:00Z"
image = "registry/clnrm:1.0.0"

[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "clnrm", "env" = "ci" }

[otel.headers]
Authorization = "Bearer test-token"

[otel.propagators]
use = ["tracecontext", "baggage"]

[service.clnrm]
type = "generic_container"
plugin = "generic"
image = "registry/clnrm:1.0.0"

[[scenario]]
name = "otel_only_proof"
steps = [
    { name = "run_test", command = ["clnrm", "run"] }
]

[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "clnrm.step:hello_world"
parent = "clnrm.run"
kind = "internal"
events.any = ["container.start", "container.exec", "container.stop"]

[expect.graph]
must_include = [["clnrm.run", "clnrm.step:hello_world"]]
acyclic = true

[expect.status]
all = "OK"

[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = { "service.name" = "clnrm", "env" = "ci" }

[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "report.json"
digest = "trace.sha256"

[limits]
cpu_millicores = 1000
memory_mb = 512
"#;

    // Parse the TOML
    let config: TestConfig = toml::from_str(toml_content).expect("Failed to parse v1.0 schema");

    // Validate metadata
    assert_eq!(config.meta.as_ref().unwrap().name, "clnrm_otel_proof");
    assert_eq!(config.meta.as_ref().unwrap().version, "1.0.0");

    // Validate OTEL config
    let otel = config.otel.as_ref().expect("OTEL config missing");
    assert_eq!(otel.exporter, "otlp");
    assert_eq!(otel.endpoint.as_ref().unwrap(), "http://localhost:4318");
    assert_eq!(otel.protocol.as_ref().unwrap(), "http/protobuf");
    assert_eq!(otel.sample_ratio.unwrap(), 1.0);

    // Validate expectations
    let expect = config.expect.as_ref().expect("Expectations missing");

    // Validate span expectations
    assert_eq!(expect.span.len(), 2);
    assert_eq!(expect.span[0].name, "clnrm.run");
    assert_eq!(expect.span[0].kind.as_ref().unwrap(), "internal");

    // Validate graph expectations
    let graph = expect.graph.as_ref().expect("Graph expectations missing");
    assert!(graph.acyclic.unwrap());
    let edges = graph.must_include.as_ref().unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0][0], "clnrm.run");
    assert_eq!(edges[0][1], "clnrm.step:hello_world");

    // Validate status expectations
    let status = expect.status.as_ref().expect("Status expectations missing");
    assert_eq!(status.all.as_ref().unwrap(), "OK");

    // Validate hermeticity expectations
    let hermetic = expect
        .hermeticity
        .as_ref()
        .expect("Hermeticity expectations missing");
    assert!(hermetic.no_external_services.unwrap());

    // Validate determinism config
    let determinism = config
        .determinism
        .as_ref()
        .expect("Determinism config missing");
    assert_eq!(determinism.seed.unwrap(), 42);
    assert_eq!(
        determinism.freeze_clock.as_ref().unwrap(),
        "2025-01-01T00:00:00Z"
    );

    // Validate report config
    let report = config.report.as_ref().expect("Report config missing");
    assert_eq!(report.json.as_ref().unwrap(), "report.json");
    assert_eq!(report.digest.as_ref().unwrap(), "trace.sha256");

    // Validate limits config
    let limits = config.limits.as_ref().expect("Limits config missing");
    assert_eq!(limits.cpu_millicores.unwrap(), 1000);
    assert_eq!(limits.memory_mb.unwrap(), 512);

    // Run validation
    config.validate().expect("Validation failed");
}

#[test]
fn test_v1_minimal_schema() {
    let toml_content = r#"
[meta]
name = "minimal_test"
version = "1.0.0"

[[scenario]]
name = "basic"
steps = [
    { name = "test", command = ["echo", "hello"] }
]
"#;

    let config: TestConfig = toml::from_str(toml_content).expect("Failed to parse minimal schema");
    config.validate().expect("Minimal schema validation failed");
}

#[test]
fn test_v1_order_expectations() {
    let toml_content = r#"
[meta]
name = "order_test"
version = "1.0.0"

[[scenario]]
name = "test"
steps = [
    { name = "test", command = ["echo", "test"] }
]

[expect.order]
must_precede = [["A", "B"], ["B", "C"]]
must_follow = [["D", "C"]]
"#;

    let config: TestConfig = toml::from_str(toml_content).expect("Failed to parse order schema");

    let expect = config.expect.as_ref().expect("Expectations missing");
    let order = expect.order.as_ref().expect("Order expectations missing");

    assert_eq!(order.must_precede.as_ref().unwrap().len(), 2);
    assert_eq!(order.must_follow.as_ref().unwrap().len(), 1);
}

#[test]
fn test_v1_count_expectations() {
    let toml_content = r#"
[meta]
name = "count_test"
version = "1.0.0"

[[scenario]]
name = "test"
steps = [
    { name = "test", command = ["echo", "test"] }
]

[expect.counts]
spans_total = { gte = 10, lte = 100 }
events_total = { eq = 50 }
errors_total = { lte = 5 }

[expect.counts.by_name]
"test_span" = { eq = 10 }
"other_span" = { gte = 1 }
"#;

    let config: TestConfig = toml::from_str(toml_content).expect("Failed to parse count schema");

    let expect = config.expect.as_ref().expect("Expectations missing");
    let counts = expect.counts.as_ref().expect("Count expectations missing");

    assert_eq!(counts.spans_total.as_ref().unwrap().gte.unwrap(), 10);
    assert_eq!(counts.spans_total.as_ref().unwrap().lte.unwrap(), 100);
    assert_eq!(counts.events_total.as_ref().unwrap().eq.unwrap(), 50);
    assert_eq!(counts.errors_total.as_ref().unwrap().lte.unwrap(), 5);

    assert_eq!(
        counts
            .by_name
            .as_ref()
            .unwrap()
            .get("test_span")
            .unwrap()
            .eq
            .unwrap(),
        10
    );
}

#[test]
fn test_v1_window_expectations() {
    let toml_content = r#"
[meta]
name = "window_test"
version = "1.0.0"

[[scenario]]
name = "test"
steps = [
    { name = "test", command = ["echo", "test"] }
]

[[expect.window]]
outer = "parent_span"
contains = ["child_a", "child_b", "child_c"]

[[expect.window]]
outer = "root"
contains = ["sub_operation"]
"#;

    let config: TestConfig = toml::from_str(toml_content).expect("Failed to parse window schema");

    let expect = config.expect.as_ref().expect("Expectations missing");
    assert_eq!(expect.window.len(), 2);
    assert_eq!(expect.window[0].outer, "parent_span");
    assert_eq!(expect.window[0].contains.len(), 3);
    assert_eq!(expect.window[1].outer, "root");
}

#[test]
fn test_v1_span_events_and_duration() {
    let toml_content = r#"
[meta]
name = "span_details_test"
version = "1.0.0"

[[scenario]]
name = "test"
steps = [
    { name = "test", command = ["echo", "test"] }
]

[[expect.span]]
name = "test_span"
kind = "internal"
events.any = ["start", "end"]
duration_ms = { min = 100.0, max = 5000.0 }

[[expect.span]]
name = "other_span"
events.all = ["init", "process", "cleanup"]
"#;

    let config: TestConfig = toml::from_str(toml_content).expect("Failed to parse span details");

    let expect = config.expect.as_ref().expect("Expectations missing");
    assert_eq!(expect.span.len(), 2);

    let span1 = &expect.span[0];
    assert_eq!(
        span1.events.as_ref().unwrap().any.as_ref().unwrap().len(),
        2
    );
    assert_eq!(span1.duration_ms.as_ref().unwrap().min.unwrap(), 100.0);
    assert_eq!(span1.duration_ms.as_ref().unwrap().max.unwrap(), 5000.0);

    let span2 = &expect.span[1];
    assert_eq!(
        span2.events.as_ref().unwrap().all.as_ref().unwrap().len(),
        3
    );
}
