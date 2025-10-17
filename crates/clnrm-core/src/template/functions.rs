//! Custom Tera functions for clnrm templates
//!
//! Provides built-in functions for template rendering:
//! - `env(name)` - Get environment variable
//! - `now_rfc3339()` - Current timestamp (respects freeze_clock)
//! - `sha256(s)` - SHA-256 hex digest
//! - `toml_encode(value)` - Encode as TOML literal

use crate::error::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tera::{Function, Tera, Value};

/// Register all custom functions with Tera
pub fn register_functions(tera: &mut Tera) -> Result<()> {
    tera.register_function("env", EnvFunction);
    tera.register_function("now_rfc3339", NowRfc3339Function::new());
    tera.register_function("sha256", Sha256Function);
    tera.register_function("toml_encode", TomlEncodeFunction);
    Ok(())
}

/// env(name) - Get environment variable
///
/// Usage: `{{ env(name="HOME") }}`
struct EnvFunction;

impl Function for EnvFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("env() requires 'name' parameter"))?;

        std::env::var(name)
            .map(Value::String)
            .map_err(|_| tera::Error::msg(format!("Environment variable '{}' not found", name)))
    }
}

/// now_rfc3339() - Current timestamp (respects freeze_clock)
///
/// Usage: `{{ now_rfc3339() }}`
///
/// Returns RFC3339 formatted timestamp. Can be frozen for deterministic tests.
struct NowRfc3339Function {
    frozen: Arc<Mutex<Option<String>>>,
}

impl NowRfc3339Function {
    fn new() -> Self {
        Self {
            frozen: Arc::new(Mutex::new(None)),
        }
    }

    /// Freeze the clock to a specific timestamp for deterministic testing
    #[allow(dead_code)]
    pub fn freeze(&self, timestamp: String) {
        if let Ok(mut frozen) = self.frozen.lock() {
            *frozen = Some(timestamp);
        }
    }

    /// Unfreeze the clock to use real time
    #[allow(dead_code)]
    pub fn unfreeze(&self) {
        if let Ok(mut frozen) = self.frozen.lock() {
            *frozen = None;
        }
    }
}

impl Function for NowRfc3339Function {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Ok(frozen) = self.frozen.lock() {
            if let Some(ref frozen_time) = *frozen {
                return Ok(Value::String(frozen_time.clone()));
            }
        }
        Ok(Value::String(Utc::now().to_rfc3339()))
    }
}

/// sha256(s) - SHA-256 hex digest
///
/// Usage: `{{ sha256(s="hello") }}`
struct Sha256Function;

impl Function for Sha256Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let input = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("sha256() requires 's' parameter"))?;

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        Ok(Value::String(format!("{:x}", result)))
    }
}

/// toml_encode(value) - Encode as TOML literal
///
/// Usage: `{{ toml_encode(value=vars.myvar) }}`
struct TomlEncodeFunction;

impl Function for TomlEncodeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let value = args
            .get("value")
            .ok_or_else(|| tera::Error::msg("toml_encode() requires 'value' parameter"))?;

        // Convert JSON value to TOML string
        let toml_str = match value {
            Value::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
                        _ => v.to_string(),
                    })
                    .collect();
                format!("[{}]", items.join(","))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| match v {
                        Value::String(s) => {
                            format!("\"{}\"=\"{}\"", k, s.replace('\"', "\\\""))
                        }
                        _ => format!("\"{}\"={}", k, v),
                    })
                    .collect();
                format!("{{{}}}", items.join(","))
            }
            Value::Null => "null".to_string(),
        };

        Ok(Value::String(toml_str))
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;

    #[test]
    fn test_env_function() {
        std::env::set_var("TEST_VAR", "test_value");

        let func = EnvFunction;
        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::String("TEST_VAR".to_string()));

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "test_value");
    }

    #[test]
    fn test_env_function_missing() {
        let func = EnvFunction;
        let mut args = HashMap::new();
        args.insert(
            "name".to_string(),
            Value::String("NONEXISTENT_VAR_12345".to_string()),
        );

        let result = func.call(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_env_function_no_args() {
        let func = EnvFunction;
        let args = HashMap::new();

        let result = func.call(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_now_rfc3339_function() {
        let func = NowRfc3339Function::new();
        let args = HashMap::new();

        let result = func.call(&args).unwrap();
        assert!(result.is_string());

        // Verify it's a valid RFC3339 timestamp
        let timestamp = result.as_str().unwrap();
        assert!(timestamp.contains('T'));
        assert!(timestamp.contains(':'));
    }

    #[test]
    fn test_now_rfc3339_frozen() {
        let func = NowRfc3339Function::new();
        let frozen_time = "2024-01-01T00:00:00Z".to_string();
        func.freeze(frozen_time.clone());

        let args = HashMap::new();
        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), frozen_time);
    }

    #[test]
    fn test_now_rfc3339_unfreeze() {
        let func = NowRfc3339Function::new();
        let frozen_time = "2024-01-01T00:00:00Z".to_string();

        // Freeze, then unfreeze
        func.freeze(frozen_time.clone());
        func.unfreeze();

        let args = HashMap::new();
        let result = func.call(&args).unwrap();
        // Should not be the frozen time anymore
        assert_ne!(result.as_str().unwrap(), frozen_time);
    }

    #[test]
    fn test_sha256_function() {
        let func = Sha256Function;
        let mut args = HashMap::new();
        args.insert("s".to_string(), Value::String("hello".to_string()));

        let result = func.call(&args).unwrap();
        assert!(result.as_str().unwrap().starts_with("2cf24dba"));
    }

    #[test]
    fn test_sha256_function_no_args() {
        let func = Sha256Function;
        let args = HashMap::new();

        let result = func.call(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_sha256_deterministic() {
        let func = Sha256Function;
        let mut args = HashMap::new();
        args.insert("s".to_string(), Value::String("test".to_string()));

        let result1 = func.call(&args).unwrap();
        let result2 = func.call(&args).unwrap();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_toml_encode_string() {
        let func = TomlEncodeFunction;
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("hello".to_string()));

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "\"hello\"");
    }

    #[test]
    fn test_toml_encode_string_with_quotes() {
        let func = TomlEncodeFunction;
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::String("hello \"world\"".to_string()),
        );

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "\"hello \\\"world\\\"\"");
    }

    #[test]
    fn test_toml_encode_number() {
        let func = TomlEncodeFunction;
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::Number(42.into()));

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "42");
    }

    #[test]
    fn test_toml_encode_bool() {
        let func = TomlEncodeFunction;
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::Bool(true));

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "true");
    }

    #[test]
    fn test_toml_encode_array() {
        let func = TomlEncodeFunction;
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::Array(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ]),
        );

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "[\"a\",\"b\"]");
    }

    #[test]
    fn test_toml_encode_null() {
        let func = TomlEncodeFunction;
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::Null);

        let result = func.call(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "null");
    }

    #[test]
    fn test_toml_encode_no_args() {
        let func = TomlEncodeFunction;
        let args = HashMap::new();

        let result = func.call(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_functions() {
        let mut tera = Tera::default();
        let result = register_functions(&mut tera);
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration_with_tera() {
        let mut tera = Tera::default();
        register_functions(&mut tera).unwrap();

        std::env::set_var("INTEGRATION_TEST_VAR", "success");

        let template = r#"
env: {{ env(name="INTEGRATION_TEST_VAR") }}
sha: {{ sha256(s="test") }}
now: {{ now_rfc3339() }}
"#;

        let result = tera.render_str(template, &tera::Context::new());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("env: success"));
        assert!(rendered.contains("sha: 9f86d081"));
        assert!(rendered.contains("now: "));
    }
}
