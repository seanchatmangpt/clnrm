# gVisor Test Migration Guide (v3.0.0)

**Companion to:** GVISOR_MIGRATION_PLAN.md
**Status:** gVisor is now the default and only production backend. Testcontainers is legacy-only.

## Overview

Starting with v3.0, clnrm has migrated to a gVisor-first architecture. This guide provides detailed test-by-test migration instructions for moving from legacy Testcontainers-based tests to the gVisor-native backend.

## Quick Reference

| Test Pattern | Migration Complexity | Changes Required |
|--------------|---------------------|------------------|
| **Basic Container Execution** | ✅ Low | None (Backend trait abstraction) |
| **Service Plugin Tests** | ⚠️ Medium | Update plugin initialization |
| **Port Mapping Tests** | ⚠️ Medium | Explicit port allocation |
| **Volume Mount Tests** | ⚠️ Medium | OCI bundle volume config |
| **Telemetry Tests** | ✅ Low | None (CleanroomEnvironment handles) |
| **Concurrent Execution** | ⚠️ Medium | Ensure unique container IDs |
| **Performance Benchmarks** | 🔴 High | New baseline measurements |

## Migration Patterns

### Pattern 1: Basic Container Execution (NO CHANGES)

#### File: `crates/clnrm-core/tests/docker_integration.rs`

**Test Function:** `test_container_execution_exports_container_id`

**Before (Legacy Testcontainers):**
```rust
#[tokio::test]
async fn test_container_execution_exports_container_id() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    let result = env
        .execute_in_container(
            "test_container_exec",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.stdout.contains("test"));
    assert_eq!(result.exit_code, 0);

    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**After (gVisor):**
```rust
#[tokio::test]
async fn test_container_execution_exports_container_id() -> Result<()> {
    // MIGRATION: No changes needed!
    // CleanroomEnvironment::new() will auto-detect gVisor backend
    // when TestcontainerBackend is not available or gvisor feature flag is set

    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    let result = env
        .execute_in_container(
            "test_container_exec",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.stdout.contains("test"));
    assert_eq!(result.exit_code, 0);

    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**Migration Steps:**
1. ✅ No code changes
2. ✅ Run test with gVisor: `cargo test --features gvisor test_container_execution_exports_container_id`
3. ✅ Verify same behavior

---

### Pattern 2: Explicit Backend Selection (OPTIONAL CHANGE)

**When to use:** When you want to explicitly test gVisor backend during transition period

**Before (Auto-detection):**
```rust
let env = CleanroomEnvironment::new().await?;
```

**After (Explicit gVisor):**
```rust
use clnrm_core::backend::GVisorBackend;

let backend = Arc::new(GVisorBackend::new("docker.io/library/alpine:latest")?);
let env = CleanroomEnvironment::with_backend(backend).await?;
```

**Migration Steps:**
1. Import `GVisorBackend`
2. Create explicit backend instance
3. Use `CleanroomEnvironment::with_backend()`
4. Rest of test unchanged

---

### Pattern 3: Service Plugin Tests (MEDIUM CHANGES)

#### File: `crates/clnrm-core/tests/integration_v1_3_0/e2e_basic_workflow.rs`

**Test Function:** `test_basic_workflow_single_container`

**Before (Legacy Testcontainers):**
```rust
#[tokio::test]
async fn test_basic_workflow_single_container() -> Result<()> {
    let env = create_test_environment().await?;

    // Register generic container plugin
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Start service
    let handle = env.start_service("alpine").await?;

    // Execute command
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "hello".to_string()], None, None)
        .await?;

    assert!(result.succeeded());
    assert!(result.stdout.contains("hello"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}
```

**After (gVisor):**
```rust
#[tokio::test]
async fn test_basic_workflow_single_container() -> Result<()> {
    let env = create_test_environment().await?;

    // MIGRATION: Plugin creation unchanged - GenericContainerPlugin
    // will use gVisor backend internally
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Start service - same API
    let handle = env.start_service("alpine").await?;

    // Execute command - same API
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "hello".to_string()], None, None)
        .await?;

    assert!(result.succeeded());
    assert!(result.stdout.contains("hello"));

    // Cleanup - same API
    env.stop_service(&handle.id).await?;
    Ok(())
}
```

**Migration Steps:**
1. ✅ No test code changes
2. ⚠️ Update `GenericContainerPlugin` implementation (see Pattern 6)
3. ✅ Run test with gVisor
4. ✅ Verify same behavior

---

### Pattern 4: Concurrent Execution Tests (LOW CHANGES)

#### File: `crates/clnrm-core/tests/docker_integration.rs`

**Test Function:** `test_concurrent_execution_exports_individual_telemetry`

**Before (Legacy Testcontainers):**
```rust
#[tokio::test]
async fn test_concurrent_execution_exports_individual_telemetry() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    let tasks: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                let env = CleanroomEnvironment::new().await.unwrap();
                let container_name = format!("test_concurrent_{}", i);
                let command = vec!["echo".to_string(), format!("task_{}", i)];
                env.execute_in_container(&container_name, &command, None, None)
                    .await
            })
        })
        .collect();

    let results = futures_util::future::join_all(tasks).await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(exec_result) => match exec_result {
                Ok(res) => assert_eq!(res.exit_code, 0),
                Err(e) => panic!("Task {} failed: {}", i, e),
            },
            Err(e) => panic!("Task {} panicked: {}", i, e),
        }
    }

    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**After (gVisor):**
```rust
#[tokio::test]
async fn test_concurrent_execution_exports_individual_telemetry() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    // MIGRATION: Same code - gVisor backend ensures unique container IDs
    // via UUID generation and separate bundle directories
    let tasks: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                let env = CleanroomEnvironment::new().await.unwrap();
                let container_name = format!("test_concurrent_{}", i);
                let command = vec!["echo".to_string(), format!("task_{}", i)];
                env.execute_in_container(&container_name, &command, None, None)
                    .await
            })
        })
        .collect();

    let results = futures_util::future::join_all(tasks).await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(exec_result) => match exec_result {
                Ok(res) => assert_eq!(res.exit_code, 0),
                Err(e) => panic!("Task {} failed: {}", i, e),
            },
            Err(e) => panic!("Task {} panicked: {}", i, e),
        }
    }

    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**Migration Steps:**
1. ✅ No code changes
2. ✅ gVisor backend ensures isolation via unique bundle directories
3. ⚠️ Monitor for resource exhaustion (max concurrent containers)
4. ✅ Run test with gVisor
5. ✅ Verify concurrent isolation

---

### Pattern 5: Environment Variable Tests (NO CHANGES)

#### File: `crates/clnrm-core/tests/docker_integration.rs`

**Test Function:** `test_env_var_propagation_exports_telemetry`

**Before (Legacy Testcontainers):**
```rust
#[tokio::test]
async fn test_env_var_propagation_exports_telemetry() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    let container_name = "test_env_var";
    let command = vec![
        "sh".to_string(),
        "-c".to_string(),
        "echo $TEST_VAR".to_string(),
    ];

    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    let result = env
        .execute_in_container(container_name, &command, None, Some(&env_vars))
        .await?;

    assert_eq!(result.exit_code, 0);

    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**After (gVisor):**
```rust
#[tokio::test]
async fn test_env_var_propagation_exports_telemetry() -> Result<()> {
    let _guard = init_test_otel()?;
    let env = CleanroomEnvironment::new().await?;

    let container_name = "test_env_var";
    let command = vec![
        "sh".to_string(),
        "-c".to_string(),
        "echo $TEST_VAR".to_string(),
    ];

    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    // MIGRATION: No changes - gVisor sets env vars in config.json
    let result = env
        .execute_in_container(container_name, &command, None, Some(&env_vars))
        .await?;

    assert_eq!(result.exit_code, 0);

    let telemetry_exported = check_otlp_export_occurred().await;
    assert!(telemetry_exported);

    Ok(())
}
```

**Migration Steps:**
1. ✅ No code changes
2. ✅ gVisor backend writes env vars to OCI config.json
3. ✅ Run test with gVisor
4. ✅ Verify env vars propagated

---

### Pattern 6: Service Plugin Implementation (HIGH CHANGES)

#### File: `crates/clnrm-core/src/services/generic.rs`

**Before (Legacy Testcontainers):**
```rust
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let image = GenericImage::new(self.image.clone(), self.tag.clone());
                let mut container_request = image.into();

                for (key, value) in &self.env_vars {
                    container_request = container_request.with_env_var(key, value);
                }

                for port in &self.ports {
                    container_request = container_request
                        .with_mapped_port(*port, testcontainers::core::ContainerPort::Tcp(*port));
                }

                let node = container_request.start().await?;

                let container_id = format!("generic-{}", Uuid::new_v4());

                let mut metadata = HashMap::new();
                metadata.insert("image".to_string(), format!("{}:{}", self.image, self.tag));
                metadata.insert("container_id".to_string(), container_id.clone());

                for port in &self.ports {
                    if let Ok(host_port) = node.get_host_port_ipv4(*port).await {
                        metadata.insert(format!("port_{}", port), host_port.to_string());
                    }
                }

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }
}
```

**After (gVisor):**
```rust
use crate::backend::{GVisorBackend, Backend, Cmd};

impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create gVisor backend for this service
                let image_ref = format!("docker.io/library/{}:{}", self.image, self.tag);
                let mut backend = GVisorBackend::new(&image_ref)?;

                // Configure environment variables
                for (key, value) in &self.env_vars {
                    backend = backend.with_env(key, value);
                }

                // Allocate ports
                let port_allocator = backend.port_allocator();
                let mut allocated_ports = HashMap::new();

                for container_port in &self.ports {
                    let host_port = port_allocator.allocate().await?;
                    allocated_ports.insert(*container_port, host_port);
                }

                // Start long-running service container
                // gVisor backend needs to support background services
                let container_id = backend.start_service(&["sleep", "3600"]).await?;

                // Build metadata
                let mut metadata = HashMap::new();
                metadata.insert("image".to_string(), image_ref);
                metadata.insert("container_id".to_string(), container_id.clone());

                for (container_port, host_port) in allocated_ports {
                    metadata.insert(format!("port_{}", container_port), host_port.to_string());
                }

                // Store backend reference for future exec commands
                let mut container_guard = self.container_id.write().await;
                *container_guard = Some(container_id);

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut container_guard = self.container_id.write().await;
                if let Some(container_id) = container_guard.take() {
                    // Explicitly stop and clean up gVisor container
                    let _ = GVisorBackend::stop_container(&container_id).await;
                    let _ = GVisorBackend::cleanup_bundle(&container_id).await;
                }
                Ok(())
            })
        })
    }
}
```

**Migration Steps:**
1. Replace `testcontainers` imports with `GVisorBackend`
2. Create `GVisorBackend` instance instead of `GenericImage`
3. Implement port allocation via `PortAllocator`
4. Call `backend.start_service()` for long-running containers
5. Store container_id for cleanup
6. Implement explicit cleanup in `stop()`

**New GVisorBackend Methods Needed:**
```rust
impl GVisorBackend {
    /// Start a long-running service container (doesn't wait for completion)
    pub async fn start_service(&self, command: &[&str]) -> Result<String> {
        // 1. Create OCI bundle
        // 2. Generate container ID
        // 3. Start container with `runsc create` + `runsc start`
        // 4. Return container ID for future exec/stop
    }

    /// Stop a running container
    pub async fn stop_container(container_id: &str) -> Result<()> {
        // Execute `runsc kill container-id`
    }

    /// Clean up container bundle
    pub async fn cleanup_bundle(container_id: &str) -> Result<()> {
        // Execute `runsc delete container-id`
        // Remove bundle directory
    }
}
```

---

### Pattern 7: Volume Mount Tests (MEDIUM CHANGES)

#### Test Function: `test_volume_mount_read_only`

**Before (Legacy Testcontainers):**
```rust
#[tokio::test]
async fn test_volume_mount_read_only() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "readonly content")?;

    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume_ro(
            temp_dir.path().to_str().unwrap(),
            "/mnt/data",
        )?;

    let env = CleanroomEnvironment::with_backend(Arc::new(backend)).await?;

    let result = env
        .execute_in_container(
            "test",
            &["cat".to_string(), "/mnt/data/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.stdout.contains("readonly content"));

    // Try to write (should fail)
    let write_result = env
        .execute_in_container(
            "test",
            &["sh".to_string(), "-c".to_string(), "echo 'write' > /mnt/data/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    assert_ne!(write_result.exit_code, 0); // Should fail

    Ok(())
}
```

**After (gVisor):**
```rust
#[tokio::test]
async fn test_volume_mount_read_only() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "readonly content")?;

    // MIGRATION: Same API - gVisor sets volume config in OCI bundle
    let backend = GVisorBackend::new("docker.io/library/alpine:latest")?
        .with_volume_ro(
            temp_dir.path().to_str().unwrap(),
            "/mnt/data",
        )?;

    let env = CleanroomEnvironment::with_backend(Arc::new(backend)).await?;

    let result = env
        .execute_in_container(
            "test",
            &["cat".to_string(), "/mnt/data/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.stdout.contains("readonly content"));

    // Try to write (should fail)
    let write_result = env
        .execute_in_container(
            "test",
            &["sh".to_string(), "-c".to_string(), "echo 'write' > /mnt/data/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    assert_ne!(write_result.exit_code, 0); // Should fail

    Ok(())
}
```

**Migration Steps:**
1. ✅ Same test code
2. ✅ gVisor backend configures mounts in OCI config.json:
   ```json
   {
     "mounts": [
       {
         "destination": "/mnt/data",
         "source": "/host/path",
         "type": "bind",
         "options": ["rbind", "ro"]
       }
     ]
   }
   ```
3. ✅ Run test with gVisor
4. ✅ Verify read-only enforcement

---

## Test-by-Test Migration Tracker

### docker_integration.rs (12 tests)

| Test Function | Status | Changes Required | Notes |
|--------------|--------|------------------|-------|
| `test_container_execution_exports_container_id` | ✅ Ready | None | Backend abstraction works |
| `test_container_lifecycle_telemetry` | ✅ Ready | None | Backend abstraction works |
| `test_hermetic_isolation_exports_isolation_flag` | ✅ Ready | None | gVisor provides better isolation |
| `test_container_failure_exports_error_telemetry` | ✅ Ready | None | Exit codes captured same way |
| `test_multiple_operations_export_metrics` | ✅ Ready | None | Metrics collected by CleanroomEnvironment |
| `test_container_timeout_exports_telemetry` | ⚠️ Review | Timeout impl | Verify timeout behavior |
| `test_service_lifecycle_exports_telemetry` | ⚠️ Update | Service plugin | Requires GenericContainerPlugin update |
| `test_concurrent_execution_exports_individual_telemetry` | ✅ Ready | None | Unique bundle directories ensure isolation |
| `test_env_var_propagation_exports_telemetry` | ✅ Ready | None | Env vars in config.json |
| `test_container_reuse_stats_telemetry` | ⚠️ Review | Pool impl | May need gVisor pool implementation |
| `test_complete_workflow_weaver_ready` | ⚠️ Update | Service plugin | End-to-end test |
| `test_telemetry_performance_overhead` | 🔴 Baseline | Benchmark | Need new performance baseline |

### integration_v1_3_0/e2e_basic_workflow.rs (8 tests)

| Test Function | Status | Changes Required | Notes |
|--------------|--------|------------------|-------|
| `test_basic_workflow_single_container` | ⚠️ Update | Service plugin | GenericContainerPlugin update |
| `test_basic_workflow_with_environment_variables` | ⚠️ Update | Service plugin | GenericContainerPlugin update |
| `test_basic_workflow_with_multiple_steps` | ⚠️ Update | Service plugin | GenericContainerPlugin update |
| `test_basic_workflow_command_failure_handling` | ⚠️ Update | Service plugin | GenericContainerPlugin update |
| `test_basic_workflow_with_workdir` | ✅ Ready | None | Workdir in config.json |
| `test_basic_workflow_cleanup_on_error` | ⚠️ Review | Cleanup | Verify explicit cleanup works |
| `test_basic_workflow_stdout_stderr_capture` | ✅ Ready | None | Pipe capture works same |

### integration_v1_3_0/e2e_multi_service.rs

| Test Function | Status | Changes Required | Notes |
|--------------|--------|------------------|-------|
| `test_multi_service_startup` | ⚠️ Update | Service plugins | Multiple services concurrently |
| `test_service_communication` | ⚠️ Update | Port mapping | Network namespaces |
| `test_service_dependencies` | ⚠️ Update | Service order | Startup ordering |

### Performance Benchmarks

| Benchmark | Status | Changes Required | Notes |
|-----------|--------|------------------|-------|
| `container_reuse_benchmark.rs` | 🔴 Baseline | New baseline | Measure gVisor performance |
| `cleanroom_benchmarks.rs` | 🔴 Baseline | New baseline | Compare backends |
| `memory_benchmarks.rs` | 🔴 Baseline | New baseline | Memory usage comparison |

---

## Common Migration Issues & Solutions

### Issue 1: Container Not Found

**Symptom:**
```
Error: Container 'test_container' not found
```

**Cause:** Container ID not tracked properly in service plugin

**Solution:**
```rust
// Store container_id in metadata
metadata.insert("container_id".to_string(), container_id.clone());

// Retrieve container_id for exec
let container_id = handle.metadata.get("container_id")
    .ok_or_else(|| CleanroomError::internal_error("container_id missing"))?;
```

### Issue 2: Port Already Allocated

**Symptom:**
```
Error: Port 8080 already in use
```

**Cause:** Port not released after test cleanup

**Solution:**
```rust
// In service stop()
if let Some(port_str) = handle.metadata.get("port_8080") {
    if let Ok(port) = port_str.parse::<u16>() {
        port_allocator.release(port).await?;
    }
}
```

### Issue 3: Bundle Directory Leak

**Symptom:**
```
Error: Disk space full
```

**Cause:** Bundle directories not cleaned up

**Solution:**
```rust
// Implement Drop for GVisorBackend
impl Drop for GVisorBackend {
    fn drop(&mut self) {
        // Clean up any running containers
        tokio::task::spawn_blocking(|| {
            // Cleanup logic
        });
    }
}
```

### Issue 4: Image Pull Timeout

**Symptom:**
```
Error: Timeout pulling image alpine:latest
```

**Cause:** Network slow or image not cached

**Solution:**
```rust
// Pre-pull images in CI setup
# .github/workflows/test.yml
- name: Pre-pull Test Images
  run: |
    skopeo copy docker://alpine:latest oci:/tmp/test-images/alpine:latest
    skopeo copy docker://ubuntu:22.04 oci:/tmp/test-images/ubuntu:22.04
```

### Issue 5: Telemetry Attributes Missing

**Symptom:**
```
Weaver validation failed: Missing attribute 'container.id'
```

**Cause:** gVisor backend not emitting same attributes as testcontainers

**Solution:**
```rust
// In GVisorBackend::run_cmd()
span.set_attributes(vec![
    KeyValue::new("container.id", &container_id),
    KeyValue::new("container.image.name", &self.image_ref),
    KeyValue::new("container.state", "running"),
    // ... all required attributes
]);
```

---

## Testing Best Practices

### 1. Test Isolation

```rust
// Always use unique container names
let container_name = format!("test_{}_{}", test_name, Uuid::new_v4());

// Always use unique bundle directories
let bundle_dir = format!("/tmp/clnrm-bundles/{}", Uuid::new_v4());
```

### 2. Resource Cleanup

```rust
// Use RAII pattern for cleanup
struct TestFixture {
    env: CleanroomEnvironment,
    handles: Vec<ServiceHandle>,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Cleanup all services
        for handle in &self.handles {
            let _ = self.env.stop_service(&handle.id);
        }
    }
}
```

### 3. Timeout Configuration

```rust
// Set reasonable timeouts for CI
let backend = GVisorBackend::new("docker.io/library/alpine:latest")?
    .with_timeout(Duration::from_secs(30)) // Command timeout
    .with_startup_timeout(Duration::from_secs(60)); // Container startup
```

### 4. Error Messages

```rust
// Provide detailed error context
let result = env.execute_in_container("test", &command, None, None)
    .await
    .map_err(|e| CleanroomError::container_error(
        format!("Test '{}' failed: {}", test_name, e)
    ))?;
```

### 5. Parallel Test Execution

```rust
// Use #[tokio::test(flavor = "multi_thread")] for concurrent tests
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_containers() -> Result<()> {
    // Spawn multiple tasks
    let tasks = (0..10).map(|i| tokio::spawn(async move {
        // Each task gets unique container
    }));

    futures::future::join_all(tasks).await;
}
```

---

## Verification Checklist

After migrating each test, verify:

- [ ] Test passes with gVisor backend
- [ ] Same exit code as testcontainers
- [ ] Same stdout/stderr output
- [ ] Same telemetry attributes emitted
- [ ] No resource leaks (check `runsc list`)
- [ ] No bundle directory leaks (check `/tmp/clnrm-bundles`)
- [ ] Performance within acceptable range (< 20% slower)
- [ ] Concurrent execution works
- [ ] Error cases handled correctly

---

## Next Steps

1. Review this guide before starting migration
2. Start with low-risk tests (basic container execution)
3. Run both testcontainers and gVisor in parallel during transition
4. Update CI to validate both backends
5. Gradually deprecate testcontainers tests

---

**End of Test Migration Guide**
