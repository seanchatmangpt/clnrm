/// Test fixtures for v0.7.0 DX features
///
/// Provides reusable test data and templates

pub mod templates;
pub mod traces;

// ============================================================================
// Template Fixtures
// ============================================================================

pub const VALID_MINIMAL_TEMPLATE: &str = r#"
[meta]
name = "minimal_test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
"#;

pub const VALID_COMPLETE_TEMPLATE: &str = r#"
[meta]
name = "complete_test"
description = "Complete test template"
version = "1.0.0"
author = "Test Author"

[otel]
service_name = "test_service"
endpoint = "http://localhost:4318"
sample_rate = 1.0

[service.postgres]
type = "postgres"
image = "postgres:15"
environment = { POSTGRES_PASSWORD = "test", POSTGRES_DB = "testdb" }
ports = ["5432:5432"]

[service.redis]
type = "redis"
image = "redis:7"
ports = ["6379:6379"]

[[scenario]]
name = "test_database_connection"
description = "Verify PostgreSQL connectivity"
service = "postgres"
timeout = "30s"

[[scenario]]
name = "test_cache_operations"
description = "Verify Redis operations"
service = "redis"
timeout = "10s"
"#;

pub const INVALID_MISSING_META: &str = r#"
[otel]
service_name = "test"

[service.db]
type = "postgres"
"#;

pub const INVALID_MISSING_OTEL: &str = r#"
[meta]
name = "test"

[service.db]
type = "postgres"
"#;

pub const INVALID_MISSING_SERVICE: &str = r#"
[meta]
name = "test"

[otel]
service_name = "test"
"#;

pub const INVALID_ORPHAN_SCENARIO: &str = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "orphan"
service = "nonexistent_service"
"#;

pub const INVALID_TOML_SYNTAX: &str = r#"
[meta
name = "test"
"#;

pub const UNFORMATTED_TEMPLATE: &str = r#"
[meta]
name="test"
version="1.0"
[otel]
service_name="test_service"
[service.db]
type="postgres"
port=5432
"#;

pub const FORMATTED_TEMPLATE: &str = r#"
[meta]
name = "test"
version = "1.0"

[otel]
service_name = "test_service"

[service.db]
type = "postgres"
port = 5432
"#;

// ============================================================================
// Trace Fixtures
// ============================================================================

pub const TRACE_SIMPLE: &str = r#"{
  "spans": [
    {
      "name": "http_request",
      "duration": 100,
      "attributes": {
        "http.method": "GET",
        "http.status_code": 200
      }
    }
  ]
}"#;

pub const TRACE_COMPLEX: &str = r#"{
  "spans": [
    {
      "name": "http_request",
      "duration": 250,
      "attributes": {
        "http.method": "POST",
        "http.url": "/api/users",
        "http.status_code": 201
      },
      "children": [
        {
          "name": "database_query",
          "duration": 150,
          "attributes": {
            "db.system": "postgresql",
            "db.statement": "INSERT INTO users VALUES ($1, $2)",
            "db.rows_affected": 1
          }
        },
        {
          "name": "cache_set",
          "duration": 50,
          "attributes": {
            "cache.key": "user:123",
            "cache.ttl": 3600
          }
        }
      ]
    }
  ]
}"#;

pub const TRACE_WITH_ERRORS: &str = r#"{
  "spans": [
    {
      "name": "http_request",
      "duration": 500,
      "status": "error",
      "attributes": {
        "http.method": "GET",
        "http.status_code": 500,
        "error.type": "DatabaseConnectionError",
        "error.message": "Connection timeout"
      }
    }
  ]
}"#;

// ============================================================================
// Helper Functions
// ============================================================================

pub fn get_template(name: &str) -> Option<&'static str> {
    match name {
        "valid_minimal" => Some(VALID_MINIMAL_TEMPLATE),
        "valid_complete" => Some(VALID_COMPLETE_TEMPLATE),
        "invalid_missing_meta" => Some(INVALID_MISSING_META),
        "invalid_missing_otel" => Some(INVALID_MISSING_OTEL),
        "invalid_missing_service" => Some(INVALID_MISSING_SERVICE),
        "invalid_orphan_scenario" => Some(INVALID_ORPHAN_SCENARIO),
        "invalid_toml_syntax" => Some(INVALID_TOML_SYNTAX),
        "unformatted" => Some(UNFORMATTED_TEMPLATE),
        "formatted" => Some(FORMATTED_TEMPLATE),
        _ => None,
    }
}

pub fn get_trace(name: &str) -> Option<&'static str> {
    match name {
        "simple" => Some(TRACE_SIMPLE),
        "complex" => Some(TRACE_COMPLEX),
        "with_errors" => Some(TRACE_WITH_ERRORS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_available() {
        assert!(get_template("valid_minimal").is_some());
        assert!(get_template("valid_complete").is_some());
        assert!(get_template("invalid_missing_meta").is_some());
        assert!(get_template("invalid_missing_otel").is_some());
        assert!(get_template("invalid_missing_service").is_some());
        assert!(get_template("invalid_orphan_scenario").is_some());
        assert!(get_template("invalid_toml_syntax").is_some());
        assert!(get_template("unformatted").is_some());
        assert!(get_template("formatted").is_some());
    }

    #[test]
    fn test_all_traces_available() {
        assert!(get_trace("simple").is_some());
        assert!(get_trace("complex").is_some());
        assert!(get_trace("with_errors").is_some());
    }

    #[test]
    fn test_invalid_names_return_none() {
        assert!(get_template("nonexistent").is_none());
        assert!(get_trace("nonexistent").is_none());
    }

    #[test]
    fn test_templates_are_valid_strings() {
        for template in &[
            VALID_MINIMAL_TEMPLATE,
            VALID_COMPLETE_TEMPLATE,
            UNFORMATTED_TEMPLATE,
            FORMATTED_TEMPLATE,
        ] {
            assert!(!template.is_empty(), "Template should not be empty");
            assert!(template.contains("[meta]") || template.contains("[otel]"));
        }
    }

    #[test]
    fn test_traces_are_valid_json() {
        use serde_json;

        for trace in &[TRACE_SIMPLE, TRACE_COMPLEX, TRACE_WITH_ERRORS] {
            let result: Result<serde_json::Value, _> = serde_json::from_str(trace);
            assert!(
                result.is_ok(),
                "Trace should be valid JSON: {}",
                result.err().unwrap()
            );
        }
    }
}
