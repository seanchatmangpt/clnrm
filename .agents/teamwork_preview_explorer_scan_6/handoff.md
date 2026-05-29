# Cleanroom Service Layer Placeholder Resolution Strategy

This report details the findings and concrete implementation strategy to resolve the integrity violations identified by the Forensic Audit in the Cleanroom service management layer.

---

## 1. Observation

Direct observations of stubs, facades, and placeholder systems in `crates/clnrm-core/src/service/`:

### Observation A: `crates/clnrm-core/src/service/backend.rs` (Lines 242–256)
The gVisor backend implementation executes commands by returning a mock `RunResult` containing placeholder text instead of invoking the `runsc` OCI runtime:
```rust
        // ORACLE-GAP Refusal: Implement OCI bundle creation and runsc execution
        // For now, return a EXAMPLE-ONLY: placeholder result
        warn!("gVisor backend is not fully implemented yet - returning EXAMPLE-ONLY: placeholder result");

        Ok(RunResult {
            exit_code: 0,
            stdout: "gVisor backend EXAMPLE-ONLY: placeholder".to_string(),
            stderr: String::new(),
            duration_ms: start_time.elapsed().as_millis() as u64,
            steps: Vec::new(),
            redacted_env: Vec::new(),
            backend: "gvisor".to_string(),
            concurrent: false,
            step_order: Vec::new(),
        })
```

### Observation B: `crates/clnrm-core/src/service/oci.rs` (Lines 62–81 & 109–117)
The `OciImageManager` contains stubs that bypass registry pulling and bundle creation, creating empty directories rather than actual OCI configurations and filesystems:
* **Image Pulling Stub:**
```rust
        // ORACLE-GAP Refusal: Implement actual OCI image pulling
        // This would involve:
        // 1. Fetching image manifest from registry
        // 2. Downloading image layers
        // 3. Extracting layers to create rootfs
        // 4. Generating OCI config

        warn!("OCI image pulling not yet implemented - creating EXAMPLE-ONLY: placeholder");

        // Create EXAMPLE-ONLY: placeholder directory structure
        std::fs::create_dir_all(&image_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create image directory: {}", e))
        })?;

        std::fs::create_dir_all(image_dir.join("rootfs")).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create rootfs directory: {}", e))
        })?;
```
* **Bundle Creation Stub:**
```rust
        // ORACLE-GAP Refusal: Implement actual bundle creation
        // This would involve:
        // 1. Copying/linking rootfs from image
        // 2. Generating config.json
        // 3. Setting up mounts and network

        warn!("OCI bundle creation not yet implemented - creating EXAMPLE-ONLY: placeholder");
```

### Observation C: `crates/clnrm-core/src/service/health.rs` (Lines 319–332)
The `check_exec` and `check_grpc` health methods return hardcoded `Ok(true)` and emit log warnings rather than validating container health:
```rust
    /// Execute command in container
    async fn check_exec(&self, _command: &[String]) -> Result<bool> {
        // ORACLE-GAP Refusal: Implement container exec via runsc
        // This requires executing: runsc exec <container-id> <command>
        tracing::warn!("Exec health checks not yet implemented for gVisor backend");
        Ok(true)
    }

    /// Check gRPC health endpoint
    async fn check_grpc(&self, _host: &str, _port: u16, _service: Option<&str>) -> Result<bool> {
        // ORACLE-GAP Refusal: Implement gRPC health check protocol
        // https://github.com/grpc/grpc/blob/master/doc/health-checking.md
        tracing::warn!("gRPC health checks not yet implemented for gVisor backend");
        Ok(true)
    }
```

### Observation D: `crates/clnrm-core/src/service/registry.rs` (Line 225)
The service registry health checks bypass network topology isolation by hardcoding the container IP to localhost:
```rust
                // Get container IP for health check
                let container_ip = "127.0.0.1"; // ORACLE-GAP Refusal: Get actual container IP
```

---

## 2. Logic Chain

1. The service layer (`service/`) was introduced to provide declarative container orchestration for test suites.
2. The Forensic Audit identified that the `service/` directory contains stubs that bypass the entire runtime pipeline by returning hardcoded values (`EXAMPLE-ONLY` strings, mock folders, and dummy `Ok(true)` responses).
3. The codebase contains a fully implemented, production-grade OCI pipeline under `crates/clnrm-core/src/backend/oci/` (incorporating `OciImageLoader`, `LayerManager`, `ConfigParser`, `OciBundleBuilder`, and `RunscExecutor`). This backend manages registry pulling, layer extraction, config generation, and execution.
4. By connecting the stubs in the `service` layer to the existing OCI pipeline components in `crates/clnrm-core/src/backend/oci/`, the framework can eliminate all `ORACLE-GAP` facades and run containers inside actual gVisor sandboxes.
5. In addition to the stubs, the `service` module is currently not registered as `pub mod service;` inside `crates/clnrm-core/src/lib.rs`, meaning it is not actively compiled or checked during normal library builds. The module must be registered to compile and execute properly.

---

## 3. Caveats

* **gVisor Availability on Non-Linux Systems**: Since gVisor (`runsc`) relies on Linux namespaces and kernel primitives, it cannot run natively on macOS. To maintain developer portability, the strategy relies on `RunscExecutor`'s built-in `is_mock` fallback which executes processes locally when `runsc` is absent.
* **Network IP Resolution**: Resolving isolated container IPs on bridged network modes requires external OS commands or reading state from system bridges. For simple configurations, routing traffic to mapped ports on `127.0.0.1` is preferred, falling back to network namespace introspection for strictly isolated bridge networks.

---

## 4. Conclusion

The facades and stubs in the `service/` directory must be resolved by:
1. Re-integrating the `service` module into the build system by declaring `pub mod service;` inside `crates/clnrm-core/src/lib.rs`.
2. Refactoring `service/backend.rs` to pull OCI images, build bundles, and execute containers using the backend-level OCI structures.
3. Refactoring `service/oci.rs` to download real registry layers and extract them using `OciImageLoader` and `LayerManager`.
4. Extending `RunscExecutor` to support `exec` commands, and implementing real `exec` and `grpc` probes in `service/health.rs`.
5. Resolving the actual network IPs and host ports dynamically in `service/registry.rs`.

---

## 5. Verification Method

### Step 1: Compilation Verification
Verify the service module is compiled and free of syntax errors by running:
```bash
cargo check --all-targets
```

### Step 2: Test Execution
Verify unit and integration tests run correctly under the environment:
```bash
cargo test --test gall_tests
```

### Step 3: File Inspection
Verify that the `EXAMPLE-ONLY` warnings and the `ORACLE-GAP Refusal` comment blocks have been removed from:
- `crates/clnrm-core/src/service/backend.rs`
- `crates/clnrm-core/src/service/oci.rs`
- `crates/clnrm-core/src/service/health.rs`
- `crates/clnrm-core/src/service/registry.rs`
