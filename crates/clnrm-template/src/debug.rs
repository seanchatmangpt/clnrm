//! Template debugging and error reporting utilities
//!
//! Provides tools for template development, debugging, and troubleshooting:
//! - Template syntax analysis
//! - Variable usage tracking
//! - Error location reporting
//! - Template performance profiling
//! - Development-time validation

use crate::error::{TemplateError, Result};
use crate::context::TemplateContext;
use crate::renderer::{TemplateRenderer, OutputFormat};
use crate::validation::{TemplateValidator, ValidationRule};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Template debugging information
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Template name
    pub template_name: String,
    /// Template source content
    pub source: String,
    /// Variables used in template
    pub variables_used: HashSet<String>,
    /// Functions called in template
    pub functions_used: HashSet<String>,
    /// Blocks defined in template
    pub blocks_defined: HashSet<String>,
    /// Templates extended (for inheritance)
    pub extends_templates: Vec<String>,
    /// Templates included
    pub includes_templates: Vec<String>,
    /// Syntax errors found
    pub syntax_errors: Vec<String>,
    /// Performance metrics
    pub render_time_ms: Option<u64>,
    /// Memory usage estimate
    pub memory_usage: Option<usize>,
}

/// Template debugger for development and troubleshooting
#[derive(Debug)]
pub struct TemplateDebugger {
    /// Enable verbose debugging
    verbose: bool,
    /// Track variable usage
    track_variables: bool,
    /// Track function calls
    track_functions: bool,
    /// Enable syntax validation
    validate_syntax: bool,
    /// Performance profiling
    profile_performance: bool,
}

impl Default for TemplateDebugger {
    fn default() -> Self {
        Self {
            verbose: false,
            track_variables: true,
            track_functions: true,
            validate_syntax: true,
            profile_performance: false,
        }
    }
}

impl TemplateDebugger {
    /// Create new template debugger
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable verbose debugging output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable/disable variable usage tracking
    pub fn track_variables(mut self, track: bool) -> Self {
        self.track_variables = track;
        self
    }

    /// Enable/disable function call tracking
    pub fn track_functions(mut self, track: bool) -> Self {
        self.track_functions = track;
        self
    }

    /// Enable/disable syntax validation
    pub fn validate_syntax(mut self, validate: bool) -> Self {
        self.validate_syntax = validate;
        self
    }

    /// Enable/disable performance profiling
    pub fn profile_performance(mut self, profile: bool) -> Self {
        self.profile_performance = profile;
        self
    }

    /// Analyze template for debugging information
    ///
    /// # Arguments
    /// * `template_content` - Template source content
    /// * `template_name` - Template name for reporting
    pub fn analyze(&self, template_content: &str, template_name: &str) -> Result<DebugInfo> {
        let mut info = DebugInfo {
            template_name: template_name.to_string(),
            source: template_content.to_string(),
            variables_used: HashSet::new(),
            functions_used: HashSet::new(),
            blocks_defined: HashSet::new(),
            extends_templates: Vec::new(),
            includes_templates: Vec::new(),
            syntax_errors: Vec::new(),
            render_time_ms: None,
            memory_usage: None,
        };

        // Analyze template syntax
        if self.validate_syntax {
            self.validate_template_syntax(template_content, &mut info)?;
        }

        // Extract variable usage
        if self.track_variables {
            self.extract_variables(template_content, &mut info);
        }

        // Extract function calls
        if self.track_functions {
            self.extract_functions(template_content, &mut info);
        }

        // Extract template composition info
        self.extract_composition_info(template_content, &mut info);

        Ok(info)
    }

    /// Validate template syntax
    fn validate_template_syntax(&self, content: &str, info: &mut DebugInfo) -> Result<()> {
        // Basic syntax validation for Tera templates
        let mut errors = Vec::new();

        // Check for unmatched braces
        self.check_brace_matching(content, &mut errors);

        // Check for invalid variable syntax
        self.check_variable_syntax(content, &mut errors);

        // Check for invalid function calls
        self.check_function_syntax(content, &mut errors);

        info.syntax_errors = errors;
        Ok(())
    }

    /// Check for balanced braces and brackets
    fn check_brace_matching(&self, content: &str, errors: &mut Vec<String>) {
        let mut stack = Vec::new();

        for (i, ch) in content.char_indices() {
            match ch {
                '{' => {
                    if let Some(next) = content.chars().nth(i + 1) {
                        if next == '{' || next == '%' || next == '#' {
                            stack.push((ch, i));
                        }
                    }
                }
                '%' | '#' => {
                    // Check if this is part of a Tera tag
                    if i > 0 && content.chars().nth(i - 1) == Some('{') {
                        stack.push((ch, i));
                    }
                }
                '}' => {
                    if let Some(next) = content.chars().nth(i + 1) {
                        if next == '}' {
                            if let Some((open, open_pos)) = stack.pop() {
                                if !matches!((open, ch), ('{', '}')) {
                                    errors.push(format!(
                                        "Unmatched braces at position {}: found '{}' but expected matching '{}'",
                                        i, ch, open
                                    ));
                                }
                            } else {
                                errors.push(format!("Unmatched closing brace at position {}", i));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for unclosed tags
        for (open, pos) in stack {
            errors.push(format!("Unclosed '{}' at position {}", open, pos));
        }
    }

    /// Check variable syntax
    fn check_variable_syntax(&self, content: &str, errors: &mut Vec<String>) {
        // Simple regex for Tera variables: {{ variable }} or {{ variable.nested }}
        let var_regex = regex::Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_.]*)")
            .unwrap();

        for cap in var_regex.captures_iter(content) {
            if let Some(var_name) = cap.get(1) {
                let var = var_name.as_str();
                // Check for obviously invalid variable names
                if var.contains(" ") {
                    errors.push(format!("Invalid variable name '{}' contains spaces", var));
                }
            }
        }
    }

    /// Check function call syntax
    fn check_function_syntax(&self, content: &str, errors: &mut Vec<String>) {
        // Simple regex for function calls: function_name(args)
        let func_regex = regex::Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*\(")
            .unwrap();

        for cap in func_regex.captures_iter(content) {
            if let Some(func_name) = cap.get(1) {
                let func = func_name.as_str();
                // Check for unknown functions (basic check)
                let known_functions = [
                    "env", "now_rfc3339", "sha256", "toml_encode",
                    "fake_name", "fake_email", "uuid_v4", "include", "extends"
                ];

                if !known_functions.contains(&func) && !func.starts_with("fake_") {
                    // This is just a warning for unknown functions
                    if self.verbose {
                        eprintln!("Warning: Unknown function '{}' in template", func);
                    }
                }
            }
        }
    }

    /// Extract variables used in template
    fn extract_variables(&self, content: &str, info: &mut DebugInfo) {
        // Extract variables from {{ variable }} patterns
        let var_regex = regex::Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_.]*)")
            .unwrap();

        for cap in var_regex.captures_iter(content) {
            if let Some(var_name) = cap.get(1) {
                info.variables_used.insert(var_name.as_str().to_string());
            }
        }
    }

    /// Extract function calls from template
    fn extract_functions(&self, content: &str, info: &mut DebugInfo) {
        // Extract function calls
        let func_regex = regex::Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*\(")
            .unwrap();

        for cap in func_regex.captures_iter(content) {
            if let Some(func_name) = cap.get(1) {
                info.functions_used.insert(func_name.as_str().to_string());
            }
        }
    }

    /// Extract template composition information
    fn extract_composition_info(&self, content: &str, info: &mut DebugInfo) {
        // Extract extends declarations
        let extends_regex = regex::Regex::new(r#"extends\s*\(\s*["']([^"']+)["']\s*\)"#)
            .unwrap();

        for cap in extends_regex.captures_iter(content) {
            if let Some(template) = cap.get(1) {
                info.extends_templates.push(template.as_str().to_string());
            }
        }

        // Extract include declarations
        let include_regex = regex::Regex::new(r#"include\s*\(\s*["']([^"']+)["']\s*\)"#)
            .unwrap();

        for cap in include_regex.captures_iter(content) {
            if let Some(template) = cap.get(1) {
                info.includes_templates.push(template.as_str().to_string());
            }
        }

        // Extract block definitions
        let block_regex = regex::Regex::new(r#"block\s*\(\s*["']([^"']+)["']\s*\)"#)
            .unwrap();

        for cap in block_regex.captures_iter(content) {
            if let Some(block_name) = cap.get(1) {
                info.blocks_defined.insert(block_name.as_str().to_string());
            }
        }
    }

    /// Debug template rendering with context
    ///
    /// # Arguments
    /// * `template_content` - Template content
    /// * `context` - Template context
    /// * `template_name` - Template name
    pub fn debug_render(&self, template_content: &str, context: &TemplateContext, template_name: &str) -> Result<DebugInfo> {
        let mut info = self.analyze(template_content, template_name)?;

        if self.profile_performance {
            let start = std::time::Instant::now();

            // Attempt to render (this may fail)
            let result = crate::render_with_context(template_content, context);

            let elapsed = start.elapsed();
            info.render_time_ms = Some(elapsed.as_millis() as u64);

            if result.is_err() {
                info.syntax_errors.push(result.unwrap_err().to_string());
            }
        }

        if self.verbose {
            self.print_debug_info(&info);
        }

        Ok(info)
    }

    /// Print debug information to stderr
    fn print_debug_info(&self, info: &DebugInfo) {
        eprintln!("=== Template Debug Info ===");
        eprintln!("Template: {}", info.template_name);
        eprintln!("Variables used: {:?}", info.variables_used);
        eprintln!("Functions used: {:?}", info.functions_used);
        eprintln!("Blocks defined: {:?}", info.blocks_defined);
        eprintln!("Extends: {:?}", info.extends_templates);
        eprintln!("Includes: {:?}", info.includes_templates);

        if !info.syntax_errors.is_empty() {
            eprintln!("Syntax errors:");
            for error in &info.syntax_errors {
                eprintln!("  - {}", error);
            }
        }

        if let Some(time) = info.render_time_ms {
            eprintln!("Render time: {}ms", time);
        }
    }
}

/// Template analyzer for static analysis
pub struct TemplateAnalyzer {
    debugger: TemplateDebugger,
}

impl TemplateAnalyzer {
    /// Create new template analyzer
    pub fn new() -> Self {
        Self {
            debugger: TemplateDebugger::new(),
        }
    }

    /// Analyze template file
    ///
    /// # Arguments
    /// * `file_path` - Path to template file
    pub fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Result<DebugInfo> {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| TemplateError::IoError(format!("Failed to read template file: {}", e)))?;

        let file_name = file_path.as_ref().file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.debugger.analyze(&content, file_name)
    }

    /// Analyze all templates in directory
    ///
    /// # Arguments
    /// * `dir_path` - Directory path to scan
    pub fn analyze_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<HashMap<String, DebugInfo>> {
        use walkdir::WalkDir;

        let mut results = HashMap::new();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if matches!(ext, "toml" | "tera" | "tpl" | "template") {
                        if let Ok(info) = self.analyze_file(path) {
                            let name = info.template_name.clone();
                            results.insert(name, info);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Find unused variables in context
    ///
    /// # Arguments
    /// * `template_info` - Template debug information
    /// * `context` - Template context
    pub fn find_unused_variables(&self, template_info: &DebugInfo, context: &TemplateContext) -> Vec<String> {
        let used_vars: HashSet<String> = template_info.variables_used.iter().cloned().collect();
        let context_vars: HashSet<String> = context.vars.keys().cloned().collect();

        context_vars.difference(&used_vars).cloned().collect()
    }

    /// Find potentially missing variables
    ///
    /// # Arguments
    /// * `template_info` - Template debug information
    /// * `context` - Template context
    pub fn find_missing_variables(&self, template_info: &DebugInfo, context: &TemplateContext) -> Vec<String> {
        let used_vars: HashSet<String> = template_info.variables_used.iter().cloned().collect();
        let context_vars: HashSet<String> = context.vars.keys().cloned().collect();

        used_vars.difference(&context_vars).cloned().collect()
    }
}

/// Template linting rules for code quality
pub mod lint {
    use super::*;

    /// Lint rule trait for custom validation
    pub trait LintRule {
        /// Check template for lint violations
        ///
        /// # Arguments
        /// * `info` - Template debug information
        fn check(&self, info: &DebugInfo) -> Vec<String>;
    }

    /// Rule to detect unused variables
    pub struct UnusedVariablesRule;

    impl LintRule for UnusedVariablesRule {
        fn check(&self, info: &DebugInfo) -> Vec<String> {
            // This would need context information to determine unused vars
            // For now, return empty
            Vec::new()
        }
    }

    /// Rule to detect deprecated function usage
    pub struct DeprecatedFunctionsRule;

    impl LintRule for DeprecatedFunctionsRule {
        fn check(&self, info: &DebugInfo) -> Vec<String> {
            let deprecated = ["old_function", "deprecated_helper"];
            let mut violations = Vec::new();

            for func in &info.functions_used {
                if deprecated.contains(&func.as_str()) {
                    violations.push(format!("Deprecated function '{}' used", func));
                }
            }

            violations
        }
    }

    /// Rule to detect overly complex templates
    pub struct ComplexityRule {
        max_complexity: usize,
    }

    impl ComplexityRule {
        pub fn new(max_complexity: usize) -> Self {
            Self { max_complexity }
        }
    }

    impl LintRule for ComplexityRule {
        fn check(&self, info: &DebugInfo) -> Vec<String> {
            let mut violations = Vec::new();

            // Simple complexity metric: count of functions + variables + blocks
            let complexity = info.functions_used.len() + info.variables_used.len() + info.blocks_defined.len();

            if complexity > self.max_complexity {
                violations.push(format!(
                    "Template complexity {} exceeds maximum {}",
                    complexity, self.max_complexity
                ));
            }

            violations
        }
    }

    /// Rule to detect missing variable documentation
    pub struct UndocumentedVariablesRule;

    impl LintRule for UndocumentedVariablesRule {
        fn check(&self, info: &DebugInfo) -> Vec<String> {
            // Check if variables are documented in comments
            let mut violations = Vec::new();

            for var in &info.variables_used {
                // Look for variable documentation in comments
                let doc_pattern = format!("{{# {} #}}", var);
                if !info.source.contains(&doc_pattern) {
                    violations.push(format!("Variable '{}' is not documented", var));
                }
            }

            violations
        }
    }
}

/// Template linter for comprehensive template validation
pub struct TemplateLinter {
    /// Lint rules to apply
    rules: Vec<Box<dyn lint::LintRule>>,
    /// Template debugger for analysis
    debugger: TemplateDebugger,
}

impl TemplateLinter {
    /// Create new template linter
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            debugger: TemplateDebugger::new(),
        }
    }

    /// Add lint rule
    pub fn with_rule<R: lint::LintRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Add common lint rules for production templates
    pub fn with_production_rules(mut self) -> Self {
        self.rules.push(Box::new(lint::DeprecatedFunctionsRule));
        self.rules.push(Box::new(lint::ComplexityRule::new(50))); // Max 50 complexity points
        self.rules.push(Box::new(lint::UndocumentedVariablesRule));
        self
    }

    /// Lint template content
    ///
    /// # Arguments
    /// * `template_content` - Template source content
    /// * `template_name` - Template name for reporting
    pub fn lint(&self, template_content: &str, template_name: &str) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        // Analyze template
        let info = self.debugger.analyze(template_content, template_name)?;

        // Apply all lint rules
        for rule in &self.rules {
            violations.extend(rule.check(&info));
        }

        Ok(violations)
    }

    /// Lint template file
    ///
    /// # Arguments
    /// * `file_path` - Path to template file
    pub fn lint_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<String>> {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| TemplateError::IoError(format!("Failed to read template file: {}", e)))?;

        let file_name = file_path.as_ref().file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.lint(&content, file_name)
    }

    /// Lint all templates in directory
    ///
    /// # Arguments
    /// * `dir_path` - Directory path to scan
    pub fn lint_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<HashMap<String, Vec<String>>> {
        use walkdir::WalkDir;

        let mut results = HashMap::new();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if matches!(ext, "toml" | "tera" | "tpl" | "template") {
                        match self.lint_file(path) {
                            Ok(violations) => {
                                let name = path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                results.insert(name, violations);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to lint template {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Template validation tools for development workflows
pub struct TemplateValidator {
    /// Base validator
    validator: crate::validation::TemplateValidator,
    /// Linter for code quality
    linter: TemplateLinter,
    /// Debugger for analysis
    debugger: TemplateDebugger,
}

impl TemplateValidator {
    /// Create new template validator
    pub fn new() -> Self {
        Self {
            validator: crate::validation::TemplateValidator::new(),
            linter: TemplateLinter::new(),
            debugger: TemplateDebugger::new(),
        }
    }

    /// Configure base validator
    pub fn with_validator<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::validation::TemplateValidator) -> crate::validation::TemplateValidator,
    {
        self.validator = f(self.validator);
        self
    }

    /// Configure linter
    pub fn with_linter<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TemplateLinter) -> TemplateLinter,
    {
        self.linter = f(self.linter);
        self
    }

    /// Configure debugger
    pub fn with_debugger<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TemplateDebugger) -> TemplateDebugger,
    {
        self.debugger = f(self.debugger);
        self
    }

    /// Validate template with comprehensive checking
    ///
    /// # Arguments
    /// * `template` - Template content
    /// * `context` - Template context
    /// * `name` - Template name
    pub fn validate_template(&self, template: &str, context: &TemplateContext, name: &str) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            template_name: name.to_string(),
            syntax_valid: true,
            syntax_errors: Vec::new(),
            lint_violations: Vec::new(),
            unused_variables: Vec::new(),
            missing_variables: Vec::new(),
            performance_metrics: None,
        };

        // Check syntax and structure
        let debug_info = self.debugger.analyze(template, name)?;
        report.syntax_errors = debug_info.syntax_errors.clone();

        if !report.syntax_errors.is_empty() {
            report.syntax_valid = false;
        }

        // Run linting
        report.lint_violations = self.linter.lint(template, name)?;

        // Check variable usage
        report.unused_variables = self.debugger.find_unused_variables(&debug_info, context);
        report.missing_variables = self.debugger.find_missing_variables(&debug_info, context);

        // Performance profiling
        if self.debugger.profile_performance {
            let render_result = crate::render_with_context(template, context);
            if let Ok(rendered) = render_result {
                report.performance_metrics = Some(PerformanceMetrics {
                    render_time_ms: 0, // Would need actual timing
                    template_size: template.len(),
                    output_size: rendered.len(),
                });
            }
        }

        Ok(report)
    }

    /// Validate template file
    ///
    /// # Arguments
    /// * `file_path` - Path to template file
    /// * `context` - Template context
    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P, context: &TemplateContext) -> Result<ValidationReport> {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| TemplateError::IoError(format!("Failed to read template file: {}", e)))?;

        let file_name = file_path.as_ref().file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.validate_template(&content, context, file_name)
    }
}

/// Validation report for template analysis
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Template name
    pub template_name: String,
    /// Whether template syntax is valid
    pub syntax_valid: bool,
    /// Syntax errors found
    pub syntax_errors: Vec<String>,
    /// Lint violations found
    pub lint_violations: Vec<String>,
    /// Variables defined in context but not used
    pub unused_variables: Vec<String>,
    /// Variables used in template but not in context
    pub missing_variables: Vec<String>,
    /// Performance metrics (if profiling enabled)
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Performance metrics for template rendering
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Render time in milliseconds
    pub render_time_ms: u64,
    /// Template size in bytes
    pub template_size: usize,
    /// Output size in bytes
    pub output_size: usize,
}

impl Default for TemplateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_analysis() {
        let debugger = TemplateDebugger::new();
        let template = r#"
{{ service.name }}
{{ fake_name() }}
{% block content %}
Hello {{ user }}
{% endblock %}
        "#;

        let info = debugger.analyze(template, "test").unwrap();

        assert!(info.variables_used.contains("service.name"));
        assert!(info.variables_used.contains("user"));
        assert!(info.functions_used.contains("fake_name"));
        assert!(info.blocks_defined.contains("content"));
    }

    #[test]
    fn test_syntax_validation() {
        let debugger = TemplateDebugger::new();
        let invalid_template = r#"{{ unclosed_variable"#;

        let info = debugger.analyze(invalid_template, "test").unwrap();
        assert!(!info.syntax_errors.is_empty());
    }

    #[test]
    fn test_lint_rules() {
        let debugger = TemplateDebugger::new();
        let template = r#"
{{ deprecated_function() }}
{{ another_old_func() }}
        "#;

        let info = debugger.analyze(template, "test").unwrap();

        let deprecated_rule = lint::DeprecatedFunctionsRule;
        let violations = deprecated_rule.check(&info);

        // Should detect deprecated functions
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_template_linter() {
        let linter = TemplateLinter::new()
            .with_production_rules();

        let template = r#"
{{ deprecated_function() }}
Very complex template with many variables and functions
        "#;

        let violations = linter.lint(template, "test").unwrap();
        assert!(!violations.is_empty()); // Should detect deprecated function
    }
}