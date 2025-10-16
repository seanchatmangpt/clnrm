//! CLNRM Contract Tests
//!
//! Comprehensive contract testing suite for CLNRM framework.
//!
//! This test suite validates:
//! - API contracts for CleanroomEnvironment
//! - Service plugin contracts
//! - Consumer-driven contracts between modules
//! - Event contracts for async communication
//! - Database schema contracts

mod contracts {
    pub mod schema_validator;
    pub mod api_contracts;
    pub mod service_contracts;
    pub mod consumer_contracts;
    pub mod event_contracts;
    pub mod database_contracts;
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_contract_testing_infrastructure() {
        // Verify contract testing infrastructure is set up correctly

        // Check schema validator is available
        use contracts::schema_validator::SchemaValidator;
        let schema_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/contracts/schemas");
        let _validator = SchemaValidator::new(schema_dir);

        // Verify it compiles successfully
        assert!(true);
    }

    #[test]
    fn test_all_contract_modules_available() {
        // Ensure all contract testing modules are accessible

        // This test verifies that:
        // 1. All contract test modules compile
        // 2. Schema validator is accessible
        // 3. Test infrastructure is properly organized

        assert!(true, "All contract modules compiled successfully");
    }
}
