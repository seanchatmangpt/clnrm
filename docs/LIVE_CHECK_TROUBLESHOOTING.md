# Weaver Live-Check Troubleshooting

**Version**: v1.3.0
**Last Updated**: 2025-10-31

Quick reference for resolving common Weaver live-check issues.

---

## Quick Diagnosis

```bash
# Run with debug output
clnrm run tests/test.clnrm.toml --verbose --log-level debug

# Check Weaver process
ps aux | grep weaver
lsof -i :4317  # OTLP port
lsof -i :8080  # Admin port

# Verify configuration
clnrm validate tests/test.clnrm.toml
```

---

## Common Issues

### 1. Zero Samples Received

**Symptom**: Validation passes but `sample_count = 0`

**Diagnosis**:
```bash
clnrm run tests/test.clnrm.toml --verbose 2>&1 | grep -i "weaver\|otlp\|sample"
```

**Causes & Fixes**:

| Cause | Fix |
|-------|-----|
| OTEL not configured | Add `[otel] exporter = "otlp-http"` |
| Weaver not enabled | Add `[weaver] enabled = true"` |
| Wrong endpoint | Remove custom `endpoint` (auto-configured) |
| Tests finish too fast | Increase `flush_timeout_ms` |
| Firewall blocking | Check `lsof -i :4317` and firewall rules |

**Solution**:
```toml
[weaver]
enabled = true

[weaver.performance]
flush_timeout_ms = 5000  # Increase timeout

[otel]
exporter = "otlp-http"   # Don't set custom endpoint
```

---

### 2. Port Already in Use

**Symptom**: `Address already in use` error

**Diagnosis**:
```bash
lsof -i :4317  # Check OTLP port
lsof -i :8080  # Check admin port
ps aux | grep weaver  # Find running instances
```

**Fix**:
```toml
[weaver]
otlp_port = 0    # Use auto-discovery
admin_port = 0   # Use auto-discovery
```

**Manual cleanup**:
```bash
# Kill stray Weaver processes
pkill -9 weaver

# Or specific PID
kill -9 $(lsof -t -i:4317)
```

---

### 3. Missing Required Attribute

**Symptom**: `Required attribute 'X' does not exist`

**Example**:
```
❌ Missing Required Attribute: container.id
Span: test.execute
```

**Fix**:
```rust
// Ensure attribute is emitted
let span = trace_span!(
    "test.execute",
    container.id = %container_id,  // Add missing attribute
    test.isolated = true
);
```

**Schema check**:
```bash
# Verify schema requirements
cat registry/core/test_execution.yaml | grep -A5 "container.id"
```

---

### 4. Schema Not Found

**Symptom**: `Schema not found in registry`

**Diagnosis**:
```bash
# Check registry path
ls -la registry/

# List schemas
clnrm registry list

# Check TOML config
grep -A3 "\[weaver\]" tests/test.clnrm.toml
```

**Fixes**:
```toml
# Use correct relative path
[weaver]
registry_path = "./registry"

# Or absolute path
registry_path = "/absolute/path/to/registry"
```

---

### 5. Validation Timeout

**Symptom**: Weaver times out waiting for telemetry

**Diagnosis**:
```bash
# Check timeout settings
clnrm run tests/test.clnrm.toml --verbose | grep timeout
```

**Fix**:
```toml
[weaver.performance]
startup_timeout_ms = 10000  # Weaver startup
flush_timeout_ms = 5000     # Telemetry flush
```

---

### 6. Graph Structure Violation

**Symptom**: `Graph structure violation: edge not found`

**Example**:
```
❌ Expected edge: http.server.request → db.query
   Found: http.server.request (no children)
```

**Causes**:
1. Parent-child relationship not set
2. Spans emitted in wrong context
3. Span names don't match schema

**Fix**:
```rust
// Ensure proper nesting
let http_span = trace_span!("http.server.request");
let _http_guard = http_span.enter();

// DB query is child of HTTP span
let db_span = trace_span!("db.query");
let _db_guard = db_span.enter();
// ... query code ...
```

---

### 7. Wrong Attribute Type

**Symptom**: `Invalid attribute type`

**Example**:
```
❌ Attribute: container.destroyed_at
   Expected: string (ISO 8601)
   Actual: int (unix timestamp)
```

**Fix**:
```rust
// Use ISO 8601 format
span.set_attribute(
    "container.destroyed_at",
    "2025-10-31T14:23:45Z"  // ISO 8601
);

// Not this:
// span.set_attribute("container.destroyed_at", 1699123425);  // Unix
```

---

### 8. Performance Issues

**Symptom**: Validation takes too long

**Diagnosis**:
```bash
# Check mode
grep "mode =" tests/test.clnrm.toml

# Time validation
time clnrm run tests/test.clnrm.toml
```

**Fixes**:

| Issue | Solution |
|-------|----------|
| Strict mode too slow | Use 80/20 mode for development |
| Too many spans | Add sampling or critical spans only |
| Slow flush | Reduce `flush_timeout_ms` |
| Large telemetry | Set `max_samples` limit |

```toml
[weaver.validation]
mode = "80_20"  # 6x faster

[weaver.eighty_twenty]
critical_spans = ["test.execute", "container.start"]

[weaver.performance]
max_samples = 10000  # Limit samples
```

---

### 9. Weaver Installation Issues

**Symptom**: `weaver: command not found`

**Fix**:
```bash
# Install Weaver
cargo install weaver-cli

# Verify installation
weaver --version

# Check PATH
which weaver
echo $PATH

# Add to PATH if needed
export PATH="$HOME/.cargo/bin:$PATH"
```

---

### 10. Docker Container Issues

**Symptom**: Container failures affect validation

**Diagnosis**:
```bash
# Check Docker
docker ps -a

# Check logs
docker logs <container-id>

# Check Docker daemon
docker info
```

**Common fixes**:
```bash
# Restart Docker
sudo systemctl restart docker

# Clean up
docker system prune -af

# Check disk space
df -h
```

---

## Debug Workflows

### Workflow 1: Zero Samples Debug

```bash
# 1. Verify Weaver starts
clnrm run tests/test.clnrm.toml --verbose 2>&1 | grep "Weaver started"

# 2. Check OTLP endpoint
clnrm run tests/test.clnrm.toml --verbose 2>&1 | grep "OTLP endpoint"

# 3. Verify telemetry export
clnrm run tests/test.clnrm.toml --dump-telemetry

# 4. Check telemetry dump
cat validation_output/telemetry_dump.json | jq '.spans | length'
```

### Workflow 2: Validation Failures

```bash
# 1. Run with fail-fast off
clnrm run tests/test.clnrm.toml --no-fail-fast

# 2. Check violations
cat validation_output/violations.json | jq '.violations[]'

# 3. Check specific span
cat validation_output/telemetry_dump.json | \
  jq '.spans[] | select(.name == "test.execute")'

# 4. Verify schema
cat registry/core/test_execution.yaml
```

### Workflow 3: Port Conflicts

```bash
# 1. Find conflicting process
lsof -i :4317 -i :8080

# 2. Kill conflicts
pkill -9 weaver

# 3. Use auto-discovery
# Edit TOML: otlp_port = 0, admin_port = 0

# 4. Verify ports assigned
clnrm run tests/test.clnrm.toml --verbose 2>&1 | grep "port"
```

---

## Environment-Specific Issues

### macOS

**Issue**: Port binding fails
```bash
# Check if port requires admin
sudo lsof -i :4317

# Use non-privileged ports
# Ports >= 1024 don't require admin
```

**Issue**: Docker Desktop not running
```bash
# Start Docker Desktop
open -a Docker

# Wait for ready
until docker info >/dev/null 2>&1; do sleep 1; done
```

### Linux

**Issue**: Permission denied
```bash
# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Or use rootless Docker
dockerd-rootless-setuptool.sh install
```

**Issue**: Firewall blocking ports
```bash
# Allow OTLP port
sudo ufw allow 4317/tcp
sudo ufw allow 8080/tcp
```

### Windows (WSL)

**Issue**: WSL networking issues
```bash
# Use host networking
[weaver]
otlp_port = 4317  # Fixed port
admin_port = 8080
```

**Issue**: Path issues
```toml
# Use Windows-style paths in WSL
[weaver]
registry_path = "/mnt/c/Users/username/registry"
```

---

## Getting Help

### Before Asking

1. Check this troubleshooting guide
2. Search [GitHub Issues](https://github.com/seanchatmangpt/clnrm/issues)
3. Review [Live-Check Guide](LIVE_CHECK_GUIDE.md)
4. Try `--verbose` and `--log-level debug`

### Asking for Help

Include this information:

```bash
# System info
clnrm --version
weaver --version
docker --version
uname -a

# Run with debug
clnrm run tests/test.clnrm.toml --verbose --log-level debug 2>&1 | tee debug.log

# TOML config (sanitize secrets!)
cat tests/test.clnrm.toml

# Validation report
cat validation_output/summary.json
```

### Where to Ask

- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Discussions**: https://github.com/seanchatmangpt/clnrm/discussions
- **Documentation**: https://github.com/seanchatmangpt/clnrm/docs

---

## Quick Reference

| Problem | Quick Fix |
|---------|-----------|
| Zero samples | Add `[otel] exporter = "otlp-http"` |
| Port conflict | Use `otlp_port = 0` |
| Missing attribute | Add to span: `span.set_attribute(key, value)` |
| Schema not found | Fix `registry_path` in TOML |
| Timeout | Increase `flush_timeout_ms` |
| Too slow | Use `mode = "80_20"` |
| Wrong type | Use correct format (ISO 8601 for timestamps) |
| Graph violation | Fix span parent-child relationships |
| Weaver not found | `cargo install weaver-cli` |
| Docker issues | `docker system prune -af` |

---

**Last Updated**: 2025-10-31
**Version**: v1.3.0
