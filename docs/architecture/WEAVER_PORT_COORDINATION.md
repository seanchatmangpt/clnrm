# Weaver-First Port Coordination Architecture

## Problem Statement

**Current Issue**: Port mismatch between OTEL initialization (hardcoded 4317) and Weaver listener (dynamic port discovery). This creates a chicken-and-egg problem where:

1. OTEL exporter needs endpoint before initialization
2. Weaver discovers available port during startup
3. Telemetry sent to wrong port, validation fails

**Critical Requirement**: Weaver MUST start first and OTEL MUST use Weaver's actual port.

## Architecture Decision

### Decision: Weaver-First Initialization Pattern

**Rationale**: Make Weaver the source of truth for port coordination.

```
┌─────────────────────────────────────────────────────┐
│ Weaver-First Coordination Flow                     │
└─────────────────────────────────────────────────────┘

  Phase 1: Weaver Discovery
  ┌──────────────────────────┐
  │ 1. Find available port   │
  │    (4317-4327, fallback) │
  └───────────┬──────────────┘
              │
              v
  ┌──────────────────────────┐
  │ 2. Start Weaver process  │
  │    on discovered port    │
  └───────────┬──────────────┘
              │
              v
  ┌──────────────────────────┐
  │ 3. Wait for ready state  │
  │    (health check)        │
  └───────────┬──────────────┘
              │
              v
  Phase 2: OTEL Initialization
  ┌──────────────────────────┐
  │ 4. Get Weaver's port     │
  │    via get_otlp_port()   │
  └───────────┬──────────────┘
              │
              v
  ┌──────────────────────────┐
  │ 5. Init OTEL exporter    │
  │    with Weaver endpoint  │
  └───────────┬──────────────┘
              │
              v
  Phase 3: Test Execution
  ┌──────────────────────────┐
  │ 6. Run tests with        │
  │    telemetry emission    │
  └───────────┬──────────────┘
              │
              v
  Phase 4: Validation
  ┌──────────────────────────┐
  │ 7. Flush OTEL explicitly │
  │    (ensure delivery)     │
  └───────────┬──────────────┘
              │
              v
  ┌──────────────────────────┐
  │ 8. Stop Weaver, get      │
  │    validation report     │
  └──────────────────────────┘
```

## Component Design

### 1. WeaverCoordination Structure

```rust
/// Coordination metadata from Weaver startup
#[derive(Debug, Clone)]
pub struct WeaverCoordination {
    /// Process ID of Weaver instance
    pub weaver_pid: u32,
    /// OTLP gRPC port Weaver is listening on
    pub otlp_grpc_port: u16,
    /// Admin/health port for control interface
    pub admin_port: u16,
    /// Timestamp when Weaver became ready
    pub ready_at: std::time::Instant,
}
```

**Design Rationale**:
- Immutable after creation (no port changes mid-execution)
- Contains all information needed for OTEL initialization
- Includes timestamp for coordination debugging

### 2. Enhanced WeaverController

**New Methods**:

```rust
impl WeaverController {
    /// Start Weaver and return coordination info (BLOCKING)
    ///
    /// This is the PRIMARY method for Weaver-first initialization.
    /// It blocks until Weaver is ready and returns coordination metadata.
    pub fn start_and_coordinate(&mut self) -> Result<WeaverCoordination> {
        // 1. Find available ports
        // 2. Start Weaver process
        // 3. Health check (wait for ready)
        // 4. Return coordination info
    }

    /// Get current coordination state (non-blocking)
    ///
    /// Returns None if Weaver not started, otherwise coordination info.
    pub fn coordination(&self) -> Option<WeaverCoordination> {
        // Return cached coordination from start_and_coordinate()
    }
}
```

**Key Design Decisions**:
- `start_and_coordinate()` is **synchronous/blocking** - ensures ready state before returning
- Stores coordination metadata internally for later queries
- Health check with timeout prevents indefinite blocking

### 3. Port Discovery Algorithm

```rust
/// Find available port with intelligent retry
///
/// Strategy:
/// 1. Try primary range (4317-4327) - standard OTLP gRPC ports
/// 2. Fallback to secondary range (5317-5327) if primary exhausted
/// 3. Error if no ports available
///
/// Returns first available port
fn find_available_port_with_fallback() -> Result<u16> {
    // Try primary range
    if let Ok(port) = find_available_port(4317, 4327) {
        return Ok(port);
    }

    // Fallback to secondary range
    warn!("Primary OTLP port range exhausted, trying fallback");
    find_available_port(5317, 5327)
}
```

**Design Rationale**:
- Primary range matches standard OTLP gRPC convention
- Fallback ensures robustness in congested environments
- Clear error when all ranges exhausted

### 4. Health Check Implementation

```rust
/// Wait for Weaver to become ready
///
/// Health check strategy:
/// 1. Initial delay (1000ms) for process startup
/// 2. Check process still running (not crashed)
/// 3. Optional: HTTP GET to admin port /health
/// 4. Timeout after 10 seconds
fn wait_for_ready(&mut self, timeout: Duration) -> Result<()> {
    let start = std::time::Instant::now();

    // Initial startup delay
    thread::sleep(Duration::from_millis(1000));

    // Check process state
    if let Some(ref mut process) = self.live_check_process {
        match process.try_wait()? {
            Some(status) => {
                return Err(CleanroomError::internal_error(
                    format!("Weaver exited prematurely: {}", status)
                ));
            }
            None => {
                // Still running, assume ready
                // TODO: Add HTTP health check to admin port
                return Ok(());
            }
        }
    }

    Err(CleanroomError::timeout_error("Weaver not ready within timeout"))
}
```

**Future Enhancement**: HTTP health check to admin port for robust readiness verification.

## Integration Pattern

### Run Command Integration

**Before (Broken)**:
```rust
// WRONG: OTEL initialized with hardcoded port
let _otel_guard = init_otel(OtelConfig {
    export: Export::OtlpGrpc {
        endpoint: "http://localhost:4317", // ❌ Hardcoded!
    },
    ..config
})?;

// Weaver starts AFTER OTEL (too late!)
let mut weaver = WeaverController::new(config);
weaver.start_live_check()?;
```

**After (Correct)**:
```rust
// ✅ CORRECT: Weaver-first pattern
let mut weaver = WeaverController::new(weaver_config);
let coordination = weaver.start_and_coordinate()?;

info!("🔗 Weaver ready on port {}", coordination.otlp_grpc_port);

// Initialize OTEL with Weaver's actual port
let endpoint = format!("http://localhost:{}", coordination.otlp_grpc_port);
let _otel_guard = init_otel(OtelConfig {
    export: Export::OtlpGrpc {
        endpoint: Box::leak(endpoint.into_boxed_str()),
    },
    ..config
})?;

// Now run tests - telemetry goes to Weaver
run_tests(...)?;

// Explicit flush before Weaver shutdown
drop(_otel_guard);
thread::sleep(Duration::from_millis(500));

// Get validation report
let report = weaver.stop_and_report()?;
```

## Error Handling

### Port Exhaustion
```rust
Err(CleanroomError::validation_error(
    "No available ports in range 4317-4327, 5317-5327. \
     All ports in use. Stop other OTLP services or use custom port range."
))
```

### Weaver Startup Failure
```rust
Err(CleanroomError::internal_error(
    "Weaver exited prematurely with status: exit code 1. \
     Check Weaver logs in validation_output/ for details."
))
```

### OTEL Initialization Failure
```rust
Err(CleanroomError::validation_error(
    "Failed to initialize OTEL with Weaver endpoint. \
     Weaver may not be ready or port may be blocked."
))
```

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_port_discovery_primary_range() {
    // Verify primary range tried first
}

#[test]
fn test_port_discovery_fallback() {
    // Occupy primary range, verify fallback works
}

#[test]
fn test_coordination_structure() {
    // Verify coordination metadata is correct
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_weaver_first_coordination() {
    // End-to-end: Start Weaver, init OTEL, send telemetry, verify
}

#[tokio::test]
async fn test_port_reuse_after_shutdown() {
    // Verify port is released and can be reused
}
```

## Performance Characteristics

**Startup Latency**:
- Port discovery: ~10-50ms (depends on OS TCP stack)
- Weaver process start: ~500-1000ms
- Health check delay: 1000ms
- **Total overhead**: ~1.5-2 seconds

**Rationale**: This overhead is acceptable for validation workflow. Production deployments don't use live-check, so zero overhead there.

## Operational Considerations

### Docker Environment
- Weaver container must expose OTLP port to host
- Port mapping: `-p ${DYNAMIC_PORT}:4317`
- Coordination struct must track mapped port

### CI/CD Environment
- Multiple jobs may run concurrently
- Port discovery prevents conflicts
- Fallback range increases parallelism capacity

### Development Workflow
```bash
# Developer runs validation
cargo build --features otel
clnrm run tests/ --validate

# Weaver starts automatically on available port
# OTEL configured automatically
# Validation report displayed at end
```

## Alternatives Considered

### Alternative 1: Fixed Port with Retry
**Rejected**: Doesn't solve port conflicts in CI.

### Alternative 2: OTEL-First with Port Injection
**Rejected**: Requires Weaver to accept injected port, but Weaver controls its own listener.

### Alternative 3: Proxy Layer
**Rejected**: Adds complexity and latency without solving coordination problem.

## Migration Path

### Phase 1: Implement Coordination (This Task)
- Add `WeaverCoordination` struct
- Enhance `WeaverController` with coordination methods
- Update port discovery with fallback

### Phase 2: Integrate with Run Command
- Modify `run_tests_impl_with_report()` to use Weaver-first pattern
- Add explicit OTEL flush before Weaver shutdown
- Update error messages and logging

### Phase 3: Validate in CI
- Run full test suite with Weaver validation
- Verify zero port conflicts across parallel jobs
- Measure overhead and optimize if needed

## Success Metrics

1. **Zero Port Conflicts**: All CI jobs complete without port errors
2. **100% Coordination Success**: OTEL always connects to Weaver's actual port
3. **Validation Accuracy**: Weaver receives all telemetry from tests
4. **Developer Experience**: Single `--validate` flag enables end-to-end validation

## References

- OpenTelemetry OTLP Specification: https://opentelemetry.io/docs/specs/otlp/
- Weaver Documentation: https://github.com/open-telemetry/weaver
- clnrm Weaver Integration: `docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md`
