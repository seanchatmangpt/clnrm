// Weaver Live-Check Integration Examples for clnrm
// These examples show how to use all weaver_live_check features

#![allow(dead_code, unused_imports)]

use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;

// === EXAMPLE 1: Basic OTLP Integration (CURRENTLY IMPLEMENTED) ===

#[test]
fn example_01_basic_otlp_integration() -> Result<(), Box<dyn std::error::Error>> {
    use crate::telemetry::weaver_controller::{WeaverConfig, WeaverController};

    // Configure Weaver
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry/"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: PathBuf::from("validation_output"),
        stream: false,
        inactivity_timeout: 30,
    };

    // Start Weaver live-check listener
    let mut weaver = WeaverController::new(config)?;
    weaver.start()?;

    // Run tests that emit OTLP telemetry
    run_tests_with_otlp_export()?;

    // Stop Weaver and get validation report
    let report = weaver.stop_and_get_report()?;

    // Check for violations
    println!("Violations: {}", report.violations);
    println!("Improvements: {}", report.improvements);
    println!("Registry Coverage: {:.2}%", report.registry_coverage * 100.0);

    if report.violations > 0 {
        return Err(format!("Weaver detected {} violations", report.violations).into());
    }

    Ok(())
}

// === EXAMPLE 2: Testing Span Validation ===

#[test]
fn example_02_span_validation() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_span::{SampleSpan, Status, StatusCode};
    use weaver_semconv::group::SpanKindSpec;
    use weaver_live_check::sample_attribute::SampleAttribute;
    use weaver_semconv::attribute::PrimitiveOrArrayTypeSpec;

    // Create a sample span
    let span = SampleSpan {
        name: "http.client.request".to_string(),
        kind: SpanKindSpec::Client,
        status: Some(Status {
            code: StatusCode::Ok,
            message: "Request completed successfully".to_string(),
        }),
        attributes: vec![
            SampleAttribute {
                name: "http.method".to_string(),
                value: Some(json!("GET")),
                r#type: Some(PrimitiveOrArrayTypeSpec::String),
                live_check_result: None,
            },
            SampleAttribute {
                name: "http.url".to_string(),
                value: Some(json!("https://api.example.com/users")),
                r#type: Some(PrimitiveOrArrayTypeSpec::String),
                live_check_result: None,
            },
            SampleAttribute {
                name: "http.status_code".to_string(),
                value: Some(json!(200)),
                r#type: Some(PrimitiveOrArrayTypeSpec::Int),
                live_check_result: None,
            },
        ],
        span_events: vec![],
        span_links: vec![],
        live_check_result: None,
    };

    // Emit span via OTLP
    emit_span_via_otlp(&span)?;

    // Weaver validates in real-time
    std::thread::sleep(Duration::from_millis(100));

    Ok(())
}

// === EXAMPLE 3: Testing Metric Validation ===

#[test]
fn example_03_metric_validation() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_metric::{SampleMetric, SampleInstrument, DataPoints, SampleNumberDataPoint};
    use weaver_semconv::group::InstrumentSpec;
    use weaver_live_check::sample_attribute::SampleAttribute;
    use weaver_semconv::attribute::PrimitiveOrArrayTypeSpec;

    // Create a sample metric
    let metric = SampleMetric {
        name: "http.server.request.duration".to_string(),
        instrument: SampleInstrument::Supported(InstrumentSpec::Histogram),
        unit: "ms".to_string(),
        data_points: Some(DataPoints::Number(vec![
            SampleNumberDataPoint {
                attributes: vec![
                    SampleAttribute {
                        name: "http.method".to_string(),
                        value: Some(json!("POST")),
                        r#type: Some(PrimitiveOrArrayTypeSpec::String),
                        live_check_result: None,
                    },
                ],
                value: json!(125.5),
                flags: 0,
                exemplars: vec![],
                live_check_result: None,
            },
        ])),
        live_check_result: None,
    };

    // Emit metric via OTLP
    emit_metric_via_otlp(&metric)?;

    Ok(())
}

// === EXAMPLE 4: Testing Histogram Data Points ===

#[test]
fn example_04_histogram_validation() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_metric::{
        SampleMetric, SampleInstrument, DataPoints, SampleHistogramDataPoint
    };
    use weaver_semconv::group::InstrumentSpec;

    let metric = SampleMetric {
        name: "http.server.request.duration".to_string(),
        instrument: SampleInstrument::Supported(InstrumentSpec::Histogram),
        unit: "ms".to_string(),
        data_points: Some(DataPoints::Histogram(vec![
            SampleHistogramDataPoint {
                attributes: vec![],
                count: 100,
                sum: Some(5000.0),
                bucket_counts: vec![10, 20, 30, 20, 20],
                explicit_bounds: vec![100.0, 250.0, 500.0, 1000.0],
                min: Some(50.0),
                max: Some(1500.0),
                flags: 0,
                exemplars: vec![],
                live_check_result: None,
            },
        ])),
        live_check_result: None,
    };

    emit_metric_via_otlp(&metric)?;

    Ok(())
}

// === EXAMPLE 5: Testing Exponential Histogram ===

#[test]
fn example_05_exponential_histogram() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_metric::{
        SampleMetric, SampleInstrument, DataPoints,
        SampleExponentialHistogramDataPoint, SampleExponentialHistogramBuckets
    };
    use weaver_semconv::group::InstrumentSpec;

    let metric = SampleMetric {
        name: "system.memory.usage".to_string(),
        instrument: SampleInstrument::Supported(InstrumentSpec::Histogram),
        unit: "By".to_string(),
        data_points: Some(DataPoints::ExponentialHistogram(vec![
            SampleExponentialHistogramDataPoint {
                attributes: vec![],
                count: 1000,
                sum: Some(5000000.0),
                scale: 1,
                zero_count: 10,
                positive: Some(SampleExponentialHistogramBuckets {
                    offset: 0,
                    bucket_counts: vec![100, 200, 300, 200, 100, 90, 10],
                }),
                negative: None,
                flags: 0,
                min: Some(0.0),
                max: Some(10000000.0),
                zero_threshold: 0.0,
                exemplars: vec![],
                live_check_result: None,
            },
        ])),
        live_check_result: None,
    };

    emit_metric_via_otlp(&metric)?;

    Ok(())
}

// === EXAMPLE 6: Testing Resource Attributes ===

#[test]
fn example_06_resource_validation() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_resource::SampleResource;
    use weaver_live_check::sample_attribute::SampleAttribute;
    use weaver_semconv::attribute::PrimitiveOrArrayTypeSpec;

    let resource = SampleResource {
        attributes: vec![
            SampleAttribute {
                name: "service.name".to_string(),
                value: Some(json!("clnrm-test-service")),
                r#type: Some(PrimitiveOrArrayTypeSpec::String),
                live_check_result: None,
            },
            SampleAttribute {
                name: "service.version".to_string(),
                value: Some(json!("1.2.0")),
                r#type: Some(PrimitiveOrArrayTypeSpec::String),
                live_check_result: None,
            },
            SampleAttribute {
                name: "deployment.environment".to_string(),
                value: Some(json!("test")),
                r#type: Some(PrimitiveOrArrayTypeSpec::String),
                live_check_result: None,
            },
        ],
        live_check_result: None,
    };

    emit_resource_via_otlp(&resource)?;

    Ok(())
}

// === EXAMPLE 7: Custom Rego Policy ===

#[test]
fn example_07_custom_rego_policy() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Create custom policy directory
    fs::create_dir_all("custom_policies")?;

    // Write custom Rego policy
    let policy = r#"
package live_check_advice

import rego.v1

# Enforce clnrm-specific metric naming convention
deny contains make_advice(advice_type, advice_level, advice_context, message) if {
    input.sample.metric
    not startswith(input.sample.metric.name, "clnrm.")
    advice_type := "invalid_metric_prefix"
    advice_level := "violation"
    advice_context := {
        "metric_name": input.sample.metric.name
    }
    message := sprintf("Metric name must start with 'clnrm.', got '%s'",
                       [input.sample.metric.name])
}

# Enforce attribute naming convention
deny contains make_advice(advice_type, advice_level, advice_context, message) if {
    input.sample.attribute
    contains(input.sample.attribute.name, "TEST")
    advice_type := "uppercase_in_attribute"
    advice_level := "violation"
    advice_context := {
        "attribute_name": input.sample.attribute.name
    }
    message := sprintf("Attribute name must be lowercase, got '%s'",
                       [input.sample.attribute.name])
}

make_advice(advice_type, advice_level, advice_context, message) := {
    "type": "advice",
    "advice_type": advice_type,
    "advice_level": advice_level,
    "advice_context": advice_context,
    "message": message,
}
    "#;

    fs::write("custom_policies/clnrm.rego", policy)?;

    // Run weaver with custom policies
    // weaver registry live-check \
    //   --registry registry/ \
    //   --advice-policies custom_policies/ \
    //   --otlp-grpc-port 4317

    Ok(())
}

// === EXAMPLE 8: Streaming Output ===

#[test]
fn example_08_streaming_output() -> Result<(), Box<dyn std::error::Error>> {
    use crate::telemetry::weaver_controller::{WeaverConfig, WeaverController};

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry/"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: PathBuf::from("validation_output"),
        stream: true,  // Enable streaming output
        inactivity_timeout: 30,
    };

    let mut weaver = WeaverController::new(config)?;
    weaver.start()?;

    // Emit telemetry and get real-time feedback
    emit_attribute_via_otlp("test.attribute", "value")?;

    // Streaming output appears immediately in console
    std::thread::sleep(Duration::from_millis(100));

    weaver.stop()?;

    Ok(())
}

// === EXAMPLE 9: File Input Ingestion ===

#[test]
fn example_09_file_input() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use weaver_live_check::sample_attribute::SampleAttribute;
    use weaver_semconv::attribute::PrimitiveOrArrayTypeSpec;

    // Create sample JSON file
    let samples = vec![
        SampleAttribute {
            name: "http.method".to_string(),
            value: Some(json!("GET")),
            r#type: Some(PrimitiveOrArrayTypeSpec::String),
            live_check_result: None,
        },
        SampleAttribute {
            name: "http.status_code".to_string(),
            value: Some(json!(200)),
            r#type: Some(PrimitiveOrArrayTypeSpec::Int),
            live_check_result: None,
        },
    ];

    let json = serde_json::to_string_pretty(&samples)?;
    fs::write("test_samples.json", json)?;

    // Run weaver with file input:
    // weaver registry live-check --input-source test_samples.json --format json

    Ok(())
}

// === EXAMPLE 10: Text Input (Attribute Names) ===

#[test]
fn example_10_text_input() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Create text file with attribute names
    let attributes = vec![
        "http.method=GET",
        "http.status_code=200",
        "http.url",
        "custom.attribute=value",
    ];

    fs::write("attributes.txt", attributes.join("\n"))?;

    // Run weaver with text input:
    // weaver registry live-check --input-source attributes.txt --input-format text

    Ok(())
}

// === EXAMPLE 11: Testing All Built-in Advisors ===

#[test]
fn example_11_all_advisors() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::advice::{
        DeprecatedAdvisor, StabilityAdvisor, TypeAdvisor, EnumAdvisor, RegoAdvisor
    };
    use weaver_live_check::live_checker::LiveChecker;
    use weaver_forge::registry::ResolvedRegistry;

    // Load registry
    let registry = load_registry("registry/")?;

    // Create LiveChecker with all advisors
    let advisors: Vec<Box<dyn weaver_live_check::advice::Advisor>> = vec![
        Box::new(DeprecatedAdvisor),      // Detects deprecated attributes
        Box::new(StabilityAdvisor),       // Checks stability levels
        Box::new(TypeAdvisor),            // Validates types
        Box::new(EnumAdvisor),            // Validates enum variants
        // RegoAdvisor requires initialization
    ];

    let mut live_checker = LiveChecker::new(registry, advisors);

    // Add custom Rego advisor
    let rego_advisor = RegoAdvisor::new(&live_checker, &None, &None)?;
    live_checker.add_advisor(Box::new(rego_advisor));

    // Now run validation with all advisors active
    Ok(())
}

// === EXAMPLE 12: Statistics and Coverage ===

#[test]
fn example_12_statistics() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::LiveCheckStatistics;
    use weaver_forge::registry::ResolvedRegistry;

    let registry = load_registry("registry/")?;
    let mut stats = LiveCheckStatistics::new(&registry);

    // After running validation, check statistics
    println!("Total entities: {}", stats.total_entities);
    println!("Total advisories: {}", stats.total_advisories);
    println!("Registry coverage: {:.2}%", stats.registry_coverage * 100.0);

    // Advice level breakdown
    for (level, count) in &stats.advice_level_counts {
        println!("{:?}: {}", level, count);
    }

    // Entity type breakdown
    for (entity_type, count) in &stats.total_entities_by_type {
        println!("{}: {}", entity_type, count);
    }

    // Registry coverage details
    println!("\nRegistry Attributes Seen:");
    for (attr, count) in &stats.seen_registry_attributes {
        if *count > 0 {
            println!("  {}: {} times", attr, count);
        }
    }

    println!("\nNon-Registry Attributes:");
    for (attr, count) in &stats.seen_non_registry_attributes {
        println!("  {}: {} times", attr, count);
    }

    Ok(())
}

// === EXAMPLE 13: Exemplar Validation ===

#[test]
fn example_13_exemplar_validation() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_metric::{SampleExemplar, SampleAttribute};
    use weaver_semconv::attribute::PrimitiveOrArrayTypeSpec;

    let exemplar = SampleExemplar {
        filtered_attributes: vec![
            SampleAttribute {
                name: "http.method".to_string(),
                value: Some(json!("POST")),
                r#type: Some(PrimitiveOrArrayTypeSpec::String),
                live_check_result: None,
            },
        ],
        value: json!(125.5),
        timestamp: "2025-10-30T15:00:00Z".to_string(),
        span_id: "abc123".to_string(),
        trace_id: "def456".to_string(),
        live_check_result: None,
    };

    // Exemplars are validated as part of metric data points
    Ok(())
}

// === EXAMPLE 14: Span Events and Links ===

#[test]
fn example_14_span_events_and_links() -> Result<(), Box<dyn std::error::Error>> {
    use weaver_live_check::sample_span::{SampleSpan, SampleSpanEvent, SampleSpanLink};
    use weaver_semconv::group::SpanKindSpec;
    use weaver_live_check::sample_attribute::SampleAttribute;
    use weaver_semconv::attribute::PrimitiveOrArrayTypeSpec;

    let span = SampleSpan {
        name: "process_order".to_string(),
        kind: SpanKindSpec::Internal,
        status: None,
        attributes: vec![],
        span_events: vec![
            SampleSpanEvent {
                name: "order_validated".to_string(),
                attributes: vec![
                    SampleAttribute {
                        name: "order.id".to_string(),
                        value: Some(json!("12345")),
                        r#type: Some(PrimitiveOrArrayTypeSpec::String),
                        live_check_result: None,
                    },
                ],
                live_check_result: None,
            },
        ],
        span_links: vec![
            SampleSpanLink {
                attributes: vec![
                    SampleAttribute {
                        name: "link.type".to_string(),
                        value: Some(json!("follows_from")),
                        r#type: Some(PrimitiveOrArrayTypeSpec::String),
                        live_check_result: None,
                    },
                ],
                live_check_result: None,
            },
        ],
        live_check_result: None,
    };

    emit_span_via_otlp(&span)?;

    Ok(())
}

// === EXAMPLE 15: Complete End-to-End Test ===

#[test]
fn example_15_end_to_end_validation() -> Result<(), Box<dyn std::error::Error>> {
    use crate::telemetry::weaver_controller::{WeaverConfig, WeaverController};

    // 1. Configure Weaver
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry/"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: PathBuf::from("validation_output"),
        stream: false,
        inactivity_timeout: 30,
    };

    // 2. Start Weaver
    let mut weaver = WeaverController::new(config)?;
    weaver.start()?;

    // 3. Run comprehensive test suite
    run_attribute_tests()?;
    run_metric_tests()?;
    run_span_tests()?;
    run_resource_tests()?;

    // 4. Get validation report
    let report = weaver.stop_and_get_report()?;

    // 5. Assert validation passed
    assert_eq!(report.violations, 0, "Expected no violations");
    assert!(report.registry_coverage > 0.5, "Expected >50% registry coverage");

    // 6. Print summary
    println!("\n=== Validation Summary ===");
    println!("Status: {:?}", report.status);
    println!("Violations: {}", report.violations);
    println!("Improvements: {}", report.improvements);
    println!("Information: {}", report.information);
    println!("Registry Coverage: {:.2}%", report.registry_coverage * 100.0);

    // 7. Print details
    if !report.details.is_empty() {
        println!("\n=== Validation Details ===");
        for detail in &report.details {
            println!("[{}] {}: {}",
                     detail.level,
                     detail.metric_name.as_ref().unwrap_or(&"N/A".to_string()),
                     detail.message);
        }
    }

    Ok(())
}

// === Helper Functions ===

fn run_tests_with_otlp_export() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Run actual clnrm tests with OTLP exporter configured
    Ok(())
}

fn emit_span_via_otlp(span: &weaver_live_check::sample_span::SampleSpan)
    -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Emit span via OTLP gRPC
    Ok(())
}

fn emit_metric_via_otlp(metric: &weaver_live_check::sample_metric::SampleMetric)
    -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Emit metric via OTLP gRPC
    Ok(())
}

fn emit_resource_via_otlp(resource: &weaver_live_check::sample_resource::SampleResource)
    -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Emit resource via OTLP gRPC
    Ok(())
}

fn emit_attribute_via_otlp(name: &str, value: &str)
    -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Emit attribute via OTLP gRPC
    Ok(())
}

fn load_registry(path: &str) -> Result<weaver_forge::registry::ResolvedRegistry, Box<dyn std::error::Error>> {
    // Placeholder: Load and resolve semantic convention registry
    unimplemented!("Load registry from {}", path)
}

fn run_attribute_tests() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Run attribute validation tests
    Ok(())
}

fn run_metric_tests() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Run metric validation tests
    Ok(())
}

fn run_span_tests() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Run span validation tests
    Ok(())
}

fn run_resource_tests() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder: Run resource validation tests
    Ok(())
}
