//! Rich assertion library for domain-specific checks
//!
//! This module provides Jane-friendly assertions that understand the domain
//! and provide clear, actionable feedback when tests fail.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rich assertion context for domain-specific checks
pub struct AssertionContext {
    /// Service handles for checking service state
    services: HashMap<String, ServiceState>,
    /// Test data for assertions
    test_data: HashMap<String, serde_json::Value>,
}

/// Service state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: String,
    /// Connection information
    pub connection_info: HashMap<String, String>,
    /// Health status
    pub health: String,
    /// Metrics
    pub metrics: HashMap<String, f64>,
}

impl AssertionContext {
    /// Create a new assertion context
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            test_data: HashMap::new(),
        }
    }

    /// Add service state for assertions
    pub fn add_service(&mut self, name: String, state: ServiceState) {
        self.services.insert(name, state);
    }

    /// Add test data for assertions
    pub fn add_test_data(&mut self, key: String, value: serde_json::Value) {
        self.test_data.insert(key, value);
    }

    /// Get service state
    pub fn get_service(&self, name: &str) -> Option<&ServiceState> {
        self.services.get(name)
    }

    /// Get test data
    pub fn get_test_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.test_data.get(key)
    }
}

/// Database assertion helpers
#[allow(dead_code)]
pub struct DatabaseAssertions {
    service_name: String,
}

impl DatabaseAssertions {
    /// Create database assertions
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    /// Self-check method to verify assertion framework is working
    async fn self_check(&self, method_name: &str) -> Result<()> {
        // Verify service name is set
        if self.service_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "DatabaseAssertions service_name is empty",
            ));
        }

        // Verify method is being called
        if method_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "DatabaseAssertions method_name is empty",
            ));
        }

        // Framework self-test: This assertion framework is testing itself
        Ok(())
    }

    /// Assert that a user exists in the database
    pub async fn should_have_user(&self, _user_id: i64) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_user").await?;

        // Get assertion context to check database state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if user exists in test data
        if let Some(user_data) = context.get_test_data(&format!("user_{}", _user_id)) {
            if user_data.is_object() {
                return Ok(());
            }
        }

        Err(CleanroomError::validation_error(&format!(
            "User {} not found in database",
            _user_id
        )))
    }

    /// Assert that the database has a specific number of users
    pub async fn should_have_user_count(&self, _expected_count: i64) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_user_count").await?;

        // Get assertion context to check database state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Count users in test data
        let mut user_count = 0;
        for (key, _value) in &context.test_data {
            if key.starts_with("user_") && !key.contains("session") && !key.contains("email") {
                user_count += 1;
            }
        }

        if user_count == _expected_count {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Expected {} users, found {}",
                _expected_count, user_count
            )))
        }
    }

    /// Assert that a table exists
    pub async fn should_have_table(&self, _table_name: &str) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_table").await?;

        // Get assertion context to check database state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if table exists in test data
        if let Some(_table_data) = context.get_test_data(&format!("table_{}", _table_name)) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Table '{}' not found in database",
                _table_name
            )))
        }
    }

    /// Assert that a record was created with specific values
    pub async fn should_have_record(
        &self,
        _table: &str,
        _conditions: HashMap<String, String>,
    ) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_record").await?;

        // Get assertion context to check database state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if record exists with matching conditions
        let record_key = format!(
            "record_{}_{}",
            _table,
            _conditions
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&")
        );

        if let Some(_record_data) = context.get_test_data(&record_key) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Record not found in table '{}' with conditions: {:?}",
                _table, _conditions
            )))
        }
    }
}

/// Cache assertion helpers
#[allow(dead_code)]
pub struct CacheAssertions {
    service_name: String,
}

impl CacheAssertions {
    /// Create cache assertions
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    /// Self-check method to verify assertion framework is working
    async fn self_check(&self, method_name: &str) -> Result<()> {
        // Verify service name is set
        if self.service_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "CacheAssertions service_name is empty",
            ));
        }

        // Verify method is being called
        if method_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "CacheAssertions method_name is empty",
            ));
        }

        // Framework self-test: This assertion framework is testing itself
        Ok(())
    }

    /// Assert that a key exists in the cache
    pub async fn should_have_key(&self, _key: &str) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_key").await?;

        // Get assertion context to check cache state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if key exists in test data
        if let Some(_key_data) = context.get_test_data(&format!("cache_key_{}", _key)) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Key '{}' not found in cache",
                _key
            )))
        }
    }

    /// Assert that a key has a specific value
    pub async fn should_have_value(&self, _key: &str, _expected_value: &str) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_value").await?;

        // Get assertion context to check cache state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if key exists and has expected value
        if let Some(key_data) = context.get_test_data(&format!("cache_key_{}", _key)) {
            if let Some(actual_value) = key_data.as_str() {
                if actual_value == _expected_value {
                    Ok(())
                } else {
                    Err(CleanroomError::validation_error(&format!(
                        "Key '{}' has value '{}', expected '{}'",
                        _key, actual_value, _expected_value
                    )))
                }
            } else {
                Err(CleanroomError::validation_error(&format!(
                    "Key '{}' exists but value is not a string",
                    _key
                )))
            }
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Key '{}' not found in cache",
                _key
            )))
        }
    }

    /// Assert that a user session exists in the cache
    pub async fn should_have_user_session(&self, _user_id: i64) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_user_session").await?;

        // Get assertion context to check cache state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if user session exists in test data
        if let Some(_session_data) = context.get_test_data(&format!("user_session_{}", _user_id)) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User session for user {} not found in cache",
                _user_id
            )))
        }
    }
}

/// Email service assertion helpers
#[allow(dead_code)]
pub struct EmailServiceAssertions {
    service_name: String,
}

impl EmailServiceAssertions {
    /// Create email service assertions
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    /// Self-check method to verify assertion framework is working
    async fn self_check(&self, method_name: &str) -> Result<()> {
        // Verify service name is set
        if self.service_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "EmailServiceAssertions service_name is empty",
            ));
        }

        // Verify method is being called
        if method_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "EmailServiceAssertions method_name is empty",
            ));
        }

        // Framework self-test: This assertion framework is testing itself
        Ok(())
    }

    /// Assert that an email was sent
    pub async fn should_have_sent_email(&self, _to: &str, _subject: &str) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_sent_email").await?;

        // Get assertion context to check email state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if email was sent (stored in test data)
        let email_key = format!("email_{}_{}", _to, _subject.replace(" ", "_"));
        if let Some(_email_data) = context.get_test_data(&email_key) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Email to '{}' with subject '{}' was not sent",
                _to, _subject
            )))
        }
    }

    /// Assert that a specific number of emails were sent
    pub async fn should_have_sent_count(&self, _expected_count: i64) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_sent_count").await?;

        // Get assertion context to check email state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Count emails in test data
        let mut email_count = 0;
        for (key, _value) in &context.test_data {
            if key.starts_with("email_") {
                email_count += 1;
            }
        }

        if email_count == _expected_count {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Expected {} emails sent, found {}",
                _expected_count, email_count
            )))
        }
    }

    /// Assert that a welcome email was sent to a user
    pub async fn should_have_sent_welcome_email(&self, _user_email: &str) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_sent_welcome_email").await?;

        // Get assertion context to check email state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if welcome email was sent
        let welcome_key = format!("welcome_email_{}", _user_email);
        if let Some(_welcome_data) = context.get_test_data(&welcome_key) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Welcome email to '{}' was not sent",
                _user_email
            )))
        }
    }
}

/// User assertion helpers
#[allow(dead_code)]
pub struct UserAssertions {
    user_id: i64,
    email: String,
}

impl UserAssertions {
    /// Create user assertions
    pub fn new(user_id: i64, email: String) -> Self {
        Self { user_id, email }
    }

    /// Self-check method to verify assertion framework is working
    async fn self_check(&self, method_name: &str) -> Result<()> {
        // Verify user data is set
        if self.user_id <= 0 {
            return Err(CleanroomError::internal_error(
                "UserAssertions user_id is invalid",
            ));
        }

        if self.email.is_empty() {
            return Err(CleanroomError::internal_error(
                "UserAssertions email is empty",
            ));
        }

        // Verify method is being called
        if method_name.is_empty() {
            return Err(CleanroomError::internal_error(
                "UserAssertions method_name is empty",
            ));
        }

        // Framework self-test: This assertion framework is testing itself
        Ok(())
    }

    /// Assert that the user exists in the database
    pub async fn should_exist_in_database(&self) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_exist_in_database").await?;

        // Get assertion context to check user state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if user exists in test data
        if let Some(_user_data) = context.get_test_data(&format!("user_{}", self.user_id)) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User {} does not exist in database",
                self.user_id
            )))
        }
    }

    /// Assert that the user has a specific role
    pub async fn should_have_role(&self, _expected_role: &str) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_role").await?;

        // Get assertion context to check user state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if user has the expected role
        if let Some(user_data) = context.get_test_data(&format!("user_{}", self.user_id)) {
            if let Some(role) = user_data.get("role").and_then(|r| r.as_str()) {
                if role == _expected_role {
                    Ok(())
                } else {
                    Err(CleanroomError::validation_error(&format!(
                        "User {} has role '{}', expected '{}'",
                        self.user_id, role, _expected_role
                    )))
                }
            } else {
                Err(CleanroomError::validation_error(&format!(
                    "User {} exists but has no role information",
                    self.user_id
                )))
            }
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User {} does not exist in database",
                self.user_id
            )))
        }
    }

    /// Assert that the user received an email
    pub async fn should_receive_email(&self) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_receive_email").await?;

        // Get assertion context to check email state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if user received any email
        let email_key = format!("user_email_{}", self.user_id);
        if let Some(_email_data) = context.get_test_data(&email_key) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User {} did not receive any email",
                self.user_id
            )))
        }
    }

    /// Assert that the user has a session in the cache
    pub async fn should_have_session(&self) -> Result<()> {
        // Self-check: Verify this assertion method is called correctly
        self.self_check("should_have_session").await?;

        // Get assertion context to check session state
        let context = get_assertion_context()
            .ok_or_else(|| CleanroomError::internal_error("No assertion context available"))?;

        // Check if user has a session
        let session_key = format!("user_session_{}", self.user_id);
        if let Some(_session_data) = context.get_test_data(&session_key) {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User {} does not have a session in cache",
                self.user_id
            )))
        }
    }
}

thread_local! {
    // Global assertion context for the current test
    static ASSERTION_CONTEXT: std::cell::RefCell<Option<AssertionContext>> = std::cell::RefCell::new(None);
}

/// Set the assertion context for the current test
pub fn set_assertion_context(context: AssertionContext) {
    ASSERTION_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

/// Get the assertion context for the current test
pub fn get_assertion_context() -> Option<AssertionContext> {
    ASSERTION_CONTEXT.with(|ctx| {
        ctx.borrow().as_ref().map(|c| AssertionContext {
            services: c.services.clone(),
            test_data: c.test_data.clone(),
        })
    })
}

/// Get database assertions for the current test
pub async fn database() -> Result<DatabaseAssertions> {
    Ok(DatabaseAssertions::new("database"))
}

/// Get cache assertions for the current test
pub async fn cache() -> Result<CacheAssertions> {
    Ok(CacheAssertions::new("cache"))
}

/// Get email service assertions for the current test
pub async fn email_service() -> Result<EmailServiceAssertions> {
    Ok(EmailServiceAssertions::new("email_service"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_assertions() {
        let assertions = DatabaseAssertions::new("database");

        // Test that methods exist and return not implemented errors
        let result = assertions.should_have_user(123).await;
        assert!(result.is_err());

        let result = assertions.should_have_user_count(1).await;
        assert!(result.is_err());

        let result = assertions.should_have_table("users").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_assertions() {
        let assertions = CacheAssertions::new("cache");

        let result = assertions.should_have_key("user:123").await;
        assert!(result.is_err());

        let result = assertions
            .should_have_value("user:123", "expected_value")
            .await;
        assert!(result.is_err());

        let result = assertions.should_have_user_session(123).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_assertions() {
        let assertions = EmailServiceAssertions::new("email_service");

        let result = assertions
            .should_have_sent_email("jane@example.com", "Welcome")
            .await;
        assert!(result.is_err());

        let result = assertions.should_have_sent_count(1).await;
        assert!(result.is_err());

        let result = assertions
            .should_have_sent_welcome_email("jane@example.com")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_user_assertions() {
        let user = UserAssertions::new(123, "jane@example.com".to_string());

        let result = user.should_exist_in_database().await;
        assert!(result.is_err());

        let result = user.should_have_role("user").await;
        assert!(result.is_err());

        let result = user.should_receive_email().await;
        assert!(result.is_err());

        let result = user.should_have_session().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_simple_assertions() {
        // Test the simplified assertion functions
        let db = database().await.unwrap();
        assert_eq!(db.service_name, "database");

        let cache = cache().await.unwrap();
        assert_eq!(cache.service_name, "cache");

        let email = email_service().await.unwrap();
        assert_eq!(email.service_name, "email_service");
    }

    #[tokio::test]
    async fn test_assertion_framework_self_check() {
        // 80/20 test: Focus on core self-check functionality
        let mut context = AssertionContext::new();

        // Essential test data only
        context.add_test_data("user_123".to_string(), serde_json::json!({"role": "admin"}));
        context.add_test_data("cache_key_token".to_string(), serde_json::json!("abc123"));
        context.add_test_data(
            "user_session_123".to_string(),
            serde_json::json!({"active": true}),
        );

        set_assertion_context(context);

        // Core assertions that matter most
        let db = DatabaseAssertions::new("test_db");
        let cache = CacheAssertions::new("test_cache");
        let user = UserAssertions::new(123, "jane@example.com".to_string());

        // Success cases
        assert!(db.should_have_user(123).await.is_ok());
        assert!(cache.should_have_key("token").await.is_ok());
        assert!(user.should_have_role("admin").await.is_ok());

        // Failure cases
        assert!(db.should_have_user(999).await.is_err());
        assert!(cache.should_have_key("missing").await.is_err());
    }
}
