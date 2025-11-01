//! Test Helper Modules
//!
//! Reusable utilities for integration testing.

pub mod telemetry_assertions;
pub mod test_config;
pub mod weaver_mock;

// Re-export commonly used items
pub use telemetry_assertions::{
    assert_coverage_above, assert_no_violations, assert_validation_failed,
    assert_validation_passed, ConformanceReportBuilder, ViolationCounts,
};

pub use test_config::{
    disabled_live_check_config, eighty_twenty_config, minimal_live_check_config,
    multi_service_config, strict_validation_config, TestConfigBuilder, TestStep,
};

pub use weaver_mock::{
    create_mock_report, MockValidationReport, MockValidationStatus, MockWeaverManager,
    MockWeaverProcess, TelemetrySample,
};
