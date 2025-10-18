//! Template output validation and schema checking
//!
//! Provides comprehensive validation for template rendering results:
//! - Schema validation (TOML, JSON, YAML)
//! - Required field checking
//! - Format validation
//! - Custom validation rules

use crate::error::{TemplateError, Result};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/// Template output validator
///
/// Validates rendered template content for:
/// - Correct format (TOML, JSON, YAML)
/// - Required fields presence
/// - Schema compliance
/// - Custom validation rules
pub struct TemplateValidator {
    /// Required fields that must be present
    required_fields: HashSet<String>,
    /// Required top-level sections for TOML
    required_sections: HashSet<String>,
    /// Custom validation rules
    rules: Vec<ValidationRule>,
    /// Expected output format
    format: OutputFormat,
    /// Schema for validation (TOML/JSON schema)
    schema: Option<Value>,
    /// TOML-specific validation options
    toml_options: TomlValidationOptions,
}

/// TOML-specific validation options
#[derive(Debug, Clone, Default)]
pub struct TomlValidationOptions {
    /// Allow inline tables
    pub allow_inline_tables: bool,
    /// Allow multiline strings
    pub allow_multiline_strings: bool,
    /// Maximum nesting depth
    pub max_nesting_depth: Option<usize>,
    /// Maximum array length
    pub max_array_length: Option<usize>,
    /// Maximum string length
    pub max_string_length: Option<usize>,
}

/// Supported output formats for validation
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// TOML format (default for Cleanroom)
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Auto-detect based on content
    Auto,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Toml
    }
}

impl TemplateValidator {
    /// Create new template validator
    pub fn new() -> Self {
        Self {
            required_fields: HashSet::new(),
            required_sections: HashSet::new(),
            rules: Vec::new(),
            format: OutputFormat::Toml,
            schema: None,
            toml_options: TomlValidationOptions::default(),
        }
    }

    /// Set required fields that must be present in output
    ///
    /// # Arguments
    /// * `fields` - Field paths (e.g., "service.name", "meta.version")
    pub fn require_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for field in fields {
            self.required_fields.insert(field.into());
        }
        self
    }

    /// Set required top-level sections for TOML output
    ///
    /// # Arguments
    /// * `sections` - Section names that must exist (e.g., "service", "meta")
    pub fn require_sections<I, S>(mut self, sections: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for section in sections {
            self.required_sections.insert(section.into());
        }
        self
    }

    /// Add custom validation rule
    pub fn with_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set expected output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set validation schema
    ///
    /// # Arguments
    /// * `schema` - JSON schema for validation
    pub fn with_schema(mut self, schema: Value) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Set TOML validation options
    pub fn with_toml_options(mut self, options: TomlValidationOptions) -> Self {
        self.toml_options = options;
        self
    }

    /// Validate template output
    ///
    /// # Arguments
    /// * `output` - Rendered template content
    /// * `template_name` - Name of template for error reporting
    pub fn validate(&self, output: &str, template_name: &str) -> Result<()> {
        // Detect format if auto
        let format = if matches!(self.format, OutputFormat::Auto) {
            self.detect_format(output)
        } else {
            self.format.clone()
        };

        // Validate format
        self.validate_format(&format, output, template_name)?;

        // Parse content for further validation
        let parsed = self.parse_content(&format, output, template_name)?;

        // Validate required fields
        self.validate_required_fields(&parsed, template_name)?;

        // Validate required sections (TOML only)
        if matches!(format, OutputFormat::Toml) {
            self.validate_required_sections(&parsed, template_name)?;
        }

        // Validate against schema
        if let Some(schema) = &self.schema {
            self.validate_schema(&parsed, schema, template_name)?;
        }

        // Apply custom rules
        for rule in &self.rules {
            rule.validate(&parsed, template_name)?;
        }

        // TOML-specific validation
        if matches!(format, OutputFormat::Toml) {
            self.validate_toml_structure(&parsed, template_name)?;
        }

        Ok(())
    }

    /// Detect output format from content
    fn detect_format(&self, content: &str) -> OutputFormat {
        let trimmed = content.trim();

        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            OutputFormat::Json
        } else if trimmed.starts_with('[') && trimmed.contains('=') {
            OutputFormat::Toml
        } else if trimmed.contains(": ") && !trimmed.contains('=') {
            OutputFormat::Yaml
        } else {
            OutputFormat::Toml // Default fallback
        }
    }

    /// Validate content format
    fn validate_format(&self, format: &OutputFormat, content: &str, template_name: &str) -> Result<()> {
        match format {
            OutputFormat::Toml => self.validate_toml(content, template_name),
            OutputFormat::Json => self.validate_json(content, template_name),
            OutputFormat::Yaml => self.validate_yaml(content, template_name),
            OutputFormat::Auto => Ok(()), // Already handled
        }
    }

    /// Validate TOML format
    fn validate_toml(&self, content: &str, template_name: &str) -> Result<()> {
        toml::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!(
                "Invalid TOML format in template '{}': {}",
                template_name, e
            )))?;
        Ok(())
    }

    /// Validate JSON format
    fn validate_json(&self, content: &str, template_name: &str) -> Result<()> {
        serde_json::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!(
                "Invalid JSON format in template '{}': {}",
                template_name, e
            )))?;
        Ok(())
    }

    /// Validate YAML format
    fn validate_yaml(&self, content: &str, template_name: &str) -> Result<()> {
        serde_yaml::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!(
                "Invalid YAML format in template '{}': {}",
                template_name, e
            )))?;
        Ok(())
    }

    /// Parse content into JSON Value for validation
    fn parse_content(&self, format: &OutputFormat, content: &str, template_name: &str) -> Result<Value> {
        match format {
            OutputFormat::Toml => {
                toml::from_str::<Value>(content)
                    .map_err(|e| TemplateError::ValidationError(format!(
                        "Failed to parse TOML in template '{}': {}",
                        template_name, e
                    )))
            }
            OutputFormat::Json => {
                serde_json::from_str::<Value>(content)
                    .map_err(|e| TemplateError::ValidationError(format!(
                        "Failed to parse JSON in template '{}': {}",
                        template_name, e
                    )))
            }
            OutputFormat::Yaml => {
                serde_yaml::from_str::<Value>(content)
                    .map_err(|e| TemplateError::ValidationError(format!(
                        "Failed to parse YAML in template '{}': {}",
                        template_name, e
                    )))
            }
            OutputFormat::Auto => {
                // Try TOML first (most common for Cleanroom)
                if let Ok(value) = toml::from_str::<Value>(content) {
                    Ok(value)
                } else if let Ok(value) = serde_json::from_str::<Value>(content) {
                    Ok(value)
                } else if let Ok(value) = serde_yaml::from_str::<Value>(content) {
                    Ok(value)
                } else {
                    Err(TemplateError::ValidationError(format!(
                        "Could not parse template '{}' as TOML, JSON, or YAML",
                        template_name
                    )))
                }
            }
        }
    }

    /// Validate required fields are present
    fn validate_required_fields(&self, parsed: &Value, template_name: &str) -> Result<()> {
        for field_path in &self.required_fields {
            if !self.field_exists(parsed, field_path) {
                return Err(TemplateError::ValidationError(format!(
                    "Required field '{}' missing in template '{}'",
                    field_path, template_name
                )));
            }
        }
        Ok(())
    }

    /// Validate required sections exist (TOML only)
    fn validate_required_sections(&self, parsed: &Value, template_name: &str) -> Result<()> {
        let obj = parsed.as_object()
            .ok_or_else(|| TemplateError::ValidationError(format!(
                "Template '{}' must be a TOML object for section validation",
                template_name
            )))?;

        for section in &self.required_sections {
            if !obj.contains_key(section) {
                return Err(TemplateError::ValidationError(format!(
                    "Required section '{}' missing in template '{}'",
                    section, template_name
                )));
            }
        }
        Ok(())
    }

    /// Validate against JSON schema
    fn validate_schema(&self, parsed: &Value, schema: &Value, template_name: &str) -> Result<()> {
        // Simple schema validation - can be extended with proper JSON Schema
        if let (Some(obj), Some(schema_obj)) = (parsed.as_object(), schema.as_object()) {
            // Check required properties
            if let Some(required) = schema_obj.get("required").and_then(|v| v.as_array()) {
                for prop in required {
                    if let Some(prop_str) = prop.as_str() {
                        if !obj.contains_key(prop_str) {
                            return Err(TemplateError::ValidationError(format!(
                                "Schema validation failed: required property '{}' missing in template '{}'",
                                prop_str, template_name
                            )));
                        }
                    }
                }
            }

            // Check property types
            if let Some(properties) = schema_obj.get("properties").and_then(|v| v.as_object()) {
                for (prop_name, prop_schema) in properties {
                    if let Some(prop_value) = obj.get(prop_name) {
                        self.validate_property_type(prop_value, prop_schema, prop_name, template_name)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate property type against schema
    fn validate_property_type(&self, value: &Value, schema: &Value, prop_name: &str, template_name: &str) -> Result<()> {
        if let Some(expected_type) = schema.get("type").and_then(|v| v.as_str()) {
            match expected_type {
                "string" => {
                    if !value.is_string() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed: property '{}' must be string in template '{}'",
                            prop_name, template_name
                        )));
                    }
                }
                "number" => {
                    if !value.is_number() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed: property '{}' must be number in template '{}'",
                            prop_name, template_name
                        )));
                    }
                }
                "boolean" => {
                    if !value.is_boolean() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed: property '{}' must be boolean in template '{}'",
                            prop_name, template_name
                        )));
                    }
                }
                "array" => {
                    if !value.is_array() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed: property '{}' must be array in template '{}'",
                            prop_name, template_name
                        )));
                    }
                }
                "object" => {
                    if !value.is_object() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed: property '{}' must be object in template '{}'",
                            prop_name, template_name
                        )));
                    }
                }
                _ => {} // Unknown type, skip validation
            }
        }

        Ok(())
    }

    /// Check if field exists at dot-notation path
    fn field_exists(&self, value: &Value, field_path: &str) -> bool {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                Value::Object(obj) => {
                    if let Some(next) = obj.get(part) {
                        current = next;
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
    }

    /// Validate TOML-specific structure
    fn validate_toml_structure(&self, parsed: &Value, template_name: &str) -> Result<()> {
        self.validate_toml_nesting(parsed, 0, template_name)?;
        self.validate_toml_sizes(parsed, template_name)?;
        Ok(())
    }

    /// Validate TOML nesting depth
    fn validate_toml_nesting(&self, value: &Value, depth: usize, template_name: &str) -> Result<()> {
        if let Some(max_depth) = self.toml_options.max_nesting_depth {
            if depth > max_depth {
                return Err(TemplateError::ValidationError(format!(
                    "TOML nesting depth exceeds maximum {} in template '{}'",
                    max_depth, template_name
                )));
            }
        }

        match value {
            Value::Object(obj) => {
                for (_, value) in obj {
                    self.validate_toml_nesting(value, depth + 1, template_name)?;
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    self.validate_toml_nesting(value, depth + 1, template_name)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Validate TOML value sizes
    fn validate_toml_sizes(&self, value: &Value, template_name: &str) -> Result<()> {
        match value {
            Value::Array(arr) => {
                if let Some(max_len) = self.toml_options.max_array_length {
                    if arr.len() > max_len {
                        return Err(TemplateError::ValidationError(format!(
                            "Array length {} exceeds maximum {} in template '{}'",
                            arr.len(), max_len, template_name
                        )));
                    }
                }
            }
            Value::String(s) => {
                if let Some(max_len) = self.toml_options.max_string_length {
                    if s.len() > max_len {
                        return Err(TemplateError::ValidationError(format!(
                            "String length {} exceeds maximum {} in template '{}'",
                            s.len(), max_len, template_name
                        )));
                    }
                }
            }
            Value::Object(obj) => {
                for (_, value) in obj {
                    self.validate_toml_sizes(value, template_name)?;
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    self.validate_toml_sizes(value, template_name)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Validation rule types for template validation
pub enum ValidationRule {
    /// Validate service name follows naming conventions
    ServiceName,
    /// Validate version follows semver format
    Semver,
    /// Validate environment is one of allowed values
    Environment { allowed: Vec<String> },
    /// Validate required OTEL configuration is present
    OtelConfig,
    /// Custom validation function
    Custom { name: String },
}

impl ValidationRule {
    /// Validate parsed template content
    ///
    /// # Arguments
    /// * `parsed` - Parsed template content as JSON Value
    /// * `template_name` - Template name for error reporting
    pub fn validate(&self, parsed: &Value, template_name: &str) -> Result<()> {
        match self {
            ValidationRule::ServiceName => Self::validate_service_name(parsed, template_name),
            ValidationRule::Semver => Self::validate_semver(parsed, template_name),
            ValidationRule::Environment { allowed } => Self::validate_environment(parsed, template_name, allowed),
            ValidationRule::OtelConfig => Self::validate_otel_config(parsed, template_name),
            ValidationRule::Custom { .. } => {
                // For now, custom validation is not implemented in this simplified version
                // This would require a registry of custom validators
                Ok(())
            }
        }
    }

    fn validate_service_name(parsed: &Value, template_name: &str) -> Result<()> {
        if let Some(service_name) = parsed.get("service").and_then(|v| v.get("name")).and_then(|v| v.as_str()) {
            if !service_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err(TemplateError::ValidationError(format!(
                    "Service name '{}' in template '{}' contains invalid characters (only alphanumeric, '-', '_' allowed)",
                    service_name, template_name
                )));
            }

            if service_name.len() > 63 {
                return Err(TemplateError::ValidationError(format!(
                    "Service name '{}' in template '{}' is too long (max 63 characters)",
                    service_name, template_name
                )));
            }
        }
        Ok(())
    }

    fn validate_semver(parsed: &Value, template_name: &str) -> Result<()> {
        if let Some(version) = parsed.get("meta").and_then(|v| v.get("version")).and_then(|v| v.as_str()) {
            // Simple semver regex check
            let semver_regex = regex::Regex::new(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$")
                .map_err(|_| TemplateError::ValidationError("Failed to compile semver regex".to_string()))?;

            if !semver_regex.is_match(version) {
                return Err(TemplateError::ValidationError(format!(
                    "Version '{}' in template '{}' does not follow semver format (x.y.z)",
                    version, template_name
                )));
            }
        }
        Ok(())
    }

    fn validate_environment(parsed: &Value, template_name: &str, allowed: &[String]) -> Result<()> {
        if let Some(env) = parsed.get("meta").and_then(|v| v.get("environment")).and_then(|v| v.as_str()) {
            if !allowed.contains(&env.to_string()) {
                return Err(TemplateError::ValidationError(format!(
                    "Environment '{}' in template '{}' not in allowed list: {:?}",
                    env, template_name, allowed
                )));
            }
        }
        Ok(())
    }

    fn validate_otel_config(parsed: &Value, template_name: &str) -> Result<()> {
        if let Some(otel) = parsed.get("otel") {
            let required_fields = ["endpoint", "service_name"];

            for field in &required_fields {
                if !otel.get(*field).is_some() {
                    return Err(TemplateError::ValidationError(format!(
                        "Required OTEL field '{}' missing in template '{}'",
                        field, template_name
                    )));
                }
            }

            // Validate endpoint format
            if let Some(endpoint) = otel.get("endpoint").and_then(|v| v.as_str()) {
                if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                    return Err(TemplateError::ValidationError(format!(
                        "OTEL endpoint '{}' in template '{}' must start with http:// or https://",
                        endpoint, template_name
                    )));
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationRule::ServiceName => write!(f, "ServiceName"),
            ValidationRule::Semver => write!(f, "Semver"),
            ValidationRule::Environment { allowed } => write!(f, "Environment({:?})", allowed),
            ValidationRule::OtelConfig => write!(f, "OtelConfig"),
            ValidationRule::Custom { name } => write!(f, "Custom({})", name),
        }
    }
}

impl Clone for ValidationRule {
    fn clone(&self) -> Self {
        match self {
            ValidationRule::ServiceName => ValidationRule::ServiceName,
            ValidationRule::Semver => ValidationRule::Semver,
            ValidationRule::Environment { allowed } => ValidationRule::Environment { allowed: allowed.clone() },
            ValidationRule::OtelConfig => ValidationRule::OtelConfig,
            ValidationRule::Custom { name } => ValidationRule::Custom { name: name.clone() },
        }
    }
}

/// Common validation rules
pub mod rules {
    use super::*;

    /// Create service name validation rule
    pub fn service_name() -> ValidationRule {
        ValidationRule::ServiceName
    }

    /// Create semver validation rule
    pub fn semver() -> ValidationRule {
        ValidationRule::Semver
    }

    /// Create environment validation rule
    pub fn environment(allowed: Vec<&str>) -> ValidationRule {
        ValidationRule::Environment {
            allowed: allowed.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create OTEL configuration validation rule
    pub fn otel_config() -> ValidationRule {
        ValidationRule::OtelConfig
    }

    /// Create custom validation rule (simplified version)
    pub fn custom(name: &str) -> ValidationRule {
        ValidationRule::Custom {
            name: name.to_string(),
        }
    }
}

/// Schema validator for JSON Schema validation
pub struct SchemaValidator {
    schema: Value,
}

impl SchemaValidator {
    /// Create new schema validator
    ///
    /// # Arguments
    /// * `schema` - JSON schema for validation
    pub fn new(schema: Value) -> Self {
        Self { schema }
    }

    /// Validate content against schema
    pub fn validate(&self, content: &str, template_name: &str) -> Result<()> {
        let parsed: Value = serde_json::from_str(content)
            .map_err(|e| TemplateError::ValidationError(format!(
                "Failed to parse content for schema validation in template '{}': {}",
                template_name, e
            )))?;

        // Simple schema validation implementation
        // In a real implementation, this would use a proper JSON Schema validator
        self.validate_against_schema(&parsed, &self.schema, template_name)
    }

    /// Recursive schema validation
    fn validate_against_schema(&self, value: &Value, schema: &Value, template_name: &str) -> Result<()> {
        // Check type
        if let Some(expected_type) = schema.get("type").and_then(|v| v.as_str()) {
            match expected_type {
                "object" => {
                    if !value.is_object() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed in template '{}': expected object",
                            template_name
                        )));
                    }
                }
                "array" => {
                    if !value.is_array() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed in template '{}': expected array",
                            template_name
                        )));
                    }
                }
                "string" => {
                    if !value.is_string() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed in template '{}': expected string",
                            template_name
                        )));
                    }
                }
                "number" => {
                    if !value.is_number() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed in template '{}': expected number",
                            template_name
                        )));
                    }
                }
                "boolean" => {
                    if !value.is_boolean() {
                        return Err(TemplateError::ValidationError(format!(
                            "Schema validation failed in template '{}': expected boolean",
                            template_name
                        )));
                    }
                }
                _ => {}
            }
        }

        // Check required properties for objects
        if let (Value::Object(obj), Value::Object(schema_obj)) = (value, schema) {
            if let Some(required) = schema_obj.get("required").and_then(|v| v.as_array()) {
                for prop in required {
                    if let Some(prop_str) = prop.as_str() {
                        if !obj.contains_key(prop_str) {
                            return Err(TemplateError::ValidationError(format!(
                                "Schema validation failed: required property '{}' missing in template '{}'",
                                prop_str, template_name
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_validation() {
        let validator = TemplateValidator::new()
            .require_fields(vec!["service.name", "meta.version"])
            .require_sections(vec!["service", "meta"]);

        let valid_toml = r#"
[service]
name = "my-service"

[meta]
version = "1.0.0"
        "#;

        assert!(validator.validate(valid_toml, "test").is_ok());

        let invalid_toml = r#"
[service]
# missing name field
        "#;

        assert!(validator.validate(invalid_toml, "test").is_err());
    }

    #[test]
    fn test_custom_validation_rules() {
        let validator = TemplateValidator::new()
            .with_rule(rules::service_name())
            .with_rule(rules::semver());

        let valid_content = r#"
[service]
name = "my-service"

[meta]
version = "1.0.0"
        "#;

        assert!(validator.validate(valid_content, "test").is_ok());

        let invalid_content = r#"
[service]
name = "my service!"  # invalid characters

[meta]
version = "not-semver"
        "#;

        assert!(validator.validate(invalid_content, "test").is_err());
    }

    #[test]
    fn test_format_detection() {
        let validator = TemplateValidator::new().format(OutputFormat::Auto);

        assert!(validator.validate("name = \"test\"", "test").is_ok()); // TOML
        assert!(validator.validate("{\"name\": \"test\"}", "test").is_ok()); // JSON
    }
}