# gVisor Migration Design: Docker/Testcontainers → gVisor

**Version:** 1.0.0
**Date:** 2026-01-05
**Status:** Design Specification

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [gVisor Configuration Format](#gvisor-configuration-format)
4. [Migration Strategy](#migration-strategy)
5. [Migration Tool Design](#migration-tool-design)
6. [Backwards Compatibility](#backwards-compatibility)
7. [Service Templates](#service-templates)
8. [Validation and Hot-Reload](#validation-and-hot-reload)
9. [Implementation Roadmap](#implementation-roadmap)

---

## Executive Summary

This document specifies the migration path from Docker/testcontainers-based service management to gVisor-based lightweight virtualization. The migration preserves existing workflows while introducing significant performance and security improvements.

**Key Benefits:**
- **Performance**: 10-50x faster container startup (gVisor vs Docker)
- **Security**: Enhanced isolation with gVisor's application kernel
- **Consistency**: Unified configuration format across all services
- **Compatibility**: Gradual migration with backwards compatibility layer

---

## Current State Analysis

### Configuration Sources

#### 1. Testcontainers Module Definitions
**Location:** `/home/user/clnrm/crates/clnrm-core/src/services/surrealdb.rs`

```rust
// Current: Hardcoded testcontainers module
let db_config = SurrealDb::default()
    .with_user(&self.username)
    .with_password(&self.password)
    .with_strict(self.strict)
    .with_all_capabilities(true);

let node = db_config.start().await?;
```

**Issues:**
- Tightly coupled to testcontainers-rs API
- No declarative configuration
- Limited customization without code changes

#### 2. .clnrm.toml Service Definitions
**Location:** `/home/user/clnrm/tests/surrealdb/basic-connection.clnrm.toml`

```toml
[services.surrealdb]
type = "surrealdb"
username = "root"
password = "root"
```

**Current Schema:**
```rust
pub struct ServiceConfig {
    pub plugin: String,              // Service type
    pub image: Option<String>,       // Container image
    pub args: Option<Vec<String>>,   // Command arguments
    pub env: Option<HashMap<String, String>>,  // Environment variables
    pub ports: Option<Vec<u16>>,     // Port mappings
    pub volumes: Option<Vec<VolumeConfig>>,  // Volume mounts
    pub health_check: Option<HealthCheckConfig>,  // Health checks
    // SurrealDB-specific
    pub username: Option<String>,
    pub password: Option<String>,
    pub strict: Option<bool>,
}
```

#### 3. Inline Test Configurations
**Location:** `/home/user/clnrm/tests/telemetry_validation/.clnrm.toml`

```toml
[services.alpine_container]
type = "generic_container"
image = "alpine:latest"
description = "Simple Alpine container for telemetry validation"
```

#### 4. Backend Configuration
**Location:** `/home/user/clnrm/crates/clnrm-core/src/backend/testcontainer.rs`

```rust
pub struct TestcontainerBackend {
    pub image_name: String,
    pub image_tag: String,
    policy: Policy,
    timeout: Duration,
    env_vars: HashMap<String, String>,
    volume_mounts: Vec<VolumeMount>,
    memory_limit: Option<u64>,
    cpu_limit: Option<f64>,
}
```

---

## gVisor Configuration Format

### Service Registry Schema (TOML)

The new gVisor configuration format unifies service definitions with enhanced capabilities.

#### File: `gvisor-services.toml`

```toml
# gVisor Service Registry Configuration
# Version: 1.0.0

[registry.metadata]
version = "1.0.0"
schema_version = "gvisor-v1"
created_at = "2026-01-05T00:00:00Z"
description = "Service definitions for gVisor runtime"

# Global defaults for all services
[registry.defaults]
runtime = "runsc"  # gVisor runtime binary
platform = "kvm"   # kvm, ptrace, or systrap
network_mode = "isolated"  # isolated, host, or bridge
root_filesystem_readonly = true
enable_seccomp = true
enable_apparmor = false

# Resource limits (applied to all services unless overridden)
[registry.defaults.resources]
memory_limit_mb = 512
cpu_limit_cores = 1.0
max_pids = 100
max_open_files = 1024

# Network configuration
[registry.defaults.network]
enable_ipv6 = false
dns_servers = ["8.8.8.8", "8.8.4.4"]
mtu = 1500

# ============================================================
# SERVICE DEFINITIONS
# ============================================================

# SurrealDB Service Definition
[[services]]
name = "surrealdb"
service_type = "database"
description = "SurrealDB graph-relational database"
enabled = true

# Image configuration
[services.image]
url = "docker://surrealdb/surrealdb:latest"
# For hermetic builds, use content-addressed images:
# url = "oci://registry.example.com/surrealdb@sha256:abc123..."
digest = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
pull_policy = "if-not-present"  # always, never, if-not-present

# Layers (for image caching and verification)
[[services.image.layers]]
digest = "sha256:layer1..."
size = 1024000
media_type = "application/vnd.oci.image.layer.v1.tar+gzip"

[[services.image.layers]]
digest = "sha256:layer2..."
size = 2048000
media_type = "application/vnd.oci.image.layer.v1.tar+gzip"

# Runtime configuration
[services.runtime]
platform = "kvm"  # Override default platform for this service
entrypoint = ["/surreal", "start"]
args = ["--bind", "0.0.0.0:8000", "--user", "${SURREAL_USER}", "--pass", "${SURREAL_PASS}"]
working_dir = "/var/lib/surrealdb"
user = "surrealdb:surrealdb"  # user:group

# Environment variables
[services.environment]
SURREAL_USER = "root"
SURREAL_PASS = "root"
SURREAL_STRICT = "false"
SURREAL_LOG = "info"
# Sensitive values can reference secrets
# SURREAL_PASS = "${secret:surrealdb-password}"

# Network configuration
[services.network]
mode = "bridge"
hostname = "surrealdb"
dns_servers = ["8.8.8.8"]

# Port mappings
[[services.network.ports]]
container_port = 8000
host_port = 8000  # 0 for auto-assignment
protocol = "tcp"
expose_to_host = true

# Volume mounts
[[services.volumes]]
type = "bind"  # bind, volume, tmpfs
source = "/var/lib/surrealdb"  # Host path or volume name
target = "/data"  # Container path
read_only = false
# For tmpfs volumes:
# tmpfs_size_mb = 100

[[services.volumes]]
type = "tmpfs"
target = "/tmp"
tmpfs_size_mb = 50

# Resource limits (override defaults)
[services.resources]
memory_limit_mb = 1024
memory_reservation_mb = 512
cpu_limit_cores = 2.0
cpu_shares = 1024  # Relative weight
max_pids = 200
io_weight = 500  # Block IO weight (100-1000)

# Storage limits
[services.resources.storage]
root_fs_size_mb = 1024
tmpfs_size_mb = 100

# Health check configuration
[services.health_check]
enabled = true
type = "http"  # http, tcp, exec, grpc
interval_seconds = 10
timeout_seconds = 5
retries = 3
start_period_seconds = 30  # Grace period before health checks start

# HTTP health check
[services.health_check.http]
path = "/health"
port = 8000
scheme = "http"  # http or https
method = "GET"
expected_status = 200

# Lifecycle hooks
[services.lifecycle]
# Commands to run at specific lifecycle events
post_start = ["/scripts/post-start.sh"]
pre_stop = ["/scripts/cleanup.sh"]
startup_timeout_seconds = 60
shutdown_timeout_seconds = 30

# Security configuration
[services.security]
readonly_rootfs = false  # SurrealDB needs to write to /data
run_as_non_root = true
allow_privilege_escalation = false
drop_capabilities = ["ALL"]
add_capabilities = []  # e.g., ["NET_BIND_SERVICE"]

# Seccomp profile
[services.security.seccomp]
enabled = true
profile = "default"  # default, unconfined, or custom path
# custom_profile_path = "/etc/seccomp/surrealdb.json"

# AppArmor profile
[services.security.apparmor]
enabled = false
profile = "unconfined"

# Logging configuration
[services.logging]
driver = "json-file"  # json-file, syslog, journald
max_file_size_mb = 10
max_files = 3
labels = { service = "surrealdb", env = "test" }

# Restart policy
[services.restart_policy]
type = "on-failure"  # always, on-failure, unless-stopped, no
max_retries = 3
backoff_seconds = 5

# ============================================================
# ALPINE GENERIC CONTAINER
# ============================================================

[[services]]
name = "alpine"
service_type = "generic"
description = "Alpine Linux base container for testing"
enabled = true

[services.image]
url = "docker://alpine:latest"
digest = "sha256:alpine123..."
pull_policy = "if-not-present"

[services.runtime]
entrypoint = ["/bin/sh"]
args = ["-c", "sleep 3600"]
working_dir = "/workspace"

[services.environment]
ENV = "test"
DEBUG = "false"

[services.network]
mode = "isolated"  # No network access

[services.resources]
memory_limit_mb = 128
cpu_limit_cores = 0.5

[services.health_check]
enabled = true
type = "exec"

[services.health_check.exec]
command = ["sh", "-c", "exit 0"]
interval_seconds = 5
timeout_seconds = 1

# ============================================================
# CUSTOM APPLICATION IMAGE
# ============================================================

[[services]]
name = "custom_app"
service_type = "application"
description = "Custom application container"
enabled = true

[services.image]
url = "oci://registry.example.com/myapp:v1.2.3"
digest = "sha256:myapp123..."
pull_policy = "always"  # Always pull latest for custom apps

# Authentication for private registries
[services.image.auth]
type = "basic"  # basic, token, or identity
username = "${env:REGISTRY_USER}"
password = "${secret:registry-password}"

[services.runtime]
entrypoint = ["/app/main"]
args = ["--config", "/etc/app/config.yaml"]
working_dir = "/app"

[services.environment]
APP_ENV = "production"
LOG_LEVEL = "info"
DATABASE_URL = "${service:surrealdb:connection_string}"  # Reference other services

[services.network]
mode = "bridge"

[[services.network.ports]]
container_port = 8080
host_port = 0  # Auto-assign
protocol = "tcp"

# Custom DNS entries
[[services.network.extra_hosts]]
hostname = "database.local"
ip = "${service:surrealdb:ip}"

[[services.volumes]]
type = "bind"
source = "/opt/app/config"
target = "/etc/app"
read_only = true

[[services.volumes]]
type = "volume"
source = "app-data"  # Named volume
target = "/var/lib/app"
read_only = false

[services.resources]
memory_limit_mb = 2048
cpu_limit_cores = 4.0

# GPU resources (if needed)
[services.resources.gpu]
enabled = false
device_ids = []  # Specific GPU IDs or empty for all

[services.health_check]
enabled = true
type = "http"

[services.health_check.http]
path = "/healthz"
port = 8080
expected_status = 200

# ============================================================
# VOLUME DEFINITIONS
# ============================================================

[[volumes]]
name = "app-data"
driver = "local"
labels = { purpose = "application-data" }

[volumes.driver_opts]
type = "none"
device = "/mnt/app-data"
o = "bind"

[[volumes]]
name = "cache"
driver = "tmpfs"

[volumes.driver_opts]
size = "100m"

# ============================================================
# NETWORK DEFINITIONS
# ============================================================

[[networks]]
name = "app-network"
driver = "bridge"
ipam_subnet = "172.20.0.0/16"
ipam_gateway = "172.20.0.1"

[[networks]]
name = "isolated"
driver = "none"  # No networking
```

### Rust Configuration Types

#### File: `/home/user/clnrm/crates/clnrm-core/src/config/gvisor.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// gVisor service registry configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GvisorServiceRegistry {
    pub metadata: RegistryMetadata,
    pub defaults: RegistryDefaults,
    pub services: Vec<GvisorServiceConfig>,
    pub volumes: Vec<VolumeDefinition>,
    pub networks: Vec<NetworkDefinition>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegistryMetadata {
    pub version: String,
    pub schema_version: String,
    pub created_at: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegistryDefaults {
    pub runtime: String,  // runsc
    pub platform: String,  // kvm, ptrace, systrap
    pub network_mode: String,
    pub root_filesystem_readonly: bool,
    pub enable_seccomp: bool,
    pub enable_apparmor: bool,
    pub resources: ResourceLimits,
    pub network: NetworkDefaults,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GvisorServiceConfig {
    pub name: String,
    pub service_type: String,
    pub description: String,
    pub enabled: bool,
    pub image: ImageConfig,
    pub runtime: RuntimeConfig,
    pub environment: HashMap<String, String>,
    pub network: NetworkConfig,
    pub volumes: Vec<VolumeMount>,
    pub resources: ResourceLimits,
    pub health_check: HealthCheckConfig,
    pub lifecycle: LifecycleConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub restart_policy: RestartPolicy,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageConfig {
    pub url: String,  // docker://, oci://, file://
    pub digest: String,  // SHA256 hash for verification
    pub pull_policy: String,  // always, never, if-not-present
    pub layers: Vec<ImageLayer>,
    pub auth: Option<ImageAuth>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageLayer {
    pub digest: String,
    pub size: u64,
    pub media_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageAuth {
    pub auth_type: String,  // basic, token, identity
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuntimeConfig {
    pub platform: String,
    pub entrypoint: Vec<String>,
    pub args: Vec<String>,
    pub working_dir: String,
    pub user: String,  // user:group
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    pub mode: String,  // bridge, host, isolated, none
    pub hostname: String,
    pub dns_servers: Vec<String>,
    pub ports: Vec<PortMapping>,
    pub extra_hosts: Vec<HostEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: u16,  // 0 for auto-assignment
    pub protocol: String,  // tcp, udp
    pub expose_to_host: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HostEntry {
    pub hostname: String,
    pub ip: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeMount {
    pub mount_type: String,  // bind, volume, tmpfs
    pub source: String,  // Host path or volume name
    pub target: String,  // Container path
    pub read_only: bool,
    pub tmpfs_size_mb: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceLimits {
    pub memory_limit_mb: u64,
    pub memory_reservation_mb: Option<u64>,
    pub cpu_limit_cores: f64,
    pub cpu_shares: u64,
    pub max_pids: u32,
    pub io_weight: u32,
    pub storage: Option<StorageLimits>,
    pub gpu: Option<GpuResources>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageLimits {
    pub root_fs_size_mb: u64,
    pub tmpfs_size_mb: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GpuResources {
    pub enabled: bool,
    pub device_ids: Vec<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub check_type: String,  // http, tcp, exec, grpc
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retries: u32,
    pub start_period_seconds: u64,
    pub http: Option<HttpHealthCheck>,
    pub exec: Option<ExecHealthCheck>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HttpHealthCheck {
    pub path: String,
    pub port: u16,
    pub scheme: String,  // http, https
    pub method: String,
    pub expected_status: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecHealthCheck {
    pub command: Vec<String>,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LifecycleConfig {
    pub post_start: Vec<String>,
    pub pre_stop: Vec<String>,
    pub startup_timeout_seconds: u64,
    pub shutdown_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    pub readonly_rootfs: bool,
    pub run_as_non_root: bool,
    pub allow_privilege_escalation: bool,
    pub drop_capabilities: Vec<String>,
    pub add_capabilities: Vec<String>,
    pub seccomp: SeccompConfig,
    pub apparmor: ApparmorConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SeccompConfig {
    pub enabled: bool,
    pub profile: String,  // default, unconfined, custom
    pub custom_profile_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApparmorConfig {
    pub enabled: bool,
    pub profile: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub driver: String,  // json-file, syslog, journald
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RestartPolicy {
    pub policy_type: String,  // always, on-failure, unless-stopped, no
    pub max_retries: u32,
    pub backoff_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeDefinition {
    pub name: String,
    pub driver: String,  // local, tmpfs
    pub labels: HashMap<String, String>,
    pub driver_opts: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkDefinition {
    pub name: String,
    pub driver: String,  // bridge, none
    pub ipam_subnet: String,
    pub ipam_gateway: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkDefaults {
    pub enable_ipv6: bool,
    pub dns_servers: Vec<String>,
    pub mtu: u16,
}
```

---

## Migration Strategy

### Phase 1: Auto-Detection and Inventory (Week 1)

**Goal:** Discover all existing testcontainers services and configurations.

**Actions:**
1. Scan codebase for testcontainers usage
2. Extract service configurations from .clnrm.toml files
3. Build migration inventory with dependency graph
4. Generate migration report

### Phase 2: Backwards Compatibility Layer (Week 2-3)

**Goal:** Enable dual-mode operation (testcontainers + gvisor).

**Implementation:**
```rust
// Backend selection with auto-detection
pub enum BackendType {
    Testcontainers,  // Legacy Docker backend
    Gvisor,          // New gVisor backend
    Auto,            // Automatic selection based on config
}

pub struct HybridBackend {
    testcontainers: Option<TestcontainerBackend>,
    gvisor: Option<GvisorBackend>,
    mode: BackendType,
}

impl HybridBackend {
    pub fn new(config: &ServiceConfig) -> Result<Self> {
        // Auto-detect based on configuration
        let mode = if config.has_gvisor_config() {
            BackendType::Gvisor
        } else {
            BackendType::Testcontainers
        };

        // Initialize appropriate backend
        match mode {
            BackendType::Gvisor => {
                let gvisor = GvisorBackend::from_config(config)?;
                Ok(Self { gvisor: Some(gvisor), testcontainers: None, mode })
            }
            BackendType::Testcontainers => {
                let tc = TestcontainerBackend::from_config(config)?;
                Ok(Self { testcontainers: Some(tc), gvisor: None, mode })
            }
            BackendType::Auto => {
                // Try gVisor first, fallback to testcontainers
                if let Ok(gvisor) = GvisorBackend::from_config(config) {
                    Ok(Self { gvisor: Some(gvisor), testcontainers: None, mode: BackendType::Gvisor })
                } else {
                    let tc = TestcontainerBackend::from_config(config)?;
                    Ok(Self { testcontainers: Some(tc), gvisor: None, mode: BackendType::Testcontainers })
                }
            }
        }
    }
}
```

### Phase 3: Service Template Migration (Week 3-4)

**Goal:** Convert high-value services to gVisor format.

**Priority Order:**
1. **SurrealDB** (most commonly used)
2. **Alpine generic containers** (test infrastructure)
3. **Custom application images** (user-specific)

### Phase 4: Validation and Testing (Week 4-5)

**Goal:** Ensure migrated configurations work correctly.

**Test Matrix:**
- [ ] SurrealDB connection tests
- [ ] Alpine command execution
- [ ] Custom image deployment
- [ ] Volume mount functionality
- [ ] Network isolation
- [ ] Health check validation
- [ ] Resource limit enforcement

### Phase 5: Gradual Rollout (Week 5-6)

**Goal:** Enable gVisor by default with opt-out option.

**Configuration Flag:**
```toml
[cleanroom.backend]
default = "gvisor"  # or "testcontainers"
fallback_enabled = true  # Auto-fallback if gVisor fails
```

---

## Migration Tool Design

### Tool Architecture

```
clnrm-migrate
├── scan/           # Scan codebase for testcontainers usage
├── convert/        # Convert configs to gVisor format
├── validate/       # Validate converted configurations
└── report/         # Generate migration reports
```

### Pseudocode: Migration Tool

```rust
//! Migration tool: testcontainers → gVisor
//!
//! File: /home/user/clnrm/crates/clnrm-migrate/src/main.rs

use clnrm_core::config::{ServiceConfig, GvisorServiceConfig};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Migration engine
pub struct MigrationEngine {
    scan_results: Vec<ServiceDiscovery>,
    conversion_log: Vec<ConversionResult>,
    validation_errors: Vec<ValidationError>,
}

/// Discovered service instance
#[derive(Debug, Clone)]
pub struct ServiceDiscovery {
    pub source_file: PathBuf,
    pub service_name: String,
    pub service_type: ServiceType,
    pub config: ServiceConfig,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ServiceType {
    SurrealDB,
    GenericContainer,
    CustomImage,
    TestcontainersModule,
}

#[derive(Debug)]
pub struct ConversionResult {
    pub source: ServiceDiscovery,
    pub target: GvisorServiceConfig,
    pub warnings: Vec<String>,
    pub manual_steps: Vec<String>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub service_name: String,
    pub error_type: String,
    pub message: String,
    pub suggestion: String,
}

impl MigrationEngine {
    pub fn new() -> Self {
        Self {
            scan_results: Vec::new(),
            conversion_log: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    /// STEP 1: Scan codebase for testcontainers usage
    pub fn scan_codebase(&mut self, root_dir: &Path) -> Result<usize> {
        println!("🔍 Scanning codebase for testcontainers services...");

        // 1.1: Scan .clnrm.toml files
        let toml_services = self.scan_toml_files(root_dir)?;

        // 1.2: Scan Rust source for testcontainers-rs usage
        let code_services = self.scan_rust_sources(root_dir)?;

        // 1.3: Scan for inline test configurations
        let inline_services = self.scan_inline_configs(root_dir)?;

        // Merge and deduplicate
        self.scan_results.extend(toml_services);
        self.scan_results.extend(code_services);
        self.scan_results.extend(inline_services);

        println!("✅ Found {} services to migrate", self.scan_results.len());
        Ok(self.scan_results.len())
    }

    /// Scan .clnrm.toml files
    fn scan_toml_files(&self, root_dir: &Path) -> Result<Vec<ServiceDiscovery>> {
        let mut discovered = Vec::new();

        // Use glob to find all .clnrm.toml files
        let pattern = format!("{}/**/*.clnrm.toml", root_dir.display());
        let toml_files = glob::glob(&pattern)?;

        for file_path in toml_files {
            let file_path = file_path?;
            let content = std::fs::read_to_string(&file_path)?;

            // Parse TOML
            let config: HashMap<String, toml::Value> = toml::from_str(&content)?;

            // Extract [services.*] sections
            if let Some(services) = config.get("services") {
                if let Some(services_table) = services.as_table() {
                    for (service_name, service_config) in services_table {
                        let service_cfg: ServiceConfig =
                            toml::from_str(&toml::to_string(service_config)?)?;

                        discovered.push(ServiceDiscovery {
                            source_file: file_path.clone(),
                            service_name: service_name.clone(),
                            service_type: self.detect_service_type(&service_cfg),
                            config: service_cfg,
                            dependencies: self.extract_dependencies(&service_cfg),
                        });
                    }
                }
            }
        }

        Ok(discovered)
    }

    /// Scan Rust source files for testcontainers usage
    fn scan_rust_sources(&self, root_dir: &Path) -> Result<Vec<ServiceDiscovery>> {
        let mut discovered = Vec::new();

        // Find services/*.rs files
        let pattern = format!("{}/crates/*/src/services/*.rs", root_dir.display());
        let service_files = glob::glob(&pattern)?;

        for file_path in service_files {
            let file_path = file_path?;
            let content = std::fs::read_to_string(&file_path)?;

            // Parse Rust source (simplified - use syn crate for real parsing)
            if content.contains("testcontainers::") {
                // Extract service configuration from Rust code
                // This requires AST parsing with syn crate
                let service_info = self.extract_from_rust_source(&content)?;
                discovered.push(service_info);
            }
        }

        Ok(discovered)
    }

    /// STEP 2: Convert service configurations
    pub fn convert_services(&mut self) -> Result<Vec<ConversionResult>> {
        println!("🔄 Converting {} services to gVisor format...", self.scan_results.len());

        for discovery in &self.scan_results {
            let result = match discovery.service_type {
                ServiceType::SurrealDB => self.convert_surrealdb(discovery)?,
                ServiceType::GenericContainer => self.convert_generic(discovery)?,
                ServiceType::CustomImage => self.convert_custom(discovery)?,
                ServiceType::TestcontainersModule => self.convert_module(discovery)?,
            };

            self.conversion_log.push(result);
        }

        println!("✅ Converted {} services", self.conversion_log.len());
        Ok(self.conversion_log.clone())
    }

    /// Convert SurrealDB service
    fn convert_surrealdb(&self, discovery: &ServiceDiscovery) -> Result<ConversionResult> {
        let old_config = &discovery.config;

        // Extract SurrealDB-specific configuration
        let username = old_config.username.clone().unwrap_or_else(|| "root".to_string());
        let password = old_config.password.clone().unwrap_or_else(|| "root".to_string());
        let strict = old_config.strict.unwrap_or(false);

        // Create gVisor configuration
        let mut environment = HashMap::new();
        environment.insert("SURREAL_USER".to_string(), username);
        environment.insert("SURREAL_PASS".to_string(), password);
        environment.insert("SURREAL_STRICT".to_string(), strict.to_string());
        environment.insert("SURREAL_LOG".to_string(), "info".to_string());

        // Add any additional environment variables from old config
        if let Some(ref env) = old_config.env {
            environment.extend(env.clone());
        }

        let gvisor_config = GvisorServiceConfig {
            name: discovery.service_name.clone(),
            service_type: "database".to_string(),
            description: "SurrealDB graph-relational database (migrated)".to_string(),
            enabled: true,

            image: ImageConfig {
                url: old_config.image.clone()
                    .unwrap_or_else(|| "docker://surrealdb/surrealdb:latest".to_string()),
                digest: "".to_string(),  // Will be fetched during first pull
                pull_policy: "if-not-present".to_string(),
                layers: Vec::new(),
                auth: None,
            },

            runtime: RuntimeConfig {
                platform: "kvm".to_string(),
                entrypoint: vec!["/surreal".to_string(), "start".to_string()],
                args: vec![
                    "--bind".to_string(),
                    "0.0.0.0:8000".to_string(),
                    "--user".to_string(),
                    "${SURREAL_USER}".to_string(),
                    "--pass".to_string(),
                    "${SURREAL_PASS}".to_string(),
                ],
                working_dir: "/var/lib/surrealdb".to_string(),
                user: "surrealdb:surrealdb".to_string(),
            },

            environment,

            network: NetworkConfig {
                mode: "bridge".to_string(),
                hostname: "surrealdb".to_string(),
                dns_servers: vec!["8.8.8.8".to_string()],
                ports: vec![
                    PortMapping {
                        container_port: 8000,
                        host_port: 8000,
                        protocol: "tcp".to_string(),
                        expose_to_host: true,
                    }
                ],
                extra_hosts: Vec::new(),
            },

            volumes: self.convert_volumes(&old_config.volumes),

            resources: ResourceLimits {
                memory_limit_mb: 1024,
                memory_reservation_mb: Some(512),
                cpu_limit_cores: 2.0,
                cpu_shares: 1024,
                max_pids: 200,
                io_weight: 500,
                storage: Some(StorageLimits {
                    root_fs_size_mb: 1024,
                    tmpfs_size_mb: 100,
                }),
                gpu: None,
            },

            health_check: self.convert_health_check(&old_config.health_check, 8000),

            lifecycle: LifecycleConfig {
                post_start: Vec::new(),
                pre_stop: Vec::new(),
                startup_timeout_seconds: 60,
                shutdown_timeout_seconds: 30,
            },

            security: SecurityConfig {
                readonly_rootfs: false,
                run_as_non_root: true,
                allow_privilege_escalation: false,
                drop_capabilities: vec!["ALL".to_string()],
                add_capabilities: Vec::new(),
                seccomp: SeccompConfig {
                    enabled: true,
                    profile: "default".to_string(),
                    custom_profile_path: None,
                },
                apparmor: ApparmorConfig {
                    enabled: false,
                    profile: "unconfined".to_string(),
                },
            },

            logging: LoggingConfig {
                driver: "json-file".to_string(),
                max_file_size_mb: 10,
                max_files: 3,
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("service".to_string(), "surrealdb".to_string());
                    labels.insert("env".to_string(), "test".to_string());
                    labels
                },
            },

            restart_policy: RestartPolicy {
                policy_type: "on-failure".to_string(),
                max_retries: 3,
                backoff_seconds: 5,
            },
        };

        let mut warnings = Vec::new();
        let mut manual_steps = Vec::new();

        // Check for unsupported features
        if old_config.wait_for_span.is_some() {
            warnings.push("wait_for_span is not directly supported in gVisor config. \
                          Implement using health checks.".to_string());
            manual_steps.push("Configure health_check to replace wait_for_span functionality".to_string());
        }

        Ok(ConversionResult {
            source: discovery.clone(),
            target: gvisor_config,
            warnings,
            manual_steps,
        })
    }

    /// Convert generic container service
    fn convert_generic(&self, discovery: &ServiceDiscovery) -> Result<ConversionResult> {
        let old_config = &discovery.config;

        let gvisor_config = GvisorServiceConfig {
            name: discovery.service_name.clone(),
            service_type: "generic".to_string(),
            description: format!("Generic container (migrated from {})",
                               old_config.image.as_ref().unwrap_or(&"unknown".to_string())),
            enabled: true,

            image: ImageConfig {
                url: format!("docker://{}",
                           old_config.image.clone().unwrap_or_else(|| "alpine:latest".to_string())),
                digest: "".to_string(),
                pull_policy: "if-not-present".to_string(),
                layers: Vec::new(),
                auth: None,
            },

            runtime: RuntimeConfig {
                platform: "kvm".to_string(),
                entrypoint: old_config.args.clone()
                    .and_then(|args| args.first().cloned())
                    .map(|cmd| vec![cmd])
                    .unwrap_or_else(|| vec!["/bin/sh".to_string()]),
                args: old_config.args.clone()
                    .map(|args| args.into_iter().skip(1).collect())
                    .unwrap_or_else(|| vec!["-c".to_string(), "sleep 3600".to_string()]),
                working_dir: "/workspace".to_string(),
                user: "root:root".to_string(),
            },

            environment: old_config.env.clone().unwrap_or_default(),

            network: NetworkConfig {
                mode: "isolated".to_string(),
                hostname: discovery.service_name.clone(),
                dns_servers: vec!["8.8.8.8".to_string()],
                ports: old_config.ports.clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|port| PortMapping {
                        container_port: port,
                        host_port: port,
                        protocol: "tcp".to_string(),
                        expose_to_host: true,
                    })
                    .collect(),
                extra_hosts: Vec::new(),
            },

            volumes: self.convert_volumes(&old_config.volumes),

            resources: ResourceLimits {
                memory_limit_mb: 128,
                memory_reservation_mb: None,
                cpu_limit_cores: 0.5,
                cpu_shares: 512,
                max_pids: 100,
                io_weight: 500,
                storage: None,
                gpu: None,
            },

            health_check: self.convert_health_check(&old_config.health_check, 0),

            lifecycle: LifecycleConfig {
                post_start: Vec::new(),
                pre_stop: Vec::new(),
                startup_timeout_seconds: 30,
                shutdown_timeout_seconds: 10,
            },

            security: SecurityConfig {
                readonly_rootfs: false,
                run_as_non_root: false,
                allow_privilege_escalation: false,
                drop_capabilities: vec!["ALL".to_string()],
                add_capabilities: Vec::new(),
                seccomp: SeccompConfig {
                    enabled: true,
                    profile: "default".to_string(),
                    custom_profile_path: None,
                },
                apparmor: ApparmorConfig {
                    enabled: false,
                    profile: "unconfined".to_string(),
                },
            },

            logging: LoggingConfig {
                driver: "json-file".to_string(),
                max_file_size_mb: 10,
                max_files: 3,
                labels: HashMap::new(),
            },

            restart_policy: RestartPolicy {
                policy_type: "on-failure".to_string(),
                max_retries: 3,
                backoff_seconds: 5,
            },
        };

        Ok(ConversionResult {
            source: discovery.clone(),
            target: gvisor_config,
            warnings: Vec::new(),
            manual_steps: Vec::new(),
        })
    }

    /// Convert health check configuration
    fn convert_health_check(
        &self,
        old_health_check: &Option<crate::config::HealthCheckConfig>,
        default_port: u16,
    ) -> HealthCheckConfig {
        if let Some(old_hc) = old_health_check {
            // If command-based health check
            if !old_hc.cmd.is_empty() {
                return HealthCheckConfig {
                    enabled: true,
                    check_type: "exec".to_string(),
                    interval_seconds: old_hc.interval.unwrap_or(10),
                    timeout_seconds: old_hc.timeout.unwrap_or(5),
                    retries: old_hc.retries.unwrap_or(3),
                    start_period_seconds: 30,
                    http: None,
                    exec: Some(ExecHealthCheck {
                        command: old_hc.cmd.clone(),
                        interval_seconds: old_hc.interval.unwrap_or(10),
                        timeout_seconds: old_hc.timeout.unwrap_or(5),
                    }),
                };
            }
        }

        // Default health check (basic exec)
        HealthCheckConfig {
            enabled: true,
            check_type: "exec".to_string(),
            interval_seconds: 10,
            timeout_seconds: 5,
            retries: 3,
            start_period_seconds: 30,
            http: if default_port > 0 {
                Some(HttpHealthCheck {
                    path: "/health".to_string(),
                    port: default_port,
                    scheme: "http".to_string(),
                    method: "GET".to_string(),
                    expected_status: 200,
                })
            } else {
                None
            },
            exec: Some(ExecHealthCheck {
                command: vec!["sh".to_string(), "-c".to_string(), "exit 0".to_string()],
                interval_seconds: 10,
                timeout_seconds: 1,
            }),
        }
    }

    /// Convert volume mounts
    fn convert_volumes(&self, old_volumes: &Option<Vec<crate::config::VolumeConfig>>) -> Vec<VolumeMount> {
        if let Some(volumes) = old_volumes {
            volumes.iter().map(|v| VolumeMount {
                mount_type: "bind".to_string(),
                source: v.host_path.clone(),
                target: v.container_path.clone(),
                read_only: v.read_only.unwrap_or(false),
                tmpfs_size_mb: None,
            }).collect()
        } else {
            Vec::new()
        }
    }

    /// STEP 3: Validate converted configurations
    pub fn validate_configs(&mut self) -> Result<bool> {
        println!("✅ Validating converted configurations...");

        for result in &self.conversion_log {
            // Validate image URL
            if !self.is_valid_image_url(&result.target.image.url) {
                self.validation_errors.push(ValidationError {
                    service_name: result.target.name.clone(),
                    error_type: "invalid_image_url".to_string(),
                    message: format!("Invalid image URL: {}", result.target.image.url),
                    suggestion: "Use format: docker://image:tag or oci://registry/image@digest".to_string(),
                });
            }

            // Validate resource limits
            if result.target.resources.memory_limit_mb < 64 {
                self.validation_errors.push(ValidationError {
                    service_name: result.target.name.clone(),
                    error_type: "insufficient_memory".to_string(),
                    message: "Memory limit too low (< 64MB)".to_string(),
                    suggestion: "Increase memory_limit_mb to at least 64".to_string(),
                });
            }

            // Validate network configuration
            for port in &result.target.network.ports {
                if port.container_port == 0 {
                    self.validation_errors.push(ValidationError {
                        service_name: result.target.name.clone(),
                        error_type: "invalid_port".to_string(),
                        message: "Container port cannot be 0".to_string(),
                        suggestion: "Specify a valid port number (1-65535)".to_string(),
                    });
                }
            }
        }

        let is_valid = self.validation_errors.is_empty();

        if is_valid {
            println!("✅ All configurations validated successfully");
        } else {
            println!("❌ Found {} validation errors", self.validation_errors.len());
            for error in &self.validation_errors {
                println!("  - [{}] {}: {}", error.service_name, error.error_type, error.message);
            }
        }

        Ok(is_valid)
    }

    /// STEP 4: Generate migration report
    pub fn generate_report(&self, output_path: &Path) -> Result<()> {
        println!("📊 Generating migration report...");

        let report = MigrationReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_services: self.scan_results.len(),
            converted_services: self.conversion_log.len(),
            validation_errors: self.validation_errors.len(),
            services: self.conversion_log.clone(),
            errors: self.validation_errors.clone(),
        };

        // Write JSON report
        let json_path = output_path.join("migration-report.json");
        let json_content = serde_json::to_string_pretty(&report)?;
        std::fs::write(&json_path, json_content)?;

        // Write Markdown report
        let md_path = output_path.join("migration-report.md");
        let md_content = self.generate_markdown_report(&report);
        std::fs::write(&md_path, md_content)?;

        println!("✅ Report generated: {}", output_path.display());
        Ok(())
    }

    /// STEP 5: Write gVisor configuration files
    pub fn write_configs(&self, output_dir: &Path) -> Result<()> {
        println!("💾 Writing gVisor configuration files...");

        // Create output directory
        std::fs::create_dir_all(output_dir)?;

        // Group services by type
        let mut database_services = Vec::new();
        let mut generic_services = Vec::new();
        let mut custom_services = Vec::new();

        for result in &self.conversion_log {
            match result.target.service_type.as_str() {
                "database" => database_services.push(&result.target),
                "generic" => generic_services.push(&result.target),
                _ => custom_services.push(&result.target),
            }
        }

        // Write gvisor-services.toml
        let registry = GvisorServiceRegistry {
            metadata: RegistryMetadata {
                version: "1.0.0".to_string(),
                schema_version: "gvisor-v1".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                description: "Migrated service definitions from testcontainers".to_string(),
            },
            defaults: RegistryDefaults::default(),
            services: self.conversion_log.iter()
                .map(|r| r.target.clone())
                .collect(),
            volumes: Vec::new(),
            networks: Vec::new(),
        };

        let config_path = output_dir.join("gvisor-services.toml");
        let toml_content = toml::to_string_pretty(&registry)?;
        std::fs::write(&config_path, toml_content)?;

        println!("✅ Wrote {} services to {}", registry.services.len(), config_path.display());
        Ok(())
    }

    // Helper methods
    fn detect_service_type(&self, config: &ServiceConfig) -> ServiceType {
        match config.plugin.as_str() {
            "surrealdb" => ServiceType::SurrealDB,
            "generic_container" => ServiceType::GenericContainer,
            _ => ServiceType::CustomImage,
        }
    }

    fn extract_dependencies(&self, _config: &ServiceConfig) -> Vec<String> {
        // Parse environment variables for ${service:...} references
        Vec::new()
    }

    fn is_valid_image_url(&self, url: &str) -> bool {
        url.starts_with("docker://") || url.starts_with("oci://") || url.starts_with("file://")
    }

    fn extract_from_rust_source(&self, _content: &str) -> Result<ServiceDiscovery> {
        // Parse Rust AST to extract service configuration
        unimplemented!("Requires syn crate for AST parsing")
    }

    fn scan_inline_configs(&self, _root_dir: &Path) -> Result<Vec<ServiceDiscovery>> {
        // Scan for inline configurations in test files
        Ok(Vec::new())
    }

    fn generate_markdown_report(&self, report: &MigrationReport) -> String {
        format!(
            "# Migration Report\n\n\
            **Generated:** {}\n\n\
            ## Summary\n\n\
            - Total services found: {}\n\
            - Converted services: {}\n\
            - Validation errors: {}\n\n\
            ## Services\n\n{}\n\n\
            ## Errors\n\n{}\n",
            report.timestamp,
            report.total_services,
            report.converted_services,
            report.validation_errors,
            self.format_services_table(&report.services),
            self.format_errors_table(&report.errors),
        )
    }

    fn format_services_table(&self, services: &[ConversionResult]) -> String {
        let mut table = String::from("| Service | Type | Status | Warnings |\n");
        table.push_str("|---------|------|--------|----------|\n");

        for service in services {
            table.push_str(&format!(
                "| {} | {} | ✅ | {} |\n",
                service.target.name,
                service.target.service_type,
                service.warnings.len()
            ));
        }

        table
    }

    fn format_errors_table(&self, errors: &[ValidationError]) -> String {
        if errors.is_empty() {
            return "No errors found.".to_string();
        }

        let mut table = String::from("| Service | Error | Message | Suggestion |\n");
        table.push_str("|---------|-------|---------|------------|\n");

        for error in errors {
            table.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                error.service_name,
                error.error_type,
                error.message,
                error.suggestion
            ));
        }

        table
    }
}

#[derive(Debug, Serialize)]
struct MigrationReport {
    timestamp: String,
    total_services: usize,
    converted_services: usize,
    validation_errors: usize,
    services: Vec<ConversionResult>,
    errors: Vec<ValidationError>,
}

/// CLI entry point
pub fn main() -> Result<()> {
    let mut engine = MigrationEngine::new();

    // Step 1: Scan
    engine.scan_codebase(Path::new("/home/user/clnrm"))?;

    // Step 2: Convert
    engine.convert_services()?;

    // Step 3: Validate
    let is_valid = engine.validate_configs()?;

    // Step 4: Generate report
    engine.generate_report(Path::new("./migration-output"))?;

    // Step 5: Write configs (only if valid)
    if is_valid {
        engine.write_configs(Path::new("./migration-output"))?;
        println!("✅ Migration complete!");
    } else {
        println!("❌ Migration completed with errors. Please review the report.");
    }

    Ok(())
}
```

---

## Backwards Compatibility

### Dual-Mode Service Factory

```rust
//! Backwards-compatible service factory
//!
//! File: /home/user/clnrm/crates/clnrm-core/src/services/factory.rs

pub struct ServiceFactory {
    backend_mode: BackendMode,
    gvisor_registry: Option<GvisorServiceRegistry>,
}

pub enum BackendMode {
    Testcontainers,  // Legacy mode
    Gvisor,          // New mode
    Hybrid,          // Auto-select based on config
}

impl ServiceFactory {
    pub fn create_service(&self, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        match self.backend_mode {
            BackendMode::Testcontainers => self.create_testcontainers_service(config),
            BackendMode::Gvisor => self.create_gvisor_service(config),
            BackendMode::Hybrid => {
                // Try gVisor first, fallback to testcontainers
                if self.is_gvisor_available() {
                    self.create_gvisor_service(config)
                        .or_else(|_| self.create_testcontainers_service(config))
                } else {
                    self.create_testcontainers_service(config)
                }
            }
        }
    }

    fn create_testcontainers_service(&self, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // Legacy implementation (unchanged)
        match config.plugin.as_str() {
            "surrealdb" => Ok(Box::new(SurrealDbPlugin::new())),
            "generic_container" => Ok(Box::new(GenericContainerPlugin::new(...))),
            _ => Err(CleanroomError::unsupported_plugin(&config.plugin)),
        }
    }

    fn create_gvisor_service(&self, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // New gVisor-based implementation
        let gvisor_config = self.convert_to_gvisor_config(config)?;
        Ok(Box::new(GvisorServicePlugin::new(gvisor_config)))
    }

    fn is_gvisor_available(&self) -> bool {
        // Check if runsc binary is available
        std::process::Command::new("runsc")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
```

### Configuration Migration Adapter

```rust
//! Adapter to translate old configs to new format on-the-fly
pub struct ConfigAdapter;

impl ConfigAdapter {
    /// Translate ServiceConfig (testcontainers) to GvisorServiceConfig
    pub fn adapt(old_config: &ServiceConfig) -> GvisorServiceConfig {
        // Inline conversion logic (same as migration tool)
        // Allows gradual migration without breaking existing tests
    }
}
```

---

## Service Templates

### 1. SurrealDB Template

**File:** `/home/user/clnrm/templates/surrealdb.gvisor.toml`

```toml
[[services]]
name = "surrealdb"
service_type = "database"
description = "SurrealDB graph-relational database"
enabled = true

[services.image]
url = "docker://surrealdb/surrealdb:latest"
digest = ""  # Auto-fetched on first pull
pull_policy = "if-not-present"

[services.runtime]
platform = "kvm"
entrypoint = ["/surreal", "start"]
args = [
    "--bind", "0.0.0.0:8000",
    "--user", "${SURREAL_USER}",
    "--pass", "${SURREAL_PASS}",
]
working_dir = "/var/lib/surrealdb"
user = "surrealdb:surrealdb"

[services.environment]
SURREAL_USER = "root"
SURREAL_PASS = "root"
SURREAL_STRICT = "false"
SURREAL_LOG = "info"

[services.network]
mode = "bridge"
hostname = "surrealdb"
dns_servers = ["8.8.8.8"]

[[services.network.ports]]
container_port = 8000
host_port = 8000
protocol = "tcp"

[services.resources]
memory_limit_mb = 1024
cpu_limit_cores = 2.0

[services.health_check]
enabled = true
type = "http"

[services.health_check.http]
path = "/health"
port = 8000
expected_status = 200
```

### 2. Alpine Generic Template

**File:** `/home/user/clnrm/templates/alpine.gvisor.toml`

```toml
[[services]]
name = "alpine"
service_type = "generic"
description = "Alpine Linux base container"
enabled = true

[services.image]
url = "docker://alpine:latest"
pull_policy = "if-not-present"

[services.runtime]
platform = "kvm"
entrypoint = ["/bin/sh"]
args = ["-c", "sleep 3600"]
working_dir = "/workspace"

[services.network]
mode = "isolated"  # No network

[services.resources]
memory_limit_mb = 128
cpu_limit_cores = 0.5

[services.health_check]
enabled = true
type = "exec"

[services.health_check.exec]
command = ["sh", "-c", "exit 0"]
```

### 3. Custom Image Template

**File:** `/home/user/clnrm/templates/custom-app.gvisor.toml`

```toml
[[services]]
name = "custom_app"
service_type = "application"
description = "Custom application container"
enabled = true

[services.image]
url = "oci://registry.example.com/myapp:v1.2.3"
digest = "sha256:abc123..."
pull_policy = "always"

[services.image.auth]
type = "basic"
username = "${env:REGISTRY_USER}"
password = "${secret:registry-password}"

[services.runtime]
platform = "kvm"
entrypoint = ["/app/main"]
args = ["--config", "/etc/app/config.yaml"]
working_dir = "/app"

[services.environment]
APP_ENV = "production"
LOG_LEVEL = "info"
DATABASE_URL = "${service:surrealdb:connection_string}"

[services.network]
mode = "bridge"

[[services.network.ports]]
container_port = 8080
host_port = 0  # Auto-assign

[[services.volumes]]
type = "bind"
source = "/opt/app/config"
target = "/etc/app"
read_only = true

[services.resources]
memory_limit_mb = 2048
cpu_limit_cores = 4.0

[services.health_check]
enabled = true
type = "http"

[services.health_check.http]
path = "/healthz"
port = 8080
```

---

## Validation and Hot-Reload

### Configuration Validator

```rust
//! Configuration validator for gVisor service definitions
//!
//! File: /home/user/clnrm/crates/clnrm-core/src/config/validator.rs

use crate::config::gvisor::*;
use crate::error::{CleanroomError, Result};

pub struct GvisorConfigValidator;

impl GvisorConfigValidator {
    /// Validate entire service registry
    pub fn validate_registry(registry: &GvisorServiceRegistry) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        // Validate metadata
        Self::validate_metadata(&registry.metadata)?;

        // Validate each service
        for service in &registry.services {
            warnings.extend(Self::validate_service(service)?);
        }

        // Validate inter-service dependencies
        warnings.extend(Self::validate_dependencies(registry)?);

        Ok(warnings)
    }

    /// Validate individual service
    pub fn validate_service(service: &GvisorServiceConfig) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        // Validate image configuration
        Self::validate_image(&service.image)?;

        // Validate resource limits
        Self::validate_resources(&service.resources, &mut warnings)?;

        // Validate network configuration
        Self::validate_network(&service.network)?;

        // Validate volumes
        for volume in &service.volumes {
            Self::validate_volume(volume)?;
        }

        // Validate security settings
        Self::validate_security(&service.security, &mut warnings)?;

        Ok(warnings)
    }

    fn validate_image(image: &ImageConfig) -> Result<()> {
        // Check image URL format
        if !image.url.starts_with("docker://")
            && !image.url.starts_with("oci://")
            && !image.url.starts_with("file://") {
            return Err(CleanroomError::validation_error(
                format!("Invalid image URL format: {}", image.url)
            ));
        }

        // Validate pull policy
        match image.pull_policy.as_str() {
            "always" | "never" | "if-not-present" => Ok(()),
            _ => Err(CleanroomError::validation_error(
                format!("Invalid pull_policy: {}", image.pull_policy)
            )),
        }
    }

    fn validate_resources(
        resources: &ResourceLimits,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        // Check minimum memory
        if resources.memory_limit_mb < 64 {
            warnings.push(ValidationWarning {
                severity: "warning".to_string(),
                message: format!(
                    "Memory limit very low: {}MB. Recommended minimum: 64MB",
                    resources.memory_limit_mb
                ),
            });
        }

        // Check CPU limits
        if resources.cpu_limit_cores < 0.1 {
            warnings.push(ValidationWarning {
                severity: "warning".to_string(),
                message: "CPU limit very low. May cause performance issues.".to_string(),
            });
        }

        Ok(())
    }

    fn validate_network(network: &NetworkConfig) -> Result<()> {
        // Validate network mode
        match network.mode.as_str() {
            "bridge" | "host" | "isolated" | "none" => Ok(()),
            _ => Err(CleanroomError::validation_error(
                format!("Invalid network mode: {}", network.mode)
            )),
        }
    }

    fn validate_volume(volume: &VolumeMount) -> Result<()> {
        // Validate volume type
        match volume.mount_type.as_str() {
            "bind" | "volume" | "tmpfs" => Ok(()),
            _ => Err(CleanroomError::validation_error(
                format!("Invalid volume type: {}", volume.mount_type)
            )),
        }
    }

    fn validate_security(
        security: &SecurityConfig,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        // Warn if running as root
        if !security.run_as_non_root {
            warnings.push(ValidationWarning {
                severity: "security".to_string(),
                message: "Service running as root. Consider using run_as_non_root.".to_string(),
            });
        }

        // Warn if privilege escalation allowed
        if security.allow_privilege_escalation {
            warnings.push(ValidationWarning {
                severity: "security".to_string(),
                message: "Privilege escalation allowed. This is a security risk.".to_string(),
            });
        }

        Ok(())
    }

    fn validate_dependencies(registry: &GvisorServiceRegistry) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        // Check for circular dependencies
        // Check for missing service references
        // Validate ${service:...} references in environment variables

        Ok(warnings)
    }

    fn validate_metadata(metadata: &RegistryMetadata) -> Result<()> {
        if metadata.version.is_empty() {
            return Err(CleanroomError::validation_error("Metadata version is empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub severity: String,  // warning, security, performance
    pub message: String,
}
```

### Configuration Hot-Reloader

```rust
//! Hot-reload service configurations without restarting
//!
//! File: /home/user/clnrm/crates/clnrm-core/src/config/hot_reload.rs

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::Path;

pub struct ConfigHotReloader {
    watcher: Box<dyn Watcher>,
    registry: Arc<RwLock<GvisorServiceRegistry>>,
}

impl ConfigHotReloader {
    pub fn new(config_path: &Path) -> Result<Self> {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs(2))?;
        watcher.watch(config_path, RecursiveMode::NonRecursive)?;

        // Load initial configuration
        let registry = Self::load_config(config_path)?;

        Ok(Self {
            watcher: Box::new(watcher),
            registry: Arc::new(RwLock::new(registry)),
        })
    }

    pub fn start_watching(&self, config_path: PathBuf) {
        let registry = self.registry.clone();

        std::thread::spawn(move || {
            let (tx, rx) = channel();
            let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
            watcher.watch(&config_path, RecursiveMode::NonRecursive).unwrap();

            loop {
                match rx.recv() {
                    Ok(event) => {
                        println!("🔄 Configuration file changed: {:?}", event);

                        // Reload configuration
                        match Self::load_config(&config_path) {
                            Ok(new_registry) => {
                                // Validate before applying
                                match GvisorConfigValidator::validate_registry(&new_registry) {
                                    Ok(warnings) => {
                                        let mut registry_guard = registry.write().unwrap();
                                        *registry_guard = new_registry;

                                        println!("✅ Configuration reloaded successfully");

                                        if !warnings.is_empty() {
                                            println!("⚠️  Warnings:");
                                            for warning in warnings {
                                                println!("  - [{}] {}", warning.severity, warning.message);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ Configuration validation failed: {}", e);
                                        println!("   Keeping previous configuration");
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to reload configuration: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Watch error: {:?}", e);
                    }
                }
            }
        });
    }

    fn load_config(path: &Path) -> Result<GvisorServiceRegistry> {
        let content = std::fs::read_to_string(path)?;
        let registry: GvisorServiceRegistry = toml::from_str(&content)?;
        Ok(registry)
    }

    pub fn get_registry(&self) -> Arc<RwLock<GvisorServiceRegistry>> {
        self.registry.clone()
    }
}
```

---

## Implementation Roadmap

### Week 1: Foundation
- [ ] Define gVisor configuration schema (Rust types)
- [ ] Implement configuration parser and validator
- [ ] Create service registry loader
- [ ] Set up testing infrastructure

### Week 2: Migration Tool
- [ ] Implement codebase scanner
- [ ] Build conversion engine (testcontainers → gVisor)
- [ ] Add validation logic
- [ ] Create migration report generator

### Week 3: Backwards Compatibility
- [ ] Implement hybrid backend mode
- [ ] Create configuration adapter
- [ ] Build service factory with dual-mode support
- [ ] Test backwards compatibility with existing tests

### Week 4: Service Templates
- [ ] Create SurrealDB template
- [ ] Create Alpine template
- [ ] Create custom image template
- [ ] Document template usage

### Week 5: Tooling
- [ ] Implement configuration validator CLI
- [ ] Build hot-reload system
- [ ] Create gVisor backend plugin
- [ ] Integration testing

### Week 6: Rollout
- [ ] Migrate high-priority services
- [ ] Enable gVisor by default (with fallback)
- [ ] Performance benchmarking
- [ ] Documentation and training

---

## Success Metrics

1. **Migration Coverage**: 100% of testcontainers services migrated
2. **Backwards Compatibility**: 0 breaking changes to existing tests
3. **Performance**: 10-50x improvement in container startup time
4. **Validation**: 0 invalid configurations deployed
5. **Developer Experience**: Migration automated with single command

---

## Appendix: CLI Commands

### Migration Tool

```bash
# Scan codebase
clnrm migrate scan --root /home/user/clnrm

# Convert configurations
clnrm migrate convert --input scan-results.json --output gvisor-services.toml

# Validate configurations
clnrm migrate validate --config gvisor-services.toml

# Generate report
clnrm migrate report --output migration-report.md

# Full migration (all steps)
clnrm migrate all --root /home/user/clnrm --output ./migration-output
```

### Configuration Validator

```bash
# Validate gVisor configuration
clnrm config validate --file gvisor-services.toml

# Hot-reload configuration
clnrm config watch --file gvisor-services.toml

# Lint configuration
clnrm config lint --file gvisor-services.toml --strict
```

### Service Management

```bash
# Start service with gVisor
clnrm service start --name surrealdb --backend gvisor

# List available services
clnrm service list --backend gvisor

# Show service configuration
clnrm service show --name surrealdb
```

---

**End of Design Specification**
