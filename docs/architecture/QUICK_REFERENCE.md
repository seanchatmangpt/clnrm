# Container Execution Architecture - Quick Reference

## For Developers Implementing Issue #1

**Full Architecture**: See `/Users/sac/clnrm/docs/architecture/container-execution.md` (1467 lines, 86 sections)

---

## TL;DR - The Problem

Commands currently run on the HOST, not in containers. Need to:
1. Execute commands INSIDE Docker containers via docker exec
2. Maintain hermetic isolation per test
3. Support both service containers (long-lived) and ephemeral containers (one-shot)

---

## Quick Start Guide

### Key Files to Modify

1. **`crates/clnrm-core/src/cleanroom.rs`**
   - Add `ContainerRef` and `ContainerType` structs
   - Add `container_ref: Option<Arc<ContainerRef>>` to `ServiceHandle`
   - Add `execute_in_service()` method

2. **`crates/clnrm-core/src/backend/executor.rs`** (NEW FILE)
   - Create `ContainerExecutor` trait
   - Implement `TestcontainerExecutor`

3. **`crates/clnrm-core/src/services/generic.rs`**
   - Update `start()` to create `ContainerExecutor`
   - Add container reference to `ServiceHandle`

4. **`crates/clnrm-core/tests/integration_container_exec.rs`** (NEW FILE)
   - Add tests proving containers are actually used

---

## The Solution in 5 Minutes

### 1. Add Container Reference to ServiceHandle

```rust
// In src/cleanroom.rs
#[derive(Debug, Clone)]
pub struct ServiceHandle {
    pub id: String,
    pub service_name: String,
    pub metadata: HashMap<String, String>,
    // NEW: Add this field
    pub container_ref: Option<Arc<ContainerRef>>,
}

#[derive(Debug, Clone)]
pub struct ContainerRef {
    pub container_id: String,
    pub container_type: ContainerType,
    pub image: String,
    pub exec_client: Arc<dyn ContainerExecutor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerType {
    Service,
    Ephemeral,
}
```

### 2. Create ContainerExecutor Trait

```rust
// In src/backend/executor.rs (NEW FILE)
pub trait ContainerExecutor: Send + Sync + std::fmt::Debug {
    fn exec_command(&self, command: &[String]) -> Result<ExecutionResult>;
    fn is_running(&self) -> bool;
    fn container_id(&self) -> &str;
    fn stop(&self) -> Result<()>;
}

pub struct TestcontainerExecutor {
    container: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    container_id: String,
}

impl ContainerExecutor for TestcontainerExecutor {
    fn exec_command(&self, command: &[String]) -> Result<ExecutionResult> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Get container and execute via testcontainers exec
                let container = /* downcast to Container<GenericImage> */;
                let exec_cmd = ExecCommand::new(command);
                let result = container.exec(exec_cmd).await?;
                // Read stdout/stderr, return ExecutionResult
            })
        })
    }
}
```

### 3. Update GenericContainerPlugin

```rust
// In src/services/generic.rs
impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create container
                let container = container_request
                    .with_cmd(vec!["sleep", "infinity"]) // Keep alive
                    .start()
                    .await?;

                // NEW: Create executor
                let executor = Arc::new(TestcontainerExecutor::new(
                    container,
                    Uuid::new_v4().to_string(),
                ));

                // NEW: Create container reference
                let container_ref = ContainerRef {
                    container_id: executor.container_id().to_string(),
                    container_type: ContainerType::Service,
                    image: format!("{}:{}", self.image, self.tag),
                    exec_client: executor,
                };

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata: HashMap::new(),
                    container_ref: Some(Arc::new(container_ref)), // NEW
                })
            })
        })
    }
}
```

### 4. Add execute_in_service Method

```rust
// In src/cleanroom.rs
impl CleanroomEnvironment {
    pub async fn execute_in_service(
        &self,
        service_name: &str,
        command: &[String],
    ) -> Result<ExecutionResult> {
        // Get service handle
        let services = self.services.read().await;
        let handle = services.get_service_handle(service_name)?;

        // Get container reference
        let container_ref = handle.container_ref.as_ref()
            .ok_or_else(|| CleanroomError::internal_error(
                "Service has no container reference"
            ))?;

        // Execute via docker exec
        container_ref.exec_client.exec_command(command)
    }
}
```

### 5. Add Integration Test to Prove It Works

```rust
// In tests/integration_container_exec.rs (NEW FILE)
#[tokio::test]
async fn test_command_executes_in_container_not_host() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;

    // Execute command that only exists in Alpine container
    let cmd = vec!["cat".to_string(), "/etc/alpine-release".to_string()];
    let result = env.execute_in_ephemeral_container("alpine:latest", &cmd).await?;

    // Should succeed in container
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("3.")); // Alpine version

    // Prove it's not running on host
    let host_result = std::process::Command::new("cat")
        .arg("/etc/alpine-release")
        .output();

    // Should fail on macOS/Ubuntu host
    assert!(host_result.is_err() || !host_result.unwrap().status.success());

    Ok(())
}
```

---

## Implementation Checklist

### Phase 1: Foundation (Week 1)
- [ ] Create `src/backend/executor.rs`
- [ ] Add `ContainerRef` types to `src/cleanroom.rs`
- [ ] Add `container_ref` field to `ServiceHandle`
- [ ] Implement `TestcontainerExecutor`
- [ ] Add unit tests for executor

### Phase 2: Service Execution (Week 2)
- [ ] Update `GenericContainerPlugin::start()` to create executor
- [ ] Add `ServiceRegistry::get_service_handle()`
- [ ] Add `CleanroomEnvironment::execute_in_service()`
- [ ] Add integration tests for service execution

### Phase 3: Verification (Week 2-3)
- [ ] Create `tests/integration_container_exec.rs`
- [ ] Add test proving containers are used (not host)
- [ ] Add test for container isolation
- [ ] Add test for service container command execution
- [ ] Verify TestcontainerBackend uses real containers

### Phase 4: TOML Support (Week 3)
- [ ] Add `service: Option<String>` to `StepConfig`
- [ ] Add `container_image: Option<String>` to `StepConfig`
- [ ] Update test runner to route commands correctly
- [ ] Add TOML-based integration tests

### Phase 5: Production Hardening (Week 4)
- [ ] Remove all `.unwrap()` and `.expect()` from production code
- [ ] Add proper error messages
- [ ] Add resource cleanup verification
- [ ] Run `cargo clippy -- -D warnings` and fix all issues
- [ ] Update documentation

---

## Common Pitfalls to Avoid

### 1. Don't Break Async/Sync Boundary

```rust
// ❌ WRONG - async trait method breaks dyn compatibility
pub trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ CORRECT - sync trait with async inside
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Async work here
            })
        })
    }
}
```

### 2. Don't Lose Container Reference

```rust
// ❌ WRONG - container dropped immediately
let container = image.start().await?;
let container_id = container.id();
// container dropped here - container stops!

// ✅ CORRECT - store container in Arc<RwLock<>>
let container = image.start().await?;
let executor = Arc::new(TestcontainerExecutor::new(container, id));
// container kept alive via executor
```

### 3. Don't Use .unwrap() or .expect()

```rust
// ❌ WRONG - will panic
let result = container.exec(cmd).await.unwrap();

// ✅ CORRECT - proper error handling
let result = container.exec(cmd).await
    .map_err(|e| CleanroomError::container_error(
        format!("exec failed: {}", e)
    ))?;
```

### 4. Don't Skip Verification Tests

```rust
// ✅ REQUIRED - prove containers are actually used
#[tokio::test]
async fn test_runs_in_container_not_host() {
    // Execute container-specific command
    let result = env.execute_in_container("alpine", &["cat", "/etc/alpine-release"]).await?;
    assert_eq!(result.exit_code, 0);

    // Prove it's not host
    let host_result = Command::new("cat").arg("/etc/alpine-release").output();
    assert!(host_result.is_err() || !host_result.unwrap().status.success());
}
```

---

## How to Verify Success

### 1. Build and Test
```bash
cd /Users/sac/clnrm
cargo build --release
cargo test
cargo clippy -- -D warnings
```

### 2. Run Integration Tests
```bash
cargo test --test integration_container_exec -- --nocapture
```

### 3. Run Self-Test via Dogfooding
```bash
clnrm self-test --suite container-execution
```

### 4. Check Container Usage
```bash
# While tests run, check Docker containers
docker ps

# Should see containers with clnrm images
```

---

## Performance Expectations

- **Container creation (cold)**: 2-5 seconds (first image pull)
- **Container creation (warm)**: 100-500ms (cached image)
- **docker exec**: 10-50ms per command
- **Service reuse**: 10-50x faster than new containers

### Optimization Tips

1. **Use service containers for multiple commands**
   ```rust
   let handle = env.start_service("db").await?;
   for _ in 0..100 {
       env.execute_in_service("db", &["echo", "fast"]).await?;
   }
   // 100x faster than 100 new containers
   ```

2. **Pre-pull images**
   ```bash
   docker pull alpine:latest
   docker pull postgres:15-alpine
   # Tests run faster (no pull time)
   ```

3. **Run tests in parallel**
   ```rust
   let futures: Vec<_> = tests.iter()
       .map(|t| env.execute_in_container(&t.image, &t.cmd))
       .collect();
   let results = futures::future::join_all(futures).await;
   ```

---

## Questions? Read the Full Architecture

See `/Users/sac/clnrm/docs/architecture/container-execution.md` for:
- Complete component design (1467 lines)
- Security considerations
- Performance benchmarks
- Architecture decision records
- Complete code examples
- Test strategies
- Migration paths

---

## Key Contacts

- Architecture: See `/Users/sac/clnrm/docs/architecture/container-execution.md`
- Core Team Standards: See `/Users/sac/clnrm/.cursorrules`
- Testing Standards: See `/Users/sac/clnrm/docs/TESTING.md`

---

**Quick Reference Version**: 1.0
**Date**: 2025-10-17
**Status**: Ready for Implementation
