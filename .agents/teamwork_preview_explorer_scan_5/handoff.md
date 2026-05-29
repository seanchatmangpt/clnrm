# Forensic Audit Resolution Strategy: Health Check Facades

An independent Forensic Audit has identified integrity violations in `crates/clnrm-core/src/service/health.rs` (reported as `crates/clnrm-core/src/services/health.rs`) where `check_exec` and `check_grpc` return hardcoded `Ok(true)` and print warnings instead of performing actual container probing.

This document analyzes the current facade code and outlines a concrete, production-grade implementation strategy to replace these stubs with genuine, functional logic.

---

## 1. Observation

In the repository, the target file is located at `crates/clnrm-core/src/service/health.rs` (singular `service`, rather than plural `services`). 

The facade implementations for container `exec` and `gRPC` health checks are located at lines 319-324 and 326-332 respectively:

```rust
319:     async fn check_exec(&self, _command: &[String]) -> Result<bool> {
320:         // ORACLE-GAP Refusal: Implement container exec via runsc
321:         // This requires executing: runsc exec <container-id> <command>
322:         tracing::warn!("Exec health checks not yet implemented for gVisor backend");
323:         Ok(true)
324:     }
325: 
326:     /// Check gRPC health endpoint
327:     async fn check_grpc(&self, _host: &str, _port: u16, _service: Option<&str>) -> Result<bool> {
328:         // ORACLE-GAP Refusal: Implement gRPC health check protocol
329:         // https://github.com/grpc/grpc/blob/master/doc/health-checking.md
330:         tracing::warn!("gRPC health checks not yet implemented for gVisor backend");
331:         Ok(true)
332:     }
```

Both methods bypass the system checking mechanism entirely by returning a hardcoded `Ok(true)` after printing a warning, rendering these health checks useless in production.

---

## 2. Logic Chain

1. **Service Metadata & Loop Analysis:** 
   In `crates/clnrm-core/src/service/registry.rs`, the periodic health check loop `check_all_health` executes:
   ```rust
   if let Some(probe) = probes.get_mut(&service_id) {
       match probe.check(container_ip).await {
   ```
   Currently, the loop only passes the `container_ip`. However, to run `runsc exec`, gVisor requires the `container_id` (from `ServiceMetadata.container_id`), which is available in `registry.rs` at this point under `service.container_id`.

2. **Passing `container_id` to Probe Checks:**
   The signature of `HealthProbe::check` in `health.rs` must be adjusted to accept `container_id: &str`. This enables `check_exec` to run `runsc exec <container_id> <command...>` inside the specific container sandbox.

3. **Exec Probing Integration:**
   - **Production (gVisor Environment):** When `runsc` is available on the path, execution states are written to cache directories. We can spawn a `tokio::process::Command` to invoke `runsc --root <root_dir> exec <container_id> <command...>` and check the command's exit code.
   - **Fallback (Mock Environment):** On platforms without `runsc` (such as macOS local development), the container runs as a host process. In mock mode, we fallback to executing the command directly in the host environment to see if it succeeds.

4. **gRPC Probing Integration:**
   gRPC container probing is specified in [grpc/health-checking.md](https://github.com/grpc/grpc/blob/master/doc/health-checking.md). It involves performing a gRPC `Check` call to the standard service `grpc.health.v1.Health`.
   We propose two implementation paths:
   - **Option A (Tonic-Health Crate):** Add `tonic-health` as a dependency, and use the official `HealthClient` to connect and check the server's status.
   - **Option B (Pure HTTP/2 via Reqwest):** Since `reqwest` is already included and configured, we can construct the binary protobuf payload for `HealthCheckRequest` manually, post it to the standard gRPC endpoint, and parse the raw `HealthCheckResponse` payload.

---

## 3. Caveats

- **Mock Environment Limitations:** On macOS, health checks will execute against local processes in host space rather than true gVisor sandboxes. This is the existing and expected design of the Cleanroom framework's fallback modes.
- **Port Mapping:** Cleartext gRPC calls assume the host port or container port is accessible. The system should target the correct `container_ip` and port supplied by the service metadata.

---

## 4. Conclusion & Proposed Changes

To eliminate the stubs and resolve the integrity violations, we recommend applying the following changes.

### A. Modifications to `crates/clnrm-core/src/service/registry.rs`

Update the health probe call on line 230 to pass `&service.container_id`:

```rust
<<<<
                if let Some(probe) = probes.get_mut(&service_id) {
                    match probe.check(container_ip).await {
====
                if let Some(probe) = probes.get_mut(&service_id) {
                    match probe.check(&service.container_id, container_ip).await {
>>>>
```

### B. Modifications to `crates/clnrm-core/src/service/health.rs`

#### 1. Update `check` Signature and Match Branch

```rust
<<<<
    /// Execute health check
    pub async fn check(&mut self, container_ip: &str) -> Result<HealthStatus> {
        let check_result = match &self.check {
            HealthCheck::Tcp { port, .. } => self.check_tcp(container_ip, *port).await,
            HealthCheck::Http {
                port,
                path,
                scheme,
                ..
            } => self.check_http(container_ip, *port, path, scheme).await,
            HealthCheck::Exec { command, .. } => self.check_exec(command).await,
            HealthCheck::Grpc { port, service, .. } => {
                self.check_grpc(container_ip, *port, service.as_deref()).await
            }
        };
====
    /// Execute health check
    pub async fn check(&mut self, container_id: &str, container_ip: &str) -> Result<HealthStatus> {
        let check_result = match &self.check {
            HealthCheck::Tcp { port, .. } => self.check_tcp(container_ip, *port).await,
            HealthCheck::Http {
                port,
                path,
                scheme,
                ..
            } => self.check_http(container_ip, *port, path, scheme).await,
            HealthCheck::Exec { command, .. } => self.check_exec(container_id, command).await,
            HealthCheck::Grpc { port, service, .. } => {
                self.check_grpc(container_ip, *port, service.as_deref()).await
            }
        };
>>>>
```

#### 2. Implement Production-Grade `check_exec`

```rust
<<<<
    /// Execute command in container
    async fn check_exec(&self, _command: &[String]) -> Result<bool> {
        // ORACLE-GAP Refusal: Implement container exec via runsc
        // This requires executing: runsc exec <container-id> <command>
        tracing::warn!("Exec health checks not yet implemented for gVisor backend");
        Ok(true)
    }
====
    /// Execute command in container
    async fn check_exec(&self, container_id: &str, command: &[String]) -> Result<bool> {
        let has_runsc = which::which("runsc").is_ok();
        let timeout = self.check.timeout()?;

        if !has_runsc {
            // Mock mode fallback: execute the command locally on the host
            if command.is_empty() {
                return Ok(true);
            }
            let mut cmd = tokio::process::Command::new(&command[0]);
            if command.len() > 1 {
                cmd.args(&command[1..]);
            }
            match tokio::time::timeout(timeout, cmd.output()).await {
                Ok(Ok(output)) => Ok(output.status.success()),
                Ok(Err(_)) | Err(_) => Ok(false),
            }
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
>>>>
```

#### 3. Implement Production-Grade `check_grpc`

##### Option A: Using `tonic-health` (Recommended for Maintainability)

Add `tonic-health = "0.12"` to the dependencies in `crates/clnrm-core/Cargo.toml` and implement:

```rust
<<<<
    /// Check gRPC health endpoint
    async fn check_grpc(&self, _host: &str, _port: u16, _service: Option<&str>) -> Result<bool> {
        // ORACLE-GAP Refusal: Implement gRPC health check protocol
        // https://github.com/grpc/grpc/blob/master/doc/health-checking.md
        tracing::warn!("gRPC health checks not yet implemented for gVisor backend");
        Ok(true)
    }
====
    /// Check gRPC health endpoint
    async fn check_grpc(&self, host: &str, port: u16, service: Option<&str>) -> Result<bool> {
        use tonic_health::pb::health_client::HealthClient;
        use tonic_health::pb::HealthCheckRequest;

        let endpoint = format!("http://{}:{}", host, port);
        let timeout = self.check.timeout()?;

        let mut client = match HealthClient::connect(endpoint).await {
            Ok(client) => client,
            Err(_) => return Ok(false),
        };

        let request = tonic::Request::new(HealthCheckRequest {
            service: service.unwrap_or("").to_string(),
        });

        match tokio::time::timeout(timeout, client.check(request)).await {
            Ok(Ok(response)) => {
                use tonic_health::pb::health_check_response::ServingStatus;
                Ok(response.into_inner().status == ServingStatus::Serving as i32)
            }
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }
>>>>
```

##### Option B: Using Pure HTTP/2 via `reqwest` (Zero-Dependency Fallback)

If modifying `Cargo.toml` is not desired, the following cleartext HTTP/2 manual payload serializing/deserializing implementation is recommended:

```rust
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

        match client.post(&url)
            .header("content-type", "application/grpc")
            .header("te", "trailers")
            .body(body)
            .send()
            .await {
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
                    
                    let payload_len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
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
```

---

## 5. Verification Method

To independently verify correctness of the implementations:

1. **Run Project Integration Health Tests:**
   Ensure existing service health checks compile and pass:
   ```bash
   cargo test --test gall_tests -- gall_gap_test_service_health_check_tcp_probe
   ```

2. **Add Unit/Integration Tests for Probing Cases:**
   In `crates/clnrm-core/tests/gall_test_suites/service_health.rs`, add tests to:
   - Call `check_exec` under mock and runsc environments to ensure it verifies host output or gVisor exit status.
   - Start a mock gRPC health server locally, query it using the updated `check_grpc` method, and verify it returns `Ok(true)` for `SERVING` and `Ok(false)` for any other status or connection failure.

3. **Invalidation Conditions:**
   - Returning `Ok(true)` when the underlying exec command fails (exit code != 0).
   - Returning `Ok(true)` when a gRPC health endpoint is down, returns `NOT_SERVING`, or returns a gRPC status code of `UNIMPLEMENTED`.
