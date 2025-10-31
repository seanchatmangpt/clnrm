//! Phase 4: End-to-End Docker Validation Tests
//!
//! These are REAL integration tests that require Docker and Weaver installation.
//! They verify the complete flow: Weaver → OTEL → Real Docker → Validation.
//!
//! **All tests in this module require:**
//! - Docker daemon running
//! - Weaver binary installed
//! - Network connectivity
//!
//! Tests are marked `#[ignore]` to prevent CI failures when dependencies unavailable.

mod test_docker_container_lifecycle;
mod test_docker_plugin_execution;
mod test_docker_isolation_proof;
mod test_docker_cleanup_verification;
mod test_docker_weaver_validation;
