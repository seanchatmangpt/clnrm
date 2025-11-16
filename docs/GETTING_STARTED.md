# Getting Started with clnrm - 5 Minute Quick Start

**Time estimate: 5 minutes**
**Goal: Run your first test and understand the basics**

This is the fastest way to get clnrm working. After this, you'll have a working test and understand the core concepts.

## Prerequisites

- **Docker or Podman** installed and running
- **Rust 1.70+** (only if building from source; Homebrew installation includes pre-built binary)
- **Homebrew** (optional, for easy installation)

## Step 1: Install clnrm (1 minute)

### Option A: Homebrew (Easiest)
```bash
brew tap seanchatmangpt/clnrm
brew install clnrm
```

### Option B: Cargo
```bash
cargo install clnrm
```

### Verify Installation
```bash
clnrm --version
# Output: clnrm 1.4.1
```

## Step 2: Initialize Your First Project (1 minute)

```bash
# Create a directory for your tests
mkdir my-tests
cd my-tests

# Initialize the project
clnrm init

# This creates:
# - .clnrm/           (configuration directory)
# - tests/            (test files directory)
# - tests/simple.clnrm.toml  (example test)
```

## Step 3: Explore the Example Test (1 minute)

Look at the generated test file:

```bash
cat tests/simple.clnrm.toml
```

You'll see something like:

```toml
[meta]
name = "simple_container_test"
description = "A simple test that runs a basic container"
version = "1.0.0"

[service.alpine]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "container_runs_successfully"
service = "alpine"
run = "echo 'Hello from clnrm!'"

[expect.output]
stdout = "Hello from clnrm!"
```

**What this test does:**
1. Defines a service using Alpine Linux container
2. Runs `echo` command in that container
3. Validates the output matches "Hello from clnrm!"

## Step 4: Run Your First Test (1 minute)

```bash
clnrm run
```

You should see output like:
```
Testing simple_container_test...
✅ container_runs_successfully: PASSED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 1 test
Passed: 1
Failed: 0
```

**Congratulations!** You just ran your first clnrm test! 🎉

## Step 5: Understand What Happened (1 minute)

Here's what clnrm did:

```
Your Test File (.clnrm.toml)
        ↓
clnrm parse → TOML configuration
        ↓
clnrm run → Start Docker container (Alpine)
        ↓
         → Run command inside: `echo 'Hello from clnrm!'`
        ↓
         → Capture output
        ↓
         → Validate output matches expectation
        ↓
✅ Test passes!
```

## Key Concepts

### Services
A service is a containerized application. In the example:
```toml
[service.alpine]
plugin = "generic_container"     # What kind of service
image = "alpine:latest"          # Which Docker image
```

### Scenarios
A scenario is a test step that runs in a service. In the example:
```toml
[[scenario]]
name = "container_runs_successfully"
service = "alpine"               # Which service to run in
run = "echo 'Hello from clnrm!'" # What command to run
```

### Expectations
Expectations validate what actually happened. In the example:
```toml
[expect.output]
stdout = "Hello from clnrm!"     # What output we expect
```

## What You Can Do Now

Now that you have clnrm running, you can:

1. **Modify the test** — Change the command or expectation and run again
2. **Add more tests** — Create new `.clnrm.toml` files in the `tests/` directory
3. **Speed it up** — See [Tutorial 2: Container Pooling](docs/tutorials/02-container-pooling/) for 80% faster tests
4. **Validate behavior** — See [Tutorial 3: Weaver Validation](docs/tutorials/03-weaver-validation/) to catch false positives
5. **Test databases** — Create tests with PostgreSQL, MongoDB, etc.
6. **Test APIs** — Create integration tests with real services

## Next Steps

### Quick Wins (Pick One)
- **Speed up your tests** → [Container Pooling Tutorial](docs/tutorials/02-container-pooling/) (10 min)
- **Solve a specific problem** → [How-To Guides](docs/how-to/) (various)
- **Understand the architecture** → [Architecture Explanation](docs/explanation/architecture.md)

### Learn More
- **Complete Getting Started** → [Full Tutorial](docs/tutorials/01-getting-started/)
- **Write better tests** → [TOML Configuration Reference](docs/reference/toml-schema.md)
- **All CLI commands** → [CLI Reference](docs/reference/cli.md)

---

## Troubleshooting

### Error: "Docker daemon not found"
**Problem:** Docker isn't installed or not running.

**Solution:**
```bash
# Install Docker
brew install docker  # or download from docker.com

# Start Docker
open -a Docker  # On macOS
# Or start Docker Desktop normally
```

### Error: "command not found: clnrm"
**Problem:** Installation didn't complete properly.

**Solution:**
```bash
# If using Homebrew
brew uninstall clnrm
brew tap seanchatmangpt/clnrm
brew install clnrm
clnrm --version

# If using Cargo
cargo install clnrm --force
clnrm --version
```

### Error: "image not found"
**Problem:** Docker can't download the Alpine image.

**Solution:**
```bash
# Ensure you have internet connection
ping docker.io

# Pull the image manually
docker pull alpine:latest

# Then try running the test again
clnrm run
```

---

## Questions?

- **How do I test a database?** → See [How-To: Database Testing](docs/how-to/)
- **How do I test an API?** → See [How-To: API Testing](docs/how-to/)
- **What's a "scenario"?** → See [TOML Reference](docs/reference/toml-schema.md)
- **How do I...?** → Search [How-To Guides](docs/how-to/)

---

**Ready for more?** Start with [Container Pooling Tutorial](docs/tutorials/02-container-pooling/) to make your tests 80% faster!
