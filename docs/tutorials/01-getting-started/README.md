# Tutorial 1: Getting Started (15 minutes)

**⏱ Estimated Time**: 15 minutes
**📋 Prerequisites**: Docker/Podman installed, basic CLI knowledge
**🎯 Learning Objectives**: Run your first test and understand core concepts

## What You'll Learn

By the end of this tutorial, you'll:
- ✅ Install clnrm successfully
- ✅ Understand the core concepts (services, scenarios, expectations)
- ✅ Create your first test from scratch
- ✅ Run it and interpret the results
- ✅ Know where to go next

---

## Step 1: Install clnrm (2 minutes)

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

If you see the version number, installation succeeded! ✅

---

## Step 2: Set Up Your Project (2 minutes)

Create a new directory for your tests:

```bash
mkdir my-clnrm-tests
cd my-clnrm-tests
```

Initialize the project:

```bash
clnrm init
```

This creates:
```
.
├── .clnrm/                    # Configuration directory
└── tests/
    └── simple.clnrm.toml      # Example test (you can delete this)
```

---

## Step 3: Understand the Core Concepts (3 minutes)

Before writing your first test, let's understand three key concepts:

### 1. **Services** — Containerized Applications

A service is a Docker container that your test will use. Example:

```toml
[service.my_api]
plugin = "generic_container"     # Use any Docker image
image = "alpine:latest"          # Which image to use
```

### 2. **Scenarios** — Test Steps

A scenario is what happens in that container. Example:

```toml
[[scenario]]
name = "api_starts"
service = "my_api"
run = "echo 'Hello from clnrm!'"
```

### 3. **Expectations** — Validations

Expectations verify what actually happened. Example:

```toml
[expect.output]
stdout = "Hello from clnrm!"     # What output we expect
```

**These three things together = A test!**

---

## Step 4: Write Your First Test (5 minutes)

Create a file: `tests/my-first-test.clnrm.toml`

```toml
# Test metadata
[meta]
name = "hello_world_test"
description = "My first clnrm test"
version = "1.0.0"

# Define a service (containerized application)
[service.hello]
plugin = "generic_container"      # Use a generic container
image = "alpine:latest"           # Use Alpine Linux
environment = {
  MESSAGE = "Hello from clnrm!"
}

# Define what the test does
[[scenario]]
name = "echo_message"
service = "hello"
run = "echo $MESSAGE"

# Define what we expect to happen
[expect.output]
stdout = "Hello from clnrm!"
```

### What This Test Does

```
1. Creates a Docker container from alpine:latest
2. Sets environment variable MESSAGE="Hello from clnrm!"
3. Runs: echo $MESSAGE
4. Validates output contains "Hello from clnrm!"
5. Destroys container (automatic cleanup)
```

---

## Step 5: Run Your Test (2 minutes)

```bash
clnrm run
```

You should see output like:

```
Testing hello_world_test...
Scenario: echo_message
  ✅ Output validation passed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Results:
  Total: 1 test
  Passed: 1
  Failed: 0
  Status: ✅ ALL PASSED
```

**Congratulations!** 🎉 You just ran your first clnrm test!

---

## Step 6: Modify and Learn (1 minute)

Let's experiment. Edit your test to change the message:

```toml
environment = {
  MESSAGE = "clnrm is awesome!"
}

[expect.output]
stdout = "clnrm is awesome!"
```

Run again:
```bash
clnrm run
```

See how the test still passes? You can modify the test and clnrm validates your changes.

Try intentionally breaking it:

```toml
[expect.output]
stdout = "This will not match"
```

```bash
clnrm run
```

Now you'll see:
```
❌ Output validation failed
Expected: "This will not match"
Got: "clnrm is awesome!"
```

This shows you how clnrm validates behavior!

---

## Key Concepts Explained

### Service Definition
- **plugin**: The type of service (generic_container, surrealdb, ollama, etc.)
- **image**: Which Docker image to use
- **environment**: Environment variables passed to container
- **Other options**: Ports, volumes, etc. (see reference docs)

### Scenario Execution
- **name**: Unique identifier for this test step
- **service**: Which service to run in
- **run**: Command to execute
- **timeout_ms**: How long to wait (default: 5000ms)

### Expectations
- **output**: stdout/stderr validation
- **span**: OpenTelemetry span validation (advanced)
- **graph**: Trace structure validation (advanced)
- **counts**: Counter validation (advanced)

### What Actually Happens

```
Your TOML File
    ↓
clnrm parses it
    ↓
Creates Docker container from "alpine:latest"
    ↓
Sets up environment variables
    ↓
Runs: echo $MESSAGE inside container
    ↓
Captures output
    ↓
Compares output to expectation
    ↓
Shows result: ✅ PASS or ❌ FAIL
    ↓
Automatically destroys container
```

---

## Next Steps

### Want to speed up your tests?
→ [Tutorial 2: Container Pooling](../02-container-pooling/) (10 min)

### Want to catch false positives?
→ [Tutorial 3: Weaver Validation](../03-weaver-validation/) (15 min)

### Want to test a real service (database, API)?
→ [How-To: Database Testing](../../how-to/database-testing.md)

### Want to run in CI/CD (GitHub Actions)?
→ [How-To: GitHub Actions](../../how-to/github-actions.md)

### Want to understand how it works?
→ [Explanation: Architecture](../../explanation/architecture.md)

---

## Troubleshooting

### Error: "Docker daemon not found"
**Problem**: Docker isn't installed or running

**Solution**:
```bash
# Install Docker (macOS)
brew install docker

# Or download from https://docker.com
# Then start Docker or Docker Desktop
```

### Error: "command not found: clnrm"
**Problem**: Installation didn't work

**Solution**:
```bash
# Try reinstalling
brew uninstall clnrm
brew tap seanchatmangpt/clnrm
brew install clnrm
clnrm --version
```

### Error: "image not found: alpine:latest"
**Problem**: Docker can't download the image

**Solution**:
```bash
# Pull the image manually
docker pull alpine:latest

# Then try running the test again
clnrm run
```

### Test runs but expectation fails
**Problem**: Output doesn't match expected

**Solution**:
```bash
# Check what the actual output is
clnrm run --verbose

# Adjust your expectation to match
# Or fix the command to produce expected output
```

---

## Summary

You now understand:
- ✅ **Services** — Docker containers defined in TOML
- ✅ **Scenarios** — Test steps that run in services
- ✅ **Expectations** — Validations that prove behavior
- ✅ **How clnrm works** — TOML → Docker → Validate

---

## Continue Learning

**Progression Path**:
1. ✅ **Tutorial 1** (You are here) — Basic concepts
2. **Tutorial 2** — Make tests 80% faster with container pooling
3. **Tutorial 3** — Add behavior validation with Weaver
4. **Tutorial 4** — Create custom service plugins
5. **Tutorial 5** — Add observability with OpenTelemetry

Or jump to:
- **How-To Guides** — Solve specific problems (parallel, CI/CD, databases, etc.)
- **Reference Docs** — Look up technical details (CLI, TOML, API)
- **Explanations** — Understand concepts deeply (architecture, design, principles)

---

**Congratulations on completing Tutorial 1!** 🎓

Next: [Tutorial 2: Container Pooling](../02-container-pooling/)
