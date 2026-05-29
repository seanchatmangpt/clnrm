# Command Categorization Reference - clnrm v2.1.0

**Purpose**: Map all 26 CLI commands into 5 feature categories for README organization
**Format**: Feature → Use Case → Commands → Examples

---

## Overview: 5 Categories × 5-6 Commands

```
┌─────────────────────────────────────────────────────────────────┐
│                    clnrm CLI Commands (26)                      │
├─────────────────────────┬──────────────────┬────────────────────┤
│  Test Execution (6)     │ Configuration (5) │ Observation (5)    │
├─────────────────────────┼──────────────────┼────────────────────┤
│  run                    │ init             │ spans              │
│  dry-run                │ validate         │ report             │
│  record                 │ lint             │ graph              │
│  repro                  │ fmt              │ health             │
│  stress                 │ render           │ live-check         │
│  self-test              │                  │                    │
├─────────────────────────┼──────────────────┼────────────────────┤
│  System Mgmt (5)        │ Development (5)  │                    │
├─────────────────────────┼──────────────────┤                    │
│  services               │ dev              │                    │
│  collector              │ template         │                    │
│  plugins                │ diff             │                    │
│  pull                   │ analyze          │                    │
│  (reserved: future)     │ (reserved: future)                    │
└─────────────────────────┴──────────────────┴────────────────────┘
```

---

## Category 1: Test Execution (6 commands)

**User Story**: "I want to execute, debug, and stress-test my container specifications"

### Command 1: run
**Description**: Execute tests from TOML specification
**When to use**: Running your test suite, validating container behavior
**Most frequent use case**: Daily testing workflow

**Syntax**:
```bash
clnrm run <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm run tests/container-lifecycle.clnrm.toml
```

**Options**:
- `--filter <PATTERN>` - Run only tests matching pattern
- `--parallel <N>` - Run N tests in parallel
- `--format <FORMAT>` - Output format (human, json, github)

**Expected output**:
```
✓ Test: basic_container
  Status: PASSED
  Duration: 1.2s

✓ 1 passed, 0 failed in 1.2s
```

---

### Command 2: dry-run
**Description**: Preview test execution without running containers
**When to use**: Validating configuration before committing resources, previewing large test suites
**Key insight**: Fast way to check for syntax/logic errors

**Syntax**:
```bash
clnrm dry-run <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm dry-run tests/container-lifecycle.clnrm.toml
```

**What it shows**:
- Validates TOML syntax
- Parses templates
- Lists all tests that would run
- Doesn't start containers
- Estimates resource usage

**Expected output**:
```
✓ Configuration valid
✓ Would execute 5 tests
✓ Estimated time: 15 seconds
✓ Resources: 2 CPU, 4GB RAM
```

---

### Command 3: record
**Description**: Record test results for baseline comparison
**When to use**: Creating golden outputs for regression testing, benchmarking
**Key insight**: Compare against recorded baseline in CI/CD

**Syntax**:
```bash
clnrm record <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm record tests/performance.clnrm.toml --output baseline.json
```

**Purpose**:
- Save test output as baseline
- Later use in `clnrm diff` for regression detection
- Track performance changes over time

---

### Command 4: repro
**Description**: Reproduce a specific test failure
**When to use**: Debugging test failures, iterating on fixes
**Key insight**: Replay exact environment of failed test

**Syntax**:
```bash
clnrm repro <TEST_ID> [OPTIONS]
```

**Quick example**:
```bash
clnrm repro failure-123 --verbose
```

**What it does**:
- Restores exact test environment
- Runs with same random seed
- Captures detailed traces
- Useful for intermittent failures

---

### Command 5: stress
**Description**: Run tests under load, chaos conditions, or repeated execution
**When to use**: Performance testing, reliability testing, chaos engineering
**Key insight**: Find edge cases and race conditions

**Syntax**:
```bash
clnrm stress <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm stress tests/container.clnrm.toml --iterations 100 --concurrent 10
```

**Test modes**:
- `--iterations N` - Run test N times
- `--concurrent N` - Run N tests in parallel
- `--chaos <MODE>` - Inject failures (kill container, drop packets, etc.)
- `--duration 1h` - Run for specified time

**Useful for**:
- Finding race conditions
- Performance regression detection
- Reliability testing (99.9% uptime)
- Resource leak detection

---

### Command 6: self-test
**Description**: Run clnrm's own test suite (dogfooding)
**When to use**: Validating clnrm installation, CI/CD validation
**Key insight**: Framework tests itself with real Docker containers

**Syntax**:
```bash
clnrm self-test [OPTIONS]
```

**Quick example**:
```bash
clnrm self-test --verbose
```

**What it validates**:
- Docker connectivity
- OTEL collector (if configured)
- Test execution engine
- Report generation
- Plugin system

**Expected output**:
```
Running clnrm self-tests...
✓ Docker backend: OK
✓ OTEL integration: OK
✓ Test execution: OK (5/5 passed)
✓ Report generation: OK
✓ Plugin system: OK

clnrm is healthy and ready to use
```

---

## Category 2: Configuration & Validation (5 commands)

**User Story**: "I want to create, validate, and maintain test specifications"

### Command 1: init
**Description**: Generate boilerplate TOML configuration
**When to use**: Starting a new test project, creating examples
**Most useful for**: Getting started quickly

**Syntax**:
```bash
clnrm init <PROJECT_NAME> [OPTIONS]
```

**Quick example**:
```bash
clnrm init my-tests
```

**What it generates**:
```
my-tests/
├── test.clnrm.toml          # Main test specification
├── README.md                 # Documentation template
└── examples/
    └── basic.clnrm.toml     # Simple example
```

**Generated test.clnrm.toml**:
```toml
[test]
name = "example_test"
image = "ubuntu:latest"

[[test.commands]]
exec = "echo 'Hello, World!'"
expected_exit_code = 0
```

---

### Command 2: validate
**Description**: Validate TOML configuration syntax and semantics
**When to use**: Before committing, before running, in CI pipeline
**Purpose**: Catch configuration errors early

**Syntax**:
```bash
clnrm validate <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm validate test.clnrm.toml
```

**Checks**:
- TOML syntax correctness
- Required fields present
- Image names valid
- Port numbers in valid range
- Template variables defined
- Circular dependencies

**Output on success**:
```
✓ Syntax valid
✓ Schema valid
✓ 5 tests found
✓ No errors
```

**Output on failure**:
```
✗ Validation failed
  Line 15: Required field "image" missing from [test]
  Line 20: Unknown variable "{{ undefined_var }}"
```

---

### Command 3: lint
**Description**: Check configuration best practices
**When to use**: Code review, before production deployment
**Purpose**: Enforce team standards

**Syntax**:
```bash
clnrm lint <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm lint test.clnrm.toml
```

**Checks**:
- Unused variables
- Hardcoded passwords
- Missing documentation
- Performance anti-patterns
- Resource limits not set

**Output**:
```
⚠ Warnings (3):
  Line 5: Variable 'unused_var' is never referenced
  Line 12: Missing documentation for test 'complex_test'
  Line 25: No timeout_seconds set (recommend 30)

✓ No errors, 3 warnings
```

---

### Command 4: fmt
**Description**: Auto-format TOML files (like `cargo fmt`)
**When to use**: Before commit, in CI pipeline
**Purpose**: Enforce consistent style across team

**Syntax**:
```bash
clnrm fmt <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm fmt test.clnrm.toml
```

**What it does**:
- Consistent indentation (2 spaces)
- Consistent key ordering
- Comment formatting
- Removes trailing whitespace

**Safety**:
- Only reformats, never changes logic
- Safe to run automatically
- Writes to same file (or --check to preview)

**Options**:
- `--check` - Preview changes without writing
- `--recursive` - Format all .clnrm.toml files in directory

---

### Command 5: render
**Description**: Render templated TOML and show final output
**When to use**: Debugging templates, verifying variable substitution
**Purpose**: See what TOML will actually be used

**Syntax**:
```bash
clnrm render <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm render test.clnrm.toml
```

**Input** (with template):
```toml
[variables]
base_image = "ubuntu:22.04"
timeout_secs = 30

[test]
name = "demo"
image = "{{ base_image }}"
timeout_seconds = {{ timeout_secs }}
```

**Output** (rendered):
```toml
[test]
name = "demo"
image = "ubuntu:22.04"
timeout_seconds = 30
```

**Useful for**:
- Debugging template issues
- Verifying environment variable substitution
- Understanding final configuration

---

## Category 3: Observation & Debugging (5 commands)

**User Story**: "I want to observe, debug, and understand what happened during test execution"

### Command 1: spans
**Description**: View OpenTelemetry trace spans
**When to use**: Debugging test failures, performance analysis, understanding execution flow
**Key feature**: Real observability into test execution

**Syntax**:
```bash
clnrm spans [OPTIONS]
```

**Quick examples**:
```bash
# View last 100 spans
clnrm spans --last 100

# View spans for specific test
clnrm spans --filter test_name

# Filter by severity
clnrm spans --level ERROR

# Export to file
clnrm spans --export spans.json
```

**Output format**:
```
Span: container_start
├─ Duration: 1.2s
├─ Status: OK
└─ Attributes:
   ├─ container_id: abc123
   ├─ image: ubuntu:latest
   └─ start_time: 2025-12-20T10:30:00Z

Span: container_exec
├─ Duration: 100ms
├─ Status: OK
└─ Command: echo "test"
```

**Advanced**:
- `--format json` - Machine-readable output
- `--time-range 10m` - Last 10 minutes
- `--service collector` - Spans from specific service

---

### Command 2: report
**Description**: Generate test execution report
**When to use**: End of test run, CI pipeline reporting, stakeholder communication
**Formats**: Human-readable, JSON, HTML, JUnit XML

**Syntax**:
```bash
clnrm report <CONFIG> [OPTIONS]
```

**Quick examples**:
```bash
# Generate HTML report
clnrm report test.clnrm.toml --format html --output report.html

# JUnit format for CI
clnrm report test.clnrm.toml --format junit --output report.xml

# JSON for automation
clnrm report test.clnrm.toml --format json
```

**Report includes**:
- Test summary (passed/failed counts)
- Timing information
- Resource usage
- Failure details
- Trace links
- Environmental info

**Example output** (text):
```
Test Report: test.clnrm.toml
═══════════════════════════════

Summary:
  Passed: 8
  Failed: 1
  Skipped: 0
  Duration: 45.2s

Failed Tests:
  ✗ complex_networking (15s)
    Error: Port 8080 already in use
    Trace: span-id-789

Slowest Tests:
  1. stress_test_100_iterations (25s)
  2. complex_networking (15s)
  3. basic_container (5.2s)
```

---

### Command 3: graph
**Description**: Visualize test dependency graph
**When to use**: Understanding test relationships, planning parallel execution
**Output**: DOT format (can convert to PNG, SVG)

**Syntax**:
```bash
clnrm graph <CONFIG> [OPTIONS]
```

**Quick examples**:
```bash
# View graph in terminal (ASCII art)
clnrm graph test.clnrm.toml

# Export to GraphViz format
clnrm graph test.clnrm.toml --format dot --output graph.dot

# Convert to PNG
dot -Tpng graph.dot > graph.png
```

**Shows**:
- Test dependencies (what must run before what)
- Parallel execution opportunities
- Critical path (longest dependency chain)
- Resource sharing (which tests share ports/volumes)

**Example output**:
```
basic_container ──┐
                  ├─→ networking_test ──┐
logging_test  ────┤                     ├─→ integration_test
                  │                     │
stress_test   ────┴─────────────────────┘
```

---

### Command 4: health
**Description**: System health check
**When to use**: Troubleshooting, pre-flight check before running tests
**Purpose**: Verify everything is ready

**Syntax**:
```bash
clnrm health [OPTIONS]
```

**Quick examples**:
```bash
# Quick health check
clnrm health

# Verbose with details
clnrm health --verbose

# JSON format
clnrm health --format json
```

**Checks**:
- Docker daemon running
- Docker version compatibility
- OTEL collector (if configured)
- Required ports available
- Disk space
- Network connectivity

**Output**:
```
✓ Docker daemon: OK (v24.0.0)
✓ OTEL collector: OK (http://localhost:4317)
✓ Ports available: OK (ports 8080-8099 free)
✓ Disk space: OK (50GB available)
✓ Network: OK (can reach docker.io)

Overall status: HEALTHY
```

---

### Command 5: live-check
**Description**: Watch test execution in real-time
**When to use**: During test development, debugging slow tests, monitoring long-running tests
**Key feature**: Real-time streaming output

**Syntax**:
```bash
clnrm live-check <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm live-check test.clnrm.toml
```

**Output** (updating in real-time):
```
LIVE: test.clnrm.toml

[1/5] basic_container (0s)
  ↳ Starting container ubuntu:latest
  ↳ Running 'echo hello'
  ↳ [████░░░░░] 0.5s

[2/5] networking_test (pending)

Completed: 1
In Progress: 1
Pending: 3
```

**Features**:
- Real-time progress bars
- Current command being executed
- Resource usage updates
- Click on test to see details
- `--follow` to keep watching after completion

---

## Category 4: System Management (5 commands)

**User Story**: "I want to manage services, plugins, and infrastructure"

### Command 1: services
**Description**: Manage background services (collector, API, etc.)
**When to use**: Starting/stopping services, checking status
**Services**: OTEL collector, Health API, etc.

**Syntax**:
```bash
clnrm services <SUBCOMMAND> [OPTIONS]
```

**Subcommands**:
- `list` - List running services
- `start <SERVICE>` - Start service
- `stop <SERVICE>` - Stop service
- `status <SERVICE>` - Check service status
- `restart <SERVICE>` - Restart service
- `logs <SERVICE>` - View service logs

**Examples**:
```bash
# List all services
clnrm services list

# Start OTEL collector
clnrm services start collector

# View collector status
clnrm services status collector

# View logs
clnrm services logs collector
```

**Output** (services list):
```
Running Services:
  ✓ collector       http://localhost:4317 (OTLP)
  ✓ api            http://localhost:8080 (REST)
  ✗ health-check   (stopped)

Stopped Services:
  • advanced-metrics
```

---

### Command 2: collector
**Description**: Configure OpenTelemetry collector
**When to use**: Setting up observability, changing collector endpoints
**Purpose**: Manage where traces/metrics are exported

**Syntax**:
```bash
clnrm collector <SUBCOMMAND> [OPTIONS]
```

**Subcommands**:
- `config` - Show current collector config
- `set-endpoint <URL>` - Set OTLP endpoint
- `set-exporter <TYPE>` - Change exporter (otlp, jaeger, zipkin)
- `enable <FEATURE>` - Enable metrics/logs/traces
- `disable <FEATURE>` - Disable features

**Examples**:
```bash
# View collector config
clnrm collector config

# Send traces to Jaeger
clnrm collector set-exporter jaeger --endpoint http://jaeger:14250

# Send to Datadog
clnrm collector set-exporter otlp --endpoint https://api.datadoghq.com:443/v0.4/traces
```

---

### Command 3: plugins
**Description**: List and manage installed plugins
**When to use**: Extending clnrm with custom commands, listing available extensions
**Purpose**: Plugin system discovery

**Syntax**:
```bash
clnrm plugins [SUBCOMMAND]
```

**Subcommands**:
- `list` - List installed plugins
- `install <PLUGIN>` - Install plugin
- `uninstall <PLUGIN>` - Remove plugin
- `info <PLUGIN>` - Plugin details
- `enable <PLUGIN>` - Activate plugin
- `disable <PLUGIN>` - Deactivate plugin

**Examples**:
```bash
# List installed plugins
clnrm plugins list

# Install plugin
clnrm plugins install chaos-engineering

# Get plugin info
clnrm plugins info chaos-engineering
```

**Output**:
```
Installed Plugins:
  ✓ chaos-engineering (v1.2.0) - Active
    Commands: stress, chaos, inject-failure
  ✓ custom-validators (v0.5.0) - Inactive
    Commands: validate-custom

Available Plugins (from registry):
  • advanced-metrics (v2.0.0)
  • kubernetes-support (v0.1.0)
```

---

### Command 4: pull
**Description**: Pre-download Docker images
**When to use**: Pre-caching images for faster test runs, CI optimization
**Purpose**: Reduce test execution time

**Syntax**:
```bash
clnrm pull <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm pull test.clnrm.toml
```

**What it does**:
- Parses TOML for all referenced images
- Downloads/caches images locally
- Verifies image availability
- Useful for CI/CD (run once, use many times)

**Options**:
- `--registry <URL>` - Custom registry
- `--force` - Re-pull even if cached
- `--parallel N` - Pull N images in parallel

---

### Command 5: (reserved for future)
**Placeholder** for future system management command

---

## Category 5: Development (5 commands)

**User Story**: "I'm developing or extending clnrm"

### Command 1: dev
**Description**: Development mode with file watching and live reload
**When to use**: Test development, iterating on TOML specs
**Key feature**: Automatic re-run on file change

**Syntax**:
```bash
clnrm dev <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm dev test.clnrm.toml --watch
```

**Features**:
- Watches TOML file for changes
- Automatically re-runs on save
- Live reload of templates
- Fast iteration loop

**Output**:
```
[Dev Mode] Watching test.clnrm.toml...

[1] Changes detected in test.clnrm.toml
    Re-running tests...
    ✓ Test 1: PASSED
    ✓ Test 2: PASSED
    Duration: 2.3s
```

---

### Command 2: template
**Description**: Generate code from Tera templates
**When to use**: Generating test files, scaffolding new projects
**Purpose**: Code generation from templates

**Syntax**:
```bash
clnrm template <TEMPLATE_FILE> [OPTIONS]
```

**Quick example**:
```bash
clnrm template templates/docker-compose.tera --output docker-compose.yml
```

**Input** (templates/docker-compose.tera):
```yaml
version: '3'
services:
  {% for service in services %}
  {{ service.name }}:
    image: {{ service.image }}
    ports:
      - "{{ service.port }}:{{ service.port }}"
  {% endfor %}
```

**Output**:
```yaml
version: '3'
services:
  postgres:
    image: postgres:15
    ports:
      - "5432:5432"
  redis:
    image: redis:7
    ports:
      - "6379:6379"
```

---

### Command 3: diff
**Description**: Compare test outputs (human-readable diff)
**When to use**: Regression testing, comparing expected vs actual output
**Purpose**: Spot differences easily

**Syntax**:
```bash
clnrm diff <FILE1> <FILE2> [OPTIONS]
```

**Quick example**:
```bash
clnrm diff baseline.json test-output.json
```

**Output format** (side-by-side):
```
Baseline                          Test Run
─────────────────────             ──────────────────────
{                                 {
  "status": "PASSED",               "status": "PASSED",
  "duration": 1.2,                  "duration": 1.5,      ← Changed
  "tests": 5,                       "tests": 5,
  "passed": 5,                      "passed": 5,
}                                 }
```

**Options**:
- `--format unified` - Unified diff format
- `--ignore-timestamps` - Ignore time changes
- `--color` / `--no-color` - Colorized output

---

### Command 4: analyze
**Description**: Analyze TOML configuration complexity and coverage
**When to use**: Understanding test suite size, identifying gaps
**Purpose**: Configuration metrics and analysis

**Syntax**:
```bash
clnrm analyze <CONFIG> [OPTIONS]
```

**Quick example**:
```bash
clnrm analyze test.clnrm.toml
```

**Output**:
```
Configuration Analysis: test.clnrm.toml
═════════════════════════════════════════

Summary:
  Tests: 12
  Total Duration: 45.2s
  Avg Duration: 3.8s
  Slowest: stress_test (15s)
  Fastest: basic_test (0.5s)

Images Used:
  ubuntu:22.04 (5 tests, 8.3s)
  postgres:15 (4 tests, 12.0s)
  redis:7 (3 tests, 5.2s)

Ports:
  5432 (postgres) - 4 tests
  6379 (redis) - 3 tests
  8080 (custom) - 1 test

Coverage:
  Network: ✓ Yes (3 tests)
  Volumes: ✓ Yes (5 tests)
  Env Vars: ✓ Yes (8 tests)
  Signals: ✗ No
```

---

### Command 5: (reserved for future)
**Placeholder** for future development command

---

## Command Reference Matrix

| Command | Category | Primary Use | Frequency | Difficulty |
|---------|----------|-------------|-----------|------------|
| run | Test Execution | Execute tests | Daily | Beginner |
| dry-run | Test Execution | Preview | Weekly | Beginner |
| record | Test Execution | Baseline | Rare | Intermediate |
| repro | Test Execution | Debug | As needed | Intermediate |
| stress | Test Execution | Performance | Weekly | Advanced |
| self-test | Test Execution | Validate install | Once | Beginner |
| init | Configuration | Start project | Rare | Beginner |
| validate | Configuration | Check syntax | Every change | Beginner |
| lint | Configuration | Code review | Every commit | Intermediate |
| fmt | Configuration | Auto-format | Every commit | Beginner |
| render | Configuration | Debug templates | As needed | Intermediate |
| spans | Observation | Debug execution | Daily | Intermediate |
| report | Observation | Summarize | Daily | Beginner |
| graph | Observation | Understand deps | Weekly | Advanced |
| health | Observation | Troubleshoot | As needed | Beginner |
| live-check | Observation | Real-time watch | As needed | Beginner |
| services | System Mgmt | Start/stop | Rare | Intermediate |
| collector | System Mgmt | Config OTEL | Rare | Advanced |
| plugins | System Mgmt | Extend | Rare | Advanced |
| pull | System Mgmt | Cache images | Rare | Beginner |
| dev | Development | Iterate | Frequent | Beginner |
| template | Development | Generate code | Rare | Advanced |
| diff | Development | Compare | As needed | Beginner |
| analyze | Development | Metrics | Weekly | Beginner |

---

## Grouping Summary for README

### Quick Reference (Main README)

| Feature | Commands | Count |
|---------|----------|-------|
| Test Execution | run, dry-run, record, repro, stress, self-test | 6 |
| Configuration | init, validate, lint, fmt, render | 5 |
| Observation | spans, report, graph, health, live-check | 5 |
| System Mgmt | services, collector, plugins, pull | 4 |
| Development | dev, template, diff, analyze | 4 |
| **TOTAL** | | **24** |

*(2 commands reserved for future use)*

### README Organization Strategy

**Section 1: Quick Command Reference** (in main README)
- List all 24 commands in 5 categories
- One-liner per command
- Link to detailed reference in book

**Section 2: Common Workflows** (in main README)
- Show practical combinations of commands
- Example: "Write and Run" → init, validate, run
- Example: "Debug Failure" → spans, repro, report
- Example: "Performance Analysis" → stress, graph, report

**Section 3: Detailed Reference** (in book/src/reference/cli-reference.md)
- Full documentation per command
- Examples
- Options
- Expected output
- Common issues

---

## Implementation Notes for README

### Use This Structure in README

```markdown
## Command Reference

Quick reference organized by feature. See [Full Reference](book/src/reference/cli-reference.md) for detailed usage.

### Test Execution
- `clnrm run` - Execute tests
- `clnrm dry-run` - Preview without containers
- ...

### Configuration & Validation
- `clnrm init` - Generate boilerplate
- ...

[etc.]
```

### Use This Structure in Book

```markdown
# Full CLI Reference

## Test Execution Commands

### clnrm run
**Purpose**: Execute tests from TOML specification
**Syntax**: clnrm run <CONFIG> [OPTIONS]
**Options**: [detailed list]
**Examples**: [multiple examples]
**Expected Output**: [sample output]
**Common Issues**: [troubleshooting]

### clnrm dry-run
[same structure]

[etc.]
```

---

## Next Steps

1. **Update main README**:
   - Add section: "## Command Reference (Quick)"
   - Paste 5 categories with one-liner per command
   - Add note linking to full reference

2. **Create book/src/reference/cli-reference.md**:
   - Use template from "Detailed Reference" section above
   - Fill in details for all 24 commands
   - Add examples for each

3. **Link everywhere**:
   - README links to book reference
   - Troubleshooting links back to design principles
   - Common workflows mention related commands

