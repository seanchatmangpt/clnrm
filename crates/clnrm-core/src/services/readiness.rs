//! Service readiness checks based on OTEL span detection
//!
//! This module provides span-based health checking for services.
//! Services can specify a span name to wait for before being marked ready,
//! enabling precise synchronization based on actual service behavior.

use crate::error::{CleanroomError, Result};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use url::Url;

/// Default timeout for waiting for spans (30 seconds)
pub const DEFAULT_SPAN_WAIT_TIMEOUT_SECS: u64 = 30;

/// Poll interval for checking span appearance (500ms)
const SPAN_POLL_INTERVAL_MS: u64 = 500;

/// OTLP Health Check Port (standard)
const OTLP_HEALTH_CHECK_PORT: u16 = 13133;

/// Configuration for span-based readiness checks
#[derive(Debug, Clone)]
pub struct SpanReadinessConfig {
    /// Name of the span to wait for
    pub span_name: String,
    /// Timeout duration for waiting
    pub timeout: Duration,
}

impl SpanReadinessConfig {
    /// Create a new span readiness configuration
    pub fn new(span_name: String, timeout_secs: Option<u64>) -> Self {
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(DEFAULT_SPAN_WAIT_TIMEOUT_SECS));
        Self { span_name, timeout }
    }
}

/// Span source for checking span appearance
#[derive(Debug, Clone)]
pub enum SpanSource {
    /// Check stdout stream for span (for stdout exporter)
    Stdout(String),
    /// Query OTLP collector endpoint
    OtlpHttp { endpoint: String },
    /// Query OTLP gRPC collector
    OtlpGrpc { endpoint: String },
}

/// Wait for a span to appear in the specified source
///
/// This function polls the span source until the specified span name appears
/// or the timeout is reached. It follows core team standards with proper error
/// handling and no unwrap/expect calls.
///
/// # Arguments
///
/// * `config` - Readiness configuration with span name and timeout
/// * `source` - Source to check for span appearance
///
/// # Returns
///
/// * `Ok(())` if span was detected before timeout
/// * `Err(CleanroomError)` if timeout occurred or span check failed
///
/// # Errors
///
/// Returns error if:
/// - Timeout is reached without detecting span
/// - Source is inaccessible
/// - Span parsing fails
pub async fn wait_for_span(config: &SpanReadinessConfig, source: &SpanSource) -> Result<()> {
    let start_time = Instant::now();
    let poll_interval = Duration::from_millis(SPAN_POLL_INTERVAL_MS);

    loop {
        // Check if timeout reached
        if start_time.elapsed() >= config.timeout {
            return Err(CleanroomError::timeout_error(format!(
                "Span '{}' not detected within {} seconds",
                config.span_name,
                config.timeout.as_secs()
            ))
            .with_context("Service readiness check"));
        }

        // Check span source
        match check_span_in_source(&config.span_name, source).await {
            Ok(true) => {
                // Span detected - service is ready
                tracing::info!(
                    span_name = %config.span_name,
                    elapsed_ms = start_time.elapsed().as_millis(),
                    "Service ready: span detected"
                );
                return Ok(());
            }
            Ok(false) => {
                // Span not found yet, continue polling
                sleep(poll_interval).await;
            }
            Err(e) => {
                // Error checking span - log and retry
                tracing::warn!(
                    span_name = %config.span_name,
                    error = %e,
                    "Failed to check span, retrying"
                );
                sleep(poll_interval).await;
            }
        }
    }
}

/// Check if a span exists in the specified source
///
/// # Arguments
///
/// * `span_name` - Name of the span to search for
/// * `source` - Source to check (stdout, OTLP HTTP, or OTLP gRPC)
///
/// # Returns
///
/// * `Ok(true)` if span was found
/// * `Ok(false)` if span was not found
/// * `Err(CleanroomError)` if checking failed
async fn check_span_in_source(span_name: &str, source: &SpanSource) -> Result<bool> {
    match source {
        SpanSource::Stdout(output) => check_span_in_stdout(span_name, output),
        SpanSource::OtlpHttp { endpoint } => check_span_in_otlp_http(span_name, endpoint).await,
        SpanSource::OtlpGrpc { endpoint } => check_span_in_otlp_grpc(span_name, endpoint).await,
    }
}

/// Check if span appears in stdout output
///
/// This is used when services export spans to stdout (common in testing).
/// We search for span name patterns in the captured output.
fn check_span_in_stdout(span_name: &str, output: &str) -> Result<bool> {
    // Check for span name in various formats that stdout exporter might use
    let patterns = [
        format!("\"name\":\"{}", span_name), // JSON format
        format!("name: {}", span_name),      // YAML-like format
        format!("span.name={}", span_name),  // Key-value format
        format!("SpanName({})", span_name),  // Debug format
        span_name.to_string(),               // Direct match
    ];

    for pattern in &patterns {
        if output.contains(pattern) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if span exists in OTLP HTTP collector
///
/// Queries the collector's trace endpoint to see if span has been recorded.
/// This requires the collector to expose a query API.
async fn check_span_in_otlp_http(span_name: &str, endpoint: &str) -> Result<bool> {
    // Build query URL for searching spans
    let query_url = format!("{}/v1/traces?span_name={}", endpoint, span_name);

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            CleanroomError::network_error("Failed to create HTTP client")
                .with_context("OTLP HTTP span check")
                .with_source(e.to_string())
        })?;

    // Query collector
    match client.get(&query_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.text().await.map_err(|e| {
                    CleanroomError::network_error("Failed to read response body")
                        .with_source(e.to_string())
                })?;

                // Check if response contains our span
                Ok(body.contains(span_name))
            } else {
                // Collector not ready or span not found
                Ok(false)
            }
        }
        Err(e) => {
            // Connection failed - collector might not be ready yet
            tracing::debug!(
                endpoint = %endpoint,
                error = %e,
                "OTLP HTTP check failed, collector may not be ready"
            );
            Ok(false)
        }
    }
}

/// Check if span exists in OTLP gRPC collector
///
/// Queries the collector via gRPC to check for span existence.
async fn check_span_in_otlp_grpc(span_name: &str, endpoint: &str) -> Result<bool> {
    // Robust health check query to ensure collector is available
    if !check_otlp_health(endpoint).await? {
        tracing::debug!(endpoint = %endpoint, "OTLP collector not healthy via gRPC/HTTP");
        return Ok(false);
    }

    // Since a true gRPC trace query requires complex protobuf definitions (tonic),
    // we use a robust fallback: query the collector's HTTP query API if available,
    // which is standard for most production collectors (e.g. Jaeger, OTEL Collector with query extension).
    // We explicitly avoid falling back to internal local storage to ensure remote compatibility.
    let http_endpoint = match convert_grpc_to_http_endpoint(endpoint) {
        Ok(ep) => ep,
        Err(_) => return Ok(false),
    };

    check_span_in_otlp_http(span_name, &http_endpoint).await
}

/// Check OTLP collector health via gRPC or HTTP fallback
async fn check_otlp_health(endpoint: &str) -> Result<bool> {
    // 1. Try gRPC connection health check
    if let Ok(true) = check_grpc_health(endpoint).await {
        return Ok(true);
    }

    // 2. Try HTTP health endpoint fallback (standard port 13133)
    if let Ok(true) = check_http_health(endpoint).await {
        return Ok(true);
    }

    Ok(false)
}

/// Check gRPC endpoint health
async fn check_grpc_health(endpoint: &str) -> Result<bool> {
    let channel = tonic::transport::Endpoint::from_shared(endpoint.to_string())
        .map_err(|e| CleanroomError::network_error(format!("Invalid gRPC endpoint: {}", e)))?
        .connect_timeout(Duration::from_secs(1))
        .connect()
        .await;

    match channel {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Check HTTP health endpoint (standard OTLP collector health port)
async fn check_http_health(endpoint: &str) -> Result<bool> {
    let mut url = Url::parse(endpoint)
        .map_err(|e| CleanroomError::network_error(format!("Invalid endpoint URL: {}", e)))?;

    // Fallback to standard health port 13133 if connection on primary port fails
    url.set_port(Some(OTLP_HEALTH_CHECK_PORT))
        .map_err(|_| CleanroomError::network_error("Failed to set health check port"))?;
    url.set_path("/");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .map_err(|e| {
            CleanroomError::network_error("Failed to create HTTP client").with_source(e.to_string())
        })?;

    match client.get(url).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Convert gRPC endpoint to probable HTTP endpoint
fn convert_grpc_to_http_endpoint(endpoint: &str) -> Result<String> {
    let mut url = Url::parse(endpoint)
        .map_err(|e| CleanroomError::network_error(format!("Invalid endpoint URL: {}", e)))?;

    // Typically gRPC is on 4317, HTTP on 4318
    if url.port() == Some(4317) {
        url.set_port(Some(4318))
            .map_err(|_| CleanroomError::network_error("Failed to set HTTP port"))?;
    }

    Ok(url.to_string())
}

/// Span-based readiness checker that integrates with the service manager.
///
/// Polls a [`SpanSource`] until the expected readiness span appears or the
/// timeout is reached.
pub struct SpanReadinessChecker {
    /// Source to query for spans
    source: SpanSource,
    /// Default timeout when none is provided
    default_timeout: Duration,
}

impl SpanReadinessChecker {
    /// Create a new checker backed by the given span source
    pub fn new(source: SpanSource) -> Self {
        Self {
            source,
            default_timeout: Duration::from_secs(DEFAULT_SPAN_WAIT_TIMEOUT_SECS),
        }
    }

    /// Override the default timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Block (async) until the named service emits its readiness span.
    ///
    /// The `service_name` is used directly as the span name to wait for.
    /// Returns `Ok(())` once the span is detected, or a timeout error.
    pub async fn wait_until_ready(&self, service_name: &str) -> Result<()> {
        let config = SpanReadinessConfig::new(service_name.to_string(), None);
        let config = SpanReadinessConfig {
            span_name: config.span_name,
            timeout: self.default_timeout,
        };
        wait_for_span(&config, &self.source).await
    }

    /// Non-blocking check: returns `true` if the readiness span is already
    /// visible in the source, `false` otherwise.
    pub async fn is_ready(&self, service_name: &str) -> bool {
        match check_span_in_source(service_name, &self.source).await {
            Ok(found) => found,
            Err(e) => {
                tracing::debug!(
                    service = %service_name,
                    error = %e,
                    "Readiness check failed"
                );
                false
            }
        }
    }
}
