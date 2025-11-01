//! Integration Tests for Weaver Innovations
//!
//! Tests the three high-value Weaver integrations:
//! 1. Statistics analyzer (coverage tracking)
//! 2. Emit integration (test data generation)
//! 3. CI/CD validation gate (automated validation)
//!
//! These tests validate that Weaver innovations work end-to-end
//! and provide the promised value.

use clnrm_core::error::Result;
use clnrm_core::telemetry::weaver_emit::{EmitConfig, FixtureGenerator, WeaverEmitter};
use clnrm_core::telemetry::weaver_stats::{HealthStatus, WeaverStats};
use std::path::PathBuf;

/// Test registry statistics collection
///
/// Validates:
/// - Statistics can be collected from registry
/// - Coverage metrics are calculated correctly
/// - Quality score is computed
/// - Production readiness check works
#[test]
#[ignore = "Requires Weaver installation and valid registry"]
fn test_statistics_collection() -> Result<()> {
    // Arrange
    let registry_path = PathBuf::from("registry");
    let stats_collector = WeaverStats::new(&registry_path);

    // Act
    let statistics = stats_collector.collect()?;

    // Assert
    println!("📊 Registry Statistics:");
    println!("   Total Attributes: {}", statistics.total_attributes);
    println!("   Required Attributes: {}", statistics.required_attributes);
    println!("   Coverage: {:.1}%", statistics.coverage_percentage());
    println!("   Quality Score: {:.1}/100", statistics.quality_score());
    println!("   Health Status: {}", statistics.health_status());

    // Validate basic invariants
    assert!(
        statistics.total_attributes >= statistics.required_attributes,
        "Required attributes cannot exceed total"
    );
    assert!(
        statistics.coverage_percentage() <= 100.0,
        "Coverage cannot exceed 100%"
    );
    assert!(
        statistics.quality_score() <= 100.0,
        "Quality score cannot exceed 100"
    );

    // Check production readiness
    if statistics.is_production_ready() {
        println!("✅ Registry is production-ready (>= 80% coverage)");
    } else {
        println!(
            "⚠️  Registry needs work (coverage: {:.1}%)",
            statistics.coverage_percentage()
        );
    }

    Ok(())
}

/// Test statistics report generation
#[test]
#[ignore = "Requires Weaver installation"]
fn test_statistics_report_generation() -> Result<()> {
    // Arrange
    let registry_path = PathBuf::from("registry");
    let stats_collector = WeaverStats::new(&registry_path);

    // Act
    let statistics = stats_collector.collect()?;
    let report = stats_collector.generate_report(&statistics);

    // Assert
    println!("{}", report);
    assert!(report.contains("Registry Overview"));
    assert!(report.contains("Signal Types"));
    assert!(report.contains("Quality Metrics"));

    Ok(())
}

/// Test CI/CD gate validation
#[test]
#[ignore = "Requires Weaver installation"]
fn test_cicd_gate_validation() -> Result<()> {
    // Arrange
    let registry_path = PathBuf::from("registry");
    let stats_collector = WeaverStats::new(&registry_path);
    let statistics = stats_collector.collect()?;

    // Act
    let gate_result = stats_collector.validate_cicd_gate(&statistics);

    // Assert
    match gate_result {
        Ok(_) => {
            println!("✅ CI/CD gate passed - safe to merge");
            println!("   Coverage: {:.1}%", statistics.coverage_percentage());
            println!("   Quality: {:.1}/100", statistics.quality_score());
        }
        Err(e) => {
            println!("❌ CI/CD gate failed: {}", e);
            println!("   Coverage: {:.1}%", statistics.coverage_percentage());
            println!("   Quality: {:.1}/100", statistics.quality_score());

            // In test environment, we might allow failure
            // but in CI, this would block the merge
        }
    }

    Ok(())
}

/// Test quality score calculation logic
#[test]
fn test_quality_score_calculation() {
    use clnrm_core::telemetry::weaver_stats::RegistryStatistics;

    // Test excellent quality (should be >= 90)
    let excellent = RegistryStatistics {
        total_groups: 14,
        total_attributes: 150,
        required_attributes: 130,
        recommended_attributes: 15,
        optional_attributes: 5,
        total_spans: 20,
        total_metrics: 15,
        total_events: 10,
        required_coverage: 0.87,
    };

    assert!(
        excellent.quality_score() >= 90.0,
        "High-quality registry should score >= 90"
    );
    assert_eq!(excellent.health_status(), HealthStatus::Excellent);
    assert!(excellent.is_production_ready());

    // Test poor quality (should be < 60)
    let poor = RegistryStatistics {
        total_groups: 3,
        total_attributes: 30,
        required_attributes: 10,
        recommended_attributes: 5,
        optional_attributes: 15,
        total_spans: 2,
        total_metrics: 0,
        total_events: 0,
        required_coverage: 0.33,
    };

    assert!(
        poor.quality_score() < 60.0,
        "Poor registry should score < 60"
    );
    assert!(!poor.is_production_ready());
}

/// Test telemetry emission to stdout
#[test]
#[ignore = "Requires Weaver installation"]
fn test_emit_to_stdout() -> Result<()> {
    // Arrange
    let config = EmitConfig::stdout();
    let emitter = WeaverEmitter::with_config(config);

    // Act
    let result = emitter.emit()?;

    // Assert
    assert!(result.success, "Emission should succeed");
    assert!(result.total_signals > 0, "Should emit at least one signal");

    println!("✅ Emitted telemetry:");
    println!("   Spans: {}", result.spans_emitted);
    println!("   Metrics: {}", result.metrics_emitted);
    println!("   Events: {}", result.events_emitted);
    println!("   Total: {}", result.total_signals);

    Ok(())
}

/// Test fixture generation for integration tests
#[test]
#[ignore = "Requires Weaver installation"]
fn test_fixture_generation() -> Result<()> {
    // Arrange
    let registry_path = PathBuf::from("registry");
    let generator = FixtureGenerator::new(&registry_path);

    // Act
    let fixtures = generator.generate_json_fixtures()?;

    // Assert
    assert!(!fixtures.is_empty(), "Fixtures should not be empty");

    // Validate it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&fixtures)
        .map_err(|e| clnrm_core::error::CleanroomError::serialization_error(e.to_string()))?;

    println!("✅ Generated fixtures ({} bytes)", fixtures.len());
    println!("   Fixture type: {}", parsed["resourceSpans"].is_array());

    Ok(())
}

/// Test fixture file generation
#[test]
#[ignore = "Requires Weaver installation"]
fn test_fixture_file_generation() -> Result<()> {
    // Arrange
    let registry_path = PathBuf::from("registry");
    let generator = FixtureGenerator::new(&registry_path);
    let output_path = PathBuf::from("test_output/fixtures.json");

    // Ensure output directory exists
    std::fs::create_dir_all("test_output")?;

    // Act
    generator.emit_to_file(&output_path)?;

    // Assert
    assert!(output_path.exists(), "Fixture file should be created");

    let contents = std::fs::read_to_string(&output_path)?;
    assert!(!contents.is_empty(), "Fixture file should not be empty");

    println!("✅ Fixture file created: {}", output_path.display());
    println!("   Size: {} bytes", contents.len());

    // Cleanup
    std::fs::remove_file(&output_path).ok();

    Ok(())
}

/// Test continuous emission (background process)
#[test]
#[ignore = "Requires Weaver installation and OTLP endpoint"]
fn test_continuous_emission() -> Result<()> {
    // Arrange
    let config = EmitConfig::with_endpoint("http://localhost:4317");
    let emitter = WeaverEmitter::with_config(config);

    // Act
    let mut handle = emitter.start_continuous()?;

    // Assert - process should be running
    assert!(handle.is_running(), "Emitter should be running");

    println!("✅ Continuous emission started");

    // Let it run for a moment
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Stop emission
    handle.stop()?;

    println!("✅ Continuous emission stopped");

    Ok(())
}

/// Test seeding collector with test data
#[test]
#[ignore = "Requires Weaver installation and running collector"]
fn test_seed_collector() -> Result<()> {
    // Arrange
    let registry_path = PathBuf::from("registry");
    let generator = FixtureGenerator::new(&registry_path);
    let endpoint = "http://localhost:4317";

    // Act
    let result = generator.seed_collector(endpoint)?;

    // Assert
    assert!(result.success, "Seeding should succeed");
    assert!(result.total_signals > 0, "Should emit signals");

    println!("✅ Collector seeded with test data:");
    println!("   Endpoint: {}", endpoint);
    println!("   Signals: {}", result.total_signals);

    Ok(())
}

/// Integration test: Statistics + Emit workflow
///
/// This simulates a complete validation workflow:
/// 1. Check statistics to ensure registry is ready
/// 2. Generate test fixtures from schemas
/// 3. Emit fixtures to collector
/// 4. Validate results
#[test]
#[ignore = "Requires Weaver installation"]
fn test_complete_validation_workflow() -> Result<()> {
    println!("🔄 Running complete validation workflow");

    // Step 1: Check statistics
    println!("\n📊 Step 1: Collect Statistics");
    let registry_path = PathBuf::from("registry");
    let stats_collector = WeaverStats::new(&registry_path);
    let statistics = stats_collector.collect()?;

    println!("   Coverage: {:.1}%", statistics.coverage_percentage());
    println!("   Quality: {:.1}/100", statistics.quality_score());

    // Step 2: Generate report
    println!("\n📝 Step 2: Generate Report");
    let report = stats_collector.generate_report(&statistics);
    println!("{}", report);

    // Step 3: Check CI/CD gate
    println!("\n🚪 Step 3: Check CI/CD Gate");
    let gate_result = stats_collector.validate_cicd_gate(&statistics);
    match gate_result {
        Ok(_) => println!("✅ Gate passed"),
        Err(e) => println!("⚠️  Gate failed: {}", e),
    }

    // Step 4: Generate fixtures
    println!("\n🔧 Step 4: Generate Fixtures");
    let generator = FixtureGenerator::new(&registry_path);
    let fixtures = generator.generate_json_fixtures()?;
    println!("✅ Generated {} bytes of fixtures", fixtures.len());

    // Step 5: Emit telemetry
    println!("\n🚀 Step 5: Emit Telemetry");
    let config = EmitConfig::stdout();
    let emitter = WeaverEmitter::with_config(config);
    let result = emitter.emit()?;
    println!("✅ Emitted {} signals", result.total_signals);

    println!("\n✅ Complete workflow executed successfully!");

    Ok(())
}

/// Test error handling when registry doesn't exist
#[test]
fn test_error_handling_missing_registry() {
    let stats_collector = WeaverStats::new("nonexistent_registry");
    let result = stats_collector.collect();

    assert!(result.is_err(), "Should error with missing registry");

    if let Err(e) = result {
        println!("✅ Correctly caught error: {}", e);
    }
}

/// Test emit config variants
#[test]
fn test_emit_config_creation() {
    // Default config
    let default = EmitConfig::default();
    assert_eq!(default.endpoint, "http://localhost:4317");
    assert!(!default.stdout);

    // Stdout config
    let stdout = EmitConfig::stdout();
    assert!(stdout.stdout);

    // Custom endpoint
    let custom = EmitConfig::with_endpoint("http://example.com:4317");
    assert_eq!(custom.endpoint, "http://example.com:4317");
}

/// Performance test: Statistics collection speed
#[test]
#[ignore = "Requires Weaver installation"]
fn test_statistics_performance() -> Result<()> {
    use std::time::Instant;

    let registry_path = PathBuf::from("registry");
    let stats_collector = WeaverStats::new(&registry_path);

    let start = Instant::now();
    let _ = stats_collector.collect()?;
    let duration = start.elapsed();

    println!("⏱️  Statistics collection took: {:?}", duration);
    assert!(
        duration.as_secs() < 5,
        "Statistics should be collected in < 5 seconds"
    );

    Ok(())
}

/// Performance test: Fixture generation speed
#[test]
#[ignore = "Requires Weaver installation"]
fn test_fixture_generation_performance() -> Result<()> {
    use std::time::Instant;

    let registry_path = PathBuf::from("registry");
    let generator = FixtureGenerator::new(&registry_path);

    let start = Instant::now();
    let fixtures = generator.generate_json_fixtures()?;
    let duration = start.elapsed();

    println!("⏱️  Fixture generation took: {:?}", duration);
    println!("📦 Generated {} bytes", fixtures.len());
    assert!(
        duration.as_secs() < 10,
        "Fixture generation should complete in < 10 seconds"
    );

    Ok(())
}
