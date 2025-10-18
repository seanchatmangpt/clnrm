//! API Contract Tests
//!
//! Consumer-driven contract tests for CLNRM APIs.

use super::schema_validator::{SchemaValidator, ContractValidationError};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Test the cleanroom API contract
/// Test the backend capabilities API contract