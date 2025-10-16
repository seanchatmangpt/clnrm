//! Simple Jane-friendly test example
//!
//! This shows how Jane would actually use the cleanroom testing library
//! with the new Jane-friendly API.

use clnrm_core::{cleanroom_test, with_database, with_cache, database, cache, email_service, UserAssertions, CleanroomError, Result};

/// Jane's simple user registration test
/// 
/// This is what Jane actually wants to write - simple, declarative, and focused
/// on her business logic rather than infrastructure setup.
#[tokio::main]
async fn test_user_registration() -> Result<()> {
    // 🚀 Declarative service setup (Jane's one-liners)
    with_database("postgres:15").await?;
    with_cache("redis:7").await?;
    
    // 📝 Jane's business logic (what she actually cares about)
    let user_id = register_user("jane@example.com", "password123").await?;
    
    // ✅ Rich assertions (Jane's domain-specific checks)
    let user = UserAssertions::new(user_id, "jane@example.com".to_string());
    user.should_exist_in_database().await?;
    user.should_have_role("user").await?;
    user.should_receive_email().await?;
    user.should_have_session().await?;
    
    // 🔍 Service-level assertions (automatic verification)
    let db = database().await?;
    let cache = cache().await?;
    let email = email_service().await?;

    // Verify services are working
    db.should_have_table("users").await?;
    cache.should_have_key("user_sessions").await?;
    email.should_have_sent_count(0).await?; // No emails sent yet
    
    println!("🎉 User registration test passed!");
    Ok(())
}

/// Mock user registration function (Jane's business logic)
async fn register_user(email: &str, _password: &str) -> Result<i64, Box<dyn std::error::Error>> {
    println!("📝 Registering user: {}", email);
    
    // Simulate user registration
    let user_id = 123; // This would come from the database
    
    println!("✅ User registered successfully: {}", user_id);
    Ok(user_id)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Running Jane-friendly cleanroom test...");
    
    // This would normally be run with: cargo test test_user_registration
    // For this example, we'll just show the structure
    
    println!("✅ Jane-friendly API is ready to use!");
    println!("📝 Jane can now write tests like:");
    println!("   #[tokio::main]");
    println!("   async fn test_my_feature() -> Result<(), CleanroomError> {{");
    println!("       with_database(\"postgres:15\").await?;");
    println!("       with_cache(\"redis:7\").await?;");
    println!("       // ... her business logic");
    println!("       user.should_exist_in_database().await?;");
    println!("       Ok(())");
    println!("   }}");

    Ok(())
}
