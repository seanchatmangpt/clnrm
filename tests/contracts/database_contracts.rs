//! Database Schema Contract Tests
//!
//! Contract tests for database schema validation and migrations.

use super::schema_validator::SchemaValidator;
use serde_json::json;

#[test]
fn test_database_schema_contract_valid() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    let valid_schema = json!({
        "schema_version": "1.0.0",
        "tables": [
            {
                "name": "users",
                "columns": [
                    {
                        "name": "id",
                        "data_type": "uuid",
                        "nullable": false,
                        "unique": true
                    },
                    {
                        "name": "email",
                        "data_type": "string",
                        "nullable": false,
                        "max_length": 255
                    },
                    {
                        "name": "created_at",
                        "data_type": "datetime",
                        "nullable": false
                    }
                ],
                "primary_key": ["id"],
                "indexes": [
                    {
                        "name": "idx_users_email",
                        "columns": ["email"],
                        "unique": true,
                        "index_type": "btree"
                    }
                ]
            }
        ],
        "migrations": [
            {
                "version": "1.0.0",
                "description": "Initial schema setup",
                "timestamp": "2026-05-30T10:00:00Z"
            }
        ]
    });

    let result = validator.validate("database_schema_contract.json", &valid_schema);
    assert!(result.is_ok(), "Expected database schema to pass validation: {:?}", result);
}

#[test]
fn test_database_schema_contract_invalid_table_name() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    // Invalid table name: contains uppercase and special chars, violates regex `^[a-z][a-z0-9_]*$`
    let invalid_schema = json!({
        "schema_version": "1.0.0",
        "tables": [
            {
                "name": "INVALID-Table-Name",
                "columns": [
                    {
                        "name": "id",
                        "data_type": "uuid",
                        "nullable": false
                    }
                ],
                "primary_key": ["id"]
            }
        ]
    });

    let result = validator.validate("database_schema_contract.json", &invalid_schema);
    assert!(result.is_err(), "Expected database schema with invalid table name to fail validation");
}

#[test]
fn test_database_schema_contract_invalid_column_type() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    // Invalid data_type: "unsupported_type" is not in enum options
    let invalid_schema = json!({
        "schema_version": "1.0.0",
        "tables": [
            {
                "name": "users",
                "columns": [
                    {
                        "name": "id",
                        "data_type": "unsupported_type",
                        "nullable": false
                    }
                ],
                "primary_key": ["id"]
            }
        ]
    });

    let result = validator.validate("database_schema_contract.json", &invalid_schema);
    assert!(result.is_err(), "Expected database schema with invalid column type to fail validation");
}
