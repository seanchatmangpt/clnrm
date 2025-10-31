//! Weaver Integration Tests - London TDD Approach
//!
//! This module contains mock-driven tests for Weaver integration following
//! London School TDD principles. Tests are organized in four phases:
//!
//! 1. **Phase 1**: WeaverController lifecycle (process management)
//! 2. **Phase 2**: Coordination patterns (Weaver-first initialization)
//! 3. **Phase 3**: OTEL integration (telemetry contract verification)
//! 4. **Phase 4**: End-to-end Docker validation (real integration)
//!
//! See LONDON_TDD_STRATEGY.md for complete testing strategy.

pub mod fixtures;
pub mod mocks;

// Phase 1: WeaverController lifecycle tests
pub mod phase1_weaver_lifecycle;

// Phase 2: Coordination pattern tests
pub mod phase2_coordination;

// Phase 3: OTEL integration tests
pub mod phase3_otel_integration;

// Phase 4: End-to-end Docker validation tests
pub mod phase4_e2e_docker;

// Legacy tests (to be refactored)
pub mod controller_tests;
pub mod mock_helpers;
pub mod schema_fixtures;
