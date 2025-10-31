//! Phase 2: Coordination Pattern Tests
//!
//! These tests verify Weaver-first initialization order and coordination between
//! Weaver, OTEL, and test execution.
//!
//! Key patterns tested:
//! - Weaver starts BEFORE OTEL initialization
//! - OTEL receives Weaver's actual port (not hardcoded)
//! - Tests wait for Weaver ready before emitting telemetry
//! - OTEL flushes BEFORE Weaver stops
//! - Orphan process cleanup

mod test_weaver_first_order;
mod test_port_handoff;
mod test_ready_wait;
mod test_shutdown_order;
mod test_orphan_cleanup;
