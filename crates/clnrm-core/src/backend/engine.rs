//! Backend-Agnostic Execution Engine (Phase 7)
//!
//! Abstract execution substrate supporting containers, WASI, micro-VMs, and μ-nodes.

use crate::environment::compiler::CompiledEnvironment;
use crate::error::{CleanroomError, Result};
use crate::receipts::receipt::TestReceipt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Priority levels for container resource allocation and startup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerPriority {
    Critical,
    Important,
    Background,
}

impl ContainerPriority {
    pub fn startup_delay_ms(&self) -> u64 {
        match self {
            Self::Critical => 0,
            Self::Important => 500,
            Self::Background => 2000,
        }
    }
}

impl std::fmt::Display for ContainerPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::Important => write!(f, "Important"),
            Self::Background => write!(f, "Background"),
        }
    }
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
pub trait ExecutionEngine: Send + Sync {
    /// Get backend type
    fn backend_type(&self) -> BackendType;

    /// Start an environment
    fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle>;

    /// Execute command in environment
    fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output>;

    /// Stop environment
    fn stop(&self, handle: &EnvironmentHandle) -> Result<()>;

    /// Health check environment
    fn health_check(&self, handle: &EnvironmentHandle) -> Result<bool>;

    /// Get telemetry exporter
    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter>;

    /// Generate receipt
    fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt>;

    /// Get resource usage
    fn get_resource_usage(&self, handle: &EnvironmentHandle) -> Result<ResourceUsage>;
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub network_io: (u64, u64),
    pub disk_io: (u64, u64),
    pub uptime_seconds: u64,
}

#[derive(Debug)]
pub struct AdaptiveState {
    pub startup_times: Vec<u64>,
    pub memory_usage: Vec<u64>,
    pub strategy_scores: HashMap<ContainerStrategy, f64>,
    pub window_size: usize,
}

pub struct ContainerEngine {
    pub pool: Option<String>,
    pub config: ContainerConfig,
    pub adaptive_state: Option<AdaptiveState>,
    pub preopen_dirs: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub max_memory: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ContainerStrategy {
    Direct,
    Pooled,
    #[default]
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct ContainerConfig {
    pub strategy: ContainerStrategy,
    pub pool_size: usize,
    pub network_mode: String,
    pub auto_remove: bool,
    pub adaptive_window: usize,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            strategy: ContainerStrategy::Adaptive,
            pool_size: 10,
            network_mode: "bridge".to_string(),
            auto_remove: true,
            adaptive_window: 100,
        }
    }
}

impl ContainerEngine {
    pub fn new(config: ContainerConfig) -> Self {
        let adaptive_state = if config.strategy == ContainerStrategy::Adaptive {
            Some(AdaptiveState {
                startup_times: Vec::new(),
                memory_usage: Vec::new(),
                strategy_scores: HashMap::from([
                    (ContainerStrategy::Direct, 1.0),
                    (ContainerStrategy::Pooled, 1.0),
                ]),
                window_size: config.adaptive_window,
            })
        } else {
            None
        };

        Self {
            pool: None,
            config,
            adaptive_state,
            preopen_dirs: vec![],
            env_vars: HashMap::new(),
            max_memory: 1 << 30,
        }
    }

    fn start_single_container(&self, _env: &CompiledEnvironment) -> Result<EnvironmentHandle> {
        Err(CleanroomError::not_implemented(
            "start_single_container migrated to gVisor",
        ))
    }

    async fn start_multi_container(&self, _env: &CompiledEnvironment) -> Result<EnvironmentHandle> {
        Err(CleanroomError::not_implemented(
            "start_multi_container migrated to gVisor",
        ))
    }
}

impl ExecutionEngine for ContainerEngine {
    fn backend_type(&self) -> BackendType {
        BackendType::Container
    }

    fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle> {
        if env.graph.nodes.len() == 1 {
            self.start_single_container(env)
        } else {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { self.start_multi_container(env).await })
            })
        }
    }

    fn exec(&self, _handle: &EnvironmentHandle, _cmd: &[String]) -> Result<Output> {
        Err(CleanroomError::not_implemented("exec migrated to gVisor"))
    }

    fn stop(&self, _handle: &EnvironmentHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &EnvironmentHandle) -> Result<bool> {
        Ok(true)
    }
    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter> {
        Arc::new(NoOpExporter)
    }
    fn generate_receipt(&self, _handle: &EnvironmentHandle) -> Result<TestReceipt> {
        Err(CleanroomError::not_implemented(
            "Receipt generation not implemented",
        ))
    }
    fn get_resource_usage(&self, _handle: &EnvironmentHandle) -> Result<ResourceUsage> {
        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: 0,
            network_io: (0, 0),
            disk_io: (0, 0),
            uptime_seconds: 0,
        })
    }
}

pub struct WasiConfig {
    pub preopen_dirs: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub max_memory: u64,
}

impl Default for WasiConfig {
    fn default() -> Self {
        Self {
            preopen_dirs: vec![],
            env_vars: HashMap::new(),
            max_memory: 1 << 30,
        }
    }
}

pub struct WasiEngine {
    pub config: WasiConfig,
}
impl WasiEngine {
    pub fn new(config: WasiConfig) -> Self {
        Self { config }
    }
}

impl ExecutionEngine for WasiEngine {
    fn backend_type(&self) -> BackendType {
        BackendType::Wasi
    }
    fn start(&self, _env: &CompiledEnvironment) -> Result<EnvironmentHandle> {
        Err(CleanroomError::execution_error("WASI not implemented"))
    }
    fn exec(&self, _h: &EnvironmentHandle, _c: &[String]) -> Result<Output> {
        Err(CleanroomError::execution_error("WASI not implemented"))
    }
    fn stop(&self, _h: &EnvironmentHandle) -> Result<()> {
        Ok(())
    }
    fn health_check(&self, _h: &EnvironmentHandle) -> Result<bool> {
        Ok(true)
    }
    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter> {
        Arc::new(NoOpExporter)
    }
    fn generate_receipt(&self, _h: &EnvironmentHandle) -> Result<TestReceipt> {
        Err(CleanroomError::internal_error(
            "Receipt generation not implemented",
        ))
    }
    fn get_resource_usage(&self, _h: &EnvironmentHandle) -> Result<ResourceUsage> {
        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: 0,
            network_io: (0, 0),
            disk_io: (0, 0),
            uptime_seconds: 0,
        })
    }
}

struct NoOpExporter;
impl OtelExporter for NoOpExporter {
    fn export(&self, _d: &[u8]) -> Result<()> {
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

pub struct BackendSelector {
    backends: HashMap<BackendType, Arc<dyn ExecutionEngine>>,
    default: BackendType,
}

impl BackendSelector {
    pub fn new(default: BackendType) -> Self {
        Self {
            backends: HashMap::new(),
            default,
        }
    }
    pub fn register(&mut self, backend: Box<dyn ExecutionEngine>) {
        self.backends
            .insert(backend.backend_type(), Arc::from(backend));
    }
    pub fn select(&self, env: &CompiledEnvironment) -> Result<Arc<dyn ExecutionEngine>> {
        let selected = if env.graph.nodes.len() > 1 {
            BackendType::Container
        } else {
            self.default
        };
        self.backends
            .get(&selected)
            .cloned()
            .ok_or_else(|| CleanroomError::internal_error("No backend"))
    }
}
