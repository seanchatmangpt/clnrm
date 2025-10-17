//! CLI Integration Tests Module
//!
//! Comprehensive CLI testing following TDD London School (mockist) methodology.
//! Tests focus on behavior verification and interaction patterns.
//!
//! ## Test Organization
//!
//! - `init_command_test` - Tests `clnrm init` command behavior
//! - `validate_command_test` - Tests `clnrm validate` command behavior
//! - `run_command_test` - Tests `clnrm run` command behavior
//! - `plugins_command_test` - Tests `clnrm plugins` command behavior
//! - `health_command_test` - Tests `clnrm health` command behavior
//! - `error_handling_test` - Tests error message formatting and propagation
//!
//! ## Testing Approach
//!
//! These tests follow the London School TDD approach:
//! - **Outside-In**: Tests verify user-facing behavior first
//! - **Interaction Testing**: Focus on how CLI collaborates with system
//! - **Behavior Verification**: Assert on outcomes, not implementation
//! - **Mock Collaborators**: Use file system mocking via tempfiles
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all CLI integration tests
//! cargo test --test '*' -p clnrm
//!
//! # Run specific command tests
//! cargo test init_command -p clnrm
//! cargo test validate_command -p clnrm
//!
//! # Run with output
//! cargo test -p clnrm -- --nocapture
//! ```

mod error_handling_test;
mod health_command_test;
mod init_command_test;
mod plugins_command_test;
mod run_command_test;
mod validate_command_test;
