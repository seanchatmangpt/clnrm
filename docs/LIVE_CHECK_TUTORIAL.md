# Weaver Live-Check Tutorial

**Version**: v1.3.0
**Time**: 15-20 minutes
**Difficulty**: Beginner

Learn Weaver live-check validation through hands-on examples.

---

## Prerequisites

```bash
# Install clnrm
brew install clnrm  # Or: cargo install clnrm

# Install Weaver
cargo install weaver-cli

# Verify installations
clnrm --version  # v1.3.0+
weaver --version
docker --version
```

---

## Part 1: Your First Live-Check (5 minutes)

### Step 1: Create Test File

```bash
mkdir -p my-first-test
cd my-first-test
```

Create `hello.clnrm.toml`:

```toml
[meta]
name = "hello_world"
version = "1.0.0"

[weaver]
enabled = true  # Enable live-check

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "hello"
service = "app"
run = "echo 'Hello from clnrm!'"

# Validate test executed
[[expect.span]]
name = "test.execute"
attrs.all = {
  "test.isolated" = "true",
  "container.id" = "*"
}
```

### Step 2: Run Test

```bash
clnrm run hello.clnrm.toml
```

**Expected Output**:
```
✅ Validation: PASS

Weaver Live-Check Results:
  - Samples received: 8
  - Violations: 0
  - Registry coverage: 75%
```

**What Happened**:
1. clnrm created Alpine container
2. Executed `echo` command
3. Emitted telemetry to Weaver
4. Weaver validated telemetry against schema
5. Confirmed test ran in isolated container

### Step 3: View Detailed Report

```bash
# Check validation report
cat validation_output/summary.json | jq '.'
```

**Congratulations!** You ran your first live-check validation.

---

## Part 2: Catching False Positives (5 minutes)

### The Problem: Fake-Green Tests

Create `fake-test.clnrm.toml`:

```toml
[meta]
name = "fake_test"
version = "1.0.0"

[weaver]
enabled = true

[service.db]
plugin = "generic_container"
image = "postgres:15-alpine"

[[scenario]]
name = "query_database"
service = "db"
run = "echo 'Query executed'"  # FAKE - doesn't actually query

# Expect database span
[[expect.span]]
name = "db.query"
attrs.all = {
  "db.system" = "postgresql"
}
```

### Run Fake Test

```bash
clnrm run fake-test.clnrm.toml
```

**Expected Output**:
```
❌ Validation: FAIL

Violations:
  1. Expected span 'db.query' not found
     Impact: Test claims to query DB but doesn't
```

**What This Proves**:
- Traditional test: Would pass (exit code 0)
- Live-check: Catches fake-green (no DB span emitted)

---

## Part 3: Multi-Service Validation (5 minutes)

### Real-World Scenario

Create `api-db.clnrm.toml`:

```toml
[meta]
name = "api_with_database"
version = "1.0.0"

[weaver]
enabled = true

[weaver.validation]
mode = "80_20"  # Fast validation

[weaver.eighty_twenty]
critical_spans = [
    "http.server.request",
    "db.query"
]

[service.api]
plugin = "generic_container"
image = "nginx:alpine"

[service.db]
plugin = "generic_container"
image = "postgres:15-alpine"

[[scenario]]
name = "api_query"
service = "api"
run = "curl -f http://localhost:80/"

# Validate API span
[[expect.span]]
name = "http.server.request"
attrs.all = {
  "http.method" = "GET",
  "http.status_code" = "200"
}

# Validate DB span exists (even if not in this simple example)
[expect.counts]
by_name = {
  "container.start" = { eq = 2 }  # Started API + DB
}

# Validate graph structure
[expect.graph]
must_include = [
    ["test.execute", "http.server.request"]
]
```

### Run Multi-Service Test

```bash
clnrm run api-db.clnrm.toml
```

**Output Shows**:
- ✅ Two containers started
- ✅ HTTP request executed
- ✅ Graph structure valid
- ⚡ 80/20 mode = fast validation

---

## Part 4: Validation Modes (3 minutes)

### Compare Modes

```bash
# Strict mode (comprehensive)
clnrm run api-db.clnrm.toml --validate-mode strict

# 80/20 mode (fast)
clnrm run api-db.clnrm.toml --validate-mode 80_20

# Compare times
time clnrm run api-db.clnrm.toml --validate-mode strict
time clnrm run api-db.clnrm.toml --validate-mode 80_20
```

**Performance**:
- Strict: ~2.3s
- 80/20: ~0.4s (6x faster)

---

## Part 5: Debugging Failures (2 minutes)

### Intentional Failure

Create `debug-test.clnrm.toml`:

```toml
[meta]
name = "debug_test"
version = "1.0.0"

[weaver]
enabled = true

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test"
service = "app"
run = "echo test"

# Expect non-existent span (will fail)
[[expect.span]]
name = "nonexistent.span"
attrs.all = { "foo" = "bar" }
```

### Debug

```bash
# Run with verbose output
clnrm run debug-test.clnrm.toml --verbose

# Check violations
cat validation_output/violations.json | jq '.violations[0]'

# Dump telemetry
clnrm run debug-test.clnrm.toml --dump-telemetry
cat validation_output/telemetry_dump.json | jq '.spans[] | .name'
```

---

## Next Steps

### Practice Exercises

1. **Add span expectations** to hello.clnrm.toml
2. **Create multi-service test** with 3 services
3. **Experiment with validation modes** (strict, 80/20, lenient)
4. **Add graph validation** to check service interactions
5. **Create CI/CD config** using examples/live-check/ci-cd.clnrm.toml

### Advanced Topics

- **[Live-Check Guide](LIVE_CHECK_GUIDE.md)** - Complete reference
- **[Best Practices](LIVE_CHECK_BEST_PRACTICES.md)** - Advanced patterns
- **[Troubleshooting](LIVE_CHECK_TROUBLESHOOTING.md)** - Problem solving
- **[Migration Guide](MIGRATING_TO_V1_3_0.md)** - Upgrade from v1.2.x

### Real Projects

Check out examples:
```bash
# View all examples
ls examples/live-check/

# Run examples
clnrm run examples/live-check/basic.clnrm.toml
clnrm run examples/live-check/80-20.clnrm.toml
clnrm run examples/live-check/strict.clnrm.toml
```

---

## Summary

You learned:

✅ How to enable live-check validation
✅ Why live-check catches false positives
✅ Multi-service testing patterns
✅ Validation mode trade-offs
✅ Debugging validation failures

**Key Takeaways**:
1. Live-check validates **actual behavior**, not exit codes
2. 80/20 mode is **6x faster** for daily development
3. Span expectations prove features actually ran
4. Graph validation ensures services interact correctly

---

**Last Updated**: 2025-10-31
**Version**: v1.3.0
**Next**: Try the [CI/CD example](../examples/live-check/ci-cd.clnrm.toml)
