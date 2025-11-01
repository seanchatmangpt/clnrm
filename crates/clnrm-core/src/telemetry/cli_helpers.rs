//! CLI Telemetry Helpers
//!
//! Provides builder pattern helpers for emitting CLI command telemetry
//! that conforms to registry schemas.

use std::time::Instant;
use tracing::{info_span, Span};

/// Builder for CLI initialization span (clnrm init)
pub struct CliInitSpanBuilder {
    project_path: String,
    exists_before: bool,
    force_used: bool,
}

impl CliInitSpanBuilder {
    pub fn new(project_path: String, exists_before: bool, force_used: bool) -> Self {
        Self {
            project_path,
            exists_before,
            force_used,
        }
    }

    pub fn start(self) -> CliInitSpan {
        let span = info_span!(
            "clnrm.cli.init",
            cli.command = "init",
            cli.version = env!("CARGO_PKG_VERSION"),
            project.path = %self.project_path,
            project.exists_before = self.exists_before,
            force.used = self.force_used,
        );

        CliInitSpan {
            span,
            start_time: Instant::now(),
        }
    }
}

pub struct CliInitSpan {
    span: Span,
    start_time: Instant,
}

impl CliInitSpan {
    pub fn finish(
        self,
        success: bool,
        config_generated: bool,
        config_path: Option<String>,
        files_created: usize,
        error: Option<(String, String)>,
    ) {
        let duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        let _enter = self.span.enter();

        // Required attributes
        self.span.record("operation.success", success);
        self.span.record("config.generated", config_generated);
        self.span.record("operation.duration_ms", duration_ms);

        // Recommended attributes
        if let Some(path) = config_path {
            self.span.record("config.path", path.as_str());
        }
        self.span.record("files.created", files_created as i64);

        // Conditional error attributes
        if let Some((error_type, error_message)) = error {
            self.span.record("error.type", error_type.as_str());
            self.span.record("error.message", error_message.as_str());
        }
    }
}

/// Builder for CLI plugins span (clnrm plugins)
pub struct CliPluginsSpanBuilder;

impl Default for CliPluginsSpanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CliPluginsSpanBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn start(self) -> CliPluginsSpan {
        let span = info_span!(
            "clnrm.cli.plugins",
            cli.command = "plugins",
            cli.version = env!("CARGO_PKG_VERSION"),
        );

        CliPluginsSpan {
            span,
            start_time: Instant::now(),
        }
    }
}

pub struct CliPluginsSpan {
    span: Span,
    start_time: Instant,
}

impl CliPluginsSpan {
    pub fn finish(
        self,
        success: bool,
        plugins_discovered: usize,
        plugins_builtin: usize,
        plugins_custom: usize,
        plugins_by_type: Option<String>,
        error: Option<(String, String)>,
    ) {
        let duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        let _enter = self.span.enter();

        // Required attributes
        self.span.record("operation.success", success);
        self.span
            .record("plugins.discovered", plugins_discovered as i64);
        self.span.record("operation.duration_ms", duration_ms);

        // Recommended attributes
        self.span.record("plugins.builtin", plugins_builtin as i64);
        self.span.record("plugins.custom", plugins_custom as i64);

        if let Some(json) = plugins_by_type {
            self.span.record("plugins.by_type", json.as_str());
        }

        // Conditional error attributes
        if let Some((error_type, error_message)) = error {
            self.span.record("error.type", error_type.as_str());
            self.span.record("error.message", error_message.as_str());
        }
    }
}

/// Builder for CLI health span (clnrm health)
pub struct CliHealthSpanBuilder {
    verbose: bool,
}

impl CliHealthSpanBuilder {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn start(self) -> CliHealthSpan {
        let span = info_span!(
            "clnrm.cli.health",
            cli.command = "health",
            cli.version = env!("CARGO_PKG_VERSION"),
            verbose.enabled = self.verbose,
        );

        CliHealthSpan {
            span,
            start_time: Instant::now(),
        }
    }
}

pub struct CliHealthSpan {
    span: Span,
    start_time: Instant,
}

/// Parameters for finishing a health check span
#[derive(Debug, Default)]
pub struct HealthCheckResult {
    pub success: bool,
    pub overall: String,
    pub checks_total: usize,
    pub checks_passed: usize,
    pub checks_failed: usize,
    pub docker_available: bool,
    pub docker_version: Option<String>,
    pub docker_type: Option<String>,
    pub weaver_available: bool,
    pub weaver_version: Option<String>,
    pub error: Option<(String, String)>,
}

impl HealthCheckResult {
    pub fn builder() -> HealthCheckResultBuilder {
        HealthCheckResultBuilder::default()
    }
}

/// Builder for HealthCheckResult
#[derive(Debug, Default)]
pub struct HealthCheckResultBuilder {
    result: HealthCheckResult,
}

impl HealthCheckResultBuilder {
    pub fn success(mut self, success: bool) -> Self {
        self.result.success = success;
        self
    }

    pub fn overall(mut self, overall: impl Into<String>) -> Self {
        self.result.overall = overall.into();
        self
    }

    pub fn checks(mut self, total: usize, passed: usize, failed: usize) -> Self {
        self.result.checks_total = total;
        self.result.checks_passed = passed;
        self.result.checks_failed = failed;
        self
    }

    pub fn docker(
        mut self,
        available: bool,
        version: Option<String>,
        dtype: Option<String>,
    ) -> Self {
        self.result.docker_available = available;
        self.result.docker_version = version;
        self.result.docker_type = dtype;
        self
    }

    pub fn weaver(mut self, available: bool, version: Option<String>) -> Self {
        self.result.weaver_available = available;
        self.result.weaver_version = version;
        self
    }

    pub fn error(mut self, error_type: String, error_message: String) -> Self {
        self.result.error = Some((error_type, error_message));
        self
    }

    pub fn build(self) -> HealthCheckResult {
        self.result
    }
}

impl CliHealthSpan {
    /// Finish the health span with a single parameter object
    pub fn finish(self, result: HealthCheckResult) {
        let duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        let _enter = self.span.enter();

        // Required attributes
        self.span.record("operation.success", result.success);
        self.span.record("health.overall", result.overall.as_str());
        self.span
            .record("health.checks_total", result.checks_total as i64);
        self.span
            .record("health.checks_passed", result.checks_passed as i64);
        self.span
            .record("health.checks_failed", result.checks_failed as i64);
        self.span
            .record("docker.available", result.docker_available);
        self.span.record("operation.duration_ms", duration_ms);

        // Recommended attributes
        if let Some(version) = result.docker_version {
            self.span.record("docker.version", version.as_str());
        }
        if let Some(dtype) = result.docker_type {
            self.span.record("docker.type", dtype.as_str());
        }
        self.span
            .record("weaver.available", result.weaver_available);
        if let Some(version) = result.weaver_version {
            self.span.record("weaver.version", version.as_str());
        }

        // Conditional error attributes
        if let Some((error_type, error_message)) = result.error {
            self.span.record("error.type", error_type.as_str());
            self.span.record("error.message", error_message.as_str());
        }
    }
}

/// Builder for CLI self-test span (clnrm self-test)
pub struct CliSelfTestSpanBuilder {
    suite: Option<String>,
}

impl CliSelfTestSpanBuilder {
    pub fn new(suite: Option<String>) -> Self {
        Self { suite }
    }

    pub fn start(self) -> CliSelfTestSpan {
        let suite_name = self.suite.as_deref().unwrap_or("all");

        let span = info_span!(
            "clnrm.cli.self_test",
            cli.command = "self-test",
            cli.version = env!("CARGO_PKG_VERSION"),
            test.suite = suite_name,
        );

        CliSelfTestSpan {
            span,
            start_time: Instant::now(),
        }
    }
}

pub struct CliSelfTestSpan {
    span: Span,
    start_time: Instant,
}

impl CliSelfTestSpan {
    pub fn finish(
        self,
        success: bool,
        tests_total: usize,
        tests_passed: usize,
        tests_failed: usize,
        error: Option<(String, String)>,
    ) {
        let duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        let _enter = self.span.enter();

        // Required attributes
        self.span.record("operation.success", success);
        self.span.record("test.count", tests_total as i64);
        self.span.record("test.passed", tests_passed as i64);
        self.span.record("test.failed", tests_failed as i64);
        self.span.record("operation.duration_ms", duration_ms);

        // Conditional error attributes
        if let Some((error_type, error_message)) = error {
            self.span.record("error.type", error_type.as_str());
            self.span.record("error.message", error_message.as_str());
        }
    }
}
