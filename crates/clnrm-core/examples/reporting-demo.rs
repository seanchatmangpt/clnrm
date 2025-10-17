//! Report Generation Demo
//!
//! Demonstrates the multi-format report generation capabilities:
//! - JSON reports for programmatic access
//! - JUnit XML for CI/CD integration
//! - SHA-256 digests for reproducibility
//!
//! Run with:
//! ```bash
//! cargo run --example reporting-demo
//! ```

use clnrm_core::{
    generate_reports, DigestReporter, JsonReporter, JunitReporter, ReportConfig, Result,
    ValidationReport,
};
use std::path::Path;

fn main() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║     Cleanroom Report Generation Demo                 ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    // Example 1: Individual report generation
    demo_individual_reporters()?;

    println!("\n{}", "=".repeat(60));

    // Example 2: Unified report generation
    demo_unified_generation()?;

    println!("\n{}", "=".repeat(60));

    // Example 3: Digest comparison for reproducibility
    demo_digest_comparison()?;

    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║     Demo Complete - Check /tmp/clnrm-reports/        ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    Ok(())
}

/// Demonstrate individual reporter usage
fn demo_individual_reporters() -> Result<()> {
    println!("📊 Example 1: Individual Report Generators\n");

    // Create a sample validation report
    let mut report = ValidationReport::new();
    report.add_pass("graph_topology");
    report.add_pass("span_counts");
    report.add_pass("temporal_windows");
    report.add_fail(
        "hermeticity",
        "Cross-container communication detected between test1 and test2".to_string(),
    );

    println!("Created validation report:");
    println!("  ✓ Passes: {}", report.pass_count());
    println!("  ✗ Failures: {}", report.failure_count());
    println!();

    // Create output directory
    std::fs::create_dir_all("/tmp/clnrm-reports").map_err(|e| {
        clnrm_core::CleanroomError::io_error(format!("Failed to create output dir: {}", e))
    })?;

    // Generate JSON report
    println!("Generating JSON report...");
    JsonReporter::write(Path::new("/tmp/clnrm-reports/report.json"), &report)?;
    println!("  ✓ Written to /tmp/clnrm-reports/report.json");

    // Generate JUnit XML report
    println!("Generating JUnit XML report...");
    JunitReporter::write(Path::new("/tmp/clnrm-reports/junit.xml"), &report)?;
    println!("  ✓ Written to /tmp/clnrm-reports/junit.xml");

    // Generate digest
    let spans_json = r#"{"spans":[{"name":"root","span_id":"1"},{"name":"child","span_id":"2"}]}"#;
    println!("Generating SHA-256 digest...");
    DigestReporter::write(Path::new("/tmp/clnrm-reports/digest.txt"), spans_json)?;
    let digest = DigestReporter::compute_digest(spans_json);
    println!("  ✓ Digest: {}", digest);

    Ok(())
}

/// Demonstrate unified report generation
fn demo_unified_generation() -> Result<()> {
    println!("🎯 Example 2: Unified Report Generation\n");

    // Create validation report with various results
    let mut report = ValidationReport::new();
    report.add_pass("validation_1");
    report.add_pass("validation_2");
    report.add_pass("validation_3");
    report.add_fail("validation_4", "Expected 5 spans but found 3".to_string());
    report.add_fail(
        "validation_5",
        "Span 'database.query' missing required attribute 'db.statement'".to_string(),
    );

    // Configure all report formats
    let config = ReportConfig::new()
        .with_json("/tmp/clnrm-reports/unified.json")
        .with_junit("/tmp/clnrm-reports/unified-junit.xml")
        .with_digest("/tmp/clnrm-reports/unified-digest.txt");

    println!("Configuration:");
    println!("  JSON:   {:?}", config.json_path);
    println!("  JUnit:  {:?}", config.junit_path);
    println!("  Digest: {:?}", config.digest_path);
    println!();

    // Sample span data
    let spans_json = r#"{
  "spans": [
    {"name": "http.request", "span_id": "1", "parent_span_id": null},
    {"name": "database.query", "span_id": "2", "parent_span_id": "1"},
    {"name": "cache.get", "span_id": "3", "parent_span_id": "1"}
  ]
}"#;

    println!("Generating all reports...");
    generate_reports(&config, &report, spans_json)?;
    println!("  ✓ All reports generated successfully");
    println!();

    // Display summary
    println!("Report Summary:");
    println!("{}", report.summary());

    Ok(())
}

/// Demonstrate digest comparison for reproducibility
fn demo_digest_comparison() -> Result<()> {
    println!("🔐 Example 3: Digest Comparison for Reproducibility\n");

    // Baseline span data
    let baseline_spans = r#"{"spans":[{"name":"test","span_id":"1"}]}"#;
    let baseline_digest = DigestReporter::compute_digest(baseline_spans);
    println!("Baseline digest: {}", baseline_digest);

    // Modified span data (different content)
    let modified_spans = r#"{"spans":[{"name":"test","span_id":"2"}]}"#;
    let modified_digest = DigestReporter::compute_digest(modified_spans);
    println!("Modified digest: {}", modified_digest);

    // Identical span data (same as baseline)
    let identical_spans = r#"{"spans":[{"name":"test","span_id":"1"}]}"#;
    let identical_digest = DigestReporter::compute_digest(identical_spans);
    println!("Identical digest: {}", identical_digest);
    println!();

    // Compare digests
    println!("Comparison Results:");
    if baseline_digest == identical_digest {
        println!("  ✓ Baseline == Identical (reproducible)");
    } else {
        println!("  ✗ Baseline != Identical (unexpected change)");
    }

    if baseline_digest == modified_digest {
        println!("  ✗ Baseline == Modified (should differ)");
    } else {
        println!("  ✓ Baseline != Modified (expected difference)");
    }
    println!();

    // Demonstrate whitespace sensitivity
    println!("Whitespace Sensitivity:");
    let spaces = r#"{"spans": []}"#;
    let no_spaces = r#"{"spans":[]}"#;
    let digest_spaces = DigestReporter::compute_digest(spaces);
    let digest_no_spaces = DigestReporter::compute_digest(no_spaces);

    if digest_spaces != digest_no_spaces {
        println!("  ✓ Digest is sensitive to whitespace");
        println!("    With spaces:    {}", digest_spaces);
        println!("    Without spaces: {}", digest_no_spaces);
    }

    Ok(())
}
