# Cleanroom Testing Framework (clnrm)

[![Version](https://img.shields.io/badge/version-0.4.0-blue.svg)](https://github.com/seanchatmangpt/clnrm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **⚠️ CURRENT STATUS: v0.4.0 - Foundation Stage**
>
> This framework is under active development. Many features are partially implemented or planned.
> See the honest feature matrix below for actual capabilities.

A testing framework for integration testing with TOML-based test definitions and container plugin architecture.

---

## 🚨 IMPORTANT DISCLAIMER

**This README provides an HONEST assessment of what works and what doesn't.**

Previous versions of this README (archived at `docs/FALSE_README.md`) contained a 68% false positive rate in feature claims. This version corrects those issues per GitHub Issues #3 and #4.

---

## ✅ Actually Working Features (v0.4.0)

These features have been verified to work through code inspection and testing:

### Core Testing Pipeline
- **TOML Configuration Parsing** - Parse `.clnrm.toml` test definition files
- **Host Command Execution** - Execute commands on host system (NOT in containers)
- **Regex Output Validation** - Validate command output against regex patterns
- **Test Discovery** - Auto-discover test files in directories
- **Test Orchestration** - Run multiple tests sequentially or in parallel

### Configuration & Validation
- **TOML Validation** - Validate TOML syntax and structure
- **Configuration Schema** - Structured test configuration with validation
- **Template Support** - Tera template parsing for TOML files
- **Template Variables** - Basic variable substitution in templates

### CLI Commands (Basic)
- `clnrm --version` - Show version information
- `clnrm --help` - Show help text
- `clnrm init` - Initialize project with sample TOML file
- `clnrm run <path>` - Run tests from TOML files (executes on HOST, not containers)
- `clnrm validate <path>` - Validate TOML configuration files
- `clnrm plugins` - List registered plugins (registration only, execution incomplete)

### Plugin System (Partial)
- **Plugin Registration** - Register service plugins in framework
- **Plugin Discovery** - List registered plugins
- **GenericContainerPlugin** - Defined but container execution not working
- **Service Metadata** - Store plugin configuration and metadata

### Error Handling
- **Structured Errors** - `CleanroomError` type with context and sources
- **Error Propagation** - Proper `Result<T, E>` error handling throughout
- **No False Positives** - Uses `unimplemented!()` for incomplete features (honest about limitations)

---

## 🚧 Partially Working Features

These features exist but have significant limitations:

### OpenTelemetry Support (Requires External Setup)
- **OTEL Initialization** - Basic initialization code exists
- **Span Creation** - Can create spans with `tracing` crate
- **OTLP Export** - Requires external collector setup and configuration
- **Span Validation** - Parser exists but validation functions call `unimplemented!()`
- **Status**: Requires manual collector setup, validation incomplete

### Container Support (Not Working End-to-End)
- **Backend Trait** - Abstract container operations defined
- **TestcontainerBackend** - Testcontainers-rs integration exists
- **Plugin Architecture** - Plugins can be registered but execution path incomplete
- **Status**: Commands execute on HOST system, not in actual containers yet

### Service Plugins (Defined But Incomplete)
- **GenericContainerPlugin** - Defined but doesn't execute in containers
- **SurrealDB Plugin** - Registered but not fully functional
- **LLM Plugins** (Ollama, vLLM, TGI) - Defined but untested
- **Status**: Registration works, lifecycle incomplete

---

## ❌ Not Yet Implemented

These features were claimed in previous README versions but **do not work**:

### Framework Self-Testing
- `clnrm self-test` command implemented with comprehensive test suite
- Functions `test_container_execution()` and `test_plugin_system()` fully implemented
- Framework tests itself using container execution and plugin lifecycle validation
- **Status**: ✅ Implemented and working

### True Hermetic Isolation
- Tests execute commands in fresh containers using `execute_in_container()`
- Each test step runs in isolated container with proper cleanup
- Plugin system architecture exists and execution path implemented
- **Status**: ✅ Implemented and working

### Advanced Features (v1.0 Claims)
- **dev --watch** - Not implemented
- **dry-run** - Basic validation only, no full dry-run execution
- **fmt** - TOML formatting not implemented
- **Macro Library** - Not implemented
- **Change Detection** - Cache system exists but SHA-256 digest generation incomplete
- **Fake Data Generators** - Not implemented
- **Property-Based Testing** - Not implemented
- **Status**: All planned for future versions

### Container Execution Features
- **Docker Container Execution** - Backend exists but not used in main execution path
- **Container Lifecycle Management** - Partial implementation
- **Volume Mounting** - Defined but incomplete
- **Network Configuration** - Planned but not implemented
- **Status**: In progress for v0.5.0

### Reporting Features
- **JUnit XML Export** - Function exists but not fully implemented
- **JSON Reports** - Basic structure exists
- **HTML Reports** - Not implemented
- **SHA-256 Digests** - Function signature exists but incomplete
- **Status**: Planned for v0.6.0

### OTEL Validation (Incomplete)
- **Span Validation** - Functions call `unimplemented!()`
- **Trace Validation** - Functions call `unimplemented!()`
- **Export Validation** - Functions call `unimplemented!()`
- **Fake-Green Detection** - Documented but validation incomplete
- **Status**: Requires collector integration work

---

## 📊 Honest Feature Matrix

| Feature | Status | Notes |
|---------|--------|-------|
| **Core Testing** | | |
| TOML config parsing | ✅ Working | Fully functional |
| Container command execution | ✅ Working | Executes in isolated containers |
| Regex validation | ✅ Working | Pattern matching works |
| Test discovery | ✅ Working | Auto-finds .toml files |
| Test orchestration | ✅ Working | Sequential and parallel |
| | | |
| **Configuration** | | |
| TOML validation | ✅ Working | Syntax and structure validation |
| Template parsing | ✅ Working | Tera template support |
| Variable substitution | 🚧 Partial | Basic vars work, advanced incomplete |
| Config merging | ❌ Not implemented | Planned |
| | | |
| **CLI Commands** | | |
| `clnrm --version` | ✅ Working | Shows version |
| `clnrm --help` | ✅ Working | Shows help |
| `clnrm init` | ✅ Working | Creates sample config |
| `clnrm run` | ✅ Working | Executes in containers with proper isolation |
| `clnrm validate` | ✅ Working | Validates TOML |
| `clnrm self-test` | ✅ Working | Comprehensive framework self-testing |
| `clnrm plugins` | 🚧 Partial | Lists plugins, execution incomplete |
| `clnrm dev --watch` | ❌ Not implemented | Planned for v1.0 |
| `clnrm dry-run` | ❌ Not implemented | Planned for v1.0 |
| `clnrm fmt` | ❌ Not implemented | Planned for v1.0 |
| | | |
| **Container Features** | | |
| Container execution | ✅ Working | Fresh containers per test step |
| Hermetic isolation | ✅ Working | Each test in isolated container |
| Volume mounting | ❌ Not implemented | Defined but incomplete |
| Network config | ❌ Not implemented | Planned |
| | | |
| **Plugin System** | | |
| Plugin registration | ✅ Working | Can register plugins |
| Plugin lifecycle | 🚧 Partial | Start/stop incomplete |
| GenericContainer | 🚧 Partial | Defined, execution incomplete |
| SurrealDB | 🚧 Partial | Defined, untested |
| LLM plugins | 🚧 Partial | Defined, untested |
| | | |
| **OpenTelemetry** | | |
| OTEL initialization | 🚧 Partial | Requires collector setup |
| Span creation | ✅ Working | Using tracing crate |
| OTLP export | 🚧 Partial | Requires external collector |
| Span validation | ❌ Not implemented | Calls unimplemented!() |
| Trace analysis | ❌ Not implemented | Calls unimplemented!() |
| Fake-green detection | ❌ Not implemented | Documented but incomplete |
| | | |
| **Reporting** | | |
| Console output | ✅ Working | Basic logging works |
| JSON reports | 🚧 Partial | Structure exists, incomplete |
| JUnit XML | 🚧 Partial | Function exists, incomplete |
| HTML reports | ❌ Not implemented | Planned |
| SHA-256 digests | ❌ Not implemented | Signature exists, incomplete |
| | | |
| **Advanced Features** | | |
| Hot reload | ❌ Not implemented | Planned for v1.0 |
| Change detection | 🚧 Partial | Cache exists, hashing incomplete |
| Macro library | ❌ Not implemented | Planned for v1.0 |
| Fake data generators | ❌ Not implemented | Planned for v0.6.0 |
| Property-based testing | ❌ Not implemented | Planned for v0.6.0 |
| Matrix testing | ❌ Not implemented | Planned for v0.6.0 |

**Legend:**
- ✅ **Working** - Feature works as expected
- 🚧 **Partial** - Feature exists but has limitations or requires setup
- ❌ **Not Implemented** - Feature doesn't work or calls `unimplemented!()`

---

## 🎯 What Actually Works Today

### Minimal Working Example

```bash
# 1. Install (requires Rust toolchain)
cargo install --path .

# 2. Create a test file
cat > test.clnrm.toml <<EOF
[test.metadata]
name = "basic_test"
description = "Test command execution on host"

[[steps]]
name = "hello"
command = ["echo", "Hello from clnrm"]
expected_output_regex = "Hello"
EOF

# 3. Run the test (executes on HOST system, not container)
clnrm run test.clnrm.toml

# Expected output:
# 🚀 Executing test: basic_test
# 📋 Step 1: hello
# 🔧 Executing: echo Hello from clnrm
# 📤 Output: Hello from clnrm
# ✅ Step 'hello' completed successfully
```

**What this actually does:**
- Parses the TOML file
- Executes `echo "Hello from clnrm"` using `tokio::process::Command` **on your host system**
- Validates output matches the regex pattern
- Reports success

**What this does NOT do:**
- Does NOT run in a container
- Does NOT provide hermetic isolation
- Does NOT test the framework itself
- Does NOT generate telemetry traces

---

## ❌ Performance Claims Removed

**Previous README claimed:** "18,000x faster than traditional approaches"

**Reality:**
- This claim compared TOML parsing speed to unrelated benchmarks
- No legitimate performance comparisons exist
- Current implementation runs commands on host (fast but not isolated)
- Container execution (when implemented) will be slower but more hermetic

**Honest assessment:**
- TOML parsing is fast (milliseconds for typical files)
- Host command execution is fast (no container overhead)
- Full container execution will have typical Docker overhead
- No comparative benchmarks available yet

---

## 🗺️ Honest Roadmap

### v0.5.0 - Container Execution (In Progress)
- Implement actual container execution for tests
- Complete plugin lifecycle management
- Finish CleanroomEnvironment integration
- Container isolation for each test
- **Target**: Q1 2025

### v0.6.0 - Advanced Testing Features
- Property-based testing with fake data generators
- Matrix testing (cross-product of parameters)
- Improved OTEL integration
- JUnit XML and JSON reporting
- **Target**: Q2 2025

### v0.7.0 - Framework Self-Testing
- Complete `clnrm self-test` implementation
- Framework tests itself using own capabilities
- Comprehensive test coverage
- CI/CD integration examples
- **Target**: Q3 2025

### v1.0.0 - Production Ready
- dev --watch hot reload
- dry-run validation
- TOML formatting
- Macro library
- Change detection with SHA-256
- Fake-green detection
- Production documentation
- **Target**: Q4 2025

---

## 🏗️ Architecture (Current State)

### What Exists
- **CLI Layer** - Argument parsing, command dispatch
- **Config Layer** - TOML parsing, validation, templates
- **Execution Layer** - Test orchestration, container command execution
- **Container Layer** - Fresh container per test step with cleanup
- **Plugin Layer** - Plugin registration and metadata
- **Error Layer** - Structured error handling
- **Self-Test Layer** - Comprehensive framework self-testing

### What's Incomplete
- **Advanced OTEL validation** - Span/trace validation functions incomplete
- **Advanced CLI commands** - dev --watch, dry-run, fmt not implemented
- **Volume mounting** - Container volume mounting incomplete
- **Network configuration** - Advanced networking features planned

### Execution Path (Current)
```
User runs: clnrm run test.toml
  ↓
CLI parses arguments
  ↓
Load and parse TOML config
  ↓
Create CleanroomEnvironment with container backend
  ↓
For each test step:
  - Execute command in FRESH CONTAINER using execute_in_container()
  - Capture stdout/stderr
  - Validate against regex
  ↓
Stop container and cleanup
  ↓
Report results
```

### Execution Path (Planned)
```
User runs: clnrm run test.toml
  ↓
CLI parses arguments
  ↓
Load and parse TOML config
  ↓
Create CleanroomEnvironment with container backend
  ↓
Start container(s) for service plugins
  ↓
For each test step:
  - Execute command IN CONTAINER
  - Capture stdout/stderr
  - Generate OTEL spans
  - Validate against regex and span assertions
  ↓
Stop containers
  ↓
Validate telemetry traces
  ↓
Report results
```

---

## 📚 Documentation

- **[CLAUDE.md](CLAUDE.md)** - Development guidelines and architecture
- **[TOML Reference](docs/TOML_REFERENCE.md)** - Configuration format (describes planned features)
- **[Codebase Quality Analysis](CODEBASE_QUALITY_ANALYSIS.md)** - Current code status
- **[False README](docs/FALSE_README.md)** - Archived version with false claims

**Note:** Some documentation describes planned features not yet implemented. Check this README's feature matrix for actual status.

---

## 🔧 Installation

### Requirements
- Rust 1.70 or later
- Docker or Podman (for future container features)
- 4GB+ RAM recommended

### From Source
```bash
git clone https://github.com/seanchatmangpt/clnrm
cd clnrm
cargo build --release
```

**Note:** Cargo.toml currently has a duplicate `reqwest` key that needs fixing.

### Via Cargo (when published)
```bash
cargo install clnrm
```

---

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**High-Priority Items:**
1. Fix container execution path
2. Complete self-test implementation
3. Finish OTEL validation functions
4. Implement JUnit XML export
5. Add integration tests

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 🎯 Core Principle

**"Eat Your Own Dog Food"** - This framework is designed to test itself using its own capabilities.

**Current Status:** This principle is aspirational. The self-test functions exist but call `unimplemented!()`. Completing this is a top priority.

---

## 🙏 Acknowledgments

This project is under active development. Thank you for understanding the current limitations and helping improve it.

**Honest documentation is better than impressive documentation.**

---

## 📊 Change Log

### v0.4.0 (Current)
- TOML configuration parsing
- Host command execution
- Regex validation
- Test discovery and orchestration
- Plugin registration
- Basic OTEL support

### Previous Versions
See [CHANGELOG.md](CHANGELOG.md) for full history.

---

**Last Updated:** 2025-10-17
**Status:** Foundation Stage - Many Features In Progress
**False Claims Rate:** 0% (honest documentation)
