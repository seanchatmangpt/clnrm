# gVisor Docker Replacement - Complete Implementation Roadmap

**Status**: Planning
**Version**: 2.0.0
**Date**: 2026-01-05
**Owner**: Platform Team

---

## Executive Summary

This roadmap provides a complete 6-week phased implementation plan to replace Docker/testcontainers with gVisor's runsc runtime across the entire clnrm codebase. The implementation ensures zero Docker daemon dependencies while maintaining 100% feature parity, improving performance, and enhancing hermetic isolation.

**Key Goals:**
- ✅ Complete elimination of Docker daemon dependency
- ✅ Replace testcontainers with gVisor backend
- ✅ Maintain 100% test pass rate
- ✅ Improve container startup performance (40% faster)
- ✅ Full OTLP telemetry integration
- ✅ Production-ready service management

**Timeline**: 6 weeks (30 business days)
**Team Size**: 2-3 engineers
**Risk Level**: Medium (mitigated with phased approach)

---

## Phase 1: Foundation (Week 1 - Days 1-5)

### 🎯 Objective
Create the foundational gVisor backend abstraction layer and basic OCI image handling capabilities.

### 📋 Tasks

#### 1.1 Create ContainerBackend Trait Abstraction
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/mod.rs`

```rust
//! gVisor backend implementation for containerized execution

pub mod config;
pub mod image;
pub mod network;
pub mod oci;
pub mod runtime;

use crate::backend::{Backend, Cmd, RunResult};
use crate::error::Result;

/// gVisor-based container backend
#[derive(Debug, Clone)]
pub struct GvisorBackend {
    /// Path to runsc binary
    runtime_path: PathBuf,
    /// Root directory for container state
    root_dir: PathBuf,
    /// Network configuration
    network_config: NetworkConfig,
    /// OCI image cache
    image_cache: Arc<ImageCache>,
    /// Platform (ptrace, kvm, systrap)
    platform: GvisorPlatform,
}

impl GvisorBackend {
    pub fn new(config: GvisorConfig) -> Result<Self>;
    pub fn with_platform(mut self, platform: GvisorPlatform) -> Self;
    pub fn with_root_dir(mut self, root_dir: PathBuf) -> Self;
}

impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult>;
    fn name(&self) -> &str { "gvisor" }
    fn is_available(&self) -> bool;
    fn supports_hermetic(&self) -> bool { true }
    fn supports_deterministic(&self) -> bool { true }
}
```

**Dependencies:**
- None (new module)

**Success Criteria:**
- [x] `GvisorBackend` struct compiles
- [x] Implements `Backend` trait
- [x] Unit tests pass
- [x] Documentation complete

**Estimated Effort**: 1 day

---

#### 1.2 Build Basic gVisor runsc Wrapper
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/runtime.rs`

```rust
//! runsc runtime wrapper

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// runsc runtime interface
pub struct RunscRuntime {
    /// Path to runsc binary
    binary_path: PathBuf,
    /// Root directory for container state
    root_dir: PathBuf,
    /// Platform configuration
    platform: GvisorPlatform,
}

impl RunscRuntime {
    /// Create a new container
    pub fn create(&self, bundle_path: &Path, container_id: &str) -> Result<()> {
        Command::new(&self.binary_path)
            .arg("create")
            .arg("--bundle").arg(bundle_path)
            .arg("--root").arg(&self.root_dir)
            .arg(container_id)
            .output()
            .map_err(|e| RuntimeError::CreateFailed(e.to_string()))?;
        Ok(())
    }

    /// Start a container
    pub fn start(&self, container_id: &str) -> Result<()>;

    /// Execute command in container
    pub fn exec(&self, container_id: &str, cmd: &[String]) -> Result<ExecOutput>;

    /// Stop container
    pub fn kill(&self, container_id: &str, signal: Signal) -> Result<()>;

    /// Delete container
    pub fn delete(&self, container_id: &str) -> Result<()>;

    /// Get container state
    pub fn state(&self, container_id: &str) -> Result<ContainerState>;
}

#[derive(Debug)]
pub enum GvisorPlatform {
    Ptrace,   // Default, works everywhere
    Kvm,      // Best performance, requires KVM
    Systrap,  // New platform, good balance
}
```

**Dependencies:**
- runsc binary installed (`gvisor-runsc` package)

**Success Criteria:**
- [x] Can execute `runsc create`
- [x] Can execute `runsc start`
- [x] Can execute `runsc exec`
- [x] Can execute `runsc delete`
- [x] Error handling for runsc failures

**Estimated Effort**: 2 days

---

#### 1.3 Implement OCI Image Loading
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/image.rs`

```rust
//! OCI image management

use std::path::{Path, PathBuf};

/// OCI image cache for gVisor
pub struct ImageCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Image registry client
    registry_client: RegistryClient,
}

impl ImageCache {
    /// Load image from registry (Docker Hub, GHCR, etc.)
    pub async fn pull_image(&self, image_ref: &ImageRef) -> Result<ImageManifest> {
        // 1. Parse image reference (name, tag, digest)
        let parsed = parse_image_reference(image_ref)?;

        // 2. Check local cache
        if let Some(cached) = self.get_cached_image(&parsed)? {
            return Ok(cached);
        }

        // 3. Pull from registry
        let manifest = self.registry_client.pull(&parsed).await?;

        // 4. Download layers
        for layer in &manifest.layers {
            self.download_layer(&parsed, layer).await?;
        }

        // 5. Extract to cache
        self.extract_image(&parsed, &manifest)?;

        Ok(manifest)
    }

    /// Create OCI bundle for runsc
    pub fn create_bundle(&self, image_ref: &ImageRef, bundle_dir: &Path) -> Result<()> {
        // 1. Copy rootfs from cache
        // 2. Generate config.json (OCI runtime spec)
        // 3. Set up container configuration
    }
}

/// Image reference (docker.io/library/alpine:latest)
#[derive(Debug, Clone)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: Option<String>,
    pub digest: Option<String>,
}

/// Parse image reference
fn parse_image_reference(image: &str) -> Result<ImageRef>;
```

**Dependencies:**
- `oci-distribution` crate (for OCI registry client)
- `oci-spec` crate (for OCI spec types)

**Add to Cargo.toml:**
```toml
oci-distribution = "0.11"
oci-spec = "0.6"
flate2 = "1.0"  # For layer decompression
tar = "0.4"     # For extracting tarballs
```

**Success Criteria:**
- [x] Can pull from Docker Hub
- [x] Can pull from GHCR
- [x] Can load from local OCI archive
- [x] Can cache images locally
- [x] Can create OCI bundles

**Estimated Effort**: 2 days

---

#### 1.4 Create Feature Flags
**File**: `/home/user/clnrm/crates/clnrm-core/Cargo.toml`

```toml
[features]
default = ["gvisor"]
gvisor = []                    # gVisor backend (new default)
testcontainers = []            # Legacy testcontainers (deprecated)
gvisor-kvm = []                # KVM platform (best performance)
gvisor-systrap = []            # Systrap platform (new)
```

**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/mod.rs`

```rust
// Conditional compilation based on features
#[cfg(feature = "gvisor")]
pub mod gvisor;

#[cfg(feature = "testcontainers")]
pub mod testcontainer;

// Auto-select backend based on features
pub fn default_backend() -> Result<Box<dyn Backend>> {
    #[cfg(feature = "gvisor")]
    {
        return Ok(Box::new(gvisor::GvisorBackend::new(
            gvisor::GvisorConfig::default()
        )?));
    }

    #[cfg(all(feature = "testcontainers", not(feature = "gvisor")))]
    {
        return Ok(Box::new(testcontainer::TestcontainerBackend::new("alpine:latest")?));
    }

    Err(CleanroomError::config_error("No backend available"))
}
```

**Success Criteria:**
- [x] Feature flags compile
- [x] Backend selection works
- [x] Both backends can coexist
- [x] Default is gvisor

**Estimated Effort**: 0.5 days

---

### 📊 Phase 1 Deliverables

| Deliverable | Status | Owner |
|-------------|--------|-------|
| ContainerBackend trait | ⏳ | TBD |
| runsc wrapper | ⏳ | TBD |
| OCI image loading | ⏳ | TBD |
| Feature flags | ⏳ | TBD |
| Unit tests | ⏳ | TBD |
| Documentation | ⏳ | TBD |

### ✅ Phase 1 Success Criteria

- [ ] gVisor backend compiles without errors
- [ ] Can create and start a basic container (alpine:latest)
- [ ] Can execute simple command (`echo hello`)
- [ ] Can pull images from Docker Hub
- [ ] Feature flags work correctly
- [ ] 100% unit test coverage for new code
- [ ] Documentation complete

### 🚨 Phase 1 Risk Areas

| Risk | Impact | Mitigation |
|------|--------|------------|
| runsc not available on all platforms | High | Provide installation docs, fallback to testcontainers |
| OCI image format compatibility | Medium | Test with wide variety of images |
| Performance slower than expected | Low | Benchmark early, optimize hot paths |

---

## Phase 2: Core Runtime (Week 2 - Days 6-10)

### 🎯 Objective
Implement complete gVisor container execution with network isolation, filesystem mounts, and port allocation.

### 📋 Tasks

#### 2.1 Implement gVisor-based Container Execution
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/execution.rs`

```rust
//! Container execution engine

/// Execute command in gVisor container
pub struct ContainerExecutor {
    runtime: RunscRuntime,
    network: NetworkManager,
    volumes: VolumeManager,
}

impl ContainerExecutor {
    pub fn execute(&self, cmd: &Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        // 1. Prepare OCI bundle
        let bundle_dir = self.prepare_bundle(&cmd)?;

        // 2. Setup network namespace
        let network_ns = self.network.create_namespace()?;

        // 3. Setup volume mounts
        self.volumes.setup_mounts(&bundle_dir, &cmd.volumes)?;

        // 4. Create container
        let container_id = format!("clnrm-{}", uuid::Uuid::new_v4());
        self.runtime.create(&bundle_dir, &container_id)?;

        // 5. Start container
        self.runtime.start(&container_id)?;

        // 6. Execute command
        let output = self.runtime.exec(&container_id, &cmd.args)?;

        // 7. Cleanup
        self.runtime.kill(&container_id, Signal::SIGTERM)?;
        self.runtime.delete(&container_id)?;
        self.network.cleanup_namespace(&network_ns)?;

        Ok(RunResult {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duration_ms: start_time.elapsed().as_millis() as u64,
            backend: "gvisor".to_string(),
            ..Default::default()
        })
    }
}
```

**Success Criteria:**
- [x] Can execute simple commands
- [x] Can execute complex commands with pipes
- [x] Exit codes captured correctly
- [x] stdout/stderr captured correctly
- [x] Cleanup happens on success and failure

**Estimated Effort**: 2 days

---

#### 2.2 Handle Network Isolation Without Docker
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/network.rs`

```rust
//! Network isolation and management

use std::net::IpAddr;

/// Network manager for gVisor containers
pub struct NetworkManager {
    /// Network mode (none, host, bridge)
    mode: NetworkMode,
    /// Bridge configuration
    bridge_config: Option<BridgeConfig>,
}

#[derive(Debug, Clone)]
pub enum NetworkMode {
    None,      // No network access
    Host,      // Share host network (testing only)
    Bridge,    // Isolated network with NAT
}

impl NetworkManager {
    /// Create isolated network namespace
    pub fn create_namespace(&self) -> Result<NetworkNamespace> {
        match self.mode {
            NetworkMode::None => {
                // Create completely isolated namespace
                self.create_isolated_namespace()
            }
            NetworkMode::Host => {
                // No namespace, use host network
                Ok(NetworkNamespace::host())
            }
            NetworkMode::Bridge => {
                // Create namespace with bridge
                self.create_bridge_namespace()
            }
        }
    }

    /// Setup port forwarding
    pub fn setup_port_mapping(&self, ns: &NetworkNamespace, mapping: &PortMapping) -> Result<()> {
        // Use iptables/nftables for port forwarding
        let rule = format!(
            "DNAT --to-destination {}:{}",
            ns.ip_address, mapping.container_port
        );

        Command::new("iptables")
            .args(["-t", "nat", "-A", "PREROUTING"])
            .arg("-p").arg("tcp")
            .arg("--dport").arg(mapping.host_port.to_string())
            .arg("-j").arg(&rule)
            .output()?;

        Ok(())
    }
}

/// Port mapping configuration
#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone)]
pub enum Protocol {
    Tcp,
    Udp,
}
```

**Success Criteria:**
- [x] Network isolation works (container can't see host network)
- [x] Port mapping works (can access container ports from host)
- [x] DNS resolution works
- [x] IPv4 and IPv6 support

**Estimated Effort**: 2 days

---

#### 2.3 Setup Filesystem Mounts
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/filesystem.rs`

```rust
//! Filesystem isolation and mounts

/// Volume manager for container mounts
pub struct VolumeManager {
    /// Mount validator
    validator: VolumeValidator,
}

impl VolumeManager {
    /// Setup volume mounts in OCI bundle
    pub fn setup_mounts(&self, bundle_dir: &Path, volumes: &[VolumeMount]) -> Result<()> {
        let config_path = bundle_dir.join("config.json");
        let mut config: oci_spec::runtime::Spec =
            serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;

        for volume in volumes {
            // Validate mount for security
            self.validator.validate(volume)?;

            // Add mount to OCI spec
            config.mounts_mut().push(oci_spec::runtime::Mount {
                destination: volume.container_path().into(),
                source: Some(volume.host_path().into()),
                typ: Some("bind".to_string()),
                options: Some(vec![
                    "bind".to_string(),
                    if volume.is_read_only() { "ro" } else { "rw" }.to_string(),
                ]),
            });
        }

        // Write updated config
        std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        Ok(())
    }
}
```

**Success Criteria:**
- [x] Can mount host directories
- [x] Read-only mounts enforced
- [x] Read-write mounts work
- [x] Security validation prevents unsafe paths

**Estimated Effort**: 1 day

---

#### 2.4 Port Allocation System
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/ports.rs`

```rust
//! Dynamic port allocation

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Port allocator for container services
pub struct PortAllocator {
    /// Range of available ports
    range: std::ops::Range<u16>,
    /// Currently allocated ports
    allocated: Arc<Mutex<HashSet<u16>>>,
}

impl PortAllocator {
    pub fn new(range: std::ops::Range<u16>) -> Self {
        Self {
            range,
            allocated: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Allocate a port
    pub fn allocate(&self) -> Result<u16> {
        let mut allocated = self.allocated.lock().unwrap();

        for port in self.range.clone() {
            if !allocated.contains(&port) && self.is_port_available(port)? {
                allocated.insert(port);
                return Ok(port);
            }
        }

        Err(CleanroomError::resource_exhausted("No ports available"))
    }

    /// Release a port
    pub fn release(&self, port: u16) {
        let mut allocated = self.allocated.lock().unwrap();
        allocated.remove(&port);
    }

    /// Check if port is available on host
    fn is_port_available(&self, port: u16) -> Result<bool> {
        use std::net::TcpListener;

        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
```

**Success Criteria:**
- [x] Can allocate ports dynamically
- [x] No port conflicts
- [x] Ports released after container stops
- [x] Deterministic allocation for tests

**Estimated Effort**: 1 day

---

### 📊 Phase 2 Deliverables

| Deliverable | Status | Owner |
|-------------|--------|-------|
| Container execution | ⏳ | TBD |
| Network isolation | ⏳ | TBD |
| Filesystem mounts | ⏳ | TBD |
| Port allocation | ⏳ | TBD |
| Integration tests | ⏳ | TBD |

### ✅ Phase 2 Success Criteria

- [ ] Can execute commands in isolated container
- [ ] Network isolation verified (no host access)
- [ ] Port mapping works (can connect to container services)
- [ ] Volume mounts work (can read/write files)
- [ ] Port allocation prevents conflicts
- [ ] Performance: cold start < 3s, warm start < 500ms

### 🚨 Phase 2 Risk Areas

| Risk | Impact | Mitigation |
|------|--------|------------|
| Network namespace creation requires root | High | Use user namespaces where possible, document requirements |
| Port allocation race conditions | Medium | Use proper locking, test concurrency |
| Filesystem permissions issues | Medium | Test with various permission scenarios |

---

## Phase 3: Services (Week 3 - Days 11-15)

### 🎯 Objective
Implement production-ready service management for SurrealDB and other services using gVisor.

### 📋 Tasks

#### 3.1 Implement SurrealDB Service on gVisor
**File**: `/home/user/clnrm/crates/clnrm-core/src/services/gvisor/surrealdb.rs`

```rust
//! SurrealDB service plugin for gVisor

use crate::backend::gvisor::GvisorBackend;
use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::Result;

/// SurrealDB service using gVisor
pub struct GvisorSurrealDbPlugin {
    name: String,
    backend: Arc<GvisorBackend>,
    container_id: Arc<RwLock<Option<String>>>,
    port: Arc<RwLock<Option<u16>>>,
    username: String,
    password: String,
}

impl GvisorSurrealDbPlugin {
    pub fn new(backend: Arc<GvisorBackend>) -> Self {
        Self {
            name: "surrealdb".to_string(),
            backend,
            container_id: Arc::new(RwLock::new(None)),
            port: Arc::new(RwLock::new(None)),
            username: "root".to_string(),
            password: "root".to_string(),
        }
    }
}

impl ServicePlugin for GvisorSurrealDbPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // 1. Pull SurrealDB image
        let image_ref = ImageRef::parse("surrealdb/surrealdb:v1.0.0")?;
        self.backend.image_cache.pull_image(&image_ref).await?;

        // 2. Allocate port
        let port = self.backend.port_allocator.allocate()?;

        // 3. Create OCI bundle
        let bundle_dir = tempfile::tempdir()?;
        self.backend.image_cache.create_bundle(&image_ref, bundle_dir.path())?;

        // 4. Configure SurrealDB
        self.configure_surrealdb(&bundle_dir, port)?;

        // 5. Start container
        let container_id = format!("surrealdb-{}", uuid::Uuid::new_v4());
        self.backend.runtime.create(bundle_dir.path(), &container_id)?;
        self.backend.runtime.start(&container_id)?;

        // 6. Wait for health
        self.wait_for_health(&container_id, port)?;

        // 7. Store state
        *self.container_id.write().await = Some(container_id.clone());
        *self.port.write().await = Some(port);

        // 8. Return handle
        Ok(ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: HashMap::from([
                ("host".to_string(), "127.0.0.1".to_string()),
                ("port".to_string(), port.to_string()),
                ("connection_string".to_string(), format!("ws://127.0.0.1:{}", port)),
            ]),
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        let container_id = self.container_id.read().await;
        if let Some(id) = container_id.as_ref() {
            self.backend.runtime.kill(id, Signal::SIGTERM)?;
            self.backend.runtime.delete(id)?;
        }

        if let Some(port) = *self.port.read().await {
            self.backend.port_allocator.release(port);
        }

        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        // Use SurrealDB health endpoint
        let port = handle.metadata.get("port")?.parse::<u16>().ok()?;
        let url = format!("http://127.0.0.1:{}/health", port);

        match reqwest::blocking::get(&url) {
            Ok(resp) if resp.status().is_success() => HealthStatus::Healthy,
            _ => HealthStatus::Unhealthy,
        }
    }
}
```

**Success Criteria:**
- [x] SurrealDB starts successfully
- [x] Can connect to SurrealDB
- [x] Health checks work
- [x] Service stops cleanly

**Estimated Effort**: 2 days

---

#### 3.2 Generic Service Plugin System
**File**: `/home/user/clnrm/crates/clnrm-core/src/services/gvisor/generic.rs`

```rust
//! Generic service plugin for any Docker image

use crate::config::ServiceConfig;

/// Generic service plugin that can run any Docker image
pub struct GenericGvisorService {
    name: String,
    config: ServiceConfig,
    backend: Arc<GvisorBackend>,
    container_id: Arc<RwLock<Option<String>>>,
}

impl GenericGvisorService {
    pub fn from_config(config: ServiceConfig, backend: Arc<GvisorBackend>) -> Self {
        Self {
            name: config.name.clone(),
            config,
            backend,
            container_id: Arc::new(RwLock::new(None)),
        }
    }
}

impl ServicePlugin for GenericGvisorService {
    // Similar implementation to SurrealDB but uses config
}
```

**Success Criteria:**
- [x] Can run any Docker image as service
- [x] Configuration from TOML
- [x] Supports environment variables
- [x] Supports port mappings
- [x] Supports volume mounts

**Estimated Effort**: 2 days

---

#### 3.3 Service Registry and Discovery
**File**: `/home/user/clnrm/crates/clnrm-core/src/services/registry.rs`

```rust
//! Service registry and discovery

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service registry for managing running services
pub struct ServiceRegistry {
    /// Registered services
    services: Arc<RwLock<HashMap<String, Arc<dyn ServicePlugin>>>>,
    /// Running service handles
    handles: Arc<RwLock<HashMap<String, ServiceHandle>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service plugin
    pub async fn register(&self, plugin: Arc<dyn ServicePlugin>) {
        let mut services = self.services.write().await;
        services.insert(plugin.name().to_string(), plugin);
    }

    /// Start a service
    pub async fn start_service(&self, name: &str) -> Result<ServiceHandle> {
        let services = self.services.read().await;
        let plugin = services.get(name)
            .ok_or_else(|| CleanroomError::service_not_found(name))?;

        let handle = plugin.start()?;

        let mut handles = self.handles.write().await;
        handles.insert(name.to_string(), handle.clone());

        Ok(handle)
    }

    /// Get service connection info
    pub async fn get_service(&self, name: &str) -> Result<ServiceHandle> {
        let handles = self.handles.read().await;
        handles.get(name)
            .cloned()
            .ok_or_else(|| CleanroomError::service_not_found(name))
    }

    /// Stop all services
    pub async fn stop_all(&self) -> Result<()> {
        let handles = self.handles.write().await;
        let services = self.services.read().await;

        for (name, handle) in handles.iter() {
            if let Some(plugin) = services.get(name) {
                plugin.stop(handle.clone())?;
            }
        }

        Ok(())
    }
}
```

**Success Criteria:**
- [x] Can register services
- [x] Can start/stop services
- [x] Can discover running services
- [x] Handles service lifecycle

**Estimated Effort**: 1 day

---

#### 3.4 Health Check Mechanism
**File**: `/home/user/clnrm/crates/clnrm-core/src/services/health.rs`

```rust
//! Health check system for services

/// Health check configuration
#[derive(Debug, Clone)]
pub enum HealthCheck {
    /// HTTP endpoint check
    Http {
        path: String,
        port: u16,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
    /// TCP port check
    Tcp {
        port: u16,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
    /// Command execution check
    Exec {
        command: Vec<String>,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
}

impl HealthCheck {
    /// Execute health check
    pub async fn check(&self) -> HealthStatus {
        match self {
            HealthCheck::Http { path, port, .. } => {
                self.check_http(path, *port).await
            }
            HealthCheck::Tcp { port, .. } => {
                self.check_tcp(*port).await
            }
            HealthCheck::Exec { command, .. } => {
                self.check_exec(command).await
            }
        }
    }

    async fn check_http(&self, path: &str, port: u16) -> HealthStatus {
        let url = format!("http://127.0.0.1:{}{}", port, path);
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => HealthStatus::Healthy,
            _ => HealthStatus::Unhealthy,
        }
    }
}
```

**Success Criteria:**
- [x] HTTP health checks work
- [x] TCP health checks work
- [x] Exec health checks work
- [x] Retry logic works
- [x] Timeout handling works

**Estimated Effort**: 1 day

---

### 📊 Phase 3 Deliverables

| Deliverable | Status | Owner |
|-------------|--------|-------|
| SurrealDB service | ⏳ | TBD |
| Generic service plugin | ⏳ | TBD |
| Service registry | ⏳ | TBD |
| Health checks | ⏳ | TBD |
| Service tests | ⏳ | TBD |

### ✅ Phase 3 Success Criteria

- [ ] SurrealDB service starts and accepts connections
- [ ] Can run multiple services simultaneously
- [ ] Service discovery works
- [ ] Health checks detect service failures
- [ ] Services stop cleanly

---

## Phase 4: Migration (Week 4 - Days 16-20)

### 🎯 Objective
Migrate all integration tests and configurations from Docker/testcontainers to gVisor.

### 📋 Tasks

#### 4.1 Migrate All Integration Tests
**Strategy**: Incremental migration with parallel testing

**Step 1**: Create migration tool
```rust
// File: /home/user/clnrm/scripts/migrate_tests.rs

/// Migrate testcontainers tests to gVisor
fn migrate_test_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)?;

    // Replace testcontainers imports
    let content = content.replace(
        "use testcontainers::",
        "use clnrm_core::backend::gvisor::"
    );

    // Replace TestcontainerBackend with GvisorBackend
    let content = content.replace(
        "TestcontainerBackend::new",
        "GvisorBackend::new"
    );

    // Replace testcontainers-modules with gVisor services
    let content = content.replace(
        "testcontainers_modules::surrealdb",
        "clnrm_core::services::gvisor::surrealdb"
    );

    std::fs::write(path, content)?;
    Ok(())
}
```

**Step 2**: Migrate test files systematically
```bash
# Migrate all test files
find crates/clnrm-core/tests -name "*.rs" -exec ./scripts/migrate_tests {} \;

# Verify compilation
cargo test --no-run --features gvisor

# Run tests
CLNRM_BACKEND=gvisor cargo test
```

**Test Categories to Migrate:**
1. Unit tests (`tests/*.rs`)
2. Integration tests (`tests/integration/*.rs`)
3. Docker integration tests (`tests/weaver/phase4_e2e_docker/*.rs`)
4. Service tests (`crates/clnrm-core/tests/cli_functional/services/*.rs`)

**Success Criteria:**
- [x] All unit tests migrated
- [x] All integration tests migrated
- [x] 100% test pass rate with gVisor
- [x] No testcontainers imports remain

**Estimated Effort**: 3 days

---

#### 4.2 Convert .clnrm.toml Configurations
**File**: `/home/user/clnrm/scripts/migrate_toml_configs.py`

```python
#!/usr/bin/env python3
"""
Migrate .clnrm.toml files to use gVisor backend
"""

import tomli
import tomli_w
from pathlib import Path

def migrate_toml_config(path: Path):
    with open(path, 'rb') as f:
        config = tomli.load(f)

    # Add gVisor backend configuration
    if 'backend' not in config:
        config['backend'] = {}

    config['backend']['type'] = 'gvisor'

    # Migrate service configurations
    if 'service' in config:
        for service_name, service_config in config['service'].items():
            # Convert testcontainers plugin to gvisor plugin
            if service_config.get('plugin') == 'testcontainers':
                service_config['plugin'] = 'gvisor_container'

    # Write updated config
    with open(path, 'wb') as f:
        tomli_w.dump(config, f)

# Migrate all TOML files
for toml_file in Path('.').rglob('*.clnrm.toml'):
    migrate_toml_config(toml_file)
```

**Success Criteria:**
- [x] All .clnrm.toml files updated
- [x] Backend configuration correct
- [x] Service configurations migrated
- [x] All tests still pass

**Estimated Effort**: 1 day

---

#### 4.3 Implement Configuration Migration Tool
**File**: `/home/user/clnrm/crates/clnrm-cli/src/cmds/migrate.rs`

```rust
//! Configuration migration command

use clap::Args;

#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// Path to config file or directory
    path: PathBuf,

    /// Dry run (don't modify files)
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

pub async fn migrate(args: MigrateArgs) -> Result<()> {
    println!("Migrating configurations to gVisor...");

    let files = if args.path.is_dir() {
        glob::glob(&format!("{}/**/*.clnrm.toml", args.path.display()))?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![args.path]
    };

    for file in files {
        println!("Processing: {}", file.display());

        // Read config
        let content = std::fs::read_to_string(&file)?;
        let mut config: toml::Value = toml::from_str(&content)?;

        // Migrate backend
        if let Some(table) = config.as_table_mut() {
            table.insert(
                "backend".to_string(),
                toml::Value::Table({
                    let mut backend = toml::map::Map::new();
                    backend.insert("type".to_string(), "gvisor".into());
                    backend
                })
            );
        }

        // Write back
        if !args.dry_run {
            std::fs::write(&file, toml::to_string_pretty(&config)?)?;
        }

        if args.verbose {
            println!("✅ Migrated: {}", file.display());
        }
    }

    println!("✅ Migration complete!");
    Ok(())
}
```

**Usage:**
```bash
# Migrate single file
clnrm migrate tests/example.clnrm.toml

# Migrate directory
clnrm migrate tests/ --verbose

# Dry run
clnrm migrate tests/ --dry-run
```

**Success Criteria:**
- [x] CLI command works
- [x] Migrates single files
- [x] Migrates directories
- [x] Dry run mode works
- [x] Preserves formatting where possible

**Estimated Effort**: 1 day

---

#### 4.4 Update Examples
**Files to update:**
- `/home/user/clnrm/examples/*.clnrm.toml`
- `/home/user/clnrm/examples/README.md`
- `/home/user/clnrm/book/src/**/*.md`

**Example migration:**
```toml
# Before (testcontainers)
[test]
name = "surrealdb_example"

[containers.db]
image = "alpine:latest"

# After (gVisor)
[test]
name = "surrealdb_example"

[backend]
type = "gvisor"
platform = "ptrace"  # or "kvm" for better performance

[containers.db]
image = "alpine:latest"

[service.surrealdb]
plugin = "gvisor_container"
image = "surrealdb/surrealdb:v1.0.0"
command = ["surreal", "start", "--bind", "0.0.0.0:8000"]

[service.surrealdb.env]
SURREAL_USER = "root"
SURREAL_PASS = "root"
```

**Success Criteria:**
- [x] All examples updated
- [x] All examples run successfully
- [x] Documentation updated
- [x] README files updated

**Estimated Effort**: 1 day

---

### 📊 Phase 4 Deliverables

| Deliverable | Status | Owner |
|-------------|--------|-------|
| Test migration | ⏳ | TBD |
| TOML migration | ⏳ | TBD |
| Migration tool | ⏳ | TBD |
| Examples update | ⏳ | TBD |
| Documentation | ⏳ | TBD |

### ✅ Phase 4 Success Criteria

- [ ] 100% of tests migrated to gVisor
- [ ] 100% test pass rate
- [ ] All .clnrm.toml files updated
- [ ] Migration tool works for user projects
- [ ] All examples run successfully
- [ ] Documentation complete

---

## Phase 5: Integration (Week 5 - Days 21-25)

### 🎯 Objective
Integrate OpenTelemetry telemetry with gVisor and validate OTLP export compatibility.

### 📋 Tasks

#### 5.1 Telemetry Integration with gVisor
**File**: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/telemetry.rs`

```rust
//! OpenTelemetry integration for gVisor backend

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::global;

/// Record container lifecycle events
pub fn record_container_lifecycle(
    container_id: &str,
    event: ContainerEvent,
) {
    let tracer = global::tracer("clnrm-gvisor");
    let mut span = tracer.start(format!("clnrm.container.{}", event.name()));

    span.set_attribute(opentelemetry::KeyValue::new("container.id", container_id.to_string()));
    span.set_attribute(opentelemetry::KeyValue::new("backend", "gvisor"));
    span.set_attribute(opentelemetry::KeyValue::new("container.runtime", "runsc"));

    match event {
        ContainerEvent::Create { image, .. } => {
            span.set_attribute(opentelemetry::KeyValue::new("container.image", image));
        }
        ContainerEvent::Start { .. } => {
            // Record start event
        }
        ContainerEvent::Exec { command, .. } => {
            span.set_attribute(opentelemetry::KeyValue::new("container.command", command));
        }
        ContainerEvent::Stop { exit_code, .. } => {
            span.set_attribute(opentelemetry::KeyValue::new("container.exit_code", exit_code as i64));
        }
    }

    span.end();
}

pub enum ContainerEvent {
    Create { image: String },
    Start,
    Exec { command: String },
    Stop { exit_code: i32 },
}
```

**Integration points:**
```rust
// In GvisorBackend::run_cmd
impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        record_container_lifecycle(&container_id, ContainerEvent::Create {
            image: cmd.image.clone()
        });

        // ... existing code ...

        record_container_lifecycle(&container_id, ContainerEvent::Start);

        // ... existing code ...

        record_container_lifecycle(&container_id, ContainerEvent::Exec {
            command: format!("{} {}", cmd.bin, cmd.args.join(" "))
        });

        // ... existing code ...

        record_container_lifecycle(&container_id, ContainerEvent::Stop {
            exit_code: result.exit_code
        });

        Ok(result)
    }
}
```

**Success Criteria:**
- [x] Spans created for container lifecycle
- [x] Attributes populated correctly
- [x] Context propagation works
- [x] Compatible with existing telemetry

**Estimated Effort**: 2 days

---

#### 5.2 OTLP Export Validation
**File**: `/home/user/clnrm/crates/clnrm-core/tests/telemetry/gvisor_otlp_test.rs`

```rust
//! Validate OTLP export with gVisor backend

use clnrm_core::backend::gvisor::GvisorBackend;
use clnrm_core::telemetry::init_otel;

#[tokio::test]
async fn test_gvisor_otlp_export() {
    // Setup in-memory exporter
    let exporter = opentelemetry_sdk::testing::TestExporter::new();
    init_otel(Some(exporter.clone())).await.unwrap();

    // Create gVisor backend
    let backend = GvisorBackend::new(GvisorConfig::default()).unwrap();

    // Execute command
    let cmd = Cmd::new("echo").arg("test");
    let result = backend.run_cmd(cmd).unwrap();

    assert!(result.success());

    // Flush telemetry
    opentelemetry::global::shutdown_tracer_provider();

    // Verify spans exported
    let spans = exporter.get_finished_spans().unwrap();
    assert!(!spans.is_empty(), "No spans exported");

    // Verify span names
    let span_names: Vec<_> = spans.iter()
        .map(|s| s.name.as_str())
        .collect();

    assert!(span_names.contains(&"clnrm.container.create"));
    assert!(span_names.contains(&"clnrm.container.start"));
    assert!(span_names.contains(&"clnrm.container.exec"));
    assert!(span_names.contains(&"clnrm.container.stop"));
}
```

**Success Criteria:**
- [x] Spans exported to OTLP collector
- [x] All span attributes present
- [x] No data loss
- [x] Performance acceptable

**Estimated Effort**: 1 day

---

#### 5.3 Weaver Compatibility
**Validation**: Ensure gVisor backend works with Weaver live-check

**File**: `/home/user/clnrm/tests/weaver/gvisor_weaver_test.rs`

```rust
//! Weaver integration test with gVisor backend

#[tokio::test]
async fn test_weaver_with_gvisor() {
    // Start OTEL collector
    let collector = start_otel_collector().await.unwrap();

    // Configure gVisor backend with OTLP export
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", collector.grpc_port));

    // Run test with gVisor
    let result = Command::new("cargo")
        .args(["run", "--bin", "clnrm", "--", "run", "tests/example.clnrm.toml"])
        .env("CLNRM_BACKEND", "gvisor")
        .output()
        .await
        .unwrap();

    assert!(result.status.success());

    // Verify spans in collector
    let spans = collector.get_spans().await.unwrap();
    assert!(!spans.is_empty());
}
```

**Success Criteria:**
- [x] Weaver receives spans from gVisor tests
- [x] Live-check works with gVisor
- [x] Advisors run successfully
- [x] No regressions in telemetry

**Estimated Effort**: 1 day

---

#### 5.4 Performance Validation
**File**: `/home/user/clnrm/benches/gvisor_performance.rs`

```rust
//! Performance benchmarks for gVisor backend

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_container_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_startup");

    // Baseline: testcontainers
    group.bench_function("testcontainers_cold", |b| {
        b.iter(|| {
            let backend = TestcontainerBackend::new("alpine:latest").unwrap();
            let cmd = Cmd::new("echo").arg("test");
            backend.run_cmd(cmd).unwrap()
        });
    });

    // gVisor cold start
    group.bench_function("gvisor_cold", |b| {
        b.iter(|| {
            let backend = GvisorBackend::new(GvisorConfig::default()).unwrap();
            let cmd = Cmd::new("echo").arg("test");
            backend.run_cmd(cmd).unwrap()
        });
    });

    // gVisor warm start (cached image)
    group.bench_function("gvisor_warm", |b| {
        let backend = GvisorBackend::new(GvisorConfig::default()).unwrap();
        // Pre-pull image
        backend.image_cache.pull_image(&ImageRef::parse("alpine:latest").unwrap())
            .await.unwrap();

        b.iter(|| {
            let cmd = Cmd::new("echo").arg("test");
            backend.run_cmd(cmd).unwrap()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_container_startup);
criterion_main!(benches);
```

**Target Performance Metrics:**

| Metric | Baseline (Docker) | Target (gVisor) | Status |
|--------|------------------|-----------------|--------|
| Cold start | 3-5s | < 3s (40% faster) | ⏳ |
| Warm start | 1-2s | < 500ms (75% faster) | ⏳ |
| Memory overhead | 150-200MB | < 100MB (50% less) | ⏳ |
| Network latency | 0.5-1ms | < 2ms | ⏳ |

**Success Criteria:**
- [x] Performance meets targets
- [x] No regressions in functionality
- [x] Benchmarks pass in CI

**Estimated Effort**: 1 day

---

### 📊 Phase 5 Deliverables

| Deliverable | Status | Owner |
|-------------|--------|-------|
| Telemetry integration | ⏳ | TBD |
| OTLP validation | ⏳ | TBD |
| Weaver compatibility | ⏳ | TBD |
| Performance benchmarks | ⏳ | TBD |

### ✅ Phase 5 Success Criteria

- [ ] OTLP export works with gVisor
- [ ] Weaver live-check compatible
- [ ] Performance targets met
- [ ] No telemetry data loss

---

## Phase 6: Validation (Week 6 - Days 26-30)

### 🎯 Objective
Complete Docker elimination verification, full test suite validation, and production readiness.

### 📋 Tasks

#### 6.1 Complete Docker Elimination Verification
**Script**: `/home/user/clnrm/scripts/validate_docker_elimination.sh`

```bash
#!/bin/bash
# Validate zero Docker dependencies

set -e

ERRORS=0

echo "=== Docker Elimination Validation ==="

# 1. Check for Docker CLI usage
echo "Checking for Docker CLI usage..."
if grep -rn "docker\s" --include="*.rs" --include="*.sh" . | grep -v "^#" | grep -v "//" | grep -v "docs/"; then
    echo "❌ Found Docker CLI usage"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No Docker CLI usage found"
fi

# 2. Check for Docker socket references
echo "Checking for Docker socket references..."
if grep -rn "/var/run/docker.sock" --include="*.rs" --include="*.sh" .; then
    echo "❌ Found Docker socket references"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No Docker socket references"
fi

# 3. Check for testcontainers dependencies
echo "Checking for testcontainers dependencies..."
if grep -rn "testcontainers" --include="Cargo.toml" . | grep -v "^#"; then
    echo "❌ Found testcontainers dependencies"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No testcontainers dependencies"
fi

# 4. Verify gVisor is default backend
echo "Checking default backend..."
if grep -q 'default = \["gvisor"\]' crates/clnrm-core/Cargo.toml; then
    echo "✅ gVisor is default backend"
else
    echo "❌ gVisor is not default backend"
    ERRORS=$((ERRORS + 1))
fi

if [ $ERRORS -eq 0 ]; then
    echo "✅ Docker elimination validation PASSED"
    exit 0
else
    echo "❌ Docker elimination validation FAILED ($ERRORS errors)"
    exit 1
fi
```

**Success Criteria:**
- [x] Zero Docker CLI references
- [x] Zero Docker socket references
- [x] Zero testcontainers dependencies
- [x] gVisor is default backend

**Estimated Effort**: 0.5 days

---

#### 6.2 Full Test Suite Validation
**Script**: `/home/user/clnrm/scripts/validate_gvisor_tests.sh`

```bash
#!/bin/bash
# Run full test suite with gVisor backend

set -e

export CLNRM_BACKEND=gvisor

echo "=== Running Full Test Suite with gVisor ==="

# 1. Unit tests
echo "Running unit tests..."
cargo test --all --lib

# 2. Integration tests
echo "Running integration tests..."
cargo test --all --test '*'

# 3. Doc tests
echo "Running doc tests..."
cargo test --doc

# 4. Examples
echo "Running examples..."
for example in examples/*.clnrm.toml; do
    echo "Testing: $example"
    cargo run --bin clnrm -- run "$example"
done

# 5. Benchmarks (smoke test)
echo "Running benchmarks..."
cargo bench --no-run

echo "✅ All tests passed with gVisor backend"
```

**Success Criteria:**
- [x] 100% unit tests pass
- [x] 100% integration tests pass
- [x] All doc tests pass
- [x] All examples run successfully
- [x] Benchmarks compile

**Estimated Effort**: 1 day

---

#### 6.3 Documentation Completion
**Files to create/update:**

1. **User Guide**: `/home/user/clnrm/book/src/backends/gvisor.md`
```markdown
# gVisor Backend

The gVisor backend provides secure, hermetic container execution using
Google's gVisor runtime (runsc).

## Installation

### Prerequisites

- Linux x86_64 or ARM64
- gVisor runtime installed

```bash
# Ubuntu/Debian
sudo apt-get install gvisor-runsc

# Arch Linux
sudo pacman -S gvisor-bin

# From source
wget https://storage.googleapis.com/gvisor/releases/release/latest/runsc
chmod +x runsc
sudo mv runsc /usr/local/bin/
```

## Configuration

```toml
[backend]
type = "gvisor"
platform = "ptrace"  # or "kvm" or "systrap"
root_dir = "/var/lib/clnrm/gvisor"
```

## Performance

| Metric | Docker | gVisor | Improvement |
|--------|--------|--------|-------------|
| Cold start | 3-5s | < 3s | 40% faster |
| Warm start | 1-2s | < 500ms | 75% faster |
| Memory | 150-200MB | < 100MB | 50% less |
```

2. **Migration Guide**: `/home/user/clnrm/docs/GVISOR_MIGRATION_GUIDE.md`
3. **Architecture**: `/home/user/clnrm/docs/GVISOR_ARCHITECTURE.md`
4. **Troubleshooting**: `/home/user/clnrm/docs/GVISOR_TROUBLESHOOTING.md`

**Success Criteria:**
- [x] User guide complete
- [x] Migration guide complete
- [x] Architecture docs complete
- [x] Troubleshooting guide complete
- [x] API docs complete

**Estimated Effort**: 2 days

---

#### 6.4 Performance Benchmarks
**File**: `/home/user/clnrm/benches/gvisor_comprehensive.rs`

```rust
//! Comprehensive performance benchmarks

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn bench_parallel_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_execution");

    for num_containers in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*num_containers as u64));

        group.bench_with_input(
            BenchmarkId::new("gvisor", num_containers),
            num_containers,
            |b, &num| {
                b.iter(|| {
                    // Spawn N containers in parallel
                    let handles: Vec<_> = (0..num)
                        .map(|_| {
                            tokio::spawn(async {
                                let backend = GvisorBackend::new(GvisorConfig::default()).unwrap();
                                let cmd = Cmd::new("echo").arg("test");
                                backend.run_cmd(cmd).unwrap()
                            })
                        })
                        .collect();

                    // Wait for all
                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_parallel_execution);
criterion_main!(benches);
```

**Benchmarks to implement:**
1. Container startup (cold/warm)
2. Parallel execution (1-16 containers)
3. Network performance (latency/throughput)
4. Filesystem I/O (read/write)
5. Memory usage
6. Service startup (SurrealDB, PostgreSQL)

**Success Criteria:**
- [x] All benchmarks pass
- [x] Performance targets met
- [x] Regression tests in place
- [x] CI integration complete

**Estimated Effort**: 1.5 days

---

### 📊 Phase 6 Deliverables

| Deliverable | Status | Owner |
|-------------|--------|-------|
| Docker elimination script | ⏳ | TBD |
| Test validation | ⏳ | TBD |
| Documentation | ⏳ | TBD |
| Benchmarks | ⏳ | TBD |
| CI integration | ⏳ | TBD |

### ✅ Phase 6 Success Criteria

- [ ] Zero Docker references in codebase
- [ ] 100% test pass rate
- [ ] All documentation complete
- [ ] Performance benchmarks pass
- [ ] CI/CD integration complete
- [ ] Production ready

---

## Cross-Phase Concerns

### CI/CD Integration

**File**: `/home/user/clnrm/.github/workflows/gvisor.yml`

```yaml
name: gVisor Integration

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-gvisor:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install gVisor
        run: |
          wget https://storage.googleapis.com/gvisor/releases/release/latest/runsc
          chmod +x runsc
          sudo mv runsc /usr/local/bin/

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: |
          export CLNRM_BACKEND=gvisor
          cargo test --all --features gvisor

      - name: Run benchmarks
        run: cargo bench --features gvisor --no-run
```

---

## Risk Mitigation Strategy

### High-Priority Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| gVisor platform incompatibility | High | Medium | Support multiple platforms (ptrace, kvm, systrap) |
| Performance degradation | High | Low | Benchmark early, optimize hot paths |
| OCI image compatibility issues | High | Medium | Test with wide variety of images |
| Network isolation failures | High | Low | Comprehensive security testing |
| Migration complexity | Medium | Medium | Incremental migration, dual-backend support |

### Contingency Plans

1. **Performance Issues**: Implement container pooling, optimize image caching
2. **Platform Issues**: Provide fallback to testcontainers with feature flag
3. **Migration Blockers**: Maintain backward compatibility during transition
4. **Test Failures**: Parallel testing with both backends during migration

---

## Success Metrics

### Quantitative Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Container startup (cold) | 3-5s | < 3s | Benchmark suite |
| Container startup (warm) | 1-2s | < 500ms | Benchmark suite |
| Memory overhead | 150-200MB | < 100MB | Runtime profiling |
| Test pass rate | 100% | 100% | CI |
| Docker references | N/A | 0 | grep validation |
| OTLP export success | 100% | 100% | Integration tests |

### Qualitative Metrics

- [ ] Code quality maintained
- [ ] Documentation complete and clear
- [ ] User migration path straightforward
- [ ] Performance improvements visible
- [ ] Security posture improved

---

## Timeline and Milestones

```
Week 1 (Days 1-5):   Foundation
├─ Day 1:  ContainerBackend trait ✅
├─ Day 2-3: runsc wrapper ✅
├─ Day 4-5: OCI image loading ✅
└─ Milestone: Basic container execution

Week 2 (Days 6-10):  Core Runtime
├─ Day 6-7:  Container execution ✅
├─ Day 8-9:  Network isolation ✅
├─ Day 10:   Filesystem & ports ✅
└─ Milestone: Full container lifecycle

Week 3 (Days 11-15): Services
├─ Day 11-12: SurrealDB service ✅
├─ Day 13-14: Generic service plugin ✅
├─ Day 15:    Registry & health checks ✅
└─ Milestone: Service management complete

Week 4 (Days 16-20): Migration
├─ Day 16-18: Test migration ✅
├─ Day 19:    TOML migration ✅
├─ Day 20:    Examples update ✅
└─ Milestone: All code migrated

Week 5 (Days 21-25): Integration
├─ Day 21-22: Telemetry integration ✅
├─ Day 23:    OTLP validation ✅
├─ Day 24:    Weaver compatibility ✅
├─ Day 25:    Performance validation ✅
└─ Milestone: Telemetry complete

Week 6 (Days 26-30): Validation
├─ Day 26:    Docker elimination ✅
├─ Day 27:    Test validation ✅
├─ Day 28-29: Documentation ✅
├─ Day 30:    Final benchmarks ✅
└─ Milestone: Production ready 🎉
```

---

## Team Structure

### Recommended Team

- **Lead Engineer** (1): Overall architecture and integration
- **Backend Engineer** (1): gVisor runtime and OCI implementation
- **Services Engineer** (1): Service management and health checks
- **QA Engineer** (0.5): Testing and validation
- **Tech Writer** (0.5): Documentation

### Responsibilities Matrix

| Task | Lead | Backend | Services | QA | Writer |
|------|------|---------|----------|-----|--------|
| Architecture | A | C | I | - | - |
| runsc wrapper | I | A | - | C | - |
| OCI images | I | A | - | C | - |
| Network | C | A | - | I | - |
| Services | I | - | A | C | - |
| Migration | A | C | C | I | - |
| Testing | C | I | I | A | - |
| Documentation | I | - | - | - | A |

Legend: A = Accountable, C = Consulted, I = Informed

---

## Dependencies and Prerequisites

### External Dependencies

1. **gVisor Runtime**
   - Version: latest (2024+)
   - Installation: `sudo apt-get install gvisor-runsc`
   - Platform: Linux x86_64, ARM64

2. **Rust Crates**
   ```toml
   oci-distribution = "0.11"
   oci-spec = "0.6"
   nix = "0.27"  # For namespaces
   ```

3. **System Requirements**
   - Linux kernel 4.14+ (for user namespaces)
   - 2GB+ RAM
   - 10GB+ disk space (for image cache)

### Internal Dependencies

1. **Existing Code**
   - Backend trait (`crates/clnrm-core/src/backend/mod.rs`)
   - Service plugin system (`crates/clnrm-core/src/services/mod.rs`)
   - Telemetry infrastructure (`crates/clnrm-core/src/telemetry/`)

2. **Test Infrastructure**
   - Existing test suite
   - Benchmark framework
   - CI/CD pipeline

---

## Post-Implementation

### Maintenance Plan

1. **Monitoring**
   - Performance metrics dashboard
   - Error rate tracking
   - User feedback collection

2. **Support**
   - GitHub issues triage
   - Community support channel
   - Documentation updates

3. **Future Enhancements**
   - Container pooling for 10x performance
   - Multi-platform support (macOS via VM)
   - Advanced security features

### Deprecation Timeline

| Date | Action |
|------|--------|
| Week 7 | Announce testcontainers deprecation |
| Week 8-10 | Parallel support (both backends) |
| Week 11 | Make gVisor default |
| Week 12 | Remove testcontainers code |

---

## Conclusion

This roadmap provides a comprehensive, phased approach to replacing Docker with gVisor across the entire clnrm codebase. By following this plan:

- ✅ **Week 1**: Foundation in place
- ✅ **Week 2**: Core runtime complete
- ✅ **Week 3**: Services working
- ✅ **Week 4**: Migration complete
- ✅ **Week 5**: Integration validated
- ✅ **Week 6**: Production ready

**Expected Outcomes:**
- Zero Docker daemon dependency
- 40% faster container startup
- 50% less memory overhead
- 100% test pass rate
- Full OTLP telemetry support
- Production-ready service management

**Next Steps:**
1. Review and approve roadmap
2. Allocate team resources
3. Begin Phase 1 implementation
4. Weekly progress reviews
5. Adjust timeline as needed

---

**Document Version**: 1.0
**Last Updated**: 2026-01-05
**Status**: Ready for Review
**Approval Required**: Platform Lead, Engineering Manager
