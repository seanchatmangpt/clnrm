//! Demo of DiagnosticFormatter capabilities
//!
//! Shows all three output formats with example conformance reports.
//!
//! Run with: cargo run --example diagnostics_demo

use chrono::Utc;
use clnrm_core::telemetry::live_check::diagnostics::*;
use std::path::PathBuf;

fn create_passing_report() -> ConformanceReport {
    ConformanceReport {
        clnrm_version: "1.3.0".to_string(),
        test_name: "passing_integration_test".to_string(),
        test_file: PathBuf::from("tests/integration/passing.clnrm.toml"),
        timestamp: Utc::now(),
        duration_ms: 1234,
        validation_status: ValidationStatus::Pass,
        spans: SpanValidation {
            required_count: 10,
            present_count: 10,
            missing: vec![],
        },
        attributes: AttributeValidation {
            required_count: 20,
            present_count: 20,
            missing_count: 0,
            missing: vec![],
        },
        violations: vec![],
        exit_code: 0,
        recommendation: None,
        environment: EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            ci: false,
            github_actions: false,
        },
    }
}

fn create_failing_report() -> ConformanceReport {
    let mut report = create_passing_report();
    report.test_name = "failing_integration_test".to_string();
    report.test_file = PathBuf::from("tests/integration/failing.clnrm.toml");
    report.validation_status = ValidationStatus::Fail;
    report.spans.present_count = 8;
    report.spans.missing = vec!["clnrm.test.setup".to_string(), "clnrm.test.cleanup".to_string()];
    report.exit_code = 1;

    report.violations = vec![
        Violation {
            type_: "missing_span".to_string(),
            severity: "error".to_string(),
            name: "clnrm.test.setup".to_string(),
            span: None,
            schema_file: PathBuf::from("registry/test.yaml"),
            schema_line: 10,
            message: "Required span 'clnrm.test.setup' not found in telemetry".to_string(),
            documentation_url: Some("https://docs.clnrm.dev/telemetry/spans#setup".to_string()),
        },
        Violation {
            type_: "missing_span".to_string(),
            severity: "error".to_string(),
            name: "clnrm.test.cleanup".to_string(),
            span: None,
            schema_file: PathBuf::from("registry/test.yaml"),
            schema_line: 15,
            message: "Required span 'clnrm.test.cleanup' not found in telemetry".to_string(),
            documentation_url: Some("https://docs.clnrm.dev/telemetry/spans#cleanup".to_string()),
        },
    ];

    report.recommendation = Some(
        "Fix 2 critical violations: missing spans 'clnrm.test.setup' and 'clnrm.test.cleanup'".to_string()
    );

    report
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║    clnrm v1.3.0 DiagnosticFormatter Demo                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Demo 1: ANSI Formatter with Passing Test
    println!("\n=== DEMO 1: ANSI Formatter (Passing Test) ===\n");
    let passing_report = create_passing_report();
    let ansi_formatter = AnsiFormatter::new(AnsiConfig::default());
    match ansi_formatter.format(&passing_report) {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Demo 2: ANSI Formatter with Failing Test
    println!("\n\n=== DEMO 2: ANSI Formatter (Failing Test) ===\n");
    let failing_report = create_failing_report();
    match ansi_formatter.format(&failing_report) {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Demo 3: JSON Formatter
    println!("\n\n=== DEMO 3: JSON Formatter ===\n");
    let json_formatter = JsonFormatter::new(JsonConfig::default());
    match json_formatter.format(&failing_report) {
        Ok(output) => {
            // Pretty-print first 50 lines
            for (i, line) in output.lines().enumerate() {
                if i >= 50 {
                    println!("... (truncated)");
                    break;
                }
                println!("{}", line);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Demo 4: GitHub Workflow Formatter
    println!("\n\n=== DEMO 4: GitHub Workflow Formatter ===\n");
    let gh_formatter = GithubWorkflowFormatter::new(GithubConfig::default());
    match gh_formatter.format(&failing_report) {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Demo 5: Auto-Detection
    println!("\n\n=== DEMO 5: Auto-Detection ===\n");
    let detected_format = detect_format();
    println!("Detected format: {:?}", detected_format);
    println!("Environment:");
    println!("  - GITHUB_ACTIONS: {:?}", std::env::var("GITHUB_ACTIONS").ok());
    println!("  - CI: {:?}", std::env::var("CI").ok());
    println!("  - TTY: {}", if cfg!(unix) { "Unix TTY check" } else { "Windows" });

    // Demo 6: DiagnosticProcessor
    println!("\n\n=== DEMO 6: DiagnosticProcessor ===\n");
    let config = DiagnosticConfig {
        format: "ansi".to_string(),
        ..Default::default()
    };
    let processor = DiagnosticProcessor::new(config);
    match processor.process(&failing_report) {
        Ok(output) => {
            println!("Processor successfully formatted report.");
            println!("Output length: {} bytes", output.len());
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Demo 7: Recommendation Generation
    println!("\n\n=== DEMO 7: Recommendation Generation ===\n");
    let rec_pass = DiagnosticProcessor::generate_recommendation(&passing_report);
    let rec_fail = DiagnosticProcessor::generate_recommendation(&failing_report);
    println!("Passing test recommendation: {:?}", rec_pass);
    println!("Failing test recommendation: {:?}", rec_fail);

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║    Demo Complete - All Formatters Working!                   ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
}
