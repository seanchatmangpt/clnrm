# Quick Start: clnrm in 5 Minutes

**Goal**: Run your first hermetic container test and see clnrm in action.

**Prerequisites**:
- Docker running (`docker ps` should work without errors)
- Rust 1.75+ installed
- 5 minutes of your time

---

## Step 1: Install clnrm (1 minute)

### Option A: From crates.io (recommended)
```bash
cargo install clnrm-cli
```

### Option B: From source (for contributors)
```bash
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm
cargo make build
```

**Verify installation**:
```bash
clnrm --version
# Expected: clnrm 2.1.0
```

---

## Step 2: Create Your First Test (1 minute)

Create a test file that validates an Ubuntu container can execute a simple command:

```bash
# Create test directory
mkdir -p my-first-test
cd my-first-test

# Initialize a new test specification
clnrm init hello.clnrm.toml
```

This creates `hello.clnrm.toml` with a basic template. **Edit it** to look like this:

```toml
# hello.clnrm.toml - Your first hermetic container test

[[tests]]
name = "ubuntu-echo-test"
image = "ubuntu:latest"
command = ["echo", "Hello from clnrm!"]

[tests.expectations]
exit_code = 0
stdout_contains = ["Hello from clnrm!"]
```

**What this does**:
- Pulls `ubuntu:latest` Docker image
- Runs `echo "Hello from clnrm!"` inside container
- Validates exit code is 0
- Validates stdout contains expected text

---

## Step 3: Run Your Test (2 minutes)

Execute the test specification:

```bash
clnrm run hello.clnrm.toml
```

**Expected output**:
```
🚀 clnrm v2.1.0 - Hermetic Container Testing Framework
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 Loading test specification: hello.clnrm.toml
✅ Specification valid: 1 test(s) found

🐳 Docker: Pulling ubuntu:latest...
✅ Image ready: ubuntu:latest

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪 Test 1/1: ubuntu-echo-test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🐳 Starting container: ubuntu:latest
▶  Executing: echo "Hello from clnrm!"
✅ Exit code: 0
✅ Stdout contains: "Hello from clnrm!"
✅ Test PASSED (duration: 1.23s)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Test Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Tests run:    1
Passed:       1 ✅
Failed:       0
Duration:     1.23s

✅ ALL TESTS PASSED
```

**Behind the scenes**:
1. clnrm pulls Docker image if not cached
2. Starts ephemeral container (hermetic isolation)
3. Executes command inside container
4. Validates exit code and stdout
5. Cleans up container automatically
6. Emits OpenTelemetry spans (if collector configured)

---

## Step 4: Validate Without Running (30 seconds)

Try a dry-run to validate your test specification without executing:

```bash
clnrm dry-run hello.clnrm.toml
```

**Expected output**:
```
🔍 Dry-run mode: Validating without execution
✅ Specification: hello.clnrm.toml
✅ Test count: 1
✅ Image: ubuntu:latest (available)
✅ All validations passed

🎯 Ready to run! Use: clnrm run hello.clnrm.toml
```

**What dry-run checks**:
- TOML syntax is valid
- Required fields are present
- Docker images are available (or can be pulled)
- Test expectations are well-formed

---

## Step 5: Explore More Commands (30 seconds)

Discover what else clnrm can do:

```bash
# See all available commands
clnrm --help

# Get help for a specific command
clnrm run --help

# Check framework health
clnrm health

# Validate a test specification
clnrm validate hello.clnrm.toml
```

**Common next steps**:
```bash
# Run all tests in a directory
clnrm run ./tests/

# Generate detailed report
clnrm report hello.clnrm.toml --output report.json

# Analyze OTEL traces (if collector running)
clnrm analyze traces

# Create a more complex test from template
clnrm template create --type integration
```

---

## What You Just Learned

✅ **Installed** clnrm CLI tool
✅ **Created** a hermetic container test specification
✅ **Executed** a test with Docker isolation
✅ **Validated** test configuration without running
✅ **Discovered** 26 available commands

**Time taken**: < 5 minutes ⚡

---

## Next Steps

### Beginner: Learn Test Specifications
- **Read**: [Test Specification Guide](../docs/TEST_SPECIFICATIONS.md)
- **Explore**: More complex test examples in `tests/`
- **Try**: Multi-step tests with setup/teardown phases

### Intermediate: Integrate with CI/CD
- **GitHub Actions**: Add clnrm to `.github/workflows/test.yml`
- **Exit codes**: Use in shell scripts (0 = pass, 1 = fail)
- **Reports**: Generate JUnit XML for CI platforms

### Advanced: Observability & Telemetry
- **OTEL**: Configure OpenTelemetry collector
- **Traces**: Analyze test execution traces
- **Metrics**: Track container startup times, test duration

---

## Troubleshooting

### Error: "Cannot connect to Docker daemon"
**Solution**:
```bash
# Check Docker is running
docker ps

# Start Docker Desktop (macOS/Windows)
# OR: sudo systemctl start docker (Linux)

# Verify permissions
docker run hello-world
```

### Error: "Failed to pull image"
**Cause**: Network issue or image doesn't exist

**Solution**:
```bash
# Manually pull to verify
docker pull ubuntu:latest

# Use cached image
clnrm run hello.clnrm.toml --offline
```

### Error: "Invalid TOML syntax"
**Solution**:
```bash
# Validate TOML separately
clnrm lint hello.clnrm.toml

# Use init to regenerate clean template
clnrm init hello-fixed.clnrm.toml
```

### Test fails but should pass
**Debug**:
```bash
# Run with verbose logging
RUST_LOG=debug clnrm run hello.clnrm.toml

# Record execution for replay
clnrm record hello.clnrm.toml --output debug.json

# Reproduce failed test
clnrm repro debug.json
```

---

## Example: Integration Test (Bonus 2 minutes)

Want to try something more advanced? Here's a multi-container test:

**File**: `integration.clnrm.toml`
```toml
[[tests]]
name = "redis-integration"
image = "redis:7-alpine"
command = ["redis-server", "--port", "6379"]

[tests.expectations]
exit_code = 0
stdout_contains = ["Ready to accept connections"]
timeout = 10  # seconds

[[tests]]
name = "redis-client-test"
image = "redis:7-alpine"
command = ["redis-cli", "-h", "localhost", "-p", "6379", "PING"]
depends_on = ["redis-integration"]

[tests.expectations]
exit_code = 0
stdout_contains = ["PONG"]
```

**Run it**:
```bash
clnrm run integration.clnrm.toml
```

This demonstrates:
- **Multi-container tests** with dependencies
- **Service readiness checks** (stdout_contains)
- **Timeout configuration** (10s limit)
- **Sequential execution** (depends_on relationship)

---

## Summary

In just 5 minutes, you've:
- ✅ Installed clnrm
- ✅ Created a hermetic container test
- ✅ Validated and executed the test
- ✅ Understood how Docker isolation works
- ✅ Explored command-line options

**clnrm makes container testing:**
- **Declarative**: TOML specifications, not bash scripts
- **Hermetic**: Isolated containers, reproducible results
- **Observable**: OTEL tracing, structured logging
- **Fast**: Parallel execution, incremental testing

**Ready for more?** Check out:
- [README](../README.md) - Full command reference
- [Test Specifications Guide](../docs/TEST_SPECIFICATIONS.md) - Advanced patterns
- [Constitution](../.specify/memory/constitution.md) - Development principles
- [Examples](../tests/) - Real-world test suites

---

**Questions or issues?** File an issue at: https://github.com/seanchatmangpt/clnrm/issues

**Version**: This quickstart is for clnrm v2.1.0
