//! Contract Testing Module
//!
//! This module provides comprehensive contract testing for CLNRM,
//! including API contracts, service plugin contracts, and event contracts.

pub mod api_contracts;
pub mod service_contracts;
pub mod consumer_contracts;
pub mod event_contracts;
pub mod schema_validator;

pub use schema_validator::{SchemaValidator, ContractValidationError};
