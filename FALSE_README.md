# Cleanroom Testing Framework

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/seanchatmangpt/clnrm)
[![Build Status](https://img.shields.io/badge/build-passing-green.svg)](https://github.com/seanchatmangpt/clnrm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **🚀 Production Ready:** Hermetic integration testing that actually works end-to-end.
>
> **✨ Version 1.0.0 Highlights (Production Release):**
> - **dev --watch**: Hot reload with approximately 3s latency - save and see results quickly
> - **dry-run**: Fast validation without containers (typically under 1s for 10 files)
> - **fmt**: Deterministic TOML formatting with idempotency verification
> - **Macro Pack**: Eliminate boilerplate with reusable Tera macros (up to 85% reduction)
> - **Change Detection**: Only rerun changed scenarios (significantly faster iteration)
> - **All production features**: Tera templating, temporal validation, multi-format reporting, hot reload, PRD v1.0 implementation

A testing framework for hermetic integration testing with container-based isolation and plugin architecture.

## 🧪 Experimental AI Features

AI-powered features are available but **disabled by default** to keep the production framework stable. To enable AI features, you must explicitly opt-in during installation.

### Enable AI Features

**Via Cargo:**
```bash
# Install with AI features enabled
cargo install clnrm --features ai

# Build from source with AI features
cargo build --release --features ai
```

**Via Homebrew:**
```bash
# Standard installation (AI features disabled)
brew install clnrm

# To enable AI features, build from source:
brew uninstall clnrm
git clone https://github.com/seanchatmangpt/clnrm
cd clnrm
cargo build --release --features ai
brew install --build-from-source .
```

### Available AI Commands (Requires `--features ai`)

When AI features are enabled, additional commands become available:

- **`clnrm ai-orchestrate`** - AI-powered test orchestration with predictive failure analysis
- **`clnrm ai-predict`** - Predictive analytics for test execution patterns
- **`clnrm ai-optimize`** - Automatic test optimization (execution order, resources, parallelism)
- **`clnrm ai-real`** - Real AI intelligence using SurrealDB and Ollama
- **`clnrm ai-monitor`** - Autonomous monitoring with anomaly detection and self-healing
- **`clnrm services ai-manage`** - AI-driven service lifecycle management with auto-scaling

**Status:** Experimental - requires SurrealDB and Ollama for full functionality

**Important:** AI commands will NOT appear in `clnrm --help` unless you build with `--features ai`.

## 🎯 What Works (Verified)

### ✅ **Core Testing Pipeline**
- **`clnrm init`** - Zero-config project initialization with working TOML files
- **`clnrm run`** - Real container execution with regex validation and output capture
- **`clnrm validate`** - TOML configuration validation
- **`clnrm self-test`** - Framework validates itself across 5 test suites (framework, container, plugin, cli, otel)

### ✅ **Plugin Ecosystem**
- **`clnrm plugins`** - 8 service plugins for container, database, and AI integration
- **GenericContainerPlugin** - Any Docker image with custom configuration
- **SurrealDbPlugin** - SurrealDB database with WebSocket support
- **NetworkToolsPlugin** - curl, wget, netcat for HTTP testing
- **LLM Plugins** - Ollama, vLLM, TGI for AI model inference (production-ready)
- **Chaos Engine** - Controlled failure injection (experimental - clnrm-ai crate)
- **AI Test Generator** - AI-powered test case generation (experimental - clnrm-ai crate)

### ✅ **Service Management**
- **`clnrm services`** - Service lifecycle management (status, logs, restart subcommands)

### ✅ **Template System**
- **`clnrm template <type>`** - Generate projects from templates
- **Available Templates** - default, advanced, minimal, database, api, otel
- **Usage**: `clnrm template otel` - Generate OTEL validation template
- **Output**: Generates `.clnrm.toml.tera` templates with variable substitution

### ✅ **Tera Templating** *(v1.0)*
- **Dynamic configuration** - Jinja2-like templates for test files
- **Custom functions** - `env()`, `now_rfc3339()`, `sha256()`, `toml_encode()`
- **Template namespaces** - `vars.*`, `matrix.*`, `otel.*`
- **Matrix testing** - Cross-product test generation
- **Conditional logic** - Environment-based configuration
- **Macro library** - 8 reusable macros with 85% boilerplate reduction

### ✅ **Advanced Validators** *(v1.0)*
- **Temporal ordering** - `must_precede` and `must_follow` validation
- **Status validation** - Glob patterns for span status codes
- **Count validation** - Span counts by kind and total
- **Window validation** - Time-based span containment
- **Graph validation** - Parent-child relationships and topology
- **Hermeticity validation** - Isolation and resource constraints

### ✅ **Multi-Format Reporting** *(v1.0)*
- **JSON reports** - Programmatic access and parsing
- **JUnit XML** - CI/CD integration (Jenkins, GitHub Actions)
- **SHA-256 digests** - Reproducibility verification
- **Deterministic output** - Identical digests across runs

## 🚀 Quick Start

### Initialize Project
```bash
# Zero-configuration project setup
clnrm init

# Generates: tests/basic.clnrm.toml and project structure
```

### Run Tests
```bash
# Auto-discover and run all tests
clnrm run

# Real container execution with output validation
# ✅ Container commands execute
# ✅ Regex patterns validate output
# ✅ Test results are accurate
```

### Validate Configuration
```bash
# Validate TOML syntax and structure
clnrm validate tests/

# ✅ Generated TOML files are valid
# ✅ Configuration structure is correct
```

### List Available Plugins
```bash
# Show 8 service plugins (6 production + 2 experimental)
clnrm plugins

# ✅ Generic containers, databases, network tools, LLM plugins
# ✅ Experimental: chaos engine, AI test generator (clnrm-ai crate)
```

## 🚀 **Version 1.0.0 Features (Current)**

### **✅ No-Prefix Tera Templating (Implemented)**

Create clean, readable templates with no complex namespaces:

```toml
[meta]
name = "{{ svc }}_otel_proof"
version = "1.0.0"
description = "Telemetry-only validation"

[vars]                # authoring-only; runtime ignores this table
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
freeze_clock = "{{ freeze_clock }}"
image = "{{ image }}"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "{{ svc }}", "env" = "{{ env }}" }

[service.clnrm]
plugin = "generic_container"
image = "{{ image }}"
args = ["self-test", "--otel-exporter", "{{ exporter }}", "--otel-endpoint", "{{ endpoint }}"]
env = { "OTEL_TRACES_EXPORTER" = "{{ exporter }}", "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}" }
wait_for_span = "clnrm.run"

[[scenario]]
name = "otel_only_proof"
service = "clnrm"
run = "clnrm run --otel-exporter {{ exporter }} --otel-endpoint {{ endpoint }}"
artifacts.collect = ["spans:default"]

[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "clnrm.step:hello_world"
parent = "clnrm.run"
kind = "internal"
events.any = ["container.start", "container.exec", "container.stop"]

[expect.graph]
must_include = [["clnrm.run", "clnrm.step:hello_world"]]
acyclic = true

[expect.status]
all = "OK"

[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = { "service.name" = "{{ svc }}", "env" = "{{ env }}" }

[determinism]
seed = 42
freeze_clock = "{{ freeze_clock }}"

[report]
json = "report.json"
digest = "trace.sha256"
```

### **Rust-Based Variable Resolution**

Variables are resolved in Rust with clear precedence:
- **Template variables** (highest priority)
- **Environment variables** (e.g., `$SERVICE_NAME`, `$OTEL_ENDPOINT`)
- **Defaults** (lowest priority)

**Available Variables:**
- `svc` - Service name (default: "clnrm")
- `env` - Environment (default: "ci")
- `endpoint` - OTEL endpoint (default: "http://localhost:4318")
- `exporter` - OTEL exporter (default: "otlp")
- `image` - Container image (default: "registry/clnrm:1.0.0")
- `freeze_clock` - Deterministic time (default: "2025-01-01T00:00:00Z")
- `token` - OTEL auth token (default: "")

### **Template Generation**

```bash
# Generate OTEL validation template
clnrm template otel > my-test.clnrm.toml

# The generated template uses no-prefix variables
# Variables are resolved in Rust: template vars → ENV → defaults
```

## 📚 Documentation

- **[v1.0 Documentation](docs/)** - Complete v1.0 guides and references
- **[PRD: v1.0 Tera-First Architecture](docs/PRD-v1.md)** - Product requirements
- **[TOML Reference](docs/v1.0/TOML_REFERENCE.md)** - Configuration format
- **[Migration Guide](docs/v1.0/MIGRATION_GUIDE.md)** - From v0.6.0 to v1.0
- **[Fake Green Detection](docs/FAKE_GREEN_DETECTION_USER_GUIDE.md)** - User guide
- **[CLI Analyze](docs/CLI_ANALYZE_REFERENCE.md)** - Analyze command reference

## 🎯 Legacy v0.6.0 Features

### **Property-Based Testing with Fake Data**

Generate test scenarios with fake data generators:

```toml
# tests/load-test.clnrm.toml.tera
{% for i in range(end=1000) %}
[[steps]]
name = "load_test_{{ i }}"
command = ["curl", "http://api:8080/users",
           "-d", '{"name":"{{ fake_name() }}","email":"{{ fake_email() }}"}']
expected_output_regex = "success"
{% endfor %}
```

**Key Features:**
- **50+ fake data generators** - UUIDs, names, emails, timestamps, IPs, etc.
- **Deterministic seeding** - Reproducible tests with `fake_uuid_seeded(seed=42)`
- **Matrix testing** - Generate all combinations of parameters
- **Property-based testing** - Validate properties across generated data

**Example: Generate hundreds of unique API tests from a few lines of template code.**

### **Telemetry-Only Validation (OTEL)**

**WARNING:** OpenTelemetry validation requires additional setup. See the **[OpenTelemetry Integration Guide](docs/OPENTELEMETRY_INTEGRATION_GUIDE.md)** for complete setup instructions including:
- Installing the OpenTelemetry Collector
- Configuring the collector to export traces
- Setting up clnrm to emit OTEL spans
- Using `clnrm analyze` to validate traces

Prove system correctness using OpenTelemetry spans exclusively:

```toml
# tests/otel-validation.clnrm.toml
[services.otel_collector]
plugin = "otel_collector"
image = "otel/opentelemetry-collector:latest"

[services.app_under_test]
plugin = "generic_container"
image = "myapp:latest"
env.OTEL_EXPORTER_OTLP_ENDPOINT = "http://otel_collector:4318"

[[scenario]]
name = "otel_self_validation"
service = "app_under_test"
run = "myapp --otel-endpoint http://otel_collector:4318"

# Validate spans prove correct behavior
[[expect.span]]
name = "myapp.request"
kind = "server"
duration_ms = { min = 10, max = 5000 }

[[expect.span]]
name = "myapp.db_query"
parent = "myapp.request"
kind = "client"

[expect.graph]
must_include = [["myapp.request", "myapp.db_query"]]
acyclic = true

[expect.hermeticity]
no_external_services = true
resource_attrs_must_match = { "service.name" = "myapp" }
```

**Key Features:**
- **Deterministic validation** - Consistent validation across environments
- **5-Dimensional validation** - Structural, temporal, cardinality, hermeticity, attribute
- **Span validators** - Existence, count, attributes, hierarchy, events, duration
- **Graph validators** - Parent-child relationships and cycle detection
- **Hermeticity validators** - External service detection and resource validation

**Example: Framework validates itself using its own telemetry.**

### **Advanced Validation Framework**

The framework provides comprehensive validation across multiple dimensions:

- **Structural Validation** - Span hierarchy and relationships
- **Temporal Validation** - Execution time windows and containment
- **Cardinality Validation** - Count constraints across execution paths
- **Hermeticity Validation** - Isolation and contamination detection
- **Attribute Validation** - Semantic metadata validation

**Result:** Comprehensive correctness validation with multiple detection layers.

### **Fake-Green Detection** *(v1.0)*

**The Problem:** Tests that report "PASS" but never actually executed code.

**The Solution:** OTEL-first validation with 7 independent detection layers:

```toml
# Tests must PROVE they executed by generating telemetry
[[expect.span]]
name = "container.exec"
events.any = ["container.start", "container.exec", "container.stop"]

[expect.graph]
must_include = [["test.run", "container.exec"]]

[expect.counts]
spans_total.gte = 2

[expect.status]
all = "OK"
```

**7 Detection Layers:**
1. **Lifecycle Events** - Container operations generated events
2. **Span Graph** - Parent-child relationships exist
3. **Span Counts** - Expected number of operations occurred
4. **Temporal Ordering** - Operations occurred in correct sequence
5. **Window Containment** - Child operations within parent timeframes
6. **Status Validation** - All operations completed successfully
7. **Hermeticity** - Tests run in isolation without external dependencies

**Analyze Traces:**

**NOTE:** The `clnrm analyze` command requires the OpenTelemetry Collector to be running and properly configured. See the **[OpenTelemetry Integration Guide](docs/OPENTELEMETRY_INTEGRATION_GUIDE.md)** for step-by-step setup instructions.
```bash
# Run test with OTEL
clnrm run test.toml --otel-endpoint http://localhost:4318

# Validate telemetry evidence
clnrm analyze test.toml traces.json
```

**Result:**
- ✅ **PASS** = Code actually executed with proof
- ❌ **FAIL** = Fake-green test detected (no evidence)

**Documentation:**
- [User Guide](docs/FAKE_GREEN_DETECTION_USER_GUIDE.md) - How to use it
- [Developer Guide](docs/FAKE_GREEN_DETECTION_DEV_GUIDE.md) - How to extend it
- [TOML Schema](docs/FAKE_GREEN_TOML_SCHEMA.md) - Configuration reference
- [CLI Reference](docs/CLI_ANALYZE_REFERENCE.md) - Command usage

## 🎯 **Real Evidence - Not Claims**

### **Container Execution Works**
```bash
$ clnrm run
🚀 Executing test: basic_test
📋 Step 1: hello_world
🔧 Executing: echo Hello from cleanroom!
📤 Output: Hello from cleanroom!
✅ Output matches expected regex
✅ Step 'hello_world' completed successfully
🎉 Test 'basic_test' completed successfully!
```

### **Framework Self-Tests Work**
```bash
$ clnrm self-test
Framework Self-Test Results:
Total Tests: 5
Passed: 5
Failed: 0
✅ All framework functionality validated
```

### **Plugin Ecosystem Works**
```bash
$ clnrm plugins
📦 Available Service Plugins:
✅ generic_container (alpine, ubuntu, debian)
✅ surreal_db (database integration)
✅ network_tools (curl, wget, netcat)
✅ ollama (local AI model integration)
✅ vllm (high-performance LLM inference)
✅ tgi (Hugging Face text generation inference)

🧪 Experimental Plugins (clnrm-ai crate):
🎭 chaos_engine (controlled failure injection, network partitions)
🤖 ai_test_generator (AI-powered test case generation)
```

## 🏗️ Architecture

### **Plugin-Based Architecture**
- **Service Plugins** - Extensible container service management
- **Container Isolation** - Each test runs in fresh, isolated containers
- **Configuration-Driven** - TOML-based test definitions

### **Hermetic Testing**
- **Container Isolation** - Each test runs in completely isolated containers
- **Deterministic Execution** - Consistent results across environments
- **Resource Management** - Automatic cleanup and resource limits

## 📊 **Performance**

### **Container Management**
- Infrastructure for performance optimization
- Automatic container lifecycle management
- Service registry for efficient resource usage

### **Parallel Execution**
- Multi-worker test execution
- Resource-aware scheduling

## 🎮 **Commands**

| Command | Status | Description |
|---------|--------|-------------|
| `clnrm --version` | ✅ **Working** | Show version information |
| `clnrm --help` | ✅ **Working** | Show comprehensive help |
| `clnrm init` | ✅ **Working** | Zero-config project initialization |
| `clnrm run` | ✅ **Working** | Execute tests with real containers (change-aware) |
| `clnrm validate` | ✅ **Working** | Validate templates and TOML configuration |
| `clnrm template otel` | ✅ **Working** | Generate OTEL validation template |
| `clnrm self-test` | ✅ **Working** | Framework self-validation with OTEL |
| `clnrm dev --watch` | ✅ **Working** | Hot reload development mode |
| `clnrm dry-run` | ✅ **Working** | Fast validation without containers |
| `clnrm fmt` | ✅ **Working** | Deterministic TOML formatting |

## 🚀 **Getting Started**

### Prerequisites
- Rust 1.70 or later
- Docker or Podman
- 4GB+ RAM

### Installation

#### Via Homebrew (Recommended)
```bash
# Add the tap and install
brew tap seanchatmangpt/clnrm
brew install clnrm

# Verify installation
clnrm --version  # Should show: clnrm 1.0.0
```

#### Via Cargo
```bash
cargo install clnrm
```

#### From Source
```bash
git clone https://github.com/seanchatmangpt/clnrm
cd clnrm
cargo build --release
```

### First Test
```bash
# 1. Initialize project
clnrm init

# 2. Run tests (auto-discovery)
clnrm run

# 3. Validate everything works
clnrm self-test

# 4. Explore plugins
clnrm plugins
```

## 🎯 **What Makes This Special**

### **Framework Self-Testing**
The framework tests itself through the "eat your own dog food" principle. Every feature is validated by using the framework to test its own functionality.

### **Hermetic Container Testing**
Unlike traditional testing frameworks, clnrm provides **true hermetic testing** where each test runs in completely isolated, real containers with no test interference.

### **Universal Test Definition**
Single `.clnrm.toml` files can test any technology stack - databases, APIs, microservices - all through containerized execution.

## 📚 **Documentation**

- [TOML Reference](docs/v1.0/TOML_REFERENCE.md) - Configuration format
- [Fake Green Detection](docs/FAKE_GREEN_DETECTION_USER_GUIDE.md) - User guide
- [CLI Analyze](docs/CLI_ANALYZE_REFERENCE.md) - Analyze command reference
- [v1.0 Documentation](docs/) - Complete guides and references

## 🤝 **Contributing**

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines and core team standards.

## 📄 **License**

MIT License - see [LICENSE](LICENSE) file for details.

## 🎉 **Verification**

Every feature claimed above has been verified through actual execution:

```bash
# Verify core functionality
clnrm init && clnrm run && clnrm validate tests/

# Verify framework self-testing
clnrm self-test

# Verify plugin ecosystem
clnrm plugins
```

## 📋 **Changelog**

### **Version 1.0.0** *(2025-10-17)*
**Major Release: Production Ready - Foundation Complete**

#### **🚀 New Features**
- **Hot Reload (`dev --watch`)** - <3s latency from save to results
- **Change Detection** - SHA-256 file hashing, only rerun changed scenarios (10x faster)
- **Dry Run** - Fast validation without containers (<1s for 10 files)
- **TOML Formatting** - Deterministic `fmt` command with idempotency verification
- **Macro Library** - 8 reusable macros with 85% boilerplate reduction
- **Advanced Validation** - Temporal, structural, cardinality, hermeticity validation
- **Multi-Format Reports** - JSON, JUnit XML, SHA-256 digests

#### **🔧 Improvements**
- All v0.6.0 features included and working
- Production-ready error handling (no `.unwrap()` calls)
- Comprehensive test coverage (27 cache tests pass)
- Zero clippy warnings
- 100% backward compatible with v0.6.0

#### **📚 Documentation**
- DX Architecture guide (`docs/V1.0_ARCHITECTURE.md`)
- Updated README with v1.0 features
- Macro library documentation
- Template usage examples

**Breaking Changes:** None - all v0.6.0 `.toml` and `.toml.tera` files work unchanged.

**Performance Characteristics:**
- ✅ First green: Typically under 60s
- ✅ Hot reload latency: Approximately 3s average
- ✅ Dry-run validation: Typically under 1s for 10 files
- ✅ Cache operations: Typically under 100ms

---

**Built with ❤️ for reliable, hermetic integration testing. The framework tests itself to ensure maximum reliability and performance.**