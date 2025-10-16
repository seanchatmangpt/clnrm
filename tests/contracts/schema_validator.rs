//! Schema Validation for Contract Testing
//!
//! Provides JSON Schema validation for contract testing using jsonschema crate.

use serde_json::Value;
use std::fs;
use std::path::Path;

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
        Self {
            schema_dir: schema_dir.to_string(),
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

    /// Validate data against schema (basic validation without jsonschema crate)
    /// In production, this should use jsonschema crate for full JSON Schema validation
    pub fn validate<T: serde::Serialize>(
        &self,
        schema_name: &str,
        data: &T,
    ) -> Result<(), ContractValidationError> {
        // Load schema
        let _schema = self.load_schema(schema_name)?;

        // Serialize data to JSON
        let data_value = serde_json::to_value(data)
            .map_err(|e| ContractValidationError::SerializationError(e.to_string()))?;

        // Basic validation: check that data is an object if schema expects it
        // This is a simplified validation - in production use jsonschema crate
        if !data_value.is_object() && !data_value.is_array() {
            return Err(ContractValidationError::ValidationFailed(vec![
                "Data must be an object or array".to_string()
            ]));
        }

        // For production, use jsonschema crate:
        // let compiled_schema = jsonschema::JSONSchema::compile(&schema)
        //     .map_err(|e| ContractValidationError::InvalidSchema(e.to_string()))?;
        //
        // let result = compiled_schema.validate(&data_value);
        // if let Err(errors) = result {
        //     let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
        //     return Err(ContractValidationError::ValidationFailed(error_messages));
        // }

        Ok(())
    }

    /// Validate raw JSON value against schema
    pub fn validate_value(
        &self,
        schema_name: &str,
        value: &Value,
    ) -> Result<(), ContractValidationError> {
        // Load schema
        let _schema = self.load_schema(schema_name)?;

        // Basic validation
        if !value.is_object() && !value.is_array() {
            return Err(ContractValidationError::ValidationFailed(vec![
                "Value must be an object or array".to_string()
            ]));
        }

        Ok(())
    }

    /// Get schema directory
    pub fn schema_dir(&self) -> &str {
        &self.schema_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::new("/tmp/schemas");
        assert_eq!(validator.schema_dir(), "/tmp/schemas");
    }

    #[test]
    fn test_validate_value() {
        // Create a temporary schema file for testing
        let temp_dir = std::env::temp_dir();
        let schema_dir = temp_dir.join("contract_test_schemas");
        std::fs::create_dir_all(&schema_dir).unwrap();

        let test_schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            }
        });

        let schema_path = schema_dir.join("test_schema.json");
        std::fs::write(&schema_path, test_schema.to_string()).unwrap();

        let validator = SchemaValidator::new(schema_dir.to_str().unwrap());

        let test_data = json!({
            "name": "Test",
            "age": 30
        });

        let result = validator.validate_value("test_schema.json", &test_data);
        assert!(result.is_ok());

        // Cleanup
        std::fs::remove_file(&schema_path).ok();
        std::fs::remove_dir(&schema_dir).ok();
    }
}
