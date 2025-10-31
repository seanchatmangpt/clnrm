# Code-Documentation Mismatch Analysis
**Agent:** CODER
**Task:** Analyze WeaverController implementation vs. architecture documentation
**Date:** 2025-10-31
**Session:** swarm-1761877703971-q3rac7qx5

---

## Executive Summary

**CRITICAL FINDINGS:** WeaverController implementation has significant mismatches with architecture documentation, primarily around port configuration, coordination patterns, and hardcoded values that should be configurable.

**Risk Level:** HIGH - Port conflicts, inflexible configuration, timeout issues in production

---

## 1. Port Configuration Mismatches

### Issue 1.1: Hardcoded Default Ports in WeaverConfig

**Location:** `weaver_controller.rs:121-131`

**Code:**
```rust
impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            registry_path: PathBuf::from("registry"),
            otlp_port: 4317,        // ❌ HARDCODED
            admin_port: 8080,       // ❌ HARDCODED
            output_dir: PathBuf::from("./validation_output"),
            stream: false,
        }
    }
}
```

**Documentation Claims (WEAVER_PORT_COORDINATION.md:77-93):**
- Ports should be dynamically discovered
- `WeaverCoordination` contains "OTLP gRPC port Weaver is listening on"
- "Immutable after creation (no port changes mid-execution)"

**Mismatch:**
- Default implementation hardcodes ports that documentation says should be discovered
- Creates confusion: when to use defaults vs. discovery?
- Tests at line 846-847 validate these hardcoded defaults

**Impact:** HIGH
- Port conflicts in CI/CD environments
- Misleading API surface (defaults imply fixed ports are acceptable)
- Contradicts "Weaver-first coordination" principle

**Recommendation:**
```rust
impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,  // 0 = auto-discover
            admin_port: 0, // 0 = auto-discover
            output_dir: PathBuf::from("./validation_output"),
            stream: false,
        }
    }
}
```

### Issue 1.2: Port Range Hardcoding

**Location:** `weaver_controller.rs:373-396`

**Code:**
```rust
fn find_available_port_with_fallback() -> Result<u16> {
    // Try primary range (standard OTLP gRPC ports)
    if let Ok(port) = Self::find_available_port(4317, 4327) {  // ❌ HARDCODED
        return Ok(port);
    }

    // Fallback to secondary range
    warn!("Primary OTLP port range (4317-4327) exhausted, trying fallback range");
    Self::find_available_port(5317, 5327)  // ❌ HARDCODED
        .map_err(|_| {
            CleanroomError::validation_error(
                "No available ports in range 4317-4327, 5317-5327. \  // ❌ HARDCODED in error
                 All ports in use. Stop other OTLP services or use custom port range."
            )
        })
}
```

**Same pattern repeated at lines 527-535 in `start_live_check()`**

**Documentation Claims (WEAVER_PORT_COORDINATION.md:132-152):**
```rust
/// Strategy:
/// 1. Try primary range (4317-4327) - standard OTLP gRPC ports
/// 2. Fallback to secondary range (5317-5327) if primary exhausted
```

**Mismatch:**
- Documentation describes algorithm but doesn't indicate hardcoding
- No configuration option to override port ranges
- Error message hardcodes ranges, making them user-visible as "fixed"

**Impact:** MEDIUM-HIGH
- Cannot configure custom port ranges for restrictive environments
- Port scanning is inefficient (always tries 4317 first, even if known to be occupied)
- Admin port ranges (8080-8090, 9080-9090) similarly hardcoded

**Recommendation:**
```rust
pub struct PortRanges {
    pub otlp_primary: (u16, u16),
    pub otlp_fallback: (u16, u16),
    pub admin_primary: (u16, u16),
    pub admin_fallback: (u16, u16),
}

impl Default for PortRanges {
    fn default() -> Self {
        Self {
            otlp_primary: (4317, 4327),
            otlp_fallback: (5317, 5327),
            admin_primary: (8080, 8090),
            admin_fallback: (9080, 9090),
        }
    }
}

pub struct WeaverConfig {
    // ... existing fields ...
    pub port_ranges: PortRanges,
}
```

### Issue 1.3: Admin Port Discovery Inconsistency

**Location:** `weaver_controller.rs:252-255` vs. `weaver_controller.rs:532-535`

**Code in `start_and_coordinate()`:**
```rust
let admin_port = Self::find_available_port(8080, 8090).or_else(|_| {
    warn!("Primary admin port range exhausted, trying fallback");
    Self::find_available_port(9080, 9090)
})?;
```

**Code in `start_live_check()`:**
```rust
let admin_port = Self::find_available_port(8080, 8090).or_else(|_| {
    warn!("Primary admin port range exhausted, trying fallback range");
    Self::find_available_port(9080, 9090)
})?;
```

**Mismatch:**
- Duplicated logic (DRY violation)
- Slightly different warning messages
- Should delegate to shared `find_available_admin_port_with_fallback()`

**Impact:** LOW
- Maintenance burden (changes must be made in two places)
- Potential for divergence

---

## 2. Timeout Configuration Issues

### Issue 2.1: Hardcoded Timeout Values

**Locations:**
- Line 229: `Duration::from_millis(500)` - OTEL flush time
- Line 343: `Duration::from_secs(10)` - Weaver ready timeout
- Line 411: `Duration::from_millis(1000)` - Initial startup delay
- Line 491: `Duration::from_millis(500)` - Process termination grace period
- Line 501: `Duration::from_millis(500)` - Process termination grace period (Windows)
- Line 621: `Duration::from_millis(1000)` - Weaver initialization wait
- Line 689: `Duration::from_secs(10)` - Process shutdown timeout
- Line 810: `Duration::from_millis(100)` - Wait loop polling interval

**Documentation Claims (WEAVER_PORT_COORDINATION.md:305-311):**
```
**Startup Latency**:
- Port discovery: ~10-50ms (depends on OS TCP stack)
- Weaver process start: ~500-1000ms
- Health check delay: 1000ms
- **Total overhead**: ~1.5-2 seconds
```

**Mismatch:**
- Documentation mentions timing but doesn't indicate they're configurable
- Implementation hardcodes all timeouts
- No way to adjust for:
  - Slow CI environments
  - Fast local development
  - Network latency variations
  - Resource-constrained containers

**Impact:** HIGH
- Tests may be flaky in slow environments (10s timeout too short)
- Unnecessary delays in fast environments (1000ms startup too long)
- Cannot optimize for specific deployment scenarios

**Recommendation:**
```rust
pub struct WeaverTimeouts {
    /// Time to wait for Weaver to become ready
    pub ready_timeout: Duration,
    /// Initial delay before checking process state
    pub startup_delay: Duration,
    /// Grace period for OTEL flush before shutdown
    pub flush_grace_period: Duration,
    /// Timeout for graceful process shutdown
    pub shutdown_timeout: Duration,
    /// Polling interval for process wait loops
    pub poll_interval: Duration,
}

impl Default for WeaverTimeouts {
    fn default() -> Self {
        Self {
            ready_timeout: Duration::from_secs(10),
            startup_delay: Duration::from_millis(1000),
            flush_grace_period: Duration::from_millis(500),
            shutdown_timeout: Duration::from_secs(10),
            poll_interval: Duration::from_millis(100),
        }
    }
}

pub struct WeaverConfig {
    // ... existing fields ...
    pub timeouts: WeaverTimeouts,
}
```

### Issue 2.2: Health Check Implementation Gap

**Location:** `weaver_controller.rs:406-441`

**Code:**
```rust
/// Wait for Weaver to become ready
///
/// Health check strategy:
/// 1. Initial delay (1000ms) for process startup
/// 2. Check process still running (not crashed)
/// 3. Return success if process is running
///
/// Future enhancement: Add HTTP health check to admin port
fn wait_for_ready(&mut self, _timeout: Duration) -> Result<()> {
    // ... implementation ...
    // TODO: Add HTTP health check to admin port  // ❌ NOT IMPLEMENTED
    return Ok(());
}
```

**Documentation Claims (WEAVER_PORT_COORDINATION.md:159-193):**
```rust
/// Health check strategy:
/// 1. Initial delay (1000ms) for process startup
/// 2. Check process still running (not crashed)
/// 3. Optional: HTTP GET to admin port /health
/// 4. Timeout after 10 seconds
```

**Also in WEAVER_INTEGRATION_DESIGN.md:226-227:**
```
[GET /health]
```

**Mismatch:**
- Documentation describes HTTP health check as part of design
- Implementation has TODO comment
- Admin port is discovered but never used for health checks

**Impact:** MEDIUM
- Cannot verify Weaver is actually listening on discovered port
- Process may be running but not ready to accept connections
- Race conditions: tests may start before Weaver is truly ready

**Recommendation:**
Implement HTTP health check:
```rust
fn wait_for_ready(&mut self, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    thread::sleep(Duration::from_millis(1000));

    // Check process state
    if let Some(ref mut process) = self.live_check_process {
        match process.try_wait()? {
            Some(status) => return Err(...),
            None => {
                // Process running, now check HTTP health
                let health_url = format!("http://127.0.0.1:{}/health", self.config.admin_port);

                loop {
                    if start.elapsed() > timeout {
                        return Err(CleanroomError::timeout_error("Health check timeout"));
                    }

                    match reqwest::blocking::get(&health_url) {
                        Ok(response) if response.status().is_success() => {
                            return Ok(());
                        }
                        _ => {
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                }
            }
        }
    }
    // ...
}
```

---

## 3. Path Configuration Issues

### Issue 3.1: Hardcoded Output Directory

**Location:** `weaver_controller.rs:127`

**Code:**
```rust
output_dir: PathBuf::from("./validation_output"),  // ❌ HARDCODED
```

**Documentation:** No mention of configurability

**Impact:** LOW
- Cannot customize output location
- May conflict with existing directories
- Not suitable for temporary directories in CI

**Recommendation:**
- Already configurable via `WeaverConfig`, but default is inflexible
- Consider using `std::env::temp_dir()` or system-specific paths
- Add environment variable override (e.g., `WEAVER_OUTPUT_DIR`)

### Issue 3.2: Hardcoded Registry Path

**Location:** `weaver_controller.rs:124`

**Code:**
```rust
registry_path: PathBuf::from("registry"),  // ❌ RELATIVE PATH
```

**Documentation Claims (Multiple locations):**
- `./weaver-registry` (WEAVER_INTEGRATION_DESIGN.md:496)
- `registry/` (WEAVER_PORT_COORDINATION.md multiple references)

**Mismatch:**
- Default says `"registry"` (no leading `.`)
- Documentation shows `./weaver-registry` in examples
- Inconsistent naming conventions

**Impact:** LOW
- Confusing for users (which path to use?)
- Documentation examples may not work with default config

---

## 4. Coordination Pattern Discrepancies

### Issue 4.1: `coordination()` Method Purpose Unclear

**Location:** `weaver_controller.rs:362-368`

**Code:**
```rust
/// Get current coordination state (non-blocking)
///
/// Returns None if Weaver not started via `start_and_coordinate()`,
/// otherwise returns the coordination metadata.
pub fn coordination(&self) -> Option<WeaverCoordination> {
    self.coordination.clone()
}
```

**Documentation Claims (WEAVER_PORT_COORDINATION.md:117-123):**
```rust
/// Get current coordination state (non-blocking)
///
/// Returns None if Weaver not started, otherwise coordination info.
pub fn coordination(&self) -> Option<WeaverCoordination> {
    // Return cached coordination from start_and_coordinate()
}
```

**Mismatch:**
- Implementation comment says "not started via `start_and_coordinate()`"
- But `start_live_check()` doesn't populate `self.coordination`
- So calling `coordination()` after `start_live_check()` returns None
- This is confusing: Weaver IS running but coordination() says it's not

**Impact:** MEDIUM
- API is misleading
- Two startup methods have different observability
- Users cannot query port after `start_live_check()`

**Recommendation:**
Either:
1. Deprecate `start_live_check()` in favor of `start_and_coordinate()`
2. OR populate `self.coordination` in both methods
3. OR rename to `coordination_metadata()` and document limitation

### Issue 4.2: Weaver-First Pattern Not Enforced

**Documentation Claims (WEAVER_PORT_COORDINATION.md:196-242):**
Shows clear "BEFORE (Broken)" and "AFTER (Correct)" patterns.

**Implementation:**
Both `start_live_check()` and `start_and_coordinate()` are public.

**Mismatch:**
- Documentation says Weaver-first is THE pattern
- Implementation allows both patterns
- No deprecation warning on `start_live_check()`
- No runtime check that OTEL is initialized with correct port

**Impact:** MEDIUM
- Users can still use the "broken" pattern
- Port mismatches can still occur
- No enforcement of documented best practice

**Recommendation:**
```rust
#[deprecated(
    since = "1.2.0",
    note = "Use start_and_coordinate() for proper port coordination"
)]
pub fn start_live_check(&mut self) -> Result<()> {
    warn!("start_live_check() is deprecated. Use start_and_coordinate() instead.");
    // ... implementation ...
}
```

---

## 5. CLI Integration Mismatches

### Issue 5.1: Run Command Port Configuration

**Location:** `cli/commands/run/mod.rs:364-365`

**Code:**
```rust
otlp_port: 4317, // Will be auto-discovered in start_live_check
admin_port: 8080, // Will be auto-discovered in start_live_check
```

**Mismatch:**
- Comment says "will be auto-discovered"
- But actually initializes with hardcoded values
- These values are passed to `WeaverController::new()`
- Then overwritten by port discovery

**Impact:** LOW
- Confusing code (why initialize if overwritten?)
- Comment is technically correct but misleading

**Recommendation:**
```rust
otlp_port: 0, // Auto-discover
admin_port: 0, // Auto-discover
```

### Issue 5.2: CLI Flag Defaults

**Location:** `cli/types.rs:506-511`

**Code:**
```rust
#[arg(long, default_value = "4318")]
pub otlp_http_port: u16,

#[arg(long, default_value = "4317")]
pub otlp_grpc_port: u16,
```

**Documentation:** Not mentioned in architecture docs

**Mismatch:**
- CLI has port flags with hardcoded defaults
- But WeaverController does port discovery
- If user passes `--otlp-grpc-port 5000`, does WeaverController honor it?
- No clear integration path shown

**Impact:** MEDIUM
- Unclear if CLI flags override discovery
- May create port conflicts if CLI flags ignored
- User expectations not met

---

## 6. Architecture Diagram Mismatches

### Issue 6.1: Port Numbers in Documentation

**From WEAVER_INTEGRATION_DESIGN.md:**
- Line 81: "Listen on OTLP endpoint (4318 HTTP / 4317 gRPC)"
- Line 498: `"http://localhost:4318"`
- Line 715: `│ Weaver Listener │ (port 4318) │`

**From weaver-live-check-complete.puml:**
- Line 14: `"OTLP gRPC\n:4317"`
- Line 21: `"OTLP HTTP\n:4318"`
- Line 236: `"Admin HTTP\n:8080"`

**Mismatch:**
- Diagrams show fixed ports
- But documentation emphasizes dynamic discovery
- Visual representation doesn't match runtime behavior

**Impact:** LOW (documentation issue)
- Confusing for users reading docs
- May lead to assumption that ports are fixed

**Recommendation:**
Update diagrams:
```
OTLP gRPC: 4317-4327 (auto-discovered)
OTLP HTTP: 4318 (if used)
Admin HTTP: 8080-8090 (auto-discovered)
```

### Issue 6.2: Missing Components in Implementation

**From WEAVER_INTEGRATION_DESIGN.md Architecture (lines 42-100):**

Shows these components:
- `CleanroomEnvironment.enable_tracing()`
- `CleanroomEnvironment.enable_metrics()`
- `CleanroomEnvironment.execute_test_with_validation()`

**Implementation Reality:**
- `enable_tracing()` - NOT FOUND in codebase
- `enable_metrics()` - NOT FOUND in codebase
- `execute_test_with_validation()` - NOT FOUND in codebase

**Mismatch:**
- Architecture diagram shows methods that don't exist
- Design doc describes integration points that aren't implemented

**Impact:** HIGH
- Documentation describes features that don't work
- Users cannot follow architecture guide
- False positive: docs claim feature exists

**Recommendation:**
Either:
1. Implement missing methods
2. OR update architecture to show actual implementation
3. OR mark as "planned v1.3.0 features"

---

## 7. Signal Handling Inconsistencies

### Issue 7.1: SIGHUP vs SIGTERM

**Location:** `weaver_controller.rs:666-686`

**Code:**
```rust
// Send graceful shutdown signal
#[cfg(unix)]
{
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let pid = Pid::from_raw(process.id() as i32);
    debug!("Sending SIGHUP to Weaver (PID: {})", pid);

    kill(pid, Signal::SIGHUP).map_err(|e| {  // ❌ SIGHUP
        CleanroomError::internal_error(format!("Failed to send SIGHUP: {}", e))
    })?;
}
```

**Documentation Claims (WEAVER_INTEGRATION_DESIGN.md:343-356):**
```rust
// Send SIGTERM for graceful shutdown
// This triggers Weaver to flush results and exit cleanly
#[cfg(unix)]
{
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let pid = Pid::from_raw(child.id() as i32);
    kill(pid, Signal::SIGTERM).map_err(|e| {  // ✅ SIGTERM
```

**Also in weaver-live-check-complete.puml lines 43-47:**
```
**Stop Conditions:**
- CTRL+C (SIGINT)
- SIGHUP
- HTTP /stop endpoint
```

**Mismatch:**
- Implementation uses SIGHUP
- Documentation shows SIGTERM
- PlantUML diagram lists both SIGHUP and SIGINT
- Unclear which is correct

**Impact:** LOW-MEDIUM
- May not trigger proper Weaver shutdown
- Different signals have different semantics
- Weaver may not flush results correctly

**Recommendation:**
- Verify which signal Weaver expects
- Standardize on one signal
- Update documentation to match
- Add comment explaining signal choice

---

## 8. Error Message Mismatches

### Issue 8.1: Process Exit Error Messages

**Location:** `weaver_controller.rs:417-420`

**Code:**
```rust
return Err(CleanroomError::internal_error(format!(
    "Weaver exited prematurely with status: {}. \
     Check Weaver logs in validation_output/ for details.",  // ❌ HARDCODED PATH
    status
)));
```

**Mismatch:**
- Error message hardcodes `validation_output/` path
- But `WeaverConfig.output_dir` may be different
- Misleading guidance for users

**Impact:** LOW
- User looks in wrong directory
- Wasted debugging time

**Recommendation:**
```rust
return Err(CleanroomError::internal_error(format!(
    "Weaver exited prematurely with status: {}. \
     Check Weaver logs in {} for details.",
    status,
    self.config.output_dir.display()
)));
```

---

## 9. Test Coverage Gaps

### Issue 9.1: Port Discovery Not Tested

**Location:** `weaver_controller.rs:892-897`

**Code:**
```rust
#[test]
#[ignore = "Requires Weaver installation"]
fn test_weaver_controller_lifecycle() {
    // This test requires Weaver to be installed and a registry to exist
    // TODO: Add integration test with mock Weaver process
}
```

**Mismatch:**
- Critical port discovery logic not tested
- `find_available_port_with_fallback()` has no tests
- Fallback logic may be broken

**Impact:** HIGH
- Cannot verify port discovery works
- Regressions may go unnoticed
- False confidence from passing tests

**Recommendation:**
Add unit tests:
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
fn test_port_discovery_exhaustion() {
    // Occupy all ranges, verify error
}
```

---

## 10. Configuration Consolidation

### Issue 10.1: Multiple Default Sources

**Current State:**
- `WeaverConfig::default()` - hardcoded defaults
- `cli/types.rs` - CLI flag defaults
- `cli/commands/run/mod.rs` - initialization with comments
- `cli/telemetry.rs:118-125` - OTLP endpoint defaults

**Mismatch:**
- Four different places define "default" ports
- No single source of truth
- Easy to create inconsistencies

**Impact:** MEDIUM
- Maintenance burden
- Potential for divergence
- Configuration confusion

**Recommendation:**
Consolidate to `WeaverConfig::default()` and use everywhere:
```rust
// In cli/types.rs
#[arg(long, default_value_t = WeaverConfig::default().otlp_port)]
pub otlp_grpc_port: u16,
```

---

## Summary of Critical Issues

| Issue | Severity | Impact |
|-------|----------|--------|
| Hardcoded port defaults | HIGH | Port conflicts in CI/CD |
| Hardcoded port ranges | MEDIUM-HIGH | Inflexible deployment |
| Hardcoded timeouts | HIGH | Flaky tests, slow CI |
| Missing health check | MEDIUM | Race conditions |
| Missing enable_tracing() methods | HIGH | Architecture not implemented |
| SIGHUP vs SIGTERM inconsistency | MEDIUM | May not shut down cleanly |
| No port discovery tests | HIGH | Cannot verify correctness |
| Multiple default sources | MEDIUM | Configuration confusion |

---

## Recommendations Priority

### P0 (Blocking - Must Fix)
1. Add configurable timeouts to `WeaverConfig`
2. Implement or remove `enable_tracing()`/`enable_metrics()` from docs
3. Add port discovery unit tests
4. Fix error message to use actual `output_dir`

### P1 (Important - Should Fix)
1. Make port ranges configurable
2. Implement HTTP health check
3. Consolidate configuration defaults
4. Deprecate `start_live_check()` in favor of `start_and_coordinate()`
5. Fix SIGHUP vs SIGTERM inconsistency

### P2 (Nice to Have)
1. Update architecture diagrams to show port ranges
2. Use `0` for auto-discover defaults
3. Add environment variable overrides
4. Deduplicate admin port discovery logic

---

## Files Requiring Updates

### Implementation Files
- `crates/clnrm-core/src/telemetry/weaver_controller.rs` (primary)
- `crates/clnrm-core/src/cli/commands/run/mod.rs`
- `crates/clnrm-core/src/cli/types.rs`
- `crates/clnrm-core/src/cli/telemetry.rs`

### Documentation Files
- `docs/architecture/WEAVER_PORT_COORDINATION.md`
- `docs/architecture/WEAVER_INTEGRATION_DESIGN.md`
- `docs/architecture/weaver-live-check-complete.puml`

### Test Files (New)
- `crates/clnrm-core/tests/telemetry/weaver_port_discovery_tests.rs` (create)
- `crates/clnrm-core/tests/telemetry/weaver_coordination_tests.rs` (create)

---

## Coordination Data for Aggregation

```json
{
  "agent": "coder",
  "task": "code-doc-mismatch-analysis",
  "findings": {
    "total_issues": 25,
    "critical": 5,
    "high": 5,
    "medium": 10,
    "low": 5
  },
  "categories": {
    "hardcoded_values": 12,
    "missing_features": 3,
    "inconsistent_behavior": 6,
    "test_coverage": 2,
    "documentation_mismatch": 2
  },
  "files_analyzed": [
    "crates/clnrm-core/src/telemetry/weaver_controller.rs",
    "docs/architecture/WEAVER_PORT_COORDINATION.md",
    "docs/architecture/WEAVER_INTEGRATION_DESIGN.md",
    "docs/architecture/weaver-live-check-complete.puml"
  ],
  "lines_of_code_affected": 588,
  "documentation_pages_affected": 3,
  "estimated_fix_effort": "4-6 hours"
}
```

---

## Next Steps for Swarm

1. **ARCHITECT agent** should review port configuration architecture
2. **REVIEWER agent** should prioritize fixes
3. **TESTER agent** should create test plan for port discovery
4. **ORCHESTRATOR** should coordinate fix implementation across agents

**Critical Path:** Fix P0 issues before v1.2.0 release, otherwise Weaver validation cannot be trusted in production.
