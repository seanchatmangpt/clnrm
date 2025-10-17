# Error Types for Template Operations - v0.6.0

**API Architect Report - Architecture Sub-Coordinator**

## Overview

Comprehensive error types for template rendering, validation, and plugin operations. All error types follow core team standards with structured context and error chaining.

## Template Error Types

### Enhanced `ErrorKind` Variants

```rust
/// Error kinds for different failure scenarios
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    // ... existing variants ...

    /// Template rendering error
    TemplateError,
    /// Template validation error
    TemplateValidationError,
    /// Template plugin error
    TemplatePluginError,
    /// Template context error
    TemplateContextError,
    /// Template function error
    TemplateFunctionError,
    /// Template filter error
    TemplateFilterError,
}
```

### Template-Specific Error Constructors

```rust
impl CleanroomError {
    /// Create a template rendering error
    ///
    /// # Example
    /// ```rust
    /// CleanroomError::template_error("Failed to render variable")
    ///     .with_context("Template: test.toml.tera")
    ///     .with_source("Variable 'name' not found in context")
    /// ```
    pub fn template_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateError, message)
    }

    /// Create a template validation error
    ///
    /// # Example
    /// ```rust
    /// CleanroomError::template_validation_error("Missing required section")
    ///     .with_context("Validator: schema")
    ///     .with_source("Expected [test.metadata] section")
    /// ```
    pub fn template_validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateValidationError, message)
    }

    /// Create a template plugin error
    ///
    /// # Example
    /// ```rust
    /// CleanroomError::template_plugin_error("Plugin registration failed")
    ///     .with_context("Plugin: custom-validator")
    ///     .with_source("Plugin with same name already registered")
    /// ```
    pub fn template_plugin_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplatePluginError, message)
    }

    /// Create a template context error
    ///
    /// # Example
    /// ```rust
    /// CleanroomError::template_context_error("Invalid context variable")
    ///     .with_context("Variable: matrix.version")
    ///     .with_source("Expected string, got number")
    /// ```
    pub fn template_context_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateContextError, message)
    }

    /// Create a template function error
    ///
    /// # Example
    /// ```rust
    /// CleanroomError::template_function_error("Function call failed")
    ///     .with_context("Function: env")
    ///     .with_source("Environment variable 'HOME' not found")
    /// ```
    pub fn template_function_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateFunctionError, message)
    }

    /// Create a template filter error
    ///
    /// # Example
    /// ```rust
    /// CleanroomError::template_filter_error("Filter execution failed")
    ///     .with_context("Filter: toml_encode")
    ///     .with_source("Cannot encode null value")
    /// ```
    pub fn template_filter_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateFilterError, message)
    }
}
```

## Validation Error Types

### Structured Validation Errors

```rust
/// Validation error with detailed context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Validation rule that was violated
    pub rule_id: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Location in template/output
    pub location: Option<SourceLocation>,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Related errors (for grouped failures)
    pub related: Vec<String>,
}

/// Source location for error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path (if applicable)
    pub file: Option<String>,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Code snippet at location
    pub snippet: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: impl Into<String>, rule_id: impl Into<String>, severity: ValidationSeverity) -> Self {
        Self {
            message: message.into(),
            rule_id: rule_id.into(),
            severity,
            location: None,
            suggestion: None,
            related: Vec::new(),
        }
    }

    /// Add source location
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add related error
    pub fn with_related(mut self, related: impl Into<String>) -> Self {
        self.related.push(related.into());
        self
    }

    /// Convert to CleanroomError
    pub fn into_cleanroom_error(self) -> CleanroomError {
        let mut err = CleanroomError::template_validation_error(&self.message)
            .with_context(format!("Rule: {}", self.rule_id));

        if let Some(suggestion) = self.suggestion {
            err = err.with_context(format!("Suggestion: {}", suggestion));
        }

        if let Some(location) = self.location {
            err = err.with_context(format!(
                "Location: {}:{}",
                location.line, location.column
            ));
        }

        err
    }
}
```

## Plugin Error Types

### Plugin Registration Errors

```rust
/// Plugin-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Plugin already registered
    AlreadyRegistered { name: String },
    /// Plugin not found
    NotFound { name: String },
    /// Plugin initialization failed
    InitializationFailed { name: String, reason: String },
    /// Plugin function registration failed
    FunctionRegistrationFailed { plugin: String, function: String, reason: String },
    /// Plugin filter registration failed
    FilterRegistrationFailed { plugin: String, filter: String, reason: String },
    /// Plugin context provision failed
    ContextProvisionFailed { plugin: String, reason: String },
    /// Plugin validation failed
    ValidationFailed { plugin: String, reason: String },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::AlreadyRegistered { name } => {
                write!(f, "Plugin '{}' is already registered", name)
            }
            PluginError::NotFound { name } => {
                write!(f, "Plugin '{}' not found", name)
            }
            PluginError::InitializationFailed { name, reason } => {
                write!(f, "Plugin '{}' initialization failed: {}", name, reason)
            }
            PluginError::FunctionRegistrationFailed { plugin, function, reason } => {
                write!(f, "Failed to register function '{}' from plugin '{}': {}", function, plugin, reason)
            }
            PluginError::FilterRegistrationFailed { plugin, filter, reason } => {
                write!(f, "Failed to register filter '{}' from plugin '{}': {}", filter, plugin, reason)
            }
            PluginError::ContextProvisionFailed { plugin, reason } => {
                write!(f, "Plugin '{}' failed to provide context: {}", plugin, reason)
            }
            PluginError::ValidationFailed { plugin, reason } => {
                write!(f, "Plugin '{}' validation failed: {}", plugin, reason)
            }
        }
    }
}

impl StdError for PluginError {}

impl From<PluginError> for CleanroomError {
    fn from(err: PluginError) -> Self {
        CleanroomError::template_plugin_error(err.to_string())
    }
}
```

## OTEL Validation Error Types

### Enhanced OTEL Error Variants

```rust
/// OTEL validation specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtelValidationError {
    /// Span not found
    SpanNotFound { span_name: String, trace_id: Option<String> },
    /// Span attribute missing
    AttributeMissing { span_name: String, attribute: String },
    /// Span attribute type mismatch
    AttributeTypeMismatch { span_name: String, attribute: String, expected: String, actual: String },
    /// Span duration out of bounds
    DurationOutOfBounds { span_name: String, actual_ms: f64, min_ms: Option<f64>, max_ms: Option<f64> },
    /// Trace incomplete
    TraceIncomplete { trace_id: String, expected_spans: usize, actual_spans: usize },
    /// Parent-child relationship violation
    RelationshipViolation { parent: String, expected_child: String },
    /// Export validation failed
    ExportFailed { endpoint: String, reason: String },
    /// Performance overhead exceeded
    PerformanceOverhead { actual_ms: f64, max_allowed_ms: f64 },
}

impl fmt::Display for OtelValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OtelValidationError::SpanNotFound { span_name, trace_id } => {
                if let Some(tid) = trace_id {
                    write!(f, "Span '{}' not found in trace '{}'", span_name, tid)
                } else {
                    write!(f, "Span '{}' not found", span_name)
                }
            }
            OtelValidationError::AttributeMissing { span_name, attribute } => {
                write!(f, "Span '{}' missing required attribute '{}'", span_name, attribute)
            }
            OtelValidationError::AttributeTypeMismatch { span_name, attribute, expected, actual } => {
                write!(f, "Span '{}' attribute '{}' type mismatch: expected {}, got {}",
                       span_name, attribute, expected, actual)
            }
            OtelValidationError::DurationOutOfBounds { span_name, actual_ms, min_ms, max_ms } => {
                write!(f, "Span '{}' duration {}ms out of bounds (min: {:?}, max: {:?})",
                       span_name, actual_ms, min_ms, max_ms)
            }
            OtelValidationError::TraceIncomplete { trace_id, expected_spans, actual_spans } => {
                write!(f, "Trace '{}' incomplete: expected {} spans, found {}",
                       trace_id, expected_spans, actual_spans)
            }
            OtelValidationError::RelationshipViolation { parent, expected_child } => {
                write!(f, "Parent span '{}' missing expected child '{}'", parent, expected_child)
            }
            OtelValidationError::ExportFailed { endpoint, reason } => {
                write!(f, "OTLP export to '{}' failed: {}", endpoint, reason)
            }
            OtelValidationError::PerformanceOverhead { actual_ms, max_allowed_ms } => {
                write!(f, "Performance overhead {}ms exceeds maximum allowed {}ms",
                       actual_ms, max_allowed_ms)
            }
        }
    }
}

impl StdError for OtelValidationError {}

impl From<OtelValidationError> for CleanroomError {
    fn from(err: OtelValidationError) -> Self {
        CleanroomError::validation_error(err.to_string())
            .with_context("OTEL Validation")
    }
}
```

## Error Conversion and Chaining

### Tera Error Conversion

```rust
/// Convert Tera errors to CleanroomError
impl From<tera::Error> for CleanroomError {
    fn from(err: tera::Error) -> Self {
        // Extract meaningful context from Tera error
        let message = match err.kind {
            tera::ErrorKind::Msg(ref msg) => msg.clone(),
            tera::ErrorKind::TemplateNotFound(ref name) => {
                format!("Template '{}' not found", name)
            }
            tera::ErrorKind::FilterNotFound(ref name) => {
                format!("Filter '{}' not found", name)
            }
            tera::ErrorKind::FunctionNotFound(ref name) => {
                format!("Function '{}' not found", name)
            }
            _ => err.to_string(),
        };

        CleanroomError::template_error(message)
            .with_source(err.to_string())
    }
}
```

### TOML Error Conversion

```rust
/// Convert TOML parsing errors to CleanroomError
impl From<toml::de::Error> for CleanroomError {
    fn from(err: toml::de::Error) -> Self {
        CleanroomError::configuration_error(format!("TOML parsing failed: {}", err))
            .with_source(err.to_string())
    }
}

/// Convert TOML serialization errors to CleanroomError
impl From<toml::ser::Error> for CleanroomError {
    fn from(err: toml::ser::Error) -> Self {
        CleanroomError::serialization_error(format!("TOML serialization failed: {}", err))
            .with_source(err.to_string())
    }
}
```

## Error Reporting and Formatting

### Structured Error Output

```rust
/// Error reporter for user-friendly error display
pub struct ErrorReporter;

impl ErrorReporter {
    /// Format error for CLI output
    pub fn format_for_cli(error: &CleanroomError) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("❌ {:?}: {}\n", error.kind, error.message));

        // Context
        if let Some(ref context) = error.context {
            output.push_str(&format!("   Context: {}\n", context));
        }

        // Source
        if let Some(ref source) = error.source {
            output.push_str(&format!("   Source: {}\n", source));
        }

        // Timestamp
        output.push_str(&format!("   Time: {}\n", error.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));

        output
    }

    /// Format validation report for CLI
    pub fn format_validation_report(report: &ValidationReport) -> String {
        let mut output = String::new();

        // Report header
        let status = if report.passed { "✅ PASSED" } else { "❌ FAILED" };
        output.push_str(&format!("{} - Validator: {}\n", status, report.validator_name));

        // Findings
        if !report.findings.is_empty() {
            output.push_str("\nFindings:\n");
            for finding in &report.findings {
                let icon = match finding.severity {
                    ValidationSeverity::Error => "❌",
                    ValidationSeverity::Warning => "⚠️",
                    ValidationSeverity::Info => "ℹ️",
                };
                output.push_str(&format!(
                    "  {} [{}] {}\n",
                    icon, finding.rule_id, finding.message
                ));

                if let Some(ref suggestion) = finding.suggestion {
                    output.push_str(&format!("     💡 Suggestion: {}\n", suggestion));
                }

                if let Some(ref location) = finding.location {
                    output.push_str(&format!("     📍 Location: {}\n", location));
                }
            }
        }

        output
    }

    /// Format pipeline report for CLI
    pub fn format_pipeline_report(report: &PipelineReport) -> String {
        let mut output = String::new();

        // Overall status
        let status = if report.overall_passed {
            "✅ ALL VALIDATIONS PASSED"
        } else {
            "❌ VALIDATION FAILED"
        };
        output.push_str(&format!("{}\n\n", status));

        // Template validation
        if !report.template_reports.is_empty() {
            output.push_str("Template Validation:\n");
            output.push_str("─".repeat(60).as_str());
            output.push('\n');
            for template_report in &report.template_reports {
                output.push_str(&Self::format_validation_report(template_report));
                output.push('\n');
            }
        }

        // Output validation
        if !report.output_reports.is_empty() {
            output.push_str("\nOutput Validation:\n");
            output.push_str("─".repeat(60).as_str());
            output.push('\n');
            for output_report in &report.output_reports {
                output.push_str(&Self::format_validation_report(output_report));
                output.push('\n');
            }
        }

        output
    }
}
```

## Error Recovery Strategies

### Fallback Mechanisms

```rust
/// Error recovery trait for graceful degradation
pub trait ErrorRecovery {
    /// Attempt to recover from error
    fn try_recover(&self, error: &CleanroomError) -> Result<RecoveryAction>;
}

/// Recovery action to take
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the operation
    Retry,
    /// Use fallback value
    UseFallback(String),
    /// Skip and continue
    Skip,
    /// Fail immediately
    Fail,
}

/// Template error recovery
pub struct TemplateErrorRecovery;

impl ErrorRecovery for TemplateErrorRecovery {
    fn try_recover(&self, error: &CleanroomError) -> Result<RecoveryAction> {
        match error.kind {
            ErrorKind::TemplateFunctionError => {
                // If env() fails, use empty string
                if error.message.contains("Environment variable") {
                    tracing::warn!("Recovering from missing env var with empty string");
                    return Ok(RecoveryAction::UseFallback(String::new()));
                }
                Ok(RecoveryAction::Fail)
            }
            ErrorKind::TemplateValidationError => {
                // Validation errors should not be recovered
                Ok(RecoveryAction::Fail)
            }
            _ => Ok(RecoveryAction::Fail),
        }
    }
}
```

## Usage Examples

### Example 1: Template Error Handling

```rust
// Render template with comprehensive error handling
match renderer.render_str(template, "test.toml") {
    Ok(rendered) => rendered,
    Err(e) => {
        match e.kind {
            ErrorKind::TemplateError => {
                eprintln!("{}", ErrorReporter::format_for_cli(&e));
                return Err(e);
            }
            ErrorKind::TemplateValidationError => {
                // Try to provide helpful context
                eprintln!("Template validation failed. Check your .clnrm.toml.tera file.");
                eprintln!("{}", ErrorReporter::format_for_cli(&e));
                return Err(e);
            }
            _ => return Err(e),
        }
    }
}
```

### Example 2: Validation Error Aggregation

```rust
// Collect all validation errors
let mut all_errors = Vec::new();

for report in &pipeline_report.template_reports {
    for finding in &report.findings {
        if finding.severity == ValidationSeverity::Error {
            let validation_err = ValidationError::new(
                &finding.message,
                &finding.rule_id,
                finding.severity,
            )
            .with_suggestion(finding.suggestion.clone().unwrap_or_default());

            all_errors.push(validation_err);
        }
    }
}

if !all_errors.is_empty() {
    // Combine into single error with all context
    let combined = CleanroomError::template_validation_error(
        format!("{} validation errors found", all_errors.len())
    );
    for (i, err) in all_errors.iter().enumerate() {
        combined = combined.with_context(format!("Error {}: {}", i + 1, err.message));
    }
    return Err(combined);
}
```

## API Contract Guarantees

1. **Structured Errors**: All errors include context, source, and timestamp
2. **Error Chaining**: Errors preserve full error chain for debugging
3. **User-Friendly**: Error messages are actionable and clear
4. **Machine-Parseable**: Errors are serializable for tooling integration
5. **Recovery Support**: Error types support recovery strategies
6. **No Information Loss**: All error details preserved through conversions
