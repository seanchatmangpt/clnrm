//! Health check and readiness probe implementations
//!
//! Provides multi-layer health checking for gVisor-managed services.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP scheme for health checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HttpScheme {
    Http,
    Https,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HealthCheck {
    /// Execute command in container
    Exec {
        /// Command to execute
        command: Vec<String>,
        /// Interval between checks
        #[serde(default = "default_interval")]
        interval: String,
        /// Timeout for each check
        #[serde(default = "default_timeout")]
        timeout: String,
        /// Number of consecutive failures before unhealthy
        #[serde(default = "default_retries")]
        retries: u32,
    },
    /// HTTP endpoint check
    Http {
        /// HTTP path
        path: String,
        /// Port to check
        port: u16,
        /// HTTP or HTTPS
        #[serde(default = "default_http_scheme")]
        scheme: HttpScheme,
        /// Interval between checks
        #[serde(default = "default_interval")]
        interval: String,
        /// Timeout for each check
        #[serde(default = "default_timeout")]
        timeout: String,
        /// Number of consecutive failures before unhealthy
        #[serde(default = "default_retries")]
        retries: u32,
    },
    /// TCP port check
    Tcp {
        /// Port to check
        port: u16,
        /// Interval between checks
        #[serde(default = "default_interval")]
        interval: String,
        /// Timeout for each check
        #[serde(default = "default_timeout")]
        timeout: String,
        /// Number of consecutive failures before unhealthy
        #[serde(default = "default_retries")]
        retries: u32,
    },
    /// gRPC health check
    Grpc {
        /// Port to check
        port: u16,
        /// Service name (optional)
        service: Option<String>,
        /// Interval between checks
        #[serde(default = "default_interval")]
        interval: String,
        /// Timeout for each check
        #[serde(default = "default_timeout")]
        timeout: String,
        /// Number of consecutive failures before unhealthy
        #[serde(default = "default_retries")]
        retries: u32,
    },
}

fn default_interval() -> String {
    "5s".to_string()
}

fn default_timeout() -> String {
    "3s".to_string()
}

fn default_retries() -> u32 {
    3
}

fn default_http_scheme() -> HttpScheme {
    HttpScheme::Http
}

impl HealthCheck {
    /// Parse interval duration
    pub fn interval(&self) -> Result<Duration> {
        let interval_str = match self {
            HealthCheck::Exec { interval, .. } => interval,
            HealthCheck::Http { interval, .. } => interval,
            HealthCheck::Tcp { interval, .. } => interval,
            HealthCheck::Grpc { interval, .. } => interval,
        };
        parse_duration(interval_str)
    }

    /// Parse timeout duration
    pub fn timeout(&self) -> Result<Duration> {
        let timeout_str = match self {
            HealthCheck::Exec { timeout, .. } => timeout,
            HealthCheck::Http { timeout, .. } => timeout,
            HealthCheck::Tcp { timeout, .. } => timeout,
            HealthCheck::Grpc { timeout, .. } => timeout,
        };
        parse_duration(timeout_str)
    }

    /// Get retry count
    pub fn retries(&self) -> u32 {
        match self {
            HealthCheck::Exec { retries, .. } => *retries,
            HealthCheck::Http { retries, .. } => *retries,
            HealthCheck::Tcp { retries, .. } => *retries,
            HealthCheck::Grpc { retries, .. } => *retries,
        }
    }
}

/// Readiness probe configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ReadinessProbe {
    /// TCP port check
    Tcp {
        /// Port to check
        port: u16,
        /// Initial delay before starting checks
        #[serde(default = "default_initial_delay")]
        initial_delay: String,
        /// Timeout for readiness
        #[serde(default = "default_readiness_timeout")]
        timeout: String,
    },
    /// HTTP endpoint check
    Http {
        /// HTTP path
        path: String,
        /// Port to check
        port: u16,
        /// HTTP or HTTPS
        #[serde(default = "default_http_scheme")]
        scheme: HttpScheme,
        /// Initial delay before starting checks
        #[serde(default = "default_initial_delay")]
        initial_delay: String,
        /// Timeout for readiness
        #[serde(default = "default_readiness_timeout")]
        timeout: String,
    },
    /// Execute command in container
    Exec {
        /// Command to execute
        command: Vec<String>,
        /// Initial delay before starting checks
        #[serde(default = "default_initial_delay")]
        initial_delay: String,
        /// Timeout for readiness
        #[serde(default = "default_readiness_timeout")]
        timeout: String,
    },
}

fn default_initial_delay() -> String {
    "2s".to_string()
}

fn default_readiness_timeout() -> String {
    "30s".to_string()
}

impl ReadinessProbe {
    /// Parse initial delay duration
    pub fn initial_delay(&self) -> Result<Duration> {
        let delay_str = match self {
            ReadinessProbe::Tcp { initial_delay, .. } => initial_delay,
            ReadinessProbe::Http { initial_delay, .. } => initial_delay,
            ReadinessProbe::Exec { initial_delay, .. } => initial_delay,
        };
        parse_duration(delay_str)
    }

    /// Parse timeout duration
    pub fn timeout(&self) -> Result<Duration> {
        let timeout_str = match self {
            ReadinessProbe::Tcp { timeout, .. } => timeout,
            ReadinessProbe::Http { timeout, .. } => timeout,
            ReadinessProbe::Exec { timeout, .. } => timeout,
        };
        parse_duration(timeout_str)
    }
}

/// Health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Container is starting up
    Starting,
    /// All health checks passing
    Healthy,
    /// Health checks failing
    Unhealthy,
    /// Health check status unknown
    Unknown,
}

/// Health probe executor
pub struct HealthProbe {
    /// Health check configuration
    check: HealthCheck,
    /// Current failure count
    failure_count: u32,
    /// Current health status
    status: HealthStatus,
}

impl HealthProbe {
    /// Create new health probe
    pub fn new(check: HealthCheck) -> Self {
        Self {
            check,
            failure_count: 0,
            status: HealthStatus::Starting,
        }
    }

    /// Execute health check
    pub async fn check(&mut self, container_id: &str, container_ip: &str) -> Result<HealthStatus> {
        let check_result = match &self.check {
            HealthCheck::Tcp { port, .. } => self.check_tcp(container_ip, *port).await,
            HealthCheck::Http {
                port, path, scheme, ..
            } => self.check_http(container_ip, *port, path, scheme).await,
            HealthCheck::Exec { command, .. } => self.check_exec(container_id, command).await,
            HealthCheck::Grpc { port, service, .. } => {
                self.check_grpc(container_ip, *port, service.as_deref())
                    .await
            }
        };

        match check_result {
            Ok(true) => {
                // Health check passed
                self.failure_count = 0;
                self.status = HealthStatus::Healthy;
            }
            Ok(false) | Err(_) => {
                // Health check failed
                self.failure_count += 1;
                if self.failure_count >= self.check.retries() {
                    self.status = HealthStatus::Unhealthy;
                }
            }
        }

        Ok(self.status.clone())
    }

    /// Check TCP port connectivity
    async fn check_tcp(&self, host: &str, port: u16) -> Result<bool> {
        use tokio::net::TcpStream;

        let addr = format!("{}:{}", host, port);
        let timeout = self.check.timeout()?;

        match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }

    /// Check HTTP endpoint
    async fn check_http(
        &self,
        host: &str,
        port: u16,
        path: &str,
        scheme: &HttpScheme,
    ) -> Result<bool> {
        let scheme_str = match scheme {
            HttpScheme::Http => "http",
            HttpScheme::Https => "https",
        };

        let url = format!("{}://{}:{}{}", scheme_str, host, port, path);
        let timeout = self.check.timeout()?;

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| CleanroomError::network_error(format!("HTTP client error: {}", e)))?;

        match client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Execute command in container
    async fn check_exec(&self, container_id: &str, command: &[String]) -> Result<bool> {
        let has_runsc = which::which("runsc").is_ok();
        let timeout = self.check.timeout()?;

        if !has_runsc {
            // AUTHORITATIVE: Fallback to docker exec if runsc is unavailable (e.g. on Darwin)
            if which::which("docker").is_ok() {
                let mut cmd = tokio::process::Command::new("docker");
                cmd.arg("exec").arg(container_id).args(command);

                match tokio::time::timeout(timeout, cmd.output()).await {
                    Ok(Ok(output)) => return Ok(output.status.success()),
                    Ok(Err(_)) | Err(_) => return Ok(false),
                }
            }

            return Err(CleanroomError::execution_error(
                "Execution check failed: neither runsc nor docker found for command execution",
            ));
        } else {
            // Real runsc container execution
            let root_dir = dirs::cache_dir()
                .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache directory"))?
                .join("clnrm")
                .join("runsc");

            let mut cmd = tokio::process::Command::new("runsc");
            cmd.arg("--root")
                .arg(&root_dir)
                .arg("exec")
                .arg(container_id)
                .args(command);

            match tokio::time::timeout(timeout, cmd.output()).await {
                Ok(Ok(output)) => {
                    if output.status.success() {
                        Ok(true)
                    } else {
                        tracing::warn!(
                            container_id = %container_id,
                            exit_code = ?output.status.code(),
                            stderr = %String::from_utf8_lossy(&output.stderr),
                            "Exec health check failed"
                        );
                        Ok(false)
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "Failed to run runsc exec");
                    Ok(false)
                }
                Err(_) => {
                    tracing::warn!("runsc exec health check timed out");
                    Ok(false)
                }
            }
        }
    }

    /// Check gRPC health endpoint
    async fn check_grpc(&self, host: &str, port: u16, service: Option<&str>) -> Result<bool> {
        let timeout = self.check.timeout()?;

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .http2_prior_knowledge()
            .build()
            .map_err(|e| CleanroomError::network_error(format!("gRPC client error: {}", e)))?;

        let url = format!("http://{}:{}/grpc.health.v1.Health/Check", host, port);

        let mut body = vec![0u8; 5]; // gRPC header: 1 byte uncompressed, 4 bytes length
        if let Some(svc) = service {
            if !svc.is_empty() {
                let svc_bytes = svc.as_bytes();
                let mut payload = vec![0x0a, svc_bytes.len() as u8];
                payload.extend_from_slice(svc_bytes);

                let len = payload.len() as u32;
                body[1..5].copy_from_slice(&len.to_be_bytes());
                body.extend(payload);
            }
        }

        match client
            .post(&url)
            .header("content-type", "application/grpc")
            .header("te", "trailers")
            .body(body)
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    return Ok(false);
                }

                let bytes = response.bytes().await.map_err(|e| {
                    CleanroomError::network_error(format!("Failed to read response body: {}", e))
                })?;

                if bytes.len() < 5 {
                    return Ok(false);
                }

                let payload_len =
                    u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
                if bytes.len() < 5 + payload_len {
                    return Ok(false);
                }

                let payload = &bytes[5..5 + payload_len];
                if payload.is_empty() {
                    return Ok(false);
                }

                // Parse protobuf HealthCheckResponse:
                // Field 1 (status): tag 0x08 (varint)
                // ServingStatus: 1 = SERVING
                let mut pos = 0;
                let mut serving_status = 0;

                while pos < payload.len() {
                    let tag = payload[pos];
                    pos += 1;
                    if tag == 0x08 {
                        if pos < payload.len() {
                            serving_status = payload[pos];
                            break;
                        }
                    } else {
                        break;
                    }
                }

                Ok(serving_status == 1)
            }
            Err(_) => Ok(false),
        }
    }

    /// Get current health status
    pub fn status(&self) -> &HealthStatus {
        &self.status
    }
}

/// Parse duration string (e.g., "5s", "2m", "1h")
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();

    if s.is_empty() {
        return Err(CleanroomError::validation_error("Duration cannot be empty"));
    }

    // Extract number and unit
    let (num_str, unit) = if let Some(num) = s.strip_suffix("ms") {
        (num, "ms")
    } else if let Some(num) = s.strip_suffix('s') {
        (num, "s")
    } else if let Some(num) = s.strip_suffix('m') {
        (num, "m")
    } else if let Some(num) = s.strip_suffix('h') {
        (num, "h")
    } else {
        // Default to seconds if no unit
        (s, "s")
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| CleanroomError::validation_error(format!("Invalid duration: {}", s)))?;

    let duration = match unit {
        "ms" => Duration::from_millis(num),
        "s" => Duration::from_secs(num),
        "m" => Duration::from_secs(num * 60),
        "h" => Duration::from_secs(num * 3600),
        _ => {
            return Err(CleanroomError::validation_error(format!(
                "Unknown duration unit: {}",
                unit
            )))
        }
    };

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration("10").unwrap(), Duration::from_secs(10));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("5x").is_err());
    }
}
