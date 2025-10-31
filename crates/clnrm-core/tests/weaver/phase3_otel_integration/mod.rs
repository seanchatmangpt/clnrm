//! Phase 3: OTEL Integration Tests
//!
//! These tests verify telemetry contract compliance against schemas.
//! Uses mocks to capture and validate telemetry without actual OTLP export.

mod test_contract_test_execution;
mod test_contract_container_lifecycle;
mod test_contract_plugin_execution;
mod test_contract_events;
mod test_zero_samples_detection;
mod test_attribute_validation;
