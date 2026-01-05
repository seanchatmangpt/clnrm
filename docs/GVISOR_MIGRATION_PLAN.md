# gVisor Migration Plan: Testcontainers to gVisor Backend

**Author:** Claude Code
**Date:** 2026-01-05
**Status:** Design Phase
**Branch:** claude/gvisor-testcontainers-replacement-7o2EO

## Executive Summary

This document outlines the comprehensive migration strategy for replacing testcontainers-rs with a native gVisor backend implementation using `runsc`. This migration will improve test isolation, reduce Docker daemon dependency, enable faster test execution, and provide better security guarantees.

## Table of Contents

1. [Current Architecture Analysis](#current-architecture-analysis)
2. [Migration Scope](#migration-scope)
3. [GVisor Backend Design](#gvisor-backend-design)
4. [Test Migration Strategy](#test-migration-strategy)
5. [Before/After Code Examples](#beforeafter-code-examples)
6. [Migration Checklist](#migration-checklist)
7. [New Test Utilities](#new-test-utilities)
8. [Parallel Execution Strategy](#parallel-execution-strategy)
9. [CI/CD Integration Changes](#cicd-integration-changes)
10. [Risk Assessment & Mitigation](#risk-assessment--mitigation)

---

## Current Architecture Analysis

### 1. Current Testcontainers Usage Patterns

#### Backend Implementation
**File:** `/home/user/clnrm/crates/clnrm-core/src/backend/testcontainer.rs`

**Key Dependencies:**
```rust
use testcontainers::{core::ExecCommand, runners::SyncRunner, GenericImage, ImageExt};
```

**Lifecycle Operations:**
- Container creation via `GenericImage::new(image_name, image_tag)`
- Container startup via `container_request.start()` (SyncRunner)
- Command execution via `container.exec(ExecCommand)`
- Automatic cleanup via Drop trait

**Resource Management:**
- Adaptive timeout based on image cache status
- Container creation locks to prevent race conditions
- Volume mounts via `testcontainers::core::Mount`
- Environment variable injection
- Port mappings

#### Service Plugins
**File:** `/home/user/clnrm/crates/clnrm-core/src/services/generic.rs`

**Key Dependencies:**
```rust
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
```

**Service Lifecycle:**
- Async container startup via `container_request.start().await`
- Port mapping discovery via `node.get_host_port_ipv4(port).await`
- Container storage in `Arc<RwLock<Option<String>>>`
- Cleanup on Drop

#### CleanroomEnvironment
**File:** `/home/user/clnrm/crates/clnrm-core/src/cleanroom.rs`

**Integration Pattern:**
```rust
backend: Arc<dyn Backend>
```

**Execution Methods:**
- `execute_in_container()` - Creates fresh container per command
- `execute_in_service()` - Routes to service container
- `start_service()` / `stop_service()` - Service lifecycle
- Telemetry integration via OpenTelemetry spans

### 2. Test File Inventory

#### Integration Tests (Root)
- `/home/user/clnrm/tests/integration/database_integration_test.rs` - Database persistence tests (simulated)
- `/home/user/clnrm/tests/integration/system_integration_test.rs` - System workflow tests (simulated)

#### Core Integration Tests
- `/home/user/clnrm/crates/clnrm-core/tests/docker_integration.rs` - **CRITICAL**: Real container telemetry validation
- `/home/user/clnrm/crates/clnrm-core/tests/docker_live_check.rs` - Service health checks
- `/home/user/clnrm/crates/clnrm-core/tests/integration_v1_3_0/e2e_basic_workflow.rs` - End-to-end workflows
- `/home/user/clnrm/crates/clnrm-core/tests/integration_v1_3_0/*.rs` - 12 integration test files

**Test Categories:**
1. **Container Execution Tests** - Basic command execution with stdout/stderr capture
2. **Service Startup Tests** - Plugin-based service lifecycle management
3. **Data Persistence Tests** - (Currently simulated with filesystem)
4. **Concurrent Execution Tests** - Parallel container isolation validation
5. **Telemetry Validation Tests** - OTLP export verification

### 3. Key Test Patterns

#### Pattern 1: Direct Container Execution
```rust
let env = CleanroomEnvironment::new().await?;
let result = env.execute_in_container(
    "container_name",
    &["echo", "hello"],
    None, // workdir
    None, // env vars
).await?;
```

#### Pattern 2: Service-Based Execution
```rust
let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");
env.register_service(Box::new(plugin)).await?;
let handle = env.start_service("alpine").await?;
let result = env.execute_in_service(&handle, &["echo", "hello"]).await?;
env.stop_service(&handle.id).await?;
```

#### Pattern 3: Telemetry Validation
```rust
let _guard = init_test_otel()?;
let result = env.execute_in_container("test", &command, None, None).await?;
let telemetry_exported = check_otlp_export_occurred().await;
assert!(telemetry_exported);
```

---

## Migration Scope

### Files Requiring Changes

#### 1. Backend Implementation (NEW)
- **Create:** `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor.rs`
  - Implement `GVisorBackend` struct
  - Implement `Backend` trait
  - OCI image support via `skopeo` or containerd
  - Port allocation via `PortAllocator`
  - Volume mount handling

#### 2. Backend Module (UPDATE)
- **Update:** `/home/user/clnrm/crates/clnrm-core/src/backend/mod.rs`
  - Add `pub mod gvisor;`
  - Export `pub use gvisor::GVisorBackend;`
  - Update `AutoBackend` to support gvisor detection

#### 3. Service Plugins (UPDATE)
- **Update:** `/home/user/clnrm/crates/clnrm-core/src/services/generic.rs`
  - Replace `testcontainers::AsyncRunner` with gVisor backend
  - Update container lifecycle to use gVisor API
  - Port mapping via gVisor port allocation

- **Update:** `/home/user/clnrm/crates/clnrm-core/src/services/surrealdb.rs`
- **Update:** `/home/user/clnrm/crates/clnrm-core/src/services/otel_collector.rs`

#### 4. CleanroomEnvironment (MINIMAL)
- **No changes needed** - Already uses `Arc<dyn Backend>` trait abstraction

#### 5. Test Files (UPDATE - ~100+ test functions)
- **Update:** All tests using `CleanroomEnvironment::new()`
  - May need to specify gvisor backend explicitly during transition
- **Update:** Service plugin tests
  - Update container ID extraction patterns
  - Update cleanup validation

#### 6. CI/CD (UPDATE)
- **Update:** `.github/workflows/*.yml` (if exists)
- **Update:** `Makefile.toml` or build scripts
  - Install `runsc` in CI environment
  - Configure gVisor runtime

---

## GVisor Backend Design

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CleanroomEnvironment                      │
│                  (No changes needed)                         │
├─────────────────────────────────────────────────────────────┤
│                    Arc<dyn Backend>                          │
└────────────────┬────────────────────────────────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
┌───▼──────────────┐   ┌─────▼──────────────┐
│ TestcontainerBE  │   │  GVisorBackend     │
│ (deprecated)     │   │  (new)             │
└──────────────────┘   └────────────────────┘
                              │
                 ┌────────────┼────────────┐
                 │            │            │
           ┌─────▼───┐  ┌────▼────┐  ┌───▼──────┐
           │ runsc   │  │ skopeo  │  │ PortAlloc│
           │ CLI     │  │ /containerd│ │ ator    │
           └─────────┘  └─────────┘  └──────────┘
```

### Core Components

#### 1. GVisorBackend Struct

```rust
pub struct GVisorBackend {
    /// OCI image reference (e.g., "docker.io/library/alpine:latest")
    image_ref: String,
    /// Policy for execution
    policy: Policy,
    /// Execution timeout
    timeout: Duration,
    /// Environment variables
    env_vars: HashMap<String, String>,
    /// Volume mounts
    volume_mounts: Vec<VolumeMount>,
    /// Port allocator for dynamic port assignment
    port_allocator: Arc<PortAllocator>,
    /// Root directory for container bundles
    bundle_root: PathBuf,
    /// Determinism engine
    determinism_engine: Option<Arc<DeterminismEngine>>,
}
```

#### 2. Backend Trait Implementation

```rust
impl Backend for GVisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        // 1. Pull OCI image to local bundle
        // 2. Create OCI runtime bundle
        // 3. Generate unique container ID
        // 4. Configure runsc with security options
        // 5. Execute command via runsc
        // 6. Capture stdout/stderr
        // 7. Clean up bundle
        // 8. Return RunResult
    }

    fn name(&self) -> &str { "gvisor" }
    fn is_available(&self) -> bool { /* Check runsc installed */ }
    fn supports_hermetic(&self) -> bool { true }
    fn supports_deterministic(&self) -> bool { true }
}
```

#### 3. Key Differences from Testcontainers

| Feature | Testcontainers | gVisor Backend |
|---------|----------------|----------------|
| **Container Creation** | `GenericImage::new()` + `start()` | `runsc create` + `runsc run` |
| **Image Source** | Docker daemon pulls | Direct OCI registry via skopeo/containerd |
| **Port Allocation** | Docker auto-allocates | Manual allocation via PortAllocator |
| **Volume Mounts** | Docker mount API | OCI bundle volume config |
| **Cleanup** | Drop trait (Docker handles) | Explicit `runsc delete` + bundle removal |
| **Networking** | Docker network | Network namespace + port forwarding |
| **Isolation** | Docker + optional gVisor | Native gVisor sandbox |

---

## Test Migration Strategy

### Phase 1: Backend Implementation (Week 1-2)

**Deliverables:**
1. `GVisorBackend` implementing `Backend` trait
2. OCI image pulling via `skopeo copy`
3. Bundle creation and configuration
4. `runsc` execution wrapper
5. Port allocation integration
6. Basic smoke tests

**Acceptance Criteria:**
- [ ] `GVisorBackend::run_cmd()` executes simple commands (echo, ls)
- [ ] Exit codes captured correctly
- [ ] Stdout/stderr captured correctly
- [ ] Cleanup verified (no orphaned containers or bundles)

### Phase 2: Service Plugin Migration (Week 3)

**Deliverables:**
1. Update `GenericContainerPlugin` to use gVisor
2. Port mapping via gVisor network config
3. Service health checks adapted

**Test Cases:**
- [ ] Start service container
- [ ] Execute command in service container
- [ ] Stop service and verify cleanup
- [ ] Multi-service concurrent startup

### Phase 3: Test Suite Migration (Week 4-5)

**Strategy:** **Parallel Run Approach**
- Keep testcontainers tests as `*_testcontainers.rs`
- Create gVisor equivalents as `*_gvisor.rs`
- Run both in CI to ensure parity
- Gradually deprecate testcontainers tests

**Migration Order:**
1. **Low-Risk Tests First:**
   - Unit tests (mock-based, no containers)
   - Simple command execution tests

2. **Medium-Risk Tests:**
   - Service lifecycle tests
   - Environment variable tests
   - Working directory tests

3. **High-Risk Tests:**
   - Concurrent execution tests
   - Telemetry validation tests
   - Performance benchmarks

### Phase 4: CI/CD Integration (Week 6)

**Deliverables:**
1. Install `runsc` in CI images
2. Configure gVisor runtime
3. Update test matrix to run both backends
4. Performance comparison dashboard

### Phase 5: Production Cutover (Week 7-8)

**Deliverables:**
1. Deprecate `TestcontainerBackend` (mark as deprecated)
2. Update documentation
3. Migration guide for users
4. Final performance validation

---

## Before/After Code Examples

### Example 1: Backend Creation

#### Before (Testcontainers)
```rust
use clnrm_core::backend::TestcontainerBackend;

let backend = TestcontainerBackend::new("alpine:latest")?
    .with_env("FOO", "bar")
    .with_timeout(Duration::from_secs(30));
```

#### After (gVisor)
```rust
use clnrm_core::backend::GVisorBackend;

let backend = GVisorBackend::new("docker.io/library/alpine:latest")?
    .with_env("FOO", "bar")
    .with_timeout(Duration::from_secs(30));
```

**Changes:**
- Image reference must be fully qualified OCI reference
- Same builder pattern maintained

### Example 2: Service Plugin

#### Before (Testcontainers)
```rust
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let image = GenericImage::new(self.image.clone(), self.tag.clone());
                let mut container_request = image.into();

                // Configure environment variables
                for (key, value) in &self.env_vars {
                    container_request = container_request.with_env_var(key, value);
                }

                // Start container
                let node = container_request.start().await?;

                // Get ports
                let host_port = node.get_host_port_ipv4(self.port).await?;

                // Store metadata
                let mut metadata = HashMap::new();
                metadata.insert("port".to_string(), host_port.to_string());

                Ok(ServiceHandle { /* ... */ })
            })
        })
    }
}
```

#### After (gVisor)
```rust
use crate::backend::{GVisorBackend, Backend, Cmd};

impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create gVisor backend for service
                let backend = GVisorBackend::new(
                    format!("docker.io/library/{}:{}", self.image, self.tag)
                )?;

                // Configure environment variables
                let mut backend = backend;
                for (key, value) in &self.env_vars {
                    backend = backend.with_env(key, value);
                }

                // Allocate port
                let allocated_port = backend.allocate_port()?;

                // Start container in background (long-running service)
                let container_id = backend.start_service(&["sleep", "3600"])?;

                // Store metadata
                let mut metadata = HashMap::new();
                metadata.insert("port".to_string(), allocated_port.to_string());
                metadata.insert("container_id".to_string(), container_id);

                Ok(ServiceHandle { /* ... */ })
            })
        })
    }
}
```

**Changes:**
- Replace `testcontainers` types with `GVisorBackend`
- Explicit port allocation
- Manual container lifecycle management
- Container ID tracking for cleanup

### Example 3: Test Execution

#### Before (Testcontainers)
```rust
#[tokio::test]
async fn test_container_execution() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;

    let result = env.execute_in_container(
        "test_container",
        &["echo".to_string(), "hello".to_string()],
        None,
        None,
    ).await?;

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    Ok(())
}
```

#### After (gVisor)
```rust
#[tokio::test]
async fn test_container_execution() -> Result<()> {
    // Option 1: Use default backend (auto-detected)
    let env = CleanroomEnvironment::new().await?;

    // Option 2: Explicitly use gVisor
    let gvisor_backend = GVisorBackend::new("docker.io/library/alpine:latest")?;
    let env = CleanroomEnvironment::with_backend(Arc::new(gvisor_backend)).await?;

    // Same test code - no changes needed!
    let result = env.execute_in_container(
        "test_container",
        &["echo".to_string(), "hello".to_string()],
        None,
        None,
    ).await?;

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    Ok(())
}
```

**Changes:**
- Test code remains identical (Backend trait abstraction!)
- Can explicitly specify gVisor backend if needed
- Behavior should be identical

### Example 4: Telemetry Validation

#### Before (Testcontainers)
```rust
#[tokio::test]
async fn test_container_exports_telemetry() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    let result = env.execute_in_container(
        "test",
        &["echo".to_string(), "test".to_string()],
        None,
        None,
    ).await?;

    assert_eq!(result.exit_code, 0);

    // Check telemetry exported
    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

#### After (gVisor)
```rust
#[tokio::test]
async fn test_container_exports_telemetry() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?; // Auto-detects gVisor

    let result = env.execute_in_container(
        "test",
        &["echo".to_string(), "test".to_string()],
        None,
        None,
    ).await?;

    assert_eq!(result.exit_code, 0);

    // Same telemetry validation - gVisor backend emits same events
    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**Changes:**
- **NONE** - Telemetry is emitted by `CleanroomEnvironment`, not the backend
- gVisor backend must ensure it sets same container.id attributes

---

## Migration Checklist

### Pre-Migration (Setup)

- [ ] **Install gVisor on development machines**
  ```bash
  # Ubuntu/Debian
  curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list > /dev/null
  sudo apt-get update && sudo apt-get install -y runsc
  ```

- [ ] **Install OCI tooling**
  ```bash
  # Install skopeo for OCI image operations
  sudo apt-get install -y skopeo

  # Or install containerd (alternative)
  sudo apt-get install -y containerd
  ```

- [ ] **Create feature flag**
  - Add to `Cargo.toml`: `gvisor = []`
  - Enable conditional compilation during transition

- [ ] **Set up test infrastructure**
  - Create test image registry (local or remote)
  - Pre-pull common test images (alpine, ubuntu, etc.)

### Phase 1: Backend Implementation

- [ ] **Create GVisorBackend struct** (`src/backend/gvisor.rs`)
  - [ ] Basic struct with image_ref, policy, timeout fields
  - [ ] Implement `new()` constructor
  - [ ] Implement builder methods (with_env, with_timeout, etc.)

- [ ] **Implement OCI image pulling**
  - [ ] `pull_image()` method using skopeo or containerd
  - [ ] Image cache management
  - [ ] Error handling for network failures

- [ ] **Implement OCI bundle creation**
  - [ ] `create_bundle()` method
  - [ ] Generate config.json (OCI runtime spec)
  - [ ] Set up rootfs directory
  - [ ] Configure namespaces (PID, network, mount, etc.)

- [ ] **Implement runsc execution**
  - [ ] `run_cmd()` implementation
  - [ ] Container ID generation
  - [ ] Execute `runsc create` + `runsc start`
  - [ ] Capture stdout/stderr via pipes
  - [ ] Capture exit code
  - [ ] Measure execution duration

- [ ] **Implement cleanup**
  - [ ] `cleanup_container()` method
  - [ ] Execute `runsc delete`
  - [ ] Remove bundle directory
  - [ ] Error handling for stuck containers

- [ ] **Implement Backend trait**
  - [ ] `run_cmd()` - Main execution method
  - [ ] `name()` - Return "gvisor"
  - [ ] `is_available()` - Check runsc installed
  - [ ] `supports_hermetic()` - Return true
  - [ ] `supports_deterministic()` - Return true

- [ ] **Write unit tests**
  - [ ] Test bundle creation
  - [ ] Test config.json generation
  - [ ] Test cleanup logic
  - [ ] Test error handling

- [ ] **Write integration tests**
  - [ ] Test simple command execution (echo)
  - [ ] Test command with arguments
  - [ ] Test command with environment variables
  - [ ] Test command with working directory
  - [ ] Test command failure handling
  - [ ] Test timeout handling

### Phase 2: Service Plugin Migration

- [ ] **Update GenericContainerPlugin**
  - [ ] Replace testcontainers imports
  - [ ] Implement start() using GVisorBackend
  - [ ] Implement stop() with explicit cleanup
  - [ ] Update health_check() logic
  - [ ] Add port allocation integration

- [ ] **Update SurrealDbPlugin**
  - [ ] Replace testcontainers_modules imports
  - [ ] Port configuration for SurrealDB
  - [ ] Health check via HTTP endpoint

- [ ] **Update OtelCollectorPlugin**
  - [ ] Port configuration (4317, 4318)
  - [ ] Volume mounts for config
  - [ ] Health check via gRPC/HTTP

- [ ] **Write service plugin tests**
  - [ ] Test service startup
  - [ ] Test command execution in service
  - [ ] Test service shutdown
  - [ ] Test concurrent services

### Phase 3: Test Suite Migration

#### Test Category 1: Container Execution Tests

- [ ] **Migrate docker_integration.rs**
  - [ ] Update test_container_execution_exports_container_id
  - [ ] Update test_container_lifecycle_telemetry
  - [ ] Update test_hermetic_isolation_exports_isolation_flag
  - [ ] Update test_container_failure_exports_error_telemetry
  - [ ] Update test_multiple_operations_export_metrics
  - [ ] Update test_container_timeout_exports_telemetry
  - [ ] Update test_service_lifecycle_exports_telemetry
  - [ ] Update test_concurrent_execution_exports_individual_telemetry
  - [ ] Update test_env_var_propagation_exports_telemetry
  - [ ] Update test_container_reuse_stats_telemetry
  - [ ] Update test_complete_workflow_weaver_ready
  - [ ] Update test_telemetry_performance_overhead

#### Test Category 2: Integration Tests

- [ ] **Migrate integration_v1_3_0/e2e_basic_workflow.rs**
  - [ ] test_basic_workflow_single_container
  - [ ] test_basic_workflow_with_environment_variables
  - [ ] test_basic_workflow_with_multiple_steps
  - [ ] test_basic_workflow_command_failure_handling
  - [ ] test_basic_workflow_with_workdir
  - [ ] test_basic_workflow_cleanup_on_error
  - [ ] test_basic_workflow_stdout_stderr_capture

- [ ] **Migrate integration_v1_3_0/e2e_multi_service.rs**
- [ ] **Migrate integration_v1_3_0/feature_*.rs**
- [ ] **Migrate integration_v1_3_0/perf_*.rs**
- [ ] **Migrate integration_v1_3_0/security_*.rs**

#### Test Category 3: Service Tests

- [ ] **Migrate cli_functional/services/*.rs**
  - [ ] collector_test.rs
  - [ ] health_test.rs
  - [ ] services_test.rs
  - [ ] plugins_test.rs

#### Test Category 4: Performance Tests

- [ ] **Migrate container_reuse_benchmark.rs**
- [ ] **Migrate performance_failfast.rs**
- [ ] **Benchmark gVisor vs testcontainers**
  - [ ] Container startup time
  - [ ] Command execution latency
  - [ ] Cleanup time
  - [ ] Memory usage
  - [ ] CPU usage

### Phase 4: CI/CD Integration

- [ ] **Update CI Docker images**
  - [ ] Install runsc in CI base image
  - [ ] Install skopeo/containerd
  - [ ] Pre-pull common test images

- [ ] **Update GitHub Actions workflows**
  - [ ] Add gVisor installation step
  - [ ] Configure gVisor runtime
  - [ ] Run both testcontainers and gVisor tests in parallel

- [ ] **Update Makefile.toml**
  - [ ] Add `install-gvisor` task
  - [ ] Add `test-gvisor` task
  - [ ] Add `bench-gvisor` task

- [ ] **Create performance dashboard**
  - [ ] Track test execution time
  - [ ] Track container creation time
  - [ ] Track memory usage
  - [ ] Compare gVisor vs testcontainers

### Phase 5: Documentation & Cleanup

- [ ] **Update documentation**
  - [ ] README.md - Add gVisor backend section
  - [ ] ARCHITECTURE.md - Document gVisor integration
  - [ ] CONTRIBUTING.md - Add gVisor development setup

- [ ] **Migration guide**
  - [ ] User-facing migration guide
  - [ ] Breaking changes (if any)
  - [ ] Rollback procedure

- [ ] **Deprecate testcontainers**
  - [ ] Mark TestcontainerBackend as deprecated
  - [ ] Add deprecation warnings
  - [ ] Plan for removal in future version

- [ ] **Final validation**
  - [ ] All tests passing with gVisor
  - [ ] Performance metrics acceptable
  - [ ] No regressions in test coverage
  - [ ] CI/CD stable

---

## New Test Utilities

### 1. GVisor-Specific Test Helpers

```rust
// File: crates/clnrm-core/src/testing/gvisor_helpers.rs

/// Check if gVisor (runsc) is available on the system
pub fn is_gvisor_available() -> bool {
    std::process::Command::new("runsc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Skip test if gVisor is not available (macro)
#[macro_export]
macro_rules! skip_if_no_gvisor {
    () => {
        if !$crate::testing::gvisor_helpers::is_gvisor_available() {
            eprintln!("gVisor (runsc) not available, skipping test");
            return Ok(());
        }
    };
}

/// Create a test environment with gVisor backend
pub async fn create_gvisor_test_env() -> Result<CleanroomEnvironment> {
    let backend = GVisorBackend::new("docker.io/library/alpine:latest")?;
    CleanroomEnvironment::with_backend(Arc::new(backend)).await
}

/// Pull OCI image for testing (idempotent)
pub async fn ensure_test_image_pulled(image: &str) -> Result<()> {
    // Check if image already pulled
    if check_image_cached(image)? {
        return Ok(());
    }

    // Pull image via skopeo
    let output = tokio::process::Command::new("skopeo")
        .args(&["copy", &format!("docker://{}", image), &format!("oci:{}", image)])
        .output()
        .await?;

    if !output.status.success() {
        return Err(CleanroomError::container_error(
            format!("Failed to pull image {}: {}", image, String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}
```

### 2. Backend Comparison Test Utilities

```rust
// File: crates/clnrm-core/src/testing/backend_comparison.rs

/// Run same test with both backends and compare results
pub async fn compare_backends<F, T>(
    test_name: &str,
    test_fn: F,
) -> Result<BackendComparison<T>>
where
    F: Fn(Arc<dyn Backend>) -> Pin<Box<dyn Future<Output = Result<T>>>>,
    T: PartialEq + std::fmt::Debug,
{
    // Run with testcontainers
    let tc_backend = Arc::new(TestcontainerBackend::new("alpine:latest")?);
    let tc_result = test_fn(tc_backend).await;

    // Run with gVisor
    let gvisor_backend = Arc::new(GVisorBackend::new("docker.io/library/alpine:latest")?);
    let gvisor_result = test_fn(gvisor_backend).await;

    Ok(BackendComparison {
        test_name: test_name.to_string(),
        testcontainers_result: tc_result,
        gvisor_result,
    })
}

pub struct BackendComparison<T> {
    pub test_name: String,
    pub testcontainers_result: Result<T>,
    pub gvisor_result: Result<T>,
}

impl<T: PartialEq + std::fmt::Debug> BackendComparison<T> {
    /// Assert both backends produced same result
    pub fn assert_equivalent(&self) -> Result<()> {
        match (&self.testcontainers_result, &self.gvisor_result) {
            (Ok(tc), Ok(gv)) => {
                assert_eq!(tc, gv, "Backend results differ for test {}", self.test_name);
                Ok(())
            }
            (Err(tc_err), Err(gv_err)) => {
                // Both failed - check error types match
                assert_eq!(
                    std::mem::discriminant(tc_err),
                    std::mem::discriminant(gv_err),
                    "Backend errors differ for test {}: TC={:?}, GV={:?}",
                    self.test_name, tc_err, gv_err
                );
                Ok(())
            }
            _ => {
                panic!("One backend succeeded, other failed for test {}", self.test_name);
            }
        }
    }
}
```

### 3. Parallel Execution Test Utilities

```rust
// File: crates/clnrm-core/src/testing/parallel_execution.rs

/// Execute test with both backends in parallel
pub async fn parallel_backend_test<F, T>(
    test_fn: F,
) -> Result<(Result<T>, Result<T>)>
where
    F: Fn(Arc<dyn Backend>) -> Pin<Box<dyn Future<Output = Result<T>>>> + Send + Clone + 'static,
    T: Send + 'static,
{
    let tc_backend = Arc::new(TestcontainerBackend::new("alpine:latest")?);
    let gvisor_backend = Arc::new(GVisorBackend::new("docker.io/library/alpine:latest")?);

    let tc_test = test_fn.clone();
    let gv_test = test_fn;

    let (tc_result, gv_result) = tokio::join!(
        tokio::spawn(async move { tc_test(tc_backend).await }),
        tokio::spawn(async move { gv_test(gvisor_backend).await }),
    );

    Ok((
        tc_result.map_err(|e| CleanroomError::internal_error(e.to_string()))?,
        gv_result.map_err(|e| CleanroomError::internal_error(e.to_string()))?,
    ))
}
```

---

## Parallel Execution Strategy

### Dual-Backend Test Matrix

During migration, run both backends in parallel to ensure parity:

```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    backend: [testcontainers, gvisor]
    os: [ubuntu-latest, ubuntu-22.04]

steps:
  - name: Install backend dependencies
    run: |
      if [ "${{ matrix.backend }}" = "gvisor" ]; then
        # Install gVisor
        curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
        sudo apt-get update && sudo apt-get install -y runsc skopeo
      fi

  - name: Run tests
    run: |
      if [ "${{ matrix.backend }}" = "gvisor" ]; then
        cargo test --features gvisor
      else
        cargo test
      fi
```

### Test Isolation Strategy

1. **Namespace Isolation**
   - Each test gets unique container ID
   - Unique bundle directory per test
   - No shared state between tests

2. **Resource Limits**
   - CPU limits via cgroups
   - Memory limits via cgroups
   - Disk I/O limits

3. **Cleanup Guarantees**
   - Explicit cleanup in test teardown
   - Panic handlers to ensure cleanup
   - Timeout-based cleanup for stuck containers

### Concurrent Test Execution

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_gvisor_execution() -> Result<()> {
    let tasks: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let env = create_gvisor_test_env().await?;
                env.execute_in_container(
                    &format!("test_{}", i),
                    &["echo", &format!("task_{}", i)],
                    None,
                    None,
                ).await
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All should succeed
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {} failed", i);
    }

    Ok(())
}
```

---

## CI/CD Integration Changes

### 1. CI Environment Setup

#### Dockerfile for CI Base Image

```dockerfile
# .ci/Dockerfile.gvisor
FROM ubuntu:22.04

# Install base dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install gVisor
RUN curl -fsSL https://gvisor.dev/archive.key | \
    gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | \
    tee /etc/apt/sources.list.d/gvisor.list && \
    apt-get update && \
    apt-get install -y runsc

# Install skopeo for OCI image operations
RUN apt-get update && apt-get install -y skopeo && \
    rm -rf /var/lib/apt/lists/*

# Verify installation
RUN runsc --version && skopeo --version

# Pre-pull common test images
RUN skopeo copy docker://alpine:latest oci:/var/lib/test-images/alpine:latest && \
    skopeo copy docker://ubuntu:22.04 oci:/var/lib/test-images/ubuntu:22.04
```

### 2. GitHub Actions Workflow

```yaml
# .github/workflows/test-gvisor.yml
name: Test gVisor Backend

on:
  push:
    branches: [main, develop, claude/*]
  pull_request:

jobs:
  test-gvisor:
    name: Test with gVisor Backend
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install gVisor
        run: |
          curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
          echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
          sudo apt-get update
          sudo apt-get install -y runsc skopeo

      - name: Verify gVisor Installation
        run: |
          runsc --version
          skopeo --version

      - name: Cache Cargo Registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache Cargo Build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Pre-pull Test Images
        run: |
          mkdir -p /tmp/test-images
          skopeo copy docker://alpine:latest oci:/tmp/test-images/alpine:latest
          skopeo copy docker://ubuntu:22.04 oci:/tmp/test-images/ubuntu:22.04

      - name: Run Tests (gVisor Backend)
        run: cargo test --features gvisor --no-fail-fast
        env:
          RUST_BACKTRACE: 1
          RUST_LOG: debug
          GVISOR_TEST_IMAGES: /tmp/test-images

      - name: Run Integration Tests
        run: cargo test --features gvisor --test integration_*

      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results-gvisor
          path: target/debug/test-results/

  compare-backends:
    name: Compare Testcontainers vs gVisor
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Dependencies
        run: |
          # Install Docker (for testcontainers)
          sudo apt-get update
          sudo apt-get install -y docker.io

          # Install gVisor
          curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
          echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
          sudo apt-get update
          sudo apt-get install -y runsc skopeo

      - name: Run Performance Comparison
        run: |
          # Run testcontainers benchmarks
          cargo bench --features testcontainers --bench container_benchmarks -- --save-baseline testcontainers

          # Run gVisor benchmarks
          cargo bench --features gvisor --bench container_benchmarks -- --save-baseline gvisor

          # Compare results
          cargo bench --features gvisor --bench container_benchmarks -- --baseline testcontainers

      - name: Upload Benchmark Results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-comparison
          path: target/criterion/
```

### 3. Test Execution Script

```bash
#!/bin/bash
# scripts/test-gvisor.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== gVisor Test Suite ===${NC}"

# Check gVisor installation
echo -e "${YELLOW}Checking gVisor installation...${NC}"
if ! command -v runsc &> /dev/null; then
    echo -e "${RED}Error: runsc not found. Please install gVisor.${NC}"
    exit 1
fi

if ! command -v skopeo &> /dev/null; then
    echo -e "${RED}Error: skopeo not found. Please install skopeo.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ gVisor and dependencies installed${NC}"

# Pre-pull test images
echo -e "${YELLOW}Pre-pulling test images...${NC}"
mkdir -p /tmp/clnrm-test-images
skopeo copy docker://alpine:latest oci:/tmp/clnrm-test-images/alpine:latest || true
skopeo copy docker://ubuntu:22.04 oci:/tmp/clnrm-test-images/ubuntu:22.04 || true
echo -e "${GREEN}✓ Test images ready${NC}"

# Run tests
echo -e "${YELLOW}Running gVisor backend tests...${NC}"
export GVISOR_TEST_IMAGES=/tmp/clnrm-test-images
export RUST_BACKTRACE=1

cargo test --features gvisor --no-fail-fast "$@"

echo -e "${GREEN}✓ All tests passed${NC}"
```

---

## Risk Assessment & Mitigation

### High-Risk Areas

#### 1. **Port Allocation Conflicts**

**Risk:** gVisor doesn't auto-allocate ports like Docker
**Impact:** Service tests may fail due to port conflicts
**Mitigation:**
- Implement robust `PortAllocator` with port range management
- Track allocated ports per test session
- Use ephemeral port range (49152-65535)
- Add port cleanup on test failure

#### 2. **Image Pulling Failures**

**Risk:** OCI image pulling may fail due to network issues
**Impact:** Tests become flaky in CI
**Mitigation:**
- Pre-pull images in CI setup phase
- Cache images locally in CI environment
- Implement retry logic with exponential backoff
- Provide clear error messages for image pull failures

#### 3. **Container Cleanup Failures**

**Risk:** Orphaned containers/bundles consuming disk space
**Impact:** CI runners run out of disk space
**Mitigation:**
- Explicit cleanup in Drop trait
- Panic handlers to ensure cleanup
- Background cleanup task to find orphaned containers
- Disk space monitoring in CI

#### 4. **Performance Regression**

**Risk:** gVisor may be slower than testcontainers for some operations
**Impact:** Tests take longer to run
**Mitigation:**
- Benchmark before migration to establish baseline
- Optimize bundle creation (reuse where possible)
- Parallel test execution
- Accept slight slowdown for better isolation (if needed)

#### 5. **Telemetry Attribute Drift**

**Risk:** gVisor backend may not emit same telemetry attributes
**Impact:** Weaver validation tests fail
**Mitigation:**
- Maintain exact attribute parity with testcontainers
- Integration tests to validate telemetry schemas
- Document all required attributes in gVisor backend

### Medium-Risk Areas

#### 6. **OCI Spec Compatibility**

**Risk:** Some container features may not work with gVisor
**Impact:** Tests using advanced features may fail
**Mitigation:**
- Review OCI spec compatibility matrix for gVisor
- Document unsupported features
- Provide fallback mechanisms

#### 7. **Concurrent Test Isolation**

**Risk:** Concurrent tests may interfere with each other
**Impact:** Flaky test failures
**Mitigation:**
- Unique bundle directories per test
- Unique container IDs
- Resource limits per container
- Test isolation validation suite

### Low-Risk Areas

#### 8. **Documentation Gaps**

**Risk:** Developers don't know how to use gVisor backend
**Impact:** Slow adoption, support burden
**Mitigation:**
- Comprehensive migration guide
- Code examples
- FAQ section
- Video walkthrough

---

## Success Criteria

### Functional Requirements

- [ ] All existing tests pass with gVisor backend
- [ ] No regressions in test coverage
- [ ] Telemetry parity (same spans, attributes, metrics)
- [ ] Service plugins work with gVisor
- [ ] Concurrent test execution supported

### Performance Requirements

- [ ] Test execution time within 20% of testcontainers baseline
- [ ] Container startup time < 1 second for cached images
- [ ] Memory usage reasonable (< 2x testcontainers)
- [ ] Disk space usage reasonable (cleanup works)

### Quality Requirements

- [ ] Zero orphaned containers after tests
- [ ] Zero bundle directory leaks
- [ ] Proper error messages for failures
- [ ] CI stability (no flaky tests)

### Documentation Requirements

- [ ] Migration guide complete
- [ ] Architecture documentation updated
- [ ] API documentation complete
- [ ] Examples working

---

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **Phase 1: Backend Implementation** | Week 1-2 | GVisorBackend with Backend trait |
| **Phase 2: Service Plugins** | Week 3 | Updated service plugins using gVisor |
| **Phase 3: Test Migration** | Week 4-5 | All tests migrated and passing |
| **Phase 4: CI/CD Integration** | Week 6 | CI running both backends in parallel |
| **Phase 5: Production Cutover** | Week 7-8 | Deprecate testcontainers, documentation |

**Total Duration:** 8 weeks

---

## Appendix

### A. gVisor Architecture Overview

gVisor provides application kernel for containers, running as a normal, unprivileged process:

```
┌─────────────────────────────────────────┐
│         User Application                │
├─────────────────────────────────────────┤
│         gVisor Sandbox (Sentry)         │  ← Application Kernel
├─────────────────────────────────────────┤
│         gVisor Platform (KVM/ptrace)    │  ← Secure Execution
├─────────────────────────────────────────┤
│         Host Linux Kernel               │
└─────────────────────────────────────────┘
```

**Key Benefits:**
- **Strong Isolation**: Containers cannot escape to host kernel
- **Reduced Attack Surface**: Limited syscall interface
- **Defense in Depth**: Even if container compromised, limited access to host
- **No Docker Daemon**: Direct runsc execution

### B. runsc CLI Reference

```bash
# Create container
runsc create --bundle /path/to/bundle container-id

# Start container
runsc start container-id

# Execute command
runsc exec container-id /bin/sh -c "echo hello"

# List containers
runsc list

# Kill container
runsc kill container-id

# Delete container
runsc delete container-id

# Get container state
runsc state container-id
```

### C. OCI Bundle Structure

```
/path/to/bundle/
├── config.json          # OCI runtime spec
└── rootfs/              # Container filesystem
    ├── bin/
    ├── etc/
    ├── lib/
    ├── proc/
    └── ...
```

**config.json Example:**
```json
{
  "ociVersion": "1.0.0",
  "process": {
    "terminal": false,
    "user": { "uid": 0, "gid": 0 },
    "args": ["sh", "-c", "echo hello"],
    "env": ["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"],
    "cwd": "/"
  },
  "root": {
    "path": "rootfs",
    "readonly": false
  },
  "mounts": [
    { "destination": "/proc", "type": "proc", "source": "proc" },
    { "destination": "/dev", "type": "tmpfs", "source": "tmpfs" }
  ],
  "linux": {
    "namespaces": [
      { "type": "pid" },
      { "type": "network" },
      { "type": "ipc" },
      { "type": "uts" },
      { "type": "mount" }
    ]
  }
}
```

### D. Port Allocator Design

```rust
pub struct PortAllocator {
    /// Range of ports to allocate from (ephemeral range)
    port_range: Range<u16>,
    /// Currently allocated ports
    allocated: Arc<RwLock<HashSet<u16>>>,
}

impl PortAllocator {
    pub fn new() -> Self {
        Self {
            // Use ephemeral port range to avoid conflicts
            port_range: 49152..65535,
            allocated: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn allocate(&self) -> Result<u16> {
        let mut allocated = self.allocated.write().await;

        // Find first available port
        for port in self.port_range.clone() {
            if !allocated.contains(&port) {
                allocated.insert(port);
                return Ok(port);
            }
        }

        Err(CleanroomError::resource_exhausted("No ports available"))
    }

    pub async fn release(&self, port: u16) {
        let mut allocated = self.allocated.write().await;
        allocated.remove(&port);
    }
}
```

### E. References

- [gVisor Documentation](https://gvisor.dev/docs/)
- [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec)
- [runsc CLI Reference](https://gvisor.dev/docs/user_guide/quick_start/)
- [Skopeo Documentation](https://github.com/containers/skopeo)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)

---

**End of Migration Plan**
