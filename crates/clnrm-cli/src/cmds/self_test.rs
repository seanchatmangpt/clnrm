//! Self-test command implementation

use clnrm_core::error::Result;

/// Run the self-test command
pub async fn run(suite: Option<String>, report: bool, otel_exporter: String, otel_endpoint: Option<String>) -> Result<()> {
    crate::commands::run_self_tests(suite, report, otel_exporter, otel_endpoint).await
}