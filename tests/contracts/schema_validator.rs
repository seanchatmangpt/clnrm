//! Schema Validation for Contract Testing
//!
//! Provides JSON Schema validation for contract testing using standard serde_json and regex.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

/// Contract validation error types
#[derive(Debug, Clone)]
pub enum ContractValidationError {
    /// Schema file not found
    SchemaNotFound(String),
    /// Invalid JSON in schema
    InvalidSchema(String),
    /// Validation failed
    ValidationFailed(Vec<String>),
    /// Data serialization error
    SerializationError(String),
}

impl std::fmt::Display for ContractValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaNotFound(path) => write!(f, "Schema not found: {}", path),
            Self::InvalidSchema(err) => write!(f, "Invalid schema: {}", err),
            Self::ValidationFailed(errors) => write!(f, "Validation failed: {}", errors.join(", ")),
            Self::SerializationError(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl std::error::Error for ContractValidationError {}

/// JSON Schema validator for contract testing
pub struct SchemaValidator {
    schema_dir: String,
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new(schema_dir: &str) -> Self {
        let mut resolved_path = PathBuf::from(schema_dir);
        
        if !resolved_path.exists() {
            if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let manifest_path = Path::new(&manifest_dir);
                let candidate = manifest_path.join("../../").join(schema_dir);
                if candidate.exists() {
                    resolved_path = candidate;
                } else {
                    let candidate2 = manifest_path.join(schema_dir);
                    if candidate2.exists() {
                        resolved_path = candidate2;
                    }
                }
            }
        }

        Self {
            schema_dir: resolved_path.to_string_lossy().to_string(),
        }
    }

    /// Load schema from file
    pub fn load_schema(&self, schema_name: &str) -> Result<Value, ContractValidationError> {
        let schema_path = Path::new(&self.schema_dir).join(schema_name);

        if !schema_path.exists() {
            return Err(ContractValidationError::SchemaNotFound(
                schema_path.display().to_string()
            ));
        }

        let schema_content = fs::read_to_string(&schema_path)
            .map_err(|e| ContractValidationError::InvalidSchema(e.to_string()))?;

        serde_json::from_str(&schema_content)
            .map_err(|e| ContractValidationError::InvalidSchema(e.to_string()))
    }

    /// Validate data against schema
    pub fn validate<T: serde::Serialize>(
        &self,
        schema_name: &str,
        data: &T,
    ) -> Result<(), ContractValidationError> {
        // Load schema
        let schema = self.load_schema(schema_name)?;

        // Serialize data to JSON
        let data_value = serde_json::to_value(data)
            .map_err(|e| ContractValidationError::SerializationError(e.to_string()))?;

        Self::validate_internal(&data_value, &schema, &schema)
            .map_err(ContractValidationError::ValidationFailed)
    }

    /// Validate raw JSON value against schema
    pub fn validate_value(
        &self,
        schema_name: &str,
        value: &Value,
    ) -> Result<(), ContractValidationError> {
        // Load schema
        let schema = self.load_schema(schema_name)?;

        Self::validate_internal(value, &schema, &schema)
            .map_err(ContractValidationError::ValidationFailed)
    }

    /// Get schema directory
    pub fn schema_dir(&self) -> &str {
        &self.schema_dir
    }

    /// Internal validation logic
    fn validate_internal(
        value: &Value,
        schema: &Value,
        root_schema: &Value,
    ) -> Result<(), Vec<String>> {
        // 1. Handle $ref
        if let Some(ref_str) = schema.get("$ref").and_then(|v| v.as_str()) {
            if ref_str.starts_with("#/definitions/") {
                let def_name = &ref_str["#/definitions/".len()..];
                if let Some(def_schema) = root_schema.get("definitions").and_then(|d| d.get(def_name)) {
                    return Self::validate_internal(value, def_schema, root_schema);
                } else {
                    return Err(vec![format!("Definition not found: {}", def_name)]);
                }
            }
        }

        // 2. Handle oneOf
        if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_array()) {
            let mut matched = 0;
            let mut last_errors = Vec::new();
            for sub_schema in one_of {
                match Self::validate_internal(value, sub_schema, root_schema) {
                    Ok(_) => matched += 1,
                    Err(errs) => last_errors = errs,
                }
            }
            if matched == 1 {
                return Ok(());
            } else if matched > 1 {
                return Err(vec!["Multiple schemas in oneOf matched".to_string()]);
            } else {
                return Err(vec![format!("No schemas in oneOf matched. Last error: {:?}", last_errors)]);
            }
        }

        // 3. Handle type
        if let Some(type_val) = schema.get("type") {
            let mut type_matches = false;
            let mut expected_types = Vec::new();
            if let Some(type_str) = type_val.as_str() {
                expected_types.push(type_str);
            } else if let Some(type_arr) = type_val.as_array() {
                for t in type_arr {
                    if let Some(t_str) = t.as_str() {
                        expected_types.push(t_str);
                    }
                }
            }
            for t in &expected_types {
                match *t {
                    "object" => if value.is_object() { type_matches = true; },
                    "array" => if value.is_array() { type_matches = true; },
                    "string" => if value.is_string() { type_matches = true; },
                    "integer" => if value.is_i64() || value.is_u64() || (value.is_f64() && value.as_f64().unwrap().fract() == 0.0) { type_matches = true; },
                    "number" => if value.is_number() { type_matches = true; },
                    "boolean" => if value.is_boolean() { type_matches = true; },
                    "null" => if value.is_null() { type_matches = true; },
                    _ => {}
                }
            }
            if !type_matches {
                return Err(vec![format!("Type mismatch: expected {:?}, got value {:?}", type_val, value)]);
            }
        }

        let mut errors = Vec::new();

        // 4. Handle object properties and required
        if value.is_object() {
            if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
                for req in required {
                    if let Some(req_str) = req.as_str() {
                        if !value.get(req_str).is_some() {
                            errors.push(format!("Missing required property: {}", req_str));
                        }
                    }
                }
            }
            if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
                for (prop_name, prop_schema) in properties {
                    if let Some(prop_val) = value.get(prop_name) {
                        if let Err(errs) = Self::validate_internal(prop_val, prop_schema, root_schema) {
                            for e in errs {
                                errors.push(format!("{}.{}", prop_name, e));
                            }
                        }
                    }
                }
            }
        }

        // 5. Handle array items
        if value.is_array() {
            let arr = value.as_array().unwrap();
            if let Some(items_schema) = schema.get("items") {
                for (idx, item) in arr.iter().enumerate() {
                    if let Err(errs) = Self::validate_internal(item, items_schema, root_schema) {
                        for e in errs {
                            errors.push(format!("{}[{}]", e, idx));
                        }
                    }
                }
            }
            if let Some(min_items) = schema.get("minItems").and_then(|v| v.as_u64()) {
                if (arr.len() as u64) < min_items {
                    errors.push(format!("Array has fewer than {} items (actual: {})", min_items, arr.len()));
                }
            }
            if let Some(max_items) = schema.get("maxItems").and_then(|v| v.as_u64()) {
                if (arr.len() as u64) > max_items {
                    errors.push(format!("Array has more than {} items (actual: {})", max_items, arr.len()));
                }
            }
        }

        // 6. Handle string patterns & min/max length
        if let Some(s) = value.as_str() {
            if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
                if let Ok(re) = Regex::new(pattern) {
                    if !re.is_match(s) {
                        errors.push(format!("String '{}' does not match pattern '{}'", s, pattern));
                    }
                }
            }
            if let Some(min_len) = schema.get("minLength").and_then(|v| v.as_u64()) {
                if (s.len() as u64) < min_len {
                    errors.push(format!("String '{}' is shorter than {} chars", s, min_len));
                }
            }
            if let Some(max_len) = schema.get("maxLength").and_then(|v| v.as_u64()) {
                if (s.len() as u64) > max_len {
                    errors.push(format!("String '{}' is longer than {} chars", s, max_len));
                }
            }
        }

        // 7. Handle enum values
        if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
            if !enum_vals.contains(value) {
                errors.push(format!("Value {:?} is not in enum {:?}", value, enum_vals));
            }
        }

        // 8. Handle minimum and maximum values
        if let Some(num) = value.as_i64().or_else(|| value.as_f64().map(|f| f as i64)) {
            if let Some(min_val) = schema.get("minimum").and_then(|v| v.as_i64()) {
                if num < min_val {
                    errors.push(format!("Value {} is less than minimum {}", num, min_val));
                }
            }
            if let Some(max_val) = schema.get("maximum").and_then(|v| v.as_i64()) {
                if num > max_val {
                    errors.push(format!("Value {} is greater than maximum {}", num, max_val));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
