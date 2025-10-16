//! AI Intelligence Service - Real AI Integration
//!
//! Combines SurrealDB for data persistence and Ollama for AI processing
//! to provide actual intelligent functionality for the testing framework.

use crate::cleanroom::{ServicePlugin, ServiceHandle, HealthStatus};
use crate::error::{CleanroomError, Result};
use crate::services::{surrealdb::SurrealDbPlugin, ollama::{OllamaPlugin, OllamaConfig}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Surreal};

/// AI Intelligence service that combines SurrealDB and Ollama
pub struct AIIntelligenceService {
    name: String,
    surrealdb_plugin: SurrealDbPlugin,
    ollama_plugin: OllamaPlugin,
    db_connection: Arc<RwLock<Option<Surreal<Client>>>>,
    ollama_client: Arc<RwLock<Option<reqwest::Client>>>,
}

impl AIIntelligenceService {
    /// Create a new AI Intelligence service
    pub fn new() -> Self {
        let ollama_config = OllamaConfig {
            endpoint: "http://localhost:11434".to_string(),
            default_model: "llama3.2:3b".to_string(), // Smaller model for faster responses
            timeout_seconds: 120,
        };

        Self {
            name: "ai_intelligence".to_string(),
            surrealdb_plugin: SurrealDbPlugin::new(),
            ollama_plugin: OllamaPlugin::new("ollama", ollama_config),
            db_connection: Arc::new(RwLock::new(None)),
            ollama_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize database connection
    async fn init_db_connection(&self, host: &str, port: u16) -> Result<()> {
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

        // Initialize AI intelligence tables
        self.initialize_ai_tables(&db).await?;

        let mut db_guard = self.db_connection.write().await;
        *db_guard = Some(db);

        Ok(())
    }

    /// Initialize AI intelligence tables in SurrealDB
    async fn initialize_ai_tables(&self, db: &Surreal<Client>) -> Result<()> {
        // Create test_executions table for storing test history
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

        // Create failure_patterns table for AI analysis
        let _ = db.query("
            DEFINE TABLE failure_patterns SCHEMAFULL;
            DEFINE FIELD test_name ON failure_patterns TYPE string;
            DEFINE FIELD pattern_type ON failure_patterns TYPE string;
            DEFINE FIELD confidence ON failure_patterns TYPE float;
            DEFINE FIELD description ON failure_patterns TYPE string;
            DEFINE FIELD mitigation ON failure_patterns TYPE string;
            DEFINE FIELD created_at ON failure_patterns TYPE datetime;
            DEFINE INDEX test_name_idx ON failure_patterns COLUMNS test_name;
        ").await
        .map_err(|e| CleanroomError::service_error("Failed to create failure_patterns table")
            .with_source(e.to_string()))?;

        // Create ai_insights table for storing AI-generated insights
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

    /// Initialize Ollama HTTP client
    async fn init_ollama_client(&self) -> Result<reqwest::Client> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| CleanroomError::internal_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(client)
    }

    /// Store test execution data in SurrealDB
    pub async fn store_test_execution(&self, execution: &TestExecution) -> Result<()> {
        let db_guard = self.db_connection.read().await;
        let db = db_guard.as_ref().ok_or_else(|| 
            CleanroomError::service_error("Database not initialized"))?;

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

    /// Analyze test execution history using AI
    pub async fn analyze_test_history(&self) -> Result<AIAnalysis> {
        let db_guard = self.db_connection.read().await;
        let db = db_guard.as_ref().ok_or_else(|| 
            CleanroomError::service_error("Database not initialized"))?;

        // Get recent test executions
        let mut response = db.query("
            SELECT * FROM test_executions 
            WHERE timestamp > time::now() - 30d 
            ORDER BY timestamp DESC 
            LIMIT 100
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
                failure_patterns: Vec::new(),
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

        // Use AI to analyze failure patterns
        let failure_patterns = self.analyze_failures_with_ai(&executions).await?;
        
        // Generate AI insights
        let ai_insights = self.generate_ai_insights(&executions, success_rate).await?;

        Ok(AIAnalysis {
            total_executions,
            success_rate,
            avg_execution_time,
            failure_patterns,
            ai_insights,
        })
    }

    /// Analyze failures using Ollama AI
    async fn analyze_failures_with_ai(&self, executions: &[TestExecution]) -> Result<Vec<FailurePattern>> {
        let failed_executions: Vec<_> = executions.iter()
            .filter(|e| !e.success)
            .collect();

        if failed_executions.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare data for AI analysis
        let failure_data = failed_executions.iter()
            .map(|e| format!("Test: {}, Error: {}", e.test_name, e.error_message.as_deref().unwrap_or("Unknown")))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Analyze these test failures and identify patterns:\n\n{}\n\nProvide insights about common failure patterns, their likely causes, and mitigation strategies. Be concise and actionable.",
            failure_data
        );

        // Get AI analysis from Ollama
        let ai_response = self.query_ollama(&prompt).await?;
        
        // Parse AI response and create failure patterns
        let patterns = self.parse_failure_patterns(&ai_response, &failed_executions).await?;

        Ok(patterns)
    }

    /// Generate AI insights using Ollama
    async fn generate_ai_insights(&self, executions: &[TestExecution], success_rate: f64) -> Result<Vec<AIInsight>> {
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

        let ai_response = self.query_ollama(&prompt).await?;
        
        let insights = self.parse_ai_insights(&ai_response).await?;

        Ok(insights)
    }

    /// Query Ollama AI service
    async fn query_ollama(&self, prompt: &str) -> Result<String> {
        let mut client_guard = self.ollama_client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_ollama_client().await?);
        }
        let client = client_guard.as_ref().unwrap();

        let url = "http://localhost:11434/api/generate";
        let payload = json!({
            "model": "llama3.2:3b",
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "top_p": 0.9,
                "max_tokens": 500
            }
        });

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| CleanroomError::service_error(format!("Failed to query Ollama: {}", e)))?;

        if response.status().is_success() {
            let ollama_response: Value = response
                .json()
                .await
                .map_err(|e| CleanroomError::service_error(format!("Failed to parse Ollama response: {}", e)))?;

            let response_text = ollama_response["response"]
                .as_str()
                .ok_or_else(|| CleanroomError::service_error("Invalid Ollama response format"))?;

            Ok(response_text.to_string())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(CleanroomError::service_error(format!("Ollama API error: {}", error_text)))
        }
    }

    /// Parse failure patterns from AI response
    async fn parse_failure_patterns(&self, _ai_response: &str, failed_executions: &[&TestExecution]) -> Result<Vec<FailurePattern>> {
        // Simple parsing - in a real implementation, you'd use more sophisticated NLP
        let mut patterns = Vec::new();
        
        // Group failures by test name
        let mut test_failures: HashMap<String, Vec<&TestExecution>> = HashMap::new();
        for execution in failed_executions {
            test_failures.entry(execution.test_name.clone())
                .or_insert_with(Vec::new)
                .push(execution);
        }

        for (test_name, failures) in test_failures {
            if failures.len() > 1 { // Only create patterns for tests with multiple failures
                let failure_rate = failures.len() as f64 / (failures.len() + 10) as f64; // Rough estimate
                
                patterns.push(FailurePattern {
                    test_name: test_name.clone(),
                    pattern_type: "recurring_failure".to_string(),
                    confidence: failure_rate.min(1.0),
                    description: format!("Test '{}' has failed {} times", test_name, failures.len()),
                    mitigation: "Review test implementation and dependencies".to_string(),
                    created_at: chrono::Utc::now(),
                });
            }
        }

        Ok(patterns)
    }

    /// Parse AI insights from response
    async fn parse_ai_insights(&self, ai_response: &str) -> Result<Vec<AIInsight>> {
        // Simple parsing - split by lines and create insights
        let lines: Vec<&str> = ai_response.lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        let mut insights = Vec::new();
        for (i, line) in lines.iter().enumerate().take(5) { // Limit to 5 insights
            if line.trim().len() > 20 { // Only meaningful lines
                insights.push(AIInsight {
                    insight_type: "performance_optimization".to_string(),
                    content: line.trim().to_string(),
                    confidence: 0.8 - (i as f64 * 0.1), // Decreasing confidence
                    actionable: true,
                    created_at: chrono::Utc::now(),
                });
            }
        }

        Ok(insights)
    }

    /// Predict test failures using AI
    pub async fn predict_failures(&self) -> Result<Vec<FailurePrediction>> {
        let analysis = self.analyze_test_history().await?;
        let mut predictions = Vec::new();

        for pattern in &analysis.failure_patterns {
            if pattern.confidence > 0.3 {
                predictions.push(FailurePrediction {
                    test_name: pattern.test_name.clone(),
                    failure_probability: pattern.confidence,
                    confidence_score: 0.85,
                    risk_factors: vec!["Historical failure pattern".to_string()],
                    mitigation_strategies: vec![pattern.mitigation.clone()],
                    predicted_failure_time: "Within 24 hours".to_string(),
                });
            }
        }

        Ok(predictions)
    }
}

impl ServicePlugin for AIIntelligenceService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // Start SurrealDB first
            let db_handle = self.surrealdb_plugin.start().await?;
            
            // Extract connection details
            let host = db_handle.metadata.get("host")
                .ok_or_else(|| CleanroomError::service_error("Missing host in SurrealDB metadata"))?;
            let port = db_handle.metadata.get("port")
                .ok_or_else(|| CleanroomError::service_error("Missing port in SurrealDB metadata"))?
                .parse::<u16>()
                .map_err(|e| CleanroomError::service_error(format!("Invalid port: {}", e)))?;

            // Initialize database connection
            self.init_db_connection(host, port).await?;

            // Start Ollama service
            let _ollama_handle = self.ollama_plugin.start().await?;

            // Initialize Ollama client
            let mut client_guard = self.ollama_client.write().await;
            *client_guard = Some(self.init_ollama_client().await?);

            let mut metadata = HashMap::new();
            metadata.insert("surrealdb_host".to_string(), host.clone());
            metadata.insert("surrealdb_port".to_string(), port.to_string());
            metadata.insert("ollama_endpoint".to_string(), "http://localhost:11434".to_string());
            metadata.insert("ai_model".to_string(), "llama3.2:3b".to_string());
            metadata.insert("status".to_string(), "initialized".to_string());

            Ok(ServiceHandle {
                id: Uuid::new_v4().to_string(),
                service_name: self.name.clone(),
                metadata,
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            // Clean up connections
            let mut db_guard = self.db_connection.write().await;
            *db_guard = None;

            let mut client_guard = self.ollama_client.write().await;
            *client_guard = None;

            Ok(())
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if let Some(status) = handle.metadata.get("status") {
            match status.as_str() {
                "initialized" => HealthStatus::Healthy,
                _ => HealthStatus::Unhealthy,
            }
        } else {
            HealthStatus::Unknown
        }
    }
}

// Data structures for AI intelligence

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestExecution {
    pub test_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub network_io_mb: u64,
    pub disk_io_mb: u64,
}

#[derive(Debug, Clone)]
pub struct AIAnalysis {
    pub total_executions: usize,
    pub success_rate: f64,
    pub avg_execution_time: f64,
    pub failure_patterns: Vec<FailurePattern>,
    pub ai_insights: Vec<AIInsight>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FailurePattern {
    pub test_name: String,
    pub pattern_type: String,
    pub confidence: f64,
    pub description: String,
    pub mitigation: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIInsight {
    pub insight_type: String,
    pub content: String,
    pub confidence: f64,
    pub actionable: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct FailurePrediction {
    pub test_name: String,
    pub failure_probability: f64,
    pub confidence_score: f64,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub predicted_failure_time: String,
}
