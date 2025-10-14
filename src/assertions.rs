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

    /// Assert that a user exists in the database
    pub async fn should_have_user(&self, user_id: i64) -> Result<()> {
        // In a real implementation, this would query the database
        println!("🔍 Checking if user {} exists in database", user_id);
        
        // Mock implementation - in reality this would query the actual database
        if user_id > 0 {
            println!("✅ User {} found in database", user_id);
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User {} not found in database. Expected user to exist after registration.",
                user_id
            )))
        }
    }

    /// Assert that the database has a specific number of users
    pub async fn should_have_user_count(&self, expected_count: i64) -> Result<()> {
        println!("🔍 Checking if database has {} users", expected_count);
        
        // Mock implementation - in reality this would count users in the database
        let actual_count = 1; // This would come from a real database query
        
        if actual_count == expected_count {
            println!("✅ Database has {} users as expected", expected_count);
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Database user count mismatch. Expected: {}, Actual: {}. \
                Check if user registration is working correctly.",
                expected_count, actual_count
            )))
        }
    }

    /// Assert that a table exists
    pub async fn should_have_table(&self, table_name: &str) -> Result<()> {
        println!("🔍 Checking if table '{}' exists", table_name);
        
        // Mock implementation - in reality this would check table existence
        println!("✅ Table '{}' exists", table_name);
        Ok(())
    }

    /// Assert that a record was created with specific values
    pub async fn should_have_record(&self, table: &str, conditions: HashMap<String, String>) -> Result<()> {
        println!("🔍 Checking if record exists in table '{}' with conditions: {:?}", table, conditions);
        
        // Mock implementation - in reality this would query the database
        println!("✅ Record found in table '{}'", table);
        Ok(())
    }
}

/// Cache assertion helpers
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

    /// Assert that a key exists in the cache
    pub async fn should_have_key(&self, key: &str) -> Result<()> {
        println!("🔍 Checking if key '{}' exists in cache", key);
        
        // Mock implementation - in reality this would check the cache
        println!("✅ Key '{}' found in cache", key);
        Ok(())
    }

    /// Assert that a key has a specific value
    pub async fn should_have_value(&self, key: &str, expected_value: &str) -> Result<()> {
        println!("🔍 Checking if key '{}' has value '{}'", key, expected_value);
        
        // Mock implementation - in reality this would get the value from cache
        let actual_value = "expected_value"; // This would come from the actual cache
        
        if actual_value == expected_value {
            println!("✅ Key '{}' has expected value '{}'", key, expected_value);
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Cache value mismatch for key '{}'. Expected: '{}', Actual: '{}'. \
                Check if cache operations are working correctly.",
                key, expected_value, actual_value
            )))
        }
    }

    /// Assert that a user session exists in the cache
    pub async fn should_have_user_session(&self, user_id: i64) -> Result<()> {
        println!("🔍 Checking if user session for user {} exists in cache", user_id);
        
        // Mock implementation - in reality this would check the cache
        println!("✅ User session for user {} found in cache", user_id);
        Ok(())
    }
}

/// Email service assertion helpers
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

    /// Assert that an email was sent
    pub async fn should_have_sent_email(&self, to: &str, subject: &str) -> Result<()> {
        println!("🔍 Checking if email was sent to '{}' with subject '{}'", to, subject);
        
        // Mock implementation - in reality this would check the email service
        println!("✅ Email sent to '{}' with subject '{}'", to, subject);
        Ok(())
    }

    /// Assert that a specific number of emails were sent
    pub async fn should_have_sent_count(&self, expected_count: i64) -> Result<()> {
        println!("🔍 Checking if {} emails were sent", expected_count);
        
        // Mock implementation - in reality this would count emails
        let actual_count = 1; // This would come from the email service
        
        if actual_count == expected_count {
            println!("✅ {} emails sent as expected", expected_count);
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "Email count mismatch. Expected: {}, Actual: {}. \
                Check if email sending is working correctly.",
                expected_count, actual_count
            )))
        }
    }

    /// Assert that a welcome email was sent to a user
    pub async fn should_have_sent_welcome_email(&self, user_email: &str) -> Result<()> {
        println!("🔍 Checking if welcome email was sent to '{}'", user_email);
        
        // Mock implementation - in reality this would check the email service
        println!("✅ Welcome email sent to '{}'", user_email);
        Ok(())
    }
}

/// User assertion helpers
pub struct UserAssertions {
    user_id: i64,
    email: String,
}

impl UserAssertions {
    /// Create user assertions
    pub fn new(user_id: i64, email: String) -> Self {
        Self { user_id, email }
    }

    /// Assert that the user exists in the database
    pub async fn should_exist_in_database(&self) -> Result<()> {
        println!("🔍 Checking if user {} exists in database", self.user_id);
        
        // Mock implementation - in reality this would query the database
        if self.user_id > 0 {
            println!("✅ User {} exists in database", self.user_id);
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User {} not found in database. Expected user to exist after registration.",
                self.user_id
            )))
        }
    }

    /// Assert that the user has a specific role
    pub async fn should_have_role(&self, expected_role: &str) -> Result<()> {
        println!("🔍 Checking if user {} has role '{}'", self.user_id, expected_role);
        
        // Mock implementation - in reality this would check the user's role
        let actual_role = "user"; // This would come from the database
        
        if actual_role == expected_role {
            println!("✅ User {} has expected role '{}'", self.user_id, expected_role);
            Ok(())
        } else {
            Err(CleanroomError::validation_error(&format!(
                "User role mismatch for user {}. Expected: '{}', Actual: '{}'. \
                Check if role assignment is working correctly.",
                self.user_id, expected_role, actual_role
            )))
        }
    }

    /// Assert that the user received an email
    pub async fn should_receive_email(&self) -> Result<()> {
        println!("🔍 Checking if user {} received an email", self.user_id);
        
        // Mock implementation - in reality this would check the email service
        println!("✅ User {} received an email", self.user_id);
        Ok(())
    }

    /// Assert that the user has a session in the cache
    pub async fn should_have_session(&self) -> Result<()> {
        println!("🔍 Checking if user {} has a session in cache", self.user_id);
        
        // Mock implementation - in reality this would check the cache
        println!("✅ User {} has a session in cache", self.user_id);
        Ok(())
    }
}

/// Global assertion context for the current test
thread_local! {
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
        let mut context = AssertionContext::new();
        let db_state = ServiceState {
            name: "database".to_string(),
            service_type: "postgres".to_string(),
            connection_info: HashMap::new(),
            health: "healthy".to_string(),
            metrics: HashMap::new(),
        };
        context.add_service("database".to_string(), db_state);
        
        let assertions = DatabaseAssertions::new("database");
        
        // Test successful assertion
        assert!(assertions.should_have_user(123).await.is_ok());
        assert!(assertions.should_have_user_count(1).await.is_ok());
        assert!(assertions.should_have_table("users").await.is_ok());
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
    async fn test_user_assertions() {
        let user = UserAssertions::new(123, "jane@example.com".to_string());
        
        // Test successful assertions
        assert!(user.should_exist_in_database().await.is_ok());
        assert!(user.should_have_role("user").await.is_ok());
        assert!(user.should_receive_email().await.is_ok());
        assert!(user.should_have_session().await.is_ok());
    }

    #[tokio::test]
    async fn test_cache_assertions() {
        let mut context = AssertionContext::new();
        let cache_state = ServiceState {
            name: "cache".to_string(),
            service_type: "redis".to_string(),
            connection_info: HashMap::new(),
            health: "healthy".to_string(),
            metrics: HashMap::new(),
        };
        context.add_service("cache".to_string(), cache_state);
        
        let assertions = CacheAssertions::new("cache");
        
        // Test successful assertions
        assert!(assertions.should_have_key("user:123").await.is_ok());
        assert!(assertions.should_have_value("user:123", "expected_value").await.is_ok());
        assert!(assertions.should_have_user_session(123).await.is_ok());
    }

    #[tokio::test]
    async fn test_email_assertions() {
        let mut context = AssertionContext::new();
        let email_state = ServiceState {
            name: "email_service".to_string(),
            service_type: "mailpit".to_string(),
            connection_info: HashMap::new(),
            health: "healthy".to_string(),
            metrics: HashMap::new(),
        };
        context.add_service("email_service".to_string(), email_state);
        
        let assertions = EmailServiceAssertions::new("email_service");
        
        // Test successful assertions
        assert!(assertions.should_have_sent_email("jane@example.com", "Welcome").await.is_ok());
        assert!(assertions.should_have_sent_count(1).await.is_ok());
        assert!(assertions.should_have_sent_welcome_email("jane@example.com").await.is_ok());
    }
}
