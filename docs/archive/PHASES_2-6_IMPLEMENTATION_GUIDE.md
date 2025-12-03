# Phases 2-6 Implementation Guide - Complete Roadmap

**Status**: Phase 1 Complete ✅ | Phases 2-6 Roadmap
**To-Do Items**: 15 comprehensive tasks to complete all phases
**Total Effort**: ~30-40 development hours

---

## Overview: The 15 To-Do Items

```
PHASE 2: Tutorials (3 items)
├── Item 1: Tutorial 1 - Getting Started (15 min complete)
├── Item 2: Tutorial 2 - Container Pooling (10 min complete)
└── Item 3: Tutorials 3-5 - Weaver, Plugins, OTEL (Combined)

PHASE 3: How-To Guides (3 items)
├── Item 4: Execution & Performance (5 guides)
├── Item 5: CI/CD & Configuration (11 guides)
└── Item 6: Patterns, Advanced, Troubleshooting, Migration (18 guides)

PHASE 4: Reference (3 items)
├── Item 7: CLI Commands Reference
├── Item 8: TOML Configuration Reference
└── Item 9: API, Environment Variables, Plugins Reference

PHASE 5: Explanation (3 items)
├── Item 10: Architecture & Design Explanations (3 guides)
├── Item 11: Core Concept Explanations (5 guides)
└── Item 12: Advanced Topic Explanations (4 guides)

PHASE 6: Polish & Cleanup (3 items)
├── Item 13: Archive old documentation
├── Item 14: Update all internal links
└── Item 15: Final validation and merge
```

---

## PHASE 2: Tutorials (Days 3-4)

### Item 1: Tutorial 1 - Getting Started

**File**: `/home/user/clnrm/docs/tutorials/01-getting-started/README.md`

**Content Required** (15-minute tutorial):
```markdown
# Tutorial 1: Getting Started (15 minutes)

## Learning Objectives
- Install clnrm and verify it works
- Understand basic concepts (services, scenarios, expectations)
- Create your first test from scratch
- Run it and interpret results
- Know where to go next

## Prerequisites
- Docker or Podman installed
- 5 minutes of time
- Basic CLI knowledge

## Step-by-Step Walkthrough

### Step 1: Install (2 min)
- Homebrew installation
- Cargo installation
- Verify with `clnrm --version`

### Step 2: Initialize Project (2 min)
- Run `clnrm init`
- Explain generated files
- Show project structure

### Step 3: Write Your First Test (5 min)
- Copy example from GETTING_STARTED.md
- Explain each section:
  - [meta] — Test metadata
  - [service.*] — Container definitions
  - [[scenario]] — Test steps
  - [expect.*] — Validations

### Step 4: Run and Understand (4 min)
- Run `clnrm run`
- Show output and results
- Explain what happened
- Try modifying and running again

### Step 5: What's Next (2 min)
- Links to other tutorials
- How-to guides for next steps
- Architecture explanation if interested

## Key Concepts Explained
- What is a service?
- What is a scenario?
- What is an expectation?
- How does Docker isolation work?

## Common Issues & Fixes
- Docker not running
- Image not found
- Command failed

## Advanced Topics (Links Only)
- Multiple services
- Environment variables
- Custom plugins
```

**Checklist**:
- [ ] Create README.md in 01-getting-started/
- [ ] Include all 5 steps with real, working example
- [ ] Add "Key Concepts" section
- [ ] Add "Common Issues" section
- [ ] Test the example end-to-end
- [ ] Add estimated time: ~15 minutes

---

### Item 2: Tutorial 2 - Container Pooling

**File**: `/home/user/clnrm/docs/tutorials/02-container-pooling/README.md`

**Content Required** (10-minute tutorial):
```markdown
# Tutorial 2: Container Pooling (10 minutes)

## Learning Objectives
- Understand why tests are slow (2-5s startup)
- Enable pooling with one environment variable
- Benchmark before and after
- Configure pool size and timeout
- Monitor pool hit rate

## Prerequisites
- Completed Tutorial 1
- Same test file from Tutorial 1
- ~10 minutes

## The Problem: Slow Startup

Explain:
- Docker container creation is slow (2-5 seconds)
- Creating container per test = 2-5s overhead per test
- With 100 tests = 200-500 seconds just for startup!

## The Solution: Container Pooling

Show diagram:
- Pre-warmed pool of containers (FIFO queue)
- Instead of 2-5s acquisition, 0.1-0.5ms
- 80% faster startup
- 10x higher throughput

## Step 1: Enable Pooling (1 min)

```bash
CLNRM_ENABLE_POOLING=1 clnrm run
```

Show output: "✅ Container pooling enabled"

## Step 2: Benchmark (5 min)

### Without pooling:
```bash
time clnrm run
# Real: 5.234s (startup overhead)
```

### With pooling:
```bash
CLNRM_ENABLE_POOLING=1 time clnrm run
# Real: 0.500s (80% faster!)
```

Show metrics: Pool hit rate, acquisition time

## Step 3: Configure (3 min)

In TOML:
```toml
[pool]
enabled = true
size = 10                  # Pre-warm 10 containers
idle_timeout_ms = 60000   # Keep 1 minute
health_check_interval_ms = 5000

# Or via environment
CLNRM_POOL_SIZE=10
CLNRM_POOL_IDLE_TIMEOUT_MS=60000
```

## Step 4: Monitor (1 min)

Show metrics:
- Pool hit rate (target: >90%)
- Container acquisition time (0.1-0.5ms)
- Average test duration

## Key Concepts
- Why startup is slow
- How pooling reduces overhead
- Trade-offs (memory vs. speed)
- Configuration tuning

## Advanced Topics (Links)
- Parallel execution with pooling
- Resource requirements
- Performance optimization
- Stress testing with pools
```

**Checklist**:
- [ ] Create README.md in 02-container-pooling/
- [ ] Show before/after benchmarks (real numbers)
- [ ] Include configuration examples
- [ ] Add monitoring/metrics section
- [ ] Explain trade-offs clearly
- [ ] Test pooling works end-to-end

---

### Item 3: Tutorials 3-5 - Weaver, Plugins, OTEL

**Files**:
- `/home/user/clnrm/docs/tutorials/03-weaver-validation/README.md`
- `/home/user/clnrm/docs/tutorials/04-custom-plugins/README.md`
- `/home/user/clnrm/docs/tutorials/05-otel-integration/README.md`

**Tutorial 3: Weaver Validation** (15 minutes):
```markdown
# Tutorial 3: Weaver Validation (15 minutes)

## Learning Objectives
- Understand false-positive problem in testing
- Install and configure Weaver
- Write validation schema in registry
- Enable live-checking in TOML
- See how Weaver catches broken tests

## Prerequisites
- Completed Tutorial 1
- Opentelemetry Weaver installed
- ~15 minutes

## The Problem: False Positives

Show example:
```bash
# This "test" passes but does nothing!
echo "✅ Test passed"
exit 0
```

Explain:
- Traditional testing only checks exit codes
- Test can pass while doing nothing
- No validation of actual behavior
- Hard to catch these fake-green tests

## The Solution: Weaver Schema Validation

Explain:
- Schema defines expected telemetry structure
- Weaver validates against schema
- Runtime telemetry proves behavior
- Test fails if code doesn't actually execute

## Step 1: Install Weaver (2 min)
```bash
# Install weaver CLI
cargo install weaver

# Verify
weaver --version
```

## Step 2: Create Schema (5 min)
In registry/schemas/:
```yaml
groups:
  - id: http.request
    brief: HTTP request span
    attributes:
      - id: http.method
        type: string
        required: true
      - id: http.route
        type: string
```

## Step 3: Enable in TOML (3 min)
```toml
[weaver]
enabled = true
registry_path = "registry"
otlp_port = 0
```

## Step 4: Run Live-Check (3 min)
```bash
clnrm run --live-check --registry registry/

# Output shows validation results
# ✅ All spans valid
# ❌ Missing required attributes
```

## Step 5: Fix and Re-run (2 min)
- Examine validation failures
- Update test or code to emit correct telemetry
- Re-run with live-check
- See validation pass

## Key Concepts
- Schema-first validation
- Why behavior validation matters
- OpenTelemetry semantic conventions
- Weaver's role in validation

## Advanced Topics (Links)
- Custom validators
- Schema design patterns
- Complex trace structures
```

**Tutorial 4: Custom Plugins** (20 minutes):
- How ServicePlugin trait works
- Copy example plugin
- Implement start/stop/health_check
- Register in plugin registry
- Use in TOML test
- Test your plugin

**Tutorial 5: OTEL Integration** (15 minutes):
- Why observability matters
- Configure OTLP export in TOML
- Set up Jaeger (or DataDog/New Relic)
- Run test with telemetry export
- Inspect traces in UI
- Configure sampling and propagators

**Checklist for all 3**:
- [ ] Create README.md files
- [ ] Include real, working examples
- [ ] Add code snippets (copy-paste friendly)
- [ ] Test each tutorial end-to-end
- [ ] Include "Key Concepts" section
- [ ] Add links to related How-To guides
- [ ] Verify estimated times are accurate

---

## PHASE 3: How-To Guides (Days 5-6)

### Item 4: Execution & Performance How-Tos (5 guides)

**Files to create in `/docs/how-to/`**:

1. **parallel-execution.md** — How to run tests in parallel
   - Problem: Sequential tests are slow
   - Solution: Use `--parallel --jobs N`
   - Configuration: Job count tuning
   - Trade-offs: Resource usage vs. throughput
   - Example: Benchmark 100 tests parallel vs. sequential

2. **container-pooling-setup.md** — How to enable and configure pooling
   - Problem: 2-5s startup per test
   - Solution: Enable with `CLNRM_ENABLE_POOLING=1`
   - Configuration: Pool size, idle timeout
   - Monitoring: Hit rate, metrics
   - Troubleshooting: Low hit rate, memory issues

3. **performance-tuning.md** — How to optimize for your workload
   - Job count tuning
   - Pool size optimization
   - Resource limits
   - Sampling strategies
   - Profiling and benchmarking

4. **stress-testing.md** — How to load test your setup
   - Creating 1000+ concurrent tests
   - Resource requirements
   - Bottleneck identification
   - Scaling limits

5. **performance-monitoring.md** — How to collect and analyze metrics
   - Built-in metrics
   - OTEL metrics export
   - Dashboard setup
   - Performance analysis

**Checklist**:
- [ ] Create all 5 guides
- [ ] Each has clear problem statement
- [ ] Each has copy-paste solution
- [ ] Include real examples
- [ ] Add troubleshooting section
- [ ] Link to related how-tos

---

### Item 5: CI/CD & Configuration How-Tos (11 guides)

**Files to create in `/docs/how-to/`**:

**CI/CD Integration** (5 guides):
1. **github-actions.md** — GitHub Actions workflow example
2. **gitlab-ci.md** — GitLab CI pipeline example
3. **jenkins.md** — Jenkins job example
4. **test-reporting.md** — Generate JUnit XML, HTML reports
5. **ci-fail-fast.md** — Fail on first error

**Configuration** (6 guides):
1. **environment-variables.md** — CLNRM_* and OTEL_* vars
2. **template-variables.md** — Tera template syntax
3. **multi-environment.md** — Dev/staging/prod configs
4. **container-backends.md** — Docker, Podman, testcontainers
5. **otel-configuration.md** — Export to Jaeger/DataDog/New Relic
6. **custom-toml-schemas.md** — Extend base schema

**Checklist**:
- [ ] Create all 11 guides
- [ ] Each has clear problem and solution
- [ ] Include real workflow/config files
- [ ] Add configuration examples
- [ ] Copy-paste friendly
- [ ] Test in realistic scenarios

---

### Item 6: Remaining How-Tos (18 guides)

**Testing Patterns** (5 guides):
1. **database-testing.md** — PostgreSQL, MongoDB, SurrealDB
2. **api-testing.md** — HTTP endpoint testing
3. **microservice-testing.md** — Multi-service orchestration
4. **custom-service-testing.md** — Write service plugins
5. **hermetic-patterns.md** — Isolation best practices

**Advanced Topics** (5 guides):
1. **weaver-schemas.md** — Define validation schemas
2. **custom-validators.md** — Extend validation rules
3. **plugin-development.md** — Create plugins
4. **determinism-testing.md** — Reproducible results
5. **chaos-engineering.md** — Failure injection

**Troubleshooting** (6 guides):
1. **docker.md** — Docker daemon, socket, networking
2. **debug.md** — Logging, tracing, inspection
3. **flaky-tests.md** — Timeout tuning, retry logic
4. **validation.md** — Schema and expectation errors
5. **performance.md** — Memory leaks, bottlenecks
6. **common-errors.md** — Error codes and fixes

**Migration** (2 guides):
1. **migrate-v1.3-to-v1.4.md** — Breaking changes
2. **migrate-v1.4.0-to-v1.4.1.md** — Pool improvements

**Checklist**:
- [ ] Create all 18 guides
- [ ] Each solves one specific problem
- [ ] Includes real, working examples
- [ ] Copy-paste solutions
- [ ] Related how-tos linked
- [ ] Tested in practice

---

## PHASE 4: Reference Documentation (Days 7-8)

### Item 7: CLI Commands Reference

**File**: `/docs/reference/cli.md`

**Content**:
```markdown
# CLI Commands Reference

## Overview
All clnrm commands with flags, options, and examples.

## Command Categories

### Initialization
- `clnrm init` — Initialize project
- `clnrm validate` — Validate TOML

### Test Execution
- `clnrm run` — Execute tests
  - `--parallel` — Run concurrently
  - `--jobs N` — Concurrency limit
  - `--live-check` — Weaver validation
  - `--registry PATH` — Schema registry
  - `--output FORMAT` — Output format (junit, html, json)
  - `--filter PATTERN` — Run matching tests
  - `--exclude PATTERN` — Skip matching tests
  - `-x` or `--fail-fast` — Stop on first failure
  - `--timeout MS` — Test timeout

### Inspection
- `clnrm plugins` — List available plugins
- `clnrm self-test` — Run framework tests
- `clnrm health` — Check system health

## Common Examples

### Run with pooling and parallelism
```bash
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16
```

### Run with live-check
```bash
clnrm run --live-check --registry registry/
```

### Generate reports
```bash
clnrm run --output junit > results.xml
clnrm run --output html --output-file report.html
```

## Exit Codes
- 0 — Success
- 1 — Test failure
- 2 — Configuration error
- 3 — System error (Docker, permissions)

## Help
```bash
clnrm --help
clnrm run --help
clnrm <command> --help
```
```

**Checklist**:
- [ ] Document all commands
- [ ] Include all flags with descriptions
- [ ] Add real examples
- [ ] Document exit codes
- [ ] Auto-generate if possible from clap
- [ ] Keep updated with new commands

---

### Item 8: TOML Configuration Reference

**File**: `/docs/reference/toml-schema.md`

**Content**:
```markdown
# TOML Configuration Reference

## Complete Schema

### [meta] Section
```toml
[meta]
name = "test_name"              # Required: Unique identifier
description = "what it does"    # Optional: Test description
version = "1.0.0"              # Optional: Version
tags = ["api", "integration"]   # Optional: Categorization
```

### [service.*] Sections
```toml
[service.my_service]
plugin = "generic_container"    # Which plugin to use
image = "alpine:latest"        # Docker image
# Plugin-specific options...
```

### [[scenario]] Sections
```toml
[[scenario]]
name = "scenario_name"
service = "service_name"
run = "command to execute"
timeout_ms = 5000
artifacts.collect = ["spans:default"]
```

### [expect.*] Sections
```toml
[expect.output]
stdout = "expected output"
stderr = ""

[expect.span]
name = "span_name"
kind = "server"
attrs.all = { "key" = "value" }

[expect.graph]
must_include = [["span1", "span2"]]
acyclic = true

[expect.order]
must_precede = [["span1", "span2"]]

[expect.counts]
spans_total = { gte = 1, lte = 100 }
errors_total = { eq = 0 }

[expect.hermeticity]
no_external_services = true
```

### [weaver] Section
```toml
[weaver]
enabled = true
registry_path = "registry"
otlp_port = 0
admin_port = 0
fail_fast = false
```

### [otel] Section
```toml
[otel]
exporter = "otlp-http"
endpoint = "http://localhost:4318"
sample_ratio = 1.0

[otel.resources]
"service.name" = "my_service"
"deployment.environment" = "test"
```

## Variable Substitution

Tera template variables:
- `{{ env.VAR_NAME }}` — Environment variables
- `{{ now }}` — Current timestamp
- `{{ uuid }}` — Random UUID

## Examples

[Complete working examples...]
```

**Checklist**:
- [ ] Document all TOML sections
- [ ] Include all available options
- [ ] Add examples for each section
- [ ] Document variable substitution
- [ ] Add validation rules
- [ ] Generate from serde if possible

---

### Item 9: API, Environment Variables, Plugins Reference

**File 1**: `/docs/reference/api.md`
```markdown
# Rust API Reference

## Main Types

### CleanroomEnvironment
- Methods: new(), register_service(), start_service(), execute_command()
- Used for programmatic test creation

### ServicePlugin Trait
- Methods: start(), stop(), health_check(), service_type()
- Implement for custom services

### Backend Trait
- Methods: create_container(), run_command(), cleanup()
- Implement for custom container backends

## Example: Creating Custom Plugin

[Code example...]
```

**File 2**: `/docs/reference/environment-variables.md`
```markdown
# Environment Variables Reference

## CLNRM Variables
- `CLNRM_ENABLE_POOLING` — Enable container pooling
- `CLNRM_POOL_SIZE` — Pre-warmed container count
- `CLNRM_POOL_IDLE_TIMEOUT_MS` — Idle timeout
- `CLNRM_JOBS` — Default concurrency limit
- `CLNRM_TIMEOUT_MS` — Default test timeout

## OTEL Variables
- `OTEL_EXPORTER_OTLP_ENDPOINT` — OTLP endpoint
- `OTEL_EXPORTER_OTLP_PROTOCOL` — http or grpc
- `OTEL_SDK_DISABLED` — Disable OTEL

## Rust Variables
- `RUST_LOG` — Logging level (debug, info, warn, error)
- `RUST_BACKTRACE` — 1 or full for backtraces
```

**File 3**: `/docs/reference/plugins.md`
```markdown
# Built-in Plugins Reference

## GenericContainer
- Runs any Docker image
- Options: image, command, env vars, volumes

## SurrealDB
- Database service
- Options: port, credentials

## Ollama
- LLM inference
- Options: model, port

[... more plugins ...]
```

**Checklist**:
- [ ] Document all API types
- [ ] Include usage examples
- [ ] List all environment variables
- [ ] Document all built-in plugins
- [ ] Add plugin options reference
- [ ] Keep synchronized with code

---

## PHASE 5: Explanation Documentation (Days 9-10)

### Item 10: Architecture & Design Explanations (3 guides)

**File 1**: `/docs/explanation/architecture.md`
```markdown
# System Architecture Overview

## Component Overview
```
User → CLI → Config Loader → Orchestrator → Test Executor → Backend → Docker
        ↑                        ↑              ↑              ↑
        └──────── Telemetry ────┴──────────────┴──────────────┘
```

## Key Components
1. **CLI** — Command-line interface
2. **Config Loader** — Parses TOML files
3. **Orchestrator** — Coordinates test execution
4. **Test Executor** — Runs scenarios in containers
5. **Backend** — Docker abstraction layer
6. **Telemetry** — OTEL collection

## Data Flow

### Test Execution Flow
1. User runs `clnrm run`
2. CLI finds *.clnrm.toml files
3. Config loader parses TOML
4. Orchestrator validates config
5. Test executor creates test runner
6. Backend requests container
7. Container pool provides pre-warmed container
8. Scenario runs in container
9. Telemetry captured
10. Validator checks expectations
11. Results reported

## Design Patterns

### Plugin Pattern
- Services are plugins (extensible)
- ServicePlugin trait for common interface
- Plugin registry for discovery

### Backend Abstraction
- Backend trait for container operations
- Swap implementations (Docker, Podman, Mock)
- Same tests work with different backends

### Trait-based Composition
- No inheritance, trait composition
- Loosely coupled components
- Easy testing with mocks
```

**File 2**: `/docs/explanation/plugin-system.md`
- Why plugins over hardcoding
- ServicePlugin trait design
- Plugin lifecycle
- Plugin discovery and registration
- Creating custom plugins

**File 3**: `/docs/explanation/concurrency.md`
- Why uncontrolled concurrency is bad
- Semaphore-based fairness model
- Job limiting and backpressure
- Lock-free hot paths
- Scaling characteristics

**Checklist**:
- [ ] Create all 3 guides
- [ ] Include architecture diagrams (ASCII or links to images)
- [ ] Explain design decisions
- [ ] Show data flow through system
- [ ] Discuss trade-offs
- [ ] Link to how-to guides for practical application

---

### Item 11: Core Concept Explanations (5 guides)

**File 1**: `/docs/explanation/weaver-validation.md`
- The false-positive problem in testing
- How schema validation works
- Why OpenTelemetry is source of truth
- How Weaver catches fake-green tests
- Integration with clnrm

**File 2**: `/docs/explanation/container-pooling.md`
- Why startup was slow (2-5 seconds)
- How pooling solves it (0.1-0.5ms)
- Pre-warming strategy
- FIFO queue management
- Background health checks
- Trade-offs (memory vs. speed)
- Performance characteristics

**File 3**: `/docs/explanation/hermiticity.md`
- What hermiticity means
- Why isolation matters
- Docker as isolation mechanism
- Validating hermiticity through telemetry
- Hermetic test patterns

**File 4**: `/docs/explanation/determinism.md`
- Non-determinism sources
- Deterministic test design
- Random seed control
- Timing-dependent tests
- Validation approaches

**File 5**: `/docs/explanation/false-positives.md`
- Types of false positives
- Why traditional testing fails
- How schema validation helps
- Detecting fake-green tests
- Building reliable test suites

**Checklist**:
- [ ] Create all 5 guides
- [ ] Explain "why" not just "what"
- [ ] Include analogies or diagrams
- [ ] Discuss design rationale
- [ ] Show trade-offs
- [ ] Link to related how-to guides

---

### Item 12: Advanced Topic Explanations (4 guides)

**File 1**: `/docs/explanation/otel-integration.md`
- What telemetry gets emitted
- Export formats (OTLP, stdout)
- Resource attributes and context
- Span structure and relationships
- Trace propagation

**File 2**: `/docs/explanation/performance.md`
- Startup overhead breakdown
- Throughput limits
- Resource requirements
- Scaling laws and bottlenecks
- Optimization opportunities

**File 3**: `/docs/explanation/error-handling.md`
- Error types and hierarchy
- Recovery strategies
- Graceful degradation
- Logging and debugging

**File 4**: `/docs/explanation/testing-philosophy.md`
- Hermetic testing principles
- Behavior validation approach
- Schema-first design
- End-to-end validation

**Checklist**:
- [ ] Create all 4 guides
- [ ] Deep technical content
- [ ] Explain internals and design
- [ ] Include diagrams where helpful
- [ ] Discuss implementation details
- [ ] Link to reference docs

---

## PHASE 6: Polish & Cleanup (Days 11-12)

### Item 13: Archive Old Documentation

**Task**: Move 177 historical files to archive/

**Process**:
1. Identify which files to archive
   - Agent reports
   - Completion summaries
   - Version-specific docs
   - Implementation history
   - Swarm reports

2. Create archive structure
   ```
   archive/
   ├── README.md (explain structure)
   ├── agent-reports/
   ├── completion-reports/
   ├── version-specific/
   ├── release-notes/
   ├── analysis/
   └── implementation-history/
   ```

3. Move files:
   ```bash
   git mv docs/old-file.md archive/category/old-file.md
   ```

4. Create archive README explaining:
   - Why files are archived
   - How to find what you need
   - How to reference archived docs
   - Which docs are still active

5. Update all links to archived docs
   - Change links to `/docs/archive/...`
   - Add note: "This is archived documentation"

**Checklist**:
- [ ] Identify 177 files to archive
- [ ] Create archive directory structure
- [ ] Move files to archive/
- [ ] Create archive/README.md
- [ ] Update all references
- [ ] Test no broken links

---

### Item 14: Update All Internal Links

**Task**: Fix broken references and add cross-links

**Process**:
1. Find all broken links
   ```bash
   find docs -name "*.md" -exec grep -l "docs/" {} \;
   ```

2. Fix references to moved files
   - Old: `docs/CLI_GUIDE.md`
   - New: `docs/reference/cli.md`

3. Add cross-references between Diataxis sections
   - Link tutorials to how-tos
   - Link how-tos to reference
   - Link explanations to how-tos

4. Ensure all internal links work
   - Relative paths correct
   - No circular references
   - Consistent link text

5. Test with link checker
   ```bash
   # Run markdown link checker
   find docs -name "*.md" | xargs check-links
   ```

**Checklist**:
- [ ] Find all internal links
- [ ] Update moved file references
- [ ] Add cross-references
- [ ] Test all links work
- [ ] Consistent link format
- [ ] No circular references

---

### Item 15: Final Validation & Merge

**Task**: Comprehensive validation and merge to main

**Process**:

1. **Structure Validation**
   - [ ] All 4 Diataxis sections populated
   - [ ] All 15 to-do items completed
   - [ ] No broken links
   - [ ] Consistent formatting

2. **Content Validation**
   - [ ] All tutorials have working examples
   - [ ] All how-tos are practical and copy-paste friendly
   - [ ] All references are complete and accurate
   - [ ] All explanations are conceptually sound

3. **User Testing**
   - [ ] Can new user find getting started? (< 10 sec)
   - [ ] Can user find "how to do X"? (< 30 sec)
   - [ ] Can user find technical details? (< 20 sec)
   - [ ] Can user understand concepts?
   - [ ] Navigation is clear and helpful

4. **Final Checks**
   - [ ] Run markdown lint
   - [ ] Check for dead links
   - [ ] Verify all code examples work
   - [ ] Check formatting consistency
   - [ ] Verify images/diagrams render

5. **Merge Process**
   ```bash
   # Update feature branch
   git fetch origin
   git rebase origin/main

   # Verify tests pass
   cargo test
   cargo clippy -- -D warnings

   # Create PR
   gh pr create --title "docs: complete Diataxis restructure" ...

   # After approval
   git merge --ff-only
   git push origin main
   ```

**Checklist**:
- [ ] Run all validation checks
- [ ] User testing completed
- [ ] All links verified
- [ ] Code examples tested
- [ ] Formatting consistent
- [ ] PR created and reviewed
- [ ] PR approved and merged

---

## Summary: 15 Items to Complete

```
Phase 2 (Tutorials):           3 items
Phase 3 (How-To Guides):       3 items
Phase 4 (Reference):           3 items
Phase 5 (Explanations):        3 items
Phase 6 (Polish):              3 items
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL:                         15 items
```

## Effort Estimate

| Phase | Items | Effort | Days |
|-------|-------|--------|------|
| Phase 2 | 3 | 8-10 hours | 1-2 |
| Phase 3 | 3 | 12-15 hours | 2-3 |
| Phase 4 | 3 | 6-8 hours | 1-2 |
| Phase 5 | 3 | 8-10 hours | 1-2 |
| Phase 6 | 3 | 4-5 hours | 1 |
| **TOTAL** | **15** | **38-48 hours** | **6-10 days** |

## How to Use This Guide

1. **Start with Phase 2** — Create tutorials first
2. **Follow sequence** — Each phase builds on previous
3. **Reference the checklist** — Ensure nothing is missed
4. **Track progress** — Update to-do list as you complete items
5. **Test thoroughly** — Especially tutorials and how-tos
6. **Get feedback** — User testing in Phase 6

## Success Criteria (All Phases)

✅ **All tutorials have working examples**
✅ **All how-tos are practical and copy-paste ready**
✅ **All references are complete and accurate**
✅ **All explanations are clear and conceptually sound**
✅ **No broken internal links**
✅ **User can find anything in <30 seconds**
✅ **Ready for production merge**

---

**Ready to implement? Start with Item 1 (Tutorial 1)!**
