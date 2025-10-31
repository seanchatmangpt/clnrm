//! CLI Telemetry Helpers
//!
//! Provides builder pattern helpers for emitting CLI command telemetry
//! that conforms to registry schemas.

use tracing::{info_span, Span};
use std::time::Instant;

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
        self.span.record("plugins.discovered", plugins_discovered as i64);
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

impl CliHealthSpan {
    pub fn finish(
        self,
        success: bool,
        overall: &str,
        checks_total: usize,
        checks_passed: usize,
        checks_failed: usize,
        docker_available: bool,
        docker_version: Option<String>,
        docker_type: Option<String>,
        weaver_available: bool,
        weaver_version: Option<String>,
        error: Option<(String, String)>,
    ) {
        let duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        let _enter = self.span.enter();

        // Required attributes
        self.span.record("operation.success", success);
        self.span.record("health.overall", overall);
        self.span.record("health.checks_total", checks_total as i64);
        self.span.record("health.checks_passed", checks_passed as i64);
        self.span.record("health.checks_failed", checks_failed as i64);
        self.span.record("docker.available", docker_available);
        self.span.record("operation.duration_ms", duration_ms);

        // Recommended attributes
        if let Some(version) = docker_version {
            self.span.record("docker.version", version.as_str());
        }
        if let Some(dtype) = docker_type {
            self.span.record("docker.type", dtype.as_str());
        }
        self.span.record("weaver.available", weaver_available);
        if let Some(version) = weaver_version {
            self.span.record("weaver.version", version.as_str());
        }

        // Conditional error attributes
        if let Some((error_type, error_message)) = error {
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
