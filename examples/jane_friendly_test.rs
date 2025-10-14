//! Jane-friendly cleanroom testing example
//!
//! This example shows how Jane (a typical developer) would use the cleanroom
//! testing library with the new Jane-friendly API.

use clnrm::{cleanroom_test, with_database, with_cache, with_message_queue, database, cache, email_service, UserAssertions};

/// Example user registration service (Jane's business logic)
struct UserService {
    database_url: String,
    cache_url: String,
}

impl UserService {
    fn new(database_url: String, cache_url: String) -> Self {
        Self {
            database_url,
            cache_url,
        }
    }

    /// Register a new user (Jane's business logic)
    async fn register_user(&self, email: &str, password: &str) -> Result<User, Box<dyn std::error::Error>> {
        // Simulate user registration
        println!("📝 Registering user: {}", email);
        
        // In a real implementation, this would:
        // 1. Hash the password
        // 2. Insert user into database
        // 3. Create user session in cache
        // 4. Send welcome email
        
        let user = User {
            id: 123, // This would come from the database
            email: email.to_string(),
            role: "user".to_string(),
        };
        
        println!("✅ User registered successfully: {}", user.id);
        Ok(user)
    }
}

/// User model (Jane's domain model)
#[derive(Debug, Clone)]
struct User {
    id: i64,
    email: String,
    role: String,
}

impl User {
    /// Create user assertions for this user
    fn should_exist_in_database(&self) -> UserAssertions {
        UserAssertions::new(self.id, self.email.clone())
    }
}

/// Jane's complete user registration test
/// 
/// This is what Jane actually wants to write - simple, declarative, and focused
/// on her business logic rather than infrastructure setup.
#[cleanroom_test]
async fn test_complete_user_registration() {
    // 🚀 Declarative service setup (Jane's one-liners)
    with_database("postgres:15");
    with_cache("redis:7");
    with_message_queue("rabbitmq:3");
    
    // 📝 Jane's business logic (what she actually cares about)
    let user_service = UserService::new(
        "postgresql://postgres:password@localhost:5432/testdb".to_string(),
        "redis://localhost:6379".to_string(),
    );
    
    let user = user_service.register_user("jane@example.com", "password123").await?;
    
    // ✅ Rich assertions (Jane's domain-specific checks)
    user.should_exist_in_database().should_exist_in_database().await?;
    user.should_exist_in_database().should_have_role("user").await?;
    user.should_exist_in_database().should_receive_email().await?;
    user.should_exist_in_database().should_have_session().await?;
    
    // 🔍 Service-level assertions (automatic verification)
    database().await?.should_have_user_count(1).await?;
    cache().await?.should_have_user_session(user.id).await?;
    email_service().await?.should_have_sent_welcome_email("jane@example.com").await?;
    
    println!("🎉 Complete user registration test passed!");
}

/// Jane's concurrent test (multiple users)
#[cleanroom_test]
async fn test_concurrent_user_registration() {
    // 🚀 Set up services
    with_database("postgres:15");
    with_cache("redis:7");
    
    // 📝 Jane's concurrent business logic
    let user_service = UserService::new(
        "postgresql://postgres:password@localhost:5432/testdb".to_string(),
        "redis://localhost:6379".to_string(),
    );
    
    // Register multiple users concurrently
    let users = vec![
        user_service.register_user("alice@example.com", "password123"),
        user_service.register_user("bob@example.com", "password456"),
        user_service.register_user("charlie@example.com", "password789"),
    ];
    
    let results = futures::future::join_all(users).await;
    
    // ✅ Verify all users were registered
    for result in results {
        let user = result?;
        user.should_exist_in_database().should_exist_in_database().await?;
    }
    
    // 🔍 Verify database state
    database().await?.should_have_user_count(3).await?;
    
    println!("🎉 Concurrent user registration test passed!");
}

/// Jane's error handling test
#[cleanroom_test]
async fn test_user_registration_validation() {
    // 🚀 Set up services
    with_database("postgres:15");
    
    // 📝 Jane's validation logic
    let user_service = UserService::new(
        "postgresql://postgres:password@localhost:5432/testdb".to_string(),
        "redis://localhost:6379".to_string(),
    );
    
    // Test invalid email
    let result = user_service.register_user("invalid-email", "password123").await;
    
    // ✅ Jane expects clear error messages
    assert!(result.is_err(), "Should fail with invalid email");
    
    // Test empty password
    let result = user_service.register_user("valid@example.com", "").await;
    assert!(result.is_err(), "Should fail with empty password");
    
    println!("🎉 User validation test passed!");
}

/// Jane's integration test with external services
#[cleanroom_test]
async fn test_user_registration_with_external_services() {
    // 🚀 Set up all services Jane needs
    with_database("postgres:15");
    with_cache("redis:7");
    with_message_queue("rabbitmq:3");
    with_web_server("nginx:alpine");
    
    // 📝 Jane's integration logic
    let user_service = UserService::new(
        "postgresql://postgres:password@localhost:5432/testdb".to_string(),
        "redis://localhost:6379".to_string(),
    );
    
    let user = user_service.register_user("jane@example.com", "password123").await?;
    
    // ✅ Comprehensive verification
    user.should_exist_in_database().should_exist_in_database().await?;
    user.should_exist_in_database().should_have_role("user").await?;
    user.should_exist_in_database().should_receive_email().await?;
    user.should_exist_in_database().should_have_session().await?;
    
    // 🔍 Service integration verification
    database().await?.should_have_user_count(1).await?;
    cache().await?.should_have_user_session(user.id).await?;
    email_service().await?.should_have_sent_count(1).await?;
    
    println!("🎉 Integration test with external services passed!");
}

/// Jane's performance test
#[cleanroom_test]
async fn test_user_registration_performance() {
    // 🚀 Set up services
    with_database("postgres:15");
    with_cache("redis:7");
    
    // 📝 Jane's performance test logic
    let user_service = UserService::new(
        "postgresql://postgres:password@localhost:5432/testdb".to_string(),
        "redis://localhost:6379".to_string(),
    );
    
    let start_time = std::time::Instant::now();
    
    // Register 100 users
    let mut tasks = Vec::new();
    for i in 0..100 {
        let email = format!("user{}@example.com", i);
        tasks.push(user_service.register_user(&email, "password123"));
    }
    
    let results = futures::future::join_all(tasks).await;
    
    let duration = start_time.elapsed();
    
    // ✅ Verify all registrations succeeded
    for result in results {
        result?;
    }
    
    // 🔍 Verify final state
    database().await?.should_have_user_count(100).await?;
    
    println!("🎉 Performance test passed! Registered 100 users in {:?}", duration);
    assert!(duration.as_secs() < 10, "Should complete within 10 seconds");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Running Jane-friendly cleanroom tests...");
    
    // In a real scenario, Jane would run these with: cargo test
    // For this example, we'll just show the structure
    
    println!("✅ Jane-friendly API is ready to use!");
    println!("📝 Jane can now write tests like:");
    println!("   #[cleanroom_test]");
    println!("   async fn test_my_feature() {{");
    println!("       with_database(\"postgres:15\");");
    println!("       with_cache(\"redis:7\");");
    println!("       // ... her business logic");
    println!("       user.should_exist_in_database().await?;");
    println!("   }}");
    
    Ok(())
}
