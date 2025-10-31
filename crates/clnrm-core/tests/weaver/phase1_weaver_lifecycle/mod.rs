//! Phase 1: WeaverController Lifecycle Tests
//!
//! These tests verify WeaverController process management using mocks.
//! Focus: Startup, shutdown, port discovery, coordination, health checks.

mod test_startup;
mod test_shutdown;
mod test_port_discovery;
mod test_coordination;
mod test_health_check;
mod test_failure_modes;
