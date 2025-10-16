//! SurrealDB + Ollama Integration Example
//!
//! Demonstrates real AI intelligence by combining SurrealDB for data persistence
//! and Ollama for AI processing to provide actual intelligent analysis.

use clnrm_core::{
    cleanroom::{CleanroomEnvironment, ServicePlugin, ServiceHandle},
    services::{surrealdb::SurrealDbPlugin, ollama::{OllamaPlugin, OllamaConfig}},
    error::{CleanroomError, Result},
};
use std::collections::HashMap;
use serde_json::{json, Value};
use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Surreal};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 SurrealDB + Ollama Integration Demo");
    println!("=====================================");
    println!("This example demonstrates real AI intelligence by combining:");
    println!("• SurrealDB for data persistence and querying");
    println!("• Ollama for AI-powered analysis and insights");
    println!();

    // Phase 1: Start SurrealDB
    println!("📊 Phase 1: Starting SurrealDB");
    let surrealdb_plugin = SurrealDbPlugin::new();
    let db_handle = surrealdb_plugin.start().await?;
    
    let host = db_handle.metadata.get("host").unwrap();
    let port = db_handle.metadata.get("port").unwrap().parse::<u16>()?;
    println!("✅ SurrealDB started at {}:{}", host, port);

    // Phase 2: Connect to SurrealDB and set up schema
    println!("\n📊 Phase 2: Setting up SurrealDB schema");
    let db = setup_surrealdb_connection(host, port).await?;
    setup_ai_schema(&db).await?;
    println!("✅ SurrealDB schema initialized");

    // Phase 3: Store test execution data
    println!("\n📊 Phase 3: Storing test execution data");
    let test_executions = generate_test_executions();
    for execution in &test_executions {
        store_test_execution(&db, execution).await?;
    }
    println!("✅ Stored {} test executions", test_executions.len());

    // Phase 4: Start Ollama (if available)
    println!("\n🧠 Phase 4: Starting Ollama AI service");
    let ollama_config = OllamaConfig {
        endpoint: "http://localhost:11434".to_string(),
        default_model: "llama3.2:3b".to_string(),
        timeout_seconds: 120,
    };
    let ollama_plugin = OllamaPlugin::new("ollama", ollama_config);
    
    match ollama_plugin.start().await {
        Ok(ollama_handle) => {
            println!("✅ Ollama started successfully");
            println!("   Endpoint: {}", ollama_handle.metadata.get("endpoint").unwrap_or(&"unknown".to_string()));
            println!("   Model: {}", ollama_handle.metadata.get("default_model").unwrap_or(&"unknown".to_string()));

            // Phase 5: AI-powered analysis
            println!("\n🧠 Phase 5: AI-Powered Analysis");
            let analysis = perform_ai_analysis(&db, &ollama_plugin).await?;
            display_ai_analysis(&analysis).await?;
        }
        Err(e) => {
            println!("⚠️ Ollama not available: {}", e);
            println!("   Continuing with SurrealDB-only analysis...");
            
            // Phase 5: SurrealDB-only analysis
            println!("\n📊 Phase 5: SurrealDB Analysis");
            let analysis = perform_surrealdb_analysis(&db).await?;
            display_surrealdb_analysis(&analysis).await?;
        }
    }

    // Phase 6: Cleanup
    println!("\n🧹 Phase 6: Cleanup");
    surrealdb_plugin.stop(db_handle).await?;
    println!("✅ SurrealDB stopped");

    println!("\n🎉 SurrealDB + Ollama integration demo completed!");
    println!("📊 This demonstrates real data persistence and AI processing");
    
    Ok(())
}

/// Set up SurrealDB connection
async fn setup_surrealdb_connection(host: &str, port: u16) -> Result<Surreal<Client>> {
    let url = format!("{}:{}", host, port);
    let db: Surreal<Client> = Surreal::init();
    
    db.connect::<Ws>(url)
        .await
        .map_err(|e| CleanroomError::connection_failed("Failed to connect to SurrealDB")
            .with_source(e.to_string()))?;
    
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .map_err(|e| CleanroomError::service_error("Failed to authenticate with SurrealDB")
        .with_source(e.to_string()))?;

    // Use the test namespace and database
    db.use_ns("test").use_db("test")
        .await
        .map_err(|e| CleanroomError::service_error("Failed to use test namespace/database")
            .with_source(e.to_string()))?;

    Ok(db)
}

/// Set up AI schema in SurrealDB
async fn setup_ai_schema(db: &Surreal<Client>) -> Result<()> {
    // Create test_executions table
    let _ = db.query("
        DEFINE TABLE test_executions SCHEMAFULL;
        DEFINE FIELD test_name ON test_executions TYPE string;
        DEFINE FIELD timestamp ON test_executions TYPE datetime;
        DEFINE FIELD success ON test_executions TYPE bool;
        DEFINE FIELD execution_time_ms ON test_executions TYPE int;
        DEFINE FIELD error_message ON test_executions TYPE option<string>;
        DEFINE FIELD resource_usage ON test_executions TYPE object;
        DEFINE INDEX test_name_idx ON test_executions COLUMNS test_name;
        DEFINE INDEX timestamp_idx ON test_executions COLUMNS timestamp;
    ").await
    .map_err(|e| CleanroomError::service_error("Failed to create test_executions table")
        .with_source(e.to_string()))?;

    // Create ai_insights table
    let _ = db.query("
        DEFINE TABLE ai_insights SCHEMAFULL;
        DEFINE FIELD insight_type ON ai_insights TYPE string;
        DEFINE FIELD content ON ai_insights TYPE string;
        DEFINE FIELD confidence ON ai_insights TYPE float;
        DEFINE FIELD actionable ON ai_insights TYPE bool;
        DEFINE FIELD created_at ON ai_insights TYPE datetime;
        DEFINE INDEX insight_type_idx ON ai_insights COLUMNS insight_type;
    ").await
    .map_err(|e| CleanroomError::service_error("Failed to create ai_insights table")
        .with_source(e.to_string()))?;

    Ok(())
}

/// Generate sample test execution data
fn generate_test_executions() -> Vec<TestExecution> {
    let mut executions = Vec::new();
    let now = chrono::Utc::now();

    let test_scenarios = vec![
        ("integration_test", 0.95, 2000),
        ("api_test", 0.90, 1500),
        ("performance_test", 0.75, 8000),
        ("security_test", 0.85, 5000),
        ("ui_test", 0.80, 12000),
    ];

    for (test_name, success_rate, avg_time) in test_scenarios {
        for i in 0..5 {
            let success = rand::random::<f64>() < success_rate;
            let execution_time = avg_time + (rand::random::<i32>() % 1000 - 500) as u64;
            
            executions.push(TestExecution {
                test_name: test_name.to_string(),
                timestamp: now - chrono::Duration::hours(i as i64),
                success,
                execution_time_ms: execution_time.max(100),
                error_message: if success { 
                    None 
                } else { 
                    Some(format!("{} failed: timeout after {}ms", test_name, execution_time))
                },
                resource_usage: ResourceUsage {
                    cpu_percent: rand::random::<f32>() * 50.0 + 10.0,
                    memory_mb: rand::random::<u64>() % 500 + 100,
                    network_io_mb: rand::random::<u64>() % 100,
                    disk_io_mb: rand::random::<u64>() % 50,
                },
            });
        }
    }

    executions
}

/// Store test execution in SurrealDB
async fn store_test_execution(db: &Surreal<Client>, execution: &TestExecution) -> Result<()> {
    let _: Option<Value> = db.create("test_executions")
        .content(json!({
            "test_name": execution.test_name,
            "timestamp": execution.timestamp,
            "success": execution.success,
            "execution_time_ms": execution.execution_time_ms,
            "error_message": execution.error_message,
            "resource_usage": execution.resource_usage
        }))
        .await
        .map_err(|e| CleanroomError::service_error("Failed to store test execution")
            .with_source(e.to_string()))?;

    Ok(())
}

/// Perform AI analysis using both SurrealDB and Ollama
async fn perform_ai_analysis(
    db: &Surreal<Client>, 
    ollama_plugin: &OllamaPlugin
) -> Result<AIAnalysis> {
    // Get test execution data from SurrealDB
    let mut response = db.query("
        SELECT * FROM test_executions 
        ORDER BY timestamp DESC 
        LIMIT 50
    ").await
    .map_err(|e| CleanroomError::service_error("Failed to query test executions")
        .with_source(e.to_string()))?;

    let executions: Vec<TestExecution> = response.take(0)
        .map_err(|e| CleanroomError::service_error("Failed to parse test executions")
            .with_source(e.to_string()))?;

    if executions.is_empty() {
        return Ok(AIAnalysis {
            total_executions: 0,
            success_rate: 0.0,
            avg_execution_time: 0.0,
            ai_insights: Vec::new(),
        });
    }

    // Calculate basic statistics
    let total_executions = executions.len();
    let successful_executions = executions.iter().filter(|e| e.success).count();
    let success_rate = successful_executions as f64 / total_executions as f64;
    let avg_execution_time = executions.iter()
        .map(|e| e.execution_time_ms as f64)
        .sum::<f64>() / total_executions as f64;

    // Use Ollama for AI analysis
    let ai_insights = generate_ollama_insights(ollama_plugin, &executions, success_rate).await?;

    Ok(AIAnalysis {
        total_executions,
        success_rate,
        avg_execution_time,
        ai_insights,
    })
}

/// Generate AI insights using Ollama
async fn generate_ollama_insights(
    ollama_plugin: &OllamaPlugin,
    executions: &[TestExecution],
    success_rate: f64,
) -> Result<Vec<AIInsight>> {
    let prompt = format!(
        "Analyze this test execution data and provide insights:\n\
        - Total executions: {}\n\
        - Success rate: {:.1}%\n\
        - Average execution time: {:.0}ms\n\
        - Test names: {}\n\n\
        Provide 3-5 actionable insights for improving test reliability and performance. Be specific and practical.",
        executions.len(),
        success_rate * 100.0,
        executions.iter().map(|e| e.execution_time_ms as f64).sum::<f64>() / executions.len() as f64,
        executions.iter().map(|e| e.test_name.as_str()).collect::<Vec<_>>().join(", ")
    );

    match ollama_plugin.generate_text("llama3.2:3b", &prompt).await {
        Ok(response) => {
            // Parse AI response and create insights
            let insights = parse_ollama_response(&response.response, executions.len()).await?;
            Ok(insights)
        }
        Err(e) => {
            println!("⚠️ Ollama analysis failed: {}", e);
            // Return fallback insights
            Ok(vec![
                AIInsight {
                    insight_type: "fallback".to_string(),
                    content: "AI analysis unavailable, using basic insights".to_string(),
                    confidence: 0.5,
                    actionable: false,
                    created_at: chrono::Utc::now(),
                }
            ])
        }
    }
}

/// Parse Ollama response into insights
async fn parse_ollama_response(response: &str, execution_count: usize) -> Result<Vec<AIInsight>> {
    let lines: Vec<&str> = response.lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    let mut insights = Vec::new();
    for (i, line) in lines.iter().enumerate().take(5) {
        if line.trim().len() > 20 {
            insights.push(AIInsight {
                insight_type: "ai_generated".to_string(),
                content: line.trim().to_string(),
                confidence: 0.8 - (i as f64 * 0.1),
                actionable: true,
                created_at: chrono::Utc::now(),
            });
        }
    }

    // Add a summary insight
    insights.push(AIInsight {
        insight_type: "summary".to_string(),
        content: format!("Analyzed {} test executions with AI-powered insights", execution_count),
        confidence: 0.9,
        actionable: true,
        created_at: chrono::Utc::now(),
    });

    Ok(insights)
}

/// Perform SurrealDB-only analysis
async fn perform_surrealdb_analysis(db: &Surreal<Client>) -> Result<AIAnalysis> {
    let mut response = db.query("
        SELECT * FROM test_executions 
        ORDER BY timestamp DESC 
        LIMIT 50
    ").await
    .map_err(|e| CleanroomError::service_error("Failed to query test executions")
        .with_source(e.to_string()))?;

    let executions: Vec<TestExecution> = response.take(0)
        .map_err(|e| CleanroomError::service_error("Failed to parse test executions")
            .with_source(e.to_string()))?;

    if executions.is_empty() {
        return Ok(AIAnalysis {
            total_executions: 0,
            success_rate: 0.0,
            avg_execution_time: 0.0,
            ai_insights: Vec::new(),
        });
    }

    let total_executions = executions.len();
    let successful_executions = executions.iter().filter(|e| e.success).count();
    let success_rate = successful_executions as f64 / total_executions as f64;
    let avg_execution_time = executions.iter()
        .map(|e| e.execution_time_ms as f64)
        .sum::<f64>() / total_executions as f64;

    // Generate basic insights without AI
    let insights = vec![
        AIInsight {
            insight_type: "basic_analysis".to_string(),
            content: format!("Success rate: {:.1}%", success_rate * 100.0),
            confidence: 1.0,
            actionable: true,
            created_at: chrono::Utc::now(),
        },
        AIInsight {
            insight_type: "performance".to_string(),
            content: format!("Average execution time: {:.0}ms", avg_execution_time),
            confidence: 1.0,
            actionable: true,
            created_at: chrono::Utc::now(),
        },
    ];

    Ok(AIAnalysis {
        total_executions,
        success_rate,
        avg_execution_time,
        ai_insights: insights,
    })
}

/// Display AI analysis results
async fn display_ai_analysis(analysis: &AIAnalysis) -> Result<()> {
    println!("📊 AI Analysis Results:");
    println!("   Total Executions: {}", analysis.total_executions);
    println!("   Success Rate: {:.1}%", analysis.success_rate * 100.0);
    println!("   Average Execution Time: {:.0}ms", analysis.avg_execution_time);
    
    if !analysis.ai_insights.is_empty() {
        println!("💡 AI-Generated Insights:");
        for insight in &analysis.ai_insights {
            println!("   • {} (confidence: {:.1}%)", 
                     insight.content, insight.confidence * 100.0);
        }
    }

    Ok(())
}

/// Display SurrealDB analysis results
async fn display_surrealdb_analysis(analysis: &AIAnalysis) -> Result<()> {
    println!("📊 SurrealDB Analysis Results:");
    println!("   Total Executions: {}", analysis.total_executions);
    println!("   Success Rate: {:.1}%", analysis.success_rate * 100.0);
    println!("   Average Execution Time: {:.0}ms", analysis.avg_execution_time);
    
    if !analysis.ai_insights.is_empty() {
        println!("💡 Basic Insights:");
        for insight in &analysis.ai_insights {
            println!("   • {} (confidence: {:.1}%)", 
                     insight.content, insight.confidence * 100.0);
        }
    }

    Ok(())
}

// Data structures

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TestExecution {
    test_name: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    success: bool,
    execution_time_ms: u64,
    error_message: Option<String>,
    resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ResourceUsage {
    cpu_percent: f32,
    memory_mb: u64,
    network_io_mb: u64,
    disk_io_mb: u64,
}

#[derive(Debug, Clone)]
struct AIAnalysis {
    total_executions: usize,
    success_rate: f64,
    avg_execution_time: f64,
    ai_insights: Vec<AIInsight>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AIInsight {
    insight_type: String,
    content: String,
    confidence: f64,
    actionable: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}
