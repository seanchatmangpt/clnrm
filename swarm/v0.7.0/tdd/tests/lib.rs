/// v0.7.0 DX Features Test Suite
///
/// Comprehensive acceptance tests for developer experience improvements:
/// - dev --watch: File watching and auto-execution
/// - dry-run: Template validation without Docker
/// - fmt: Template formatting
/// - lint: Template linting
/// - diff: Trace comparison
///
/// All tests use London School TDD with mock infrastructure

pub mod mocks;
pub mod acceptance;
pub mod integration;
pub mod fixtures;

// Re-export commonly used types for convenience
pub use mocks::{
    MockFileWatcher, MockTemplateRenderer, MockTomlParser,
    MockFormatter, MockTraceDiffer
};
