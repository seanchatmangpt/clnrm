# Failure Modes and Recovery Procedures

## Overview

This document catalogscritical failure modes for Weaver live-check integration and provides step-by-step recovery procedures. Every failure mode has been tested in the production validation suite.

## Failure Mode Categories

1. **Process Failures** - Weaver crashes, hangs, or fails to start
2. **Network Failures** - OTLP connectivity issues
3. **Resource Exhaustion** - Disk, memory, or CPU limits
4. **Configuration Failures** - Invalid registry, wrong ports
5. **Integration Failures** - Test/Weaver communication problems

---

## 1. Process Failures

### FM-001: Weaver Binary Not Found

**Symptom:**
```
Error: Failed to start Weaver (is it installed?): No such file or directory (os error 2)
```

**Root Cause:** Weaver not in PATH or not installed

**Recovery:**
```bash
# Check installation
which weaver

# Install via cargo
cargo install weaver-cli

# Or download binary
curl -L https://github.com/open-telemetry/weaver/releases/latest/download/weaver-$(uname -s)-$(uname -m) \
  -o /usr/local/bin/weaver
chmod +x /usr/local/bin/weaver

# Verify
weaver --version
```

**Prevention:** Add Weaver installation to CI/CD setup scripts

**Test Coverage:** `test_graceful_degradation_invalid_registry`

---

### FM-002: Weaver Crashes During Validation

**Symptom:**
```
Weaver exited prematurely with status: signal: 11 (SIGSEGV)
```

**Root Cause:**
- Corrupt telemetry data
- Out of memory
- Bug in Weaver

**Recovery:**
```bash
# Check Weaver logs
tail -f /tmp/clnrm_*/weaver.log

# Try with smaller dataset
OTEL_BSP_MAX_EXPORT_BATCH_SIZE=10 cargo test --features otel

# Update Weaver
cargo install weaver-cli --force

# Report issue
weaver --version
# Submit issue with minimal reproduction
```

**Prevention:**
- Validate telemetry before export
- Monitor Weaver memory usage
- Use latest stable Weaver version

**Test Coverage:** `test_crash_recovery_force_kill`

---

### FM-003: Zombie Weaver Processes

**Symptom:**
```
Error: Address already in use (os error 48)
```

**Root Cause:** Previous Weaver process not cleaned up

**Recovery:**
```bash
# Find zombie processes
ps aux | grep weaver

# Kill gracefully
pkill weaver

# Force kill if needed
pkill -9 weaver

# Verify cleanup
lsof -i :4317  # Should be empty
```

**Prevention:** WeaverController `Drop` impl handles cleanup

**Test Coverage:** `test_crash_recovery_force_kill`

---

### FM-004: Weaver Hangs During Shutdown

**Symptom:**
```
Error: Weaver did not stop within timeout (10s)
```

**Root Cause:**
- Large telemetry buffer being flushed
- File I/O blocking
- Network drain pending

**Recovery:**
```bash
# Identify hanging process
ps aux | grep weaver

# Send SIGTERM (graceful)
kill -TERM <PID>

# Wait 5 seconds, then SIGKILL
sleep 5
kill -9 <PID>

# Clean up resources
rm -rf /tmp/clnrm_*_test
```

**Prevention:** Increase timeout in high-volume scenarios

**Configuration:**
```rust
// In WeaverController
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30); // Increase for high volume
```

**Test Coverage:** `test_timeout_behavior_under_load`

---

## 2. Network Failures

### FM-005: OTLP Endpoint Unreachable

**Symptom:**
```
WARN  Connection to localhost:4317 failed: Connection refused
```

**Root Cause:** Weaver not listening or wrong port

**Recovery:**
```bash
# Verify Weaver is running
ps aux | grep weaver

# Check port binding
lsof -i :4317

# Verify firewall
sudo iptables -L -n | grep 4317  # Linux
sudo pfctl -sr | grep 4317       # macOS

# Test connectivity
nc -zv localhost 4317

# Start Weaver if not running
weaver registry live-check \
  --registry registry/ \
  --otlp-grpc-port 4317
```

**Prevention:**
- Health check before test execution
- Retry logic in OTLP exporter

**Test Coverage:** `test_network_failure_otlp_export_unavailable`

---

### FM-006: Network Partition

**Symptom:**
```
Telemetry export succeeded locally but Weaver shows 0% coverage
```

**Root Cause:** Network partition between test and Weaver

**Recovery:**
```bash
# Check network connectivity
ping localhost

# Verify Docker network (if containerized)
docker network inspect bridge

# Check routing
netstat -rn

# Restart network (macOS)
sudo ifconfig en0 down
sudo ifconfig en0 up

# Restart network (Linux)
sudo systemctl restart NetworkManager
```

**Prevention:** Use localhost for local testing

**Test Coverage:** `test_network_failure_otlp_export_unavailable`

---

### FM-007: OTLP Port Conflict

**Symptom:**
```
Error: Address already in use (os error 48)
```

**Root Cause:** Another service on port 4317

**Recovery:**
```bash
# Find conflicting process
lsof -i :4317
# Example output:
# COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
# weaver  12345 user    6u  IPv4  0x...      0t0  TCP *:4317 (LISTEN)

# Stop conflicting service
kill <PID>

# Or use different port
export OTLP_PORT=4327
```

**Prevention:** Use non-standard port in multi-tenant environments

**Configuration:**
```rust
let config = WeaverConfig {
    otlp_port: 4327,  // Non-standard port
    ..Default::default()
};
```

**Test Coverage:** `test_concurrent_controller_instances`

---

## 3. Resource Exhaustion

### FM-008: Disk Full

**Symptom:**
```
Error: No space left on device (os error 28)
```

**Root Cause:** Output directory full

**Recovery:**
```bash
# Check disk space
df -h

# Find large files
du -sh /tmp/clnrm_*

# Clean up
rm -rf /tmp/clnrm_*_test
rm -rf ./validation_output

# Verify space
df -h /tmp
```

**Prevention:**
- Monitor disk usage
- Implement log rotation
- Set max file size limits

**Configuration:**
```rust
// Add to WeaverConfig
pub struct WeaverConfig {
    pub max_output_size_mb: u64,  // Default: 1000 MB
}
```

**Test Coverage:** `test_resource_exhaustion_disk_full`

---

### FM-009: Out of Memory (OOM)

**Symptom:**
```
signal: 9 (SIGKILL)
# Or in dmesg:
Out of memory: Killed process 12345 (weaver) score 800
```

**Root Cause:**
- Too much telemetry buffered
- Memory leak
- Insufficient system memory

**Recovery:**
```bash
# Check system memory
free -h  # Linux
vm_stat # macOS

# Check Weaver memory usage
ps aux | grep weaver

# Kill memory-hungry processes
kill <PID>

# Increase swap
sudo swapon /swapfile

# Or reduce batch size
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=100
```

**Prevention:**
- Stream mode instead of buffering
- Implement backpressure
- Monitor memory usage

**Configuration:**
```rust
let config = WeaverConfig {
    stream: true,  // Enable streaming to reduce memory
    ..Default::default()
};
```

**Test Coverage:** `test_high_volume_telemetry_1000_spans_per_sec`

---

### FM-010: CPU Throttling

**Symptom:**
```
Validation took 5 minutes (expected < 30s)
CPU usage: 100% sustained
```

**Root Cause:**
- Too many concurrent validations
- Large telemetry volume
- Resource contention

**Recovery:**
```bash
# Check CPU usage
top
htop

# Reduce parallel tests
cargo test -- --test-threads=1

# Nice Weaver process
renice +10 $(pgrep weaver)

# Or reduce priority
sudo cpulimit -p $(pgrep weaver) -l 50  # Limit to 50% CPU
```

**Prevention:** Rate limiting in test execution

**Test Coverage:** `test_weaver_overhead_cpu_memory`

---

## 4. Configuration Failures

### FM-011: Invalid Registry Path

**Symptom:**
```
Error: Failed to load registry: No such file or directory
```

**Root Cause:** Registry path doesn't exist or wrong path

**Recovery:**
```bash
# Verify registry exists
ls -la registry/

# Check path
echo $REGISTRY_PATH

# Use absolute path
export REGISTRY_PATH=/absolute/path/to/registry

# Validate registry
weaver registry check --registry "$REGISTRY_PATH"
```

**Prevention:** Validate path before starting Weaver

**Configuration:**
```rust
// Validate in WeaverController::new()
if !config.registry_path.exists() {
    return Err(CleanroomError::configuration_error(
        format!("Registry not found: {:?}", config.registry_path)
    ));
}
```

**Test Coverage:** `test_graceful_degradation_invalid_registry`

---

### FM-012: Malformed Schema

**Symptom:**
```
Error: YAML parse error at line 15: unexpected token
```

**Root Cause:** Syntax error in schema file

**Recovery:**
```bash
# Validate schema
weaver registry check --registry registry/

# Find specific error
weaver registry check --registry registry/ --verbose

# Fix syntax (example)
# Before:
attributes
  - id: test.name

# After:
attributes:
  - id: test.name
```

**Prevention:** Schema validation in CI/CD

**CI Integration:**
```yaml
- name: Validate Schema
  run: weaver registry check --registry registry/
```

**Test Coverage:** Schema validation is prerequisite for all tests

---

### FM-013: Port Configuration Mismatch

**Symptom:**
```
Telemetry exported to :4317 but Weaver listening on :4327
Coverage: 0%
```

**Root Cause:** OTLP_PORT mismatch

**Recovery:**
```bash
# Check Weaver port
lsof -i -P | grep weaver

# Check OTLP exporter config
echo $OTEL_EXPORTER_OTLP_ENDPOINT

# Align ports
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4327
```

**Prevention:** Centralized configuration

**Configuration:**
```rust
// Ensure consistency
let port = config.otlp_port;
std::env::set_var(
    "OTEL_EXPORTER_OTLP_ENDPOINT",
    format!("http://localhost:{}", port)
);
```

**Test Coverage:** `test_different_otlp_endpoints`

---

## 5. Integration Failures

### FM-014: No Telemetry Exported

**Symptom:**
```
Weaver report: Coverage: 0.0%
No spans received
```

**Root Cause:**
- OTEL feature not enabled
- Exporter not configured
- Tests didn't emit telemetry

**Recovery:**
```bash
# Verify OTEL feature
cargo test --features otel

# Check OTLP exporter
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export RUST_LOG=debug,otel=trace

# Verify telemetry emission
cargo test --features otel -- --nocapture 2>&1 | grep "span"
```

**Prevention:** Always use `--features otel` in validation

**Test Coverage:** `test_real_clnrm_tests_with_weaver`

---

### FM-015: Schema/Telemetry Mismatch

**Symptom:**
```
Violation: Required attribute 'container.id' missing
Expected: string, Got: null
```

**Root Cause:** Code doesn't match schema requirements

**Recovery:**
```bash
# Review schema
cat registry/core/container_lifecycle.yaml

# Fix code to emit required attribute
# Before:
span.set_attribute("container.name", name);

# After:
span.set_attribute("container.id", id);  // Required
span.set_attribute("container.name", name);
```

**Prevention:** Type-safe builders from schema

**Code Generation:**
```bash
weaver registry generate rust \
  --registry registry/ \
  --output src/telemetry/generated/
```

**Test Coverage:** `test_end_to_end_validation_workflow`

---

### FM-016: Validation Report Not Found

**Symptom:**
```
WARN  Validation report not found at "/tmp/output/validation_report.json"
```

**Root Cause:**
- Weaver crashed before writing report
- Wrong output directory
- Permissions issue

**Recovery:**
```bash
# Check output directory
ls -la /tmp/clnrm_*/

# Check permissions
ls -ld /tmp/clnrm_*
# Should be: drwxr-xr-x

# Fix permissions
chmod 755 /tmp/clnrm_*

# Verify Weaver can write
touch /tmp/clnrm_test/test.txt
```

**Prevention:** Create output directory before starting Weaver

**Implementation:**
```rust
std::fs::create_dir_all(&config.output_dir)?;
```

**Test Coverage:** All tests validate report file creation

---

## Recovery Playbook

### Emergency Procedure

When validation completely fails:

```bash
#!/bin/bash
# emergency_cleanup.sh

echo "🚨 Emergency cleanup initiated"

# 1. Kill all Weaver processes
pkill -9 weaver
echo "✓ Weaver processes killed"

# 2. Clean up temporary files
rm -rf /tmp/clnrm_*
echo "✓ Temporary files removed"

# 3. Free ports
lsof -ti:4317 | xargs kill -9
lsof -ti:4318 | xargs kill -9
echo "✓ Ports freed"

# 4. Verify cleanup
echo ""
echo "Verification:"
ps aux | grep weaver || echo "✓ No Weaver processes"
lsof -i :4317 || echo "✓ Port 4317 free"
df -h /tmp | tail -1
echo ""
echo "✅ Emergency cleanup complete"
```

### Diagnostic Data Collection

When reporting issues:

```bash
#!/bin/bash
# collect_diagnostics.sh

mkdir -p diagnostics/

# System info
uname -a > diagnostics/system.txt
weaver --version > diagnostics/weaver_version.txt

# Process info
ps aux | grep weaver > diagnostics/processes.txt
lsof -i :4317 > diagnostics/network.txt

# Logs
cp -r /tmp/clnrm_* diagnostics/ 2>/dev/null
cp -r validation_output/ diagnostics/ 2>/dev/null

# Configuration
cargo tree | grep otel > diagnostics/dependencies.txt
env | grep -i otel > diagnostics/environment.txt

# Create archive
tar -czf diagnostics_$(date +%Y%m%d_%H%M%S).tar.gz diagnostics/
echo "Diagnostics collected: diagnostics_*.tar.gz"
```

---

## Monitoring and Alerting

### Health Checks

```rust
// Add to WeaverController
pub fn health_check(&self) -> Result<HealthStatus> {
    // Check if process is running
    // Check if port is listening
    // Check if admin API responds
}
```

### Metrics to Monitor

1. **Process Metrics:**
   - CPU usage < 10%
   - Memory usage < 200 MB
   - Process uptime

2. **Performance Metrics:**
   - Startup time < 5s
   - Shutdown time < 10s
   - Validation duration < 30s

3. **Telemetry Metrics:**
   - Spans/sec throughput
   - Drop rate = 0
   - Export errors = 0

4. **Validation Metrics:**
   - Violations count
   - Registry coverage %
   - Report generation time

### Alerting Rules

```yaml
alerts:
  - name: WeaverProcessDown
    condition: weaver_process_up == 0
    severity: critical
    action: restart

  - name: HighMemoryUsage
    condition: weaver_memory_mb > 200
    severity: warning
    action: investigate

  - name: ValidationFailures
    condition: weaver_violations > 0
    severity: critical
    action: block_deployment
```

---

## Testing Failure Modes

All failure modes are tested in `tests/production_validation/`:

```bash
# Test all failure modes
cargo test --test production_validation --features otel -- --ignored

# Test specific category
cargo test --test production_validation --features otel -- --ignored reliability
```

**Test Coverage Matrix:**

| Failure Mode | Test | Category |
|--------------|------|----------|
| FM-001 | test_graceful_degradation_invalid_registry | Reliability |
| FM-002 | test_crash_recovery_force_kill | Reliability |
| FM-003 | test_crash_recovery_force_kill | Reliability |
| FM-004 | test_timeout_behavior_under_load | Performance |
| FM-005 | test_network_failure_otlp_export_unavailable | Reliability |
| FM-007 | test_concurrent_controller_instances | Reliability |
| FM-008 | test_resource_exhaustion_disk_full | Reliability |
| FM-009 | test_high_volume_telemetry_1000_spans_per_sec | Performance |
| FM-010 | test_weaver_overhead_cpu_memory | Performance |
| FM-011 | test_graceful_degradation_invalid_registry | Reliability |
| FM-013 | test_different_otlp_endpoints | Integration |
| FM-014 | test_real_clnrm_tests_with_weaver | Integration |
| FM-015 | test_end_to_end_validation_workflow | Integration |

---

## Conclusion

Every failure mode has:
1. Clear symptom description
2. Root cause analysis
3. Step-by-step recovery procedure
4. Prevention strategy
5. Test coverage

**Key Principle:** Fail fast, fail informatively, recover automatically.

---

**Last Updated:** 2025-10-30
**Version:** 1.2.0
