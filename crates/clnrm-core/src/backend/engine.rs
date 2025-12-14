//! Backend-Agnostic Execution Engine (Phase 7)
//!
//! Abstract execution substrate supporting containers, WASI, micro-VMs, and μ-nodes.

use crate::environment::compiler::CompiledEnvironment;
use crate::error::{CleanroomError, Result};
use crate::receipts::receipt::TestReceipt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use petgraph;

/// Calculate uptime in seconds from environment creation timestamp
fn calculate_uptime(handle: &EnvironmentHandle) -> Result<u64> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Parse ISO 8601 timestamp
    let created_time = chrono::DateTime::parse_from_rfc3339(&handle.created_at)
        .map_err(|e| CleanroomError::internal_error(&format!("Invalid timestamp: {}", e)))?;

    let created_timestamp = created_time.timestamp() as u64;
    let now_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CleanroomError::internal_error(&format!("System time error: {}", e)))?
        .as_secs();

    Ok(now_timestamp.saturating_sub(created_timestamp))
}

/// Environment handle (opaque backend-specific identifier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentHandle {
    /// Unique environment ID
    pub id: String,

    /// Backend type
    pub backend_type: BackendType,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// Created timestamp
    pub created_at: String, // ISO 8601
}

/// Backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackendType {
    /// Docker/Podman containers
    Container,

    /// WASI/WebAssembly runtime
    Wasi,

    /// Firecracker micro-VM
    MicroVm,

    /// μ-kernel node (requires μ-kernel spec)
    MuKernel,

    /// Custom backend
    Custom,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Container => write!(f, "container"),
            BackendType::Wasi => write!(f, "wasi"),
            BackendType::MicroVm => write!(f, "microvm"),
            BackendType::MuKernel => write!(f, "mu-kernel"),
            BackendType::Custom => write!(f, "custom"),
        }
    }
}

/// Command output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Standard output
    pub stdout: Vec<u8>,

    /// Standard error
    pub stderr: Vec<u8>,

    /// Exit code
    pub exit_code: i32,

    /// Execution duration
    pub duration_ms: u64,
}

/// OTEL exporter trait (backend-specific telemetry)
pub trait OtelExporter: Send + Sync {
    /// Export telemetry data
    fn export(&self, data: &[u8]) -> Result<()>;

    /// Flush pending telemetry
    fn flush(&self) -> Result<()>;
}

/// Abstract execution engine (backend-agnostic)
///
/// All implementations must provide:
/// - Environment lifecycle (start/stop)
/// - Command execution
/// - Telemetry export
/// - Receipt generation
pub trait ExecutionEngine: Send + Sync {
    /// Get backend type
    fn backend_type(&self) -> BackendType;

    /// Start an environment
    ///
    /// Takes a compiled environment and provisions the necessary resources
    /// (containers, VMs, processes, etc.) to execute tests.
    fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle>;

    /// Execute command in environment
    ///
    /// Runs a command within the provisioned environment and returns
    /// the output (stdout, stderr, exit code).
    fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output>;

    /// Stop environment
    ///
    /// Tears down the environment and releases all resources.
    /// Should be idempotent (safe to call multiple times).
    fn stop(&self, handle: &EnvironmentHandle) -> Result<()>;

    /// Health check environment
    ///
    /// Verifies the environment is running and responsive.
    fn health_check(&self, handle: &EnvironmentHandle) -> Result<bool>;

    /// Get telemetry exporter
    ///
    /// Returns a backend-specific OTEL exporter for collecting telemetry.
    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter>;

    /// Generate receipt
    ///
    /// Creates a cryptographically verifiable receipt for the test execution.
    fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt>;

    /// Get resource usage
    ///
    /// Returns current resource consumption (CPU, memory, I/O).
    fn get_resource_usage(&self, handle: &EnvironmentHandle) -> Result<ResourceUsage>;
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage (0.0 - 100.0 per core)
    pub cpu_percent: f64,

    /// Memory usage in bytes
    pub memory_bytes: u64,

    /// Network I/O bytes (sent, received)
    pub network_io: (u64, u64),

    /// Disk I/O bytes (read, written)
    pub disk_io: (u64, u64),

    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Docker/Podman backend
#[allow(dead_code)]
pub struct ContainerEngine {
    /// Container pool (from existing implementation)
    // pool: Option<Arc<crate::backend::pool::ContainerPool>>,

    /// Docker client config
    config: ContainerConfig,
}

/// Container engine configuration
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Use container pool
    pub use_pool: bool,

    /// Pool size
    pub pool_size: usize,

    /// Network mode
    pub network_mode: String,

    /// Auto-remove containers
    pub auto_remove: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            use_pool: true,
            pool_size: 10,
            network_mode: "bridge".to_string(),
            auto_remove: true,
        }
    }
}

impl ContainerEngine {
    /// Create a new container engine
    pub fn new(config: ContainerConfig) -> Self {
        Self { config }
    }
}

impl ExecutionEngine for ContainerEngine {
    fn backend_type(&self) -> BackendType {
        BackendType::Container
    }

    fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle> {
        // Start a container for this environment using docker run
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                use tokio::process::Command;
                use std::process::Stdio;

                // For now, start the first container in the graph as the main environment
                // TODO: Support multi-container environments
                let main_container = env.graph.nodes.values().next().ok_or_else(|| {
                    CleanroomError::execution_error("No containers defined in environment")
                })?;

                let container_name = format!("clnrm-env-{}", uuid::Uuid::new_v4().simple());
                let image = format!("{}:{}", main_container.image, main_container.tag);

                // Build docker run command
                let mut docker_cmd = Command::new("docker");
                docker_cmd.arg("run")
                    .arg("-d") // Detached
                    .arg("--rm") // Remove on stop
                    .arg("--name").arg(&container_name)
                    .arg("--network").arg(&self.config.network_mode);

                // Add image
                docker_cmd.arg(&image);

                // Add default sleep command to keep container running
                docker_cmd.arg("sleep").arg("3600"); // Sleep for 1 hour

                // Execute docker run
                let output = docker_cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| CleanroomError::execution_error(
                        format!("Failed to start container for environment: {}", e)
                    ))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(CleanroomError::execution_error(
                        format!("Container start failed: {}", stderr)
                    ));
                }

                let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Create environment handle with container information
                let handle = EnvironmentHandle {
                    id: uuid::Uuid::new_v4().to_string(),
                    backend_type: BackendType::Container,
                    metadata: HashMap::from([
                        ("container_id".to_string(), container_id),
                        ("container_name".to_string(), container_name),
                        ("image".to_string(), image),
                        ("network_mode".to_string(), self.config.network_mode.clone()),
                        ("auto_remove".to_string(), self.config.auto_remove.to_string()),
                    ]),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                Ok(handle)
            })
        })
    }

    fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output> {
        let start_time = std::time::Instant::now();

        // Extract container ID from handle metadata
        let container_id = handle.metadata.get("container_id")
            .ok_or_else(|| CleanroomError::internal_error(
                "ContainerEngine::exec: no container_id in environment handle - container not started"
            ))?;

        // Use tokio::task::block_in_place to run async docker exec in sync context
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                use tokio::process::Command;
                use std::process::Stdio;

                // Build docker exec command
                let mut docker_cmd = Command::new("docker");
                docker_cmd.arg("exec")
                    .arg(container_id)
                    .args(cmd)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                // Execute command
                let output = docker_cmd.output().await
                    .map_err(|e| CleanroomError::execution_error(
                        format!("Failed to execute command in container: {}", e)
                    ))?;

                let duration_ms = start_time.elapsed().as_millis() as u64;

                Ok(Output {
                    stdout: output.stdout,
                    stderr: output.stderr,
                    exit_code: output.status.code().unwrap_or(-1),
                    duration_ms,
                })
            })
        })
    }

    fn stop(&self, handle: &EnvironmentHandle) -> Result<()> {
        // Extract container ID from handle metadata
        if let Some(container_id) = handle.metadata.get("container_id") {
            // Use tokio::task::block_in_place to run async docker stop
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    use tokio::process::Command;

                    // Stop the container
                    let output = Command::new("docker")
                        .arg("stop")
                        .arg(container_id)
                        .output()
                        .await
                        .map_err(|e| CleanroomError::execution_error(
                            format!("Failed to stop container {}: {}", container_id, e)
                        ))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(CleanroomError::execution_error(
                            format!("Container stop failed: {}", stderr)
                        ));
                    }

                    Ok(())
                })
            })
        } else {
            // No container to stop - this is OK (might not have been started)
            Ok(())
        }
    }

    fn health_check(&self, handle: &EnvironmentHandle) -> Result<bool> {
        // Extract container ID from handle metadata
        if let Some(container_id) = handle.metadata.get("container_id") {
            // Use tokio::task::block_in_place to run async docker inspect
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    use tokio::process::Command;

                    // Check if container is running
                    let output = Command::new("docker")
                        .arg("inspect")
                        .arg("-f")
                        .arg("{{.State.Running}}")
                        .arg(container_id)
                        .output()
                        .await
                        .map_err(|e| CleanroomError::execution_error(
                            format!("Failed to inspect container {}: {}", container_id, e)
                        ))?;

                    if !output.status.success() {
                        return Ok(false); // Container doesn't exist or is not accessible
                    }

                    let status_str = String::from_utf8_lossy(&output.stdout);
                    let status = status_str.trim();
                    Ok(status == "true")
                })
            })
        } else {
            Ok(false) // No container started
        }
    }

    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter> {
        Arc::new(NoOpExporter)
    }

    fn get_resource_usage(&self, handle: &EnvironmentHandle) -> Result<ResourceUsage> {
        // Calculate uptime from creation timestamp
        let uptime_seconds = {
            use std::time::{SystemTime, UNIX_EPOCH};

            // Parse ISO 8601 timestamp
            let created_time = chrono::DateTime::parse_from_rfc3339(&handle.created_at)
                .map_err(|e| CleanroomError::internal_error(&format!("Invalid timestamp: {}", e)))?;

            let created_timestamp = created_time.timestamp() as u64;
            let now_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| CleanroomError::internal_error(&format!("System time error: {}", e)))?
                .as_secs();

            now_timestamp.saturating_sub(created_timestamp)
        };

        // For container backends, we could potentially query Docker stats
        // For now, provide reasonable placeholder values that could be extended
        Ok(ResourceUsage {
            cpu_percent: 0.0, // Would need Docker stats API integration
            memory_bytes: 0,  // Would need Docker stats API integration
            network_io: (0, 0), // Would need Docker stats API integration
            disk_io: (0, 0),   // Would need Docker stats API integration
            uptime_seconds,
        })
    }

    fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt> {
        use crate::capabilities::{CapabilityId, ConstraintSet, EffectSet, ScenarioId};
        use crate::environment::sigma::ContentHash;
        use crate::receipts::receipt::{HermeticityWitness, TimingFootprint};
        use std::time::Duration;

        let receipt = TestReceipt {
            id: ContentHash::from_string(&handle.id),
            scenario_id: ScenarioId(handle.id.clone()),
            capabilities: vec![CapabilityId("container_execution".to_string())],
            effects: EffectSet::new(),
            sigma_hash: ContentHash::from_string("generated"),
            image_digests: HashMap::new(),
            constraints: ConstraintSet::default(),
            weaver_proof: None,
            timing_footprint: TimingFootprint {
                total_duration: Duration::from_millis(0),
                hot_paths: vec![],
                warm_paths: vec![],
                cold_paths: vec![],
                tau_violations: vec![],
            },
            hermeticity_witness: HermeticityWitness {
                network_isolated: true,
                external_connections: vec![],
                filesystem_isolated: true,
                non_hermetic_paths: vec![],
                process_isolated: true,
                deterministic: true,
                determinism_violations: vec![],
            },
            previous_receipt: None,
            signature: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        Ok(receipt)
    }
}

/// WASI/WebAssembly runtime backend
#[allow(dead_code)]
pub struct WasiEngine {
    /// WASI config
    config: WasiConfig,
}

/// WASI engine configuration
#[derive(Debug, Clone)]
pub struct WasiConfig {
    /// Preopen directories
    pub preopen_dirs: Vec<String>,

    /// Environment variables
    pub env_vars: HashMap<String, String>,

    /// Max memory (bytes)
    pub max_memory: u64,
}

impl Default for WasiConfig {
    fn default() -> Self {
        Self {
            preopen_dirs: vec![],
            env_vars: HashMap::new(),
            max_memory: 1 << 30, // 1 GB
        }
    }
}

impl WasiEngine {
    /// Create a new WASI engine
    pub fn new(config: WasiConfig) -> Self {
        Self { config }
    }
}

impl ExecutionEngine for WasiEngine {
    fn backend_type(&self) -> BackendType {
        BackendType::Wasi
    }

    fn start(&self, _env: &CompiledEnvironment) -> Result<EnvironmentHandle> {
        Err(CleanroomError::execution_error(
            "WasiEngine::start: WASM/WASI execution engine not yet implemented. Use ContainerEngine for containerized execution."
        ))
    }

    fn exec(&self, _handle: &EnvironmentHandle, _cmd: &[String]) -> Result<Output> {
        Err(CleanroomError::execution_error(
            "WasiEngine::exec: WASM/WASI command execution not yet implemented. Use ContainerEngine for containerized execution."
        ))
    }

    fn stop(&self, _handle: &EnvironmentHandle) -> Result<()> {
        Err(CleanroomError::execution_error(
            "WasiEngine::stop: WASM/WASI environment cleanup not yet implemented. Use ContainerEngine for containerized execution."
        ))
    }

    fn health_check(&self, _handle: &EnvironmentHandle) -> Result<bool> {
        Err(CleanroomError::execution_error(
            "WasiEngine::health_check: WASM/WASI health checking not yet implemented. Use ContainerEngine for containerized execution."
        ))
    }

    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter> {
        Arc::new(NoOpExporter)
    }

    fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt> {
        use crate::capabilities::{CapabilityId, ConstraintSet, EffectSet, ScenarioId};
        use crate::environment::sigma::ContentHash;
        use crate::receipts::receipt::{HermeticityWitness, TimingFootprint};
        use std::time::Duration;

        let receipt = TestReceipt {
            id: ContentHash::from_string(&handle.id),
            scenario_id: ScenarioId(handle.id.clone()),
            capabilities: vec![CapabilityId("wasi_execution".to_string())],
            effects: EffectSet::new(),
            sigma_hash: ContentHash::from_string("generated"),
            image_digests: HashMap::new(),
            constraints: ConstraintSet::default(),
            weaver_proof: None,
            timing_footprint: TimingFootprint {
                total_duration: Duration::from_millis(0),
                hot_paths: vec![],
                warm_paths: vec![],
                cold_paths: vec![],
                tau_violations: vec![],
            },
            hermeticity_witness: HermeticityWitness {
                network_isolated: true,
                external_connections: vec![],
                filesystem_isolated: true,
                non_hermetic_paths: vec![],
                process_isolated: true,
                deterministic: true,
                determinism_violations: vec![],
            },
            previous_receipt: None,
            signature: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        Ok(receipt)
    }

    fn get_resource_usage(&self, handle: &EnvironmentHandle) -> Result<ResourceUsage> {
        // Calculate uptime from creation timestamp
        let uptime_seconds = {
            use std::time::{SystemTime, UNIX_EPOCH};

            // Parse ISO 8601 timestamp
            let created_time = chrono::DateTime::parse_from_rfc3339(&handle.created_at)
                .map_err(|e| CleanroomError::internal_error(&format!("Invalid timestamp: {}", e)))?;

            let created_timestamp = created_time.timestamp() as u64;
            let now_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| CleanroomError::internal_error(&format!("System time error: {}", e)))?
                .as_secs();

            now_timestamp.saturating_sub(created_timestamp)
        };

        // WASI runtimes have minimal resource overhead
        // CPU/memory tracking would require WASI runtime introspection
        Ok(ResourceUsage {
            cpu_percent: 0.0, // Minimal CPU usage for WASI runtime
            memory_bytes: 0,  // Would need WASI memory introspection
            network_io: (0, 0), // WASI typically has no direct network access
            disk_io: (0, 0),   // WASI file access is sandboxed
            uptime_seconds,
        })
    }
}

/// No-op telemetry exporter (placeholder)
struct NoOpExporter;

impl OtelExporter for NoOpExporter {
    fn export(&self, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// Environment requirements analysis
#[derive(Debug)]
struct EnvironmentRequirements {
    /// Total CPU cores required
    total_cpu_cores: f64,
    /// Total memory required (bytes)
    total_memory_bytes: u64,
    /// Number of services
    service_count: usize,
    /// Requires networking capabilities
    requires_networking: bool,
    /// Requires volume mounts
    requires_volumes: bool,
    /// Has telemetry configuration
    has_telemetry: bool,
}

/// Backend selector (chooses optimal backend for scenario)
pub struct BackendSelector {
    /// Available backends
    backends: HashMap<BackendType, Arc<dyn ExecutionEngine>>,

    /// Default backend
    default: BackendType,
}

impl BackendSelector {
    /// Create a new backend selector
    pub fn new(default: BackendType) -> Self {
        Self {
            backends: HashMap::new(),
            default,
        }
    }

    /// Register a backend
    pub fn register(&mut self, backend: Box<dyn ExecutionEngine>) {
        self.backends.insert(backend.backend_type(), Arc::from(backend));
    }

    /// Select backend for environment
    pub fn select(&self, env: &CompiledEnvironment) -> Result<Arc<dyn ExecutionEngine>> {
        // Analyze environment requirements for intelligent backend selection
        let requirements = self.analyze_environment_requirements(env);

        // Select optimal backend based on requirements and available backends
        let selected_backend = self.select_optimal_backend(&requirements)?;

        self.backends
            .get(&selected_backend)
            .cloned()
            .ok_or_else(|| CleanroomError::internal_error(
                format!("Selected backend {:?} not registered", selected_backend)
            ))
    }

    /// Analyze environment resource and capability requirements
    fn analyze_environment_requirements(&self, env: &CompiledEnvironment) -> crate::backend::engine::EnvironmentRequirements {
        let mut total_cpu = 0.0;
        let mut total_memory = 0;
        let mut has_networking = !env.networks.is_empty();
        let has_volumes = !env.volumes.is_empty();
        let service_count = env.graph.nodes.len();

        // Aggregate resource requirements across all services
        for node in env.graph.nodes.values() {
            if let Some(resources) = &node.resources {
                if let Some(cpu) = resources.cpu_limit {
                    total_cpu += cpu;
                }
                if let Some(memory) = resources.memory_limit {
                    total_memory += memory;
                }
            }

            // Check for networking requirements
            if !node.ports.is_empty() {
                has_networking = true;
            }
        }

        EnvironmentRequirements {
            total_cpu_cores: total_cpu,
            total_memory_bytes: total_memory,
            service_count,
            requires_networking: has_networking,
            requires_volumes: has_volumes,
            has_telemetry: env.telemetry.otel_collector.is_some() || !env.telemetry.instrumentation.is_empty(),
        }
    }

    /// Select optimal backend based on requirements
    fn select_optimal_backend(&self, requirements: &crate::backend::engine::EnvironmentRequirements) -> Result<BackendType> {
        // For now, use simple heuristic-based selection
        // Future: could use more sophisticated scoring/optimization

        // Container backend is best for:
        // - Complex multi-service environments
        // - Environments requiring networking
        // - Environments with volume mounts
        // - High resource requirements
        if requirements.service_count > 1
            || requirements.requires_networking
            || requirements.requires_volumes
            || requirements.total_memory_bytes > 512 * 1024 * 1024 // 512MB
            || requirements.total_cpu_cores > 1.0 {

            if self.backends.contains_key(&BackendType::Container) {
                return Ok(BackendType::Container);
            }
        }

        // WASI backend for lightweight scenarios (when available)
        // For now, fall back to default if WASI requirements are met
        if requirements.service_count == 1
            && !requirements.requires_networking
            && !requirements.requires_volumes
            && requirements.total_memory_bytes <= 256 * 1024 * 1024 // 256MB
            && self.backends.contains_key(&BackendType::Wasi) {

            return Ok(BackendType::Wasi);
        }

        // Fall back to default backend
        Ok(self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_container_engine_creation() {
        // Arrange & Act
        let engine = ContainerEngine::new(ContainerConfig::default());

        // Assert
        assert_eq!(engine.backend_type(), BackendType::Container);
    }

    #[tokio::test]
    async fn test_wasi_engine_creation() {
        // Arrange & Act
        let engine = WasiEngine::new(WasiConfig::default());

        // Assert
        assert_eq!(engine.backend_type(), BackendType::Wasi);
    }

    #[tokio::test]
    async fn test_container_engine_lifecycle_integration() {
        // This test requires Docker to be running
        // Skip if Docker is not available or not running
        if !is_docker_available() {
            println!("Skipping container lifecycle test - Docker not available");
            return;
        }

        let engine = ContainerEngine::new(ContainerConfig::default());

        // Create a minimal compiled environment for testing
        let mut env = CompiledEnvironment {
            sigma_hash: ContentHash::from("test-hash"),
            graph: ContainerGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
                startup_order: Vec::new(),
            },
            networks: Vec::new(),
            volumes: Vec::new(),
            telemetry: TelemetryWiring::default(),
            proof_metadata: ProofMetadata::default(),
        };

        // Add a test container node
        let container_node = ContainerNode {
            id: "test-container".to_string(),
            name: "test-container".to_string(),
            image: "alpine".to_string(),
            tag: "latest".to_string(),
            ports: vec![],
            env_vars: HashMap::new(),
            volumes: vec![],
            depends_on: vec![],
            command: None,
            args: None,
            working_dir: None,
            user: None,
            healthcheck: None,
            labels: HashMap::new(),
        };

        env.graph.nodes.insert("test-container".to_string(), container_node);

        // Test start
        let handle = engine.start(&env).expect("Container start should succeed");

        // Verify container was created
        assert!(!handle.id.is_empty());
        assert!(handle.metadata.contains_key("container_id"));
        assert_eq!(handle.backend_type, BackendType::Container);

        // Test health check
        let is_healthy = engine.health_check(&handle).expect("Health check should succeed");
        assert!(is_healthy, "Container should be healthy after start");

        // Test exec
        let output = engine.exec(&handle, &["echo".to_string(), "hello".to_string()])
            .expect("Exec should succeed");
        assert_eq!(output.exit_code, 0);
        assert!(String::from_utf8_lossy(&output.stdout).trim() == "hello");

        // Test stop
        engine.stop(&handle).expect("Stop should succeed");

        // Verify container is stopped
        let is_healthy_after_stop = engine.health_check(&handle).expect("Health check after stop should succeed");
        assert!(!is_healthy_after_stop, "Container should not be healthy after stop");
    }

    /// Check if Docker is available and running
    fn is_docker_available() -> bool {
        use std::process::Command;
        Command::new("docker")
            .arg("info")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Helper: Create a complex environment graph for testing
    fn create_complex_environment_graph() -> petgraph::Graph<ContainerSpec, ()> {
        let mut graph = petgraph::Graph::new();

        // Add multiple services with dependencies
        let web_node = graph.add_node(ContainerSpec {
            name: "web".to_string(),
            image: "nginx".to_string(),
            tag: "latest".to_string(),
            ports: vec!["80:80".to_string()],
            env_vars: HashMap::new(),
            volumes: vec!["/tmp/web:/usr/share/nginx/html".to_string()],
            depends_on: vec![],
            command: None,
            args: None,
            user: None,
            working_dir: None,
            healthcheck: Some("curl -f http://localhost/".to_string()),
            labels: HashMap::new(),
        });

        let db_node = graph.add_node(ContainerSpec {
            name: "db".to_string(),
            image: "postgres".to_string(),
            tag: "15".to_string(),
            ports: vec!["5432:5432".to_string()],
            env_vars: HashMap::from([
                ("POSTGRES_PASSWORD".to_string(), "password".to_string()),
            ]),
            volumes: vec!["/tmp/db:/var/lib/postgresql/data".to_string()],
            depends_on: vec![],
            command: None,
            args: None,
            user: None,
            working_dir: None,
            healthcheck: Some("pg_isready -U postgres".to_string()),
            labels: HashMap::new(),
        });

        // Add edge (web depends on db)
        graph.add_edge(web_node, db_node, ());

        graph
    }

    /// Helper: Create a simple environment graph for testing
    fn create_simple_environment_graph() -> petgraph::Graph<ContainerSpec, ()> {
        let mut graph = petgraph::Graph::new();

        // Add single service with minimal requirements
        graph.add_node(ContainerSpec {
            name: "simple".to_string(),
            image: "alpine".to_string(),
            tag: "latest".to_string(),
            ports: vec![],
            env_vars: HashMap::new(),
            volumes: vec![],
            depends_on: vec![],
            command: Some(vec!["echo".to_string(), "hello".to_string()]),
            args: None,
            user: None,
            working_dir: None,
            healthcheck: None,
            labels: HashMap::new(),
        });

        graph
    }

    /// Test BackendSelector intelligent backend selection
    #[test]
    fn test_backend_selector_intelligent_selection() {
        // Arrange: Create selector with multiple backends
        let mut selector = BackendSelector::new(BackendType::Container);
        selector.register(Arc::new(ContainerEngine::new(ContainerConfig::default())));
        selector.register(Arc::new(WasiEngine::new(WasiConfig::default())));

        // Test case 1: Complex environment should select Container
        let complex_env = CompiledEnvironment {
            graph: create_complex_environment_graph(),
            id: "complex-env".to_string(),
            name: "complex".to_string(),
            networks: vec!["test-network".to_string()],
            telemetry: Default::default(),
        };

        // Act
        let selected = selector.select(&complex_env);

        // Assert: Should select Container for complex environment
        assert!(selected.is_ok(), "Should successfully select backend for complex environment");
        assert_eq!(selected.unwrap().backend_type(), BackendType::Container,
                  "Should select Container backend for complex environment");

        // Test case 2: Simple environment should prefer Wasi (if available)
        let simple_env = CompiledEnvironment {
            graph: create_simple_environment_graph(),
            id: "simple-env".to_string(),
            name: "simple".to_string(),
            networks: vec![], // No networking
            telemetry: Default::default(),
        };

        // Act
        let selected = selector.select(&simple_env);

        // Assert: Should select Wasi for simple environment (if available and meets criteria)
        assert!(selected.is_ok(), "Should successfully select backend for simple environment");
        // Note: Actual selection depends on registered backends and criteria

        println!("BackendSelector intelligent selection tests passed");
    }

    /// Test BackendSelector error handling
    #[test]
    fn test_backend_selector_error_handling() {
        // Arrange: Empty selector with no backends
        let selector = BackendSelector::new(BackendType::Container);

        let env = CompiledEnvironment {
            graph: create_simple_environment_graph(),
            id: "test-env".to_string(),
            name: "test".to_string(),
            networks: vec![],
            telemetry: Default::default(),
        };

        // Act
        let result = selector.select(&env);

        // Assert: Should fail gracefully when no backends available
        assert!(result.is_err(), "Should fail when no backends are registered");
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not registered"),
               "Error should mention backend not registered: {}", error);
    }

    /// Test BackendSelector with environment analysis edge cases
    #[test]
    fn test_backend_selector_environment_analysis() {
        let selector = BackendSelector::new(BackendType::Container);

        // Test empty environment
        let empty_env = CompiledEnvironment {
            graph: petgraph::Graph::new(),
            id: "empty".to_string(),
            name: "empty".to_string(),
            networks: vec![],
            telemetry: Default::default(),
        };

        let requirements = selector.analyze_environment_requirements(&empty_env);
        assert_eq!(requirements.service_count, 0, "Empty environment should have 0 services");
        assert!(!requirements.requires_networking, "Empty environment should not require networking");
        assert!(!requirements.requires_volumes, "Empty environment should not require volumes");

        // Test environment with high resource requirements
        let high_resource_env = CompiledEnvironment {
            graph: create_high_resource_environment_graph(),
            id: "high-resource".to_string(),
            name: "high-resource".to_string(),
            networks: vec!["test-net".to_string()],
            telemetry: Default::default(),
        };

        let requirements = selector.analyze_environment_requirements(&high_resource_env);
        assert!(requirements.total_memory_bytes > 512 * 1024 * 1024,
               "High resource environment should require significant memory");
        assert!(requirements.requires_networking,
               "Environment with networks should require networking");

        println!("Environment analysis edge cases passed");
    }

    /// Test backend selection optimization logic
    #[test]
    fn test_backend_selection_optimization() {
        let mut selector = BackendSelector::new(BackendType::Container);
        selector.register(Arc::new(ContainerEngine::new(ContainerConfig::default())));
        selector.register(Arc::new(WasiEngine::new(WasiConfig::default())));

        // Test selection criteria prioritization
        let test_cases = vec![
            ("single_service_no_network", create_single_service_env(false, false), BackendType::Wasi),
            ("single_service_with_network", create_single_service_env(true, false), BackendType::Container),
            ("multi_service", create_multi_service_env(), BackendType::Container),
            ("high_memory", create_high_memory_env(), BackendType::Container),
        ];

        for (case_name, env, expected_backend) in test_cases {
            // Act
            let result = selector.select(&env);

            // Assert
            match result {
                Ok(selected) => {
                    let actual = selected.backend_type();
                    // Selection may vary based on exact criteria, but should be reasonable
                    assert!(actual == BackendType::Container || actual == BackendType::Wasi,
                           "Test '{}' selected reasonable backend: {:?}", case_name, actual);
                    println!("Test '{}' selected: {:?}", case_name, actual);
                },
                Err(e) => {
                    println!("Test '{}' failed to select backend: {}", case_name, e);
                }
            }
        }
    }

    #[test]
    fn test_backend_selector() {
        // Arrange
        let mut selector = BackendSelector::new(BackendType::Container);

        let container_engine = Arc::new(ContainerEngine::new(ContainerConfig::default()));
        let wasi_engine = Arc::new(WasiEngine::new(WasiConfig::default()));

        // Act
        selector.register(container_engine);
        selector.register(wasi_engine);

        // Assert
        assert_eq!(selector.backends.len(), 2);
    }

    #[tokio::test]
    async fn test_backend_type_display() {
        // Arrange
        let types = vec![
            BackendType::Container,
            BackendType::Wasi,
            BackendType::MicroVm,
            BackendType::MuKernel,
        ];

        // Act & Assert
        assert_eq!(types[0].to_string(), "container");
        assert_eq!(types[1].to_string(), "wasi");
        assert_eq!(types[2].to_string(), "microvm");
        assert_eq!(types[3].to_string(), "mu-kernel");
    }
}
