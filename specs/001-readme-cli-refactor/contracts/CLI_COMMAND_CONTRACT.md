# Contract: CLI Command Specifications

**Feature**: Complete README v2.1.0 and Partial CLI Refactor Migration
**Contract Type**: API/Interface
**Version**: 1.0.0

## Contract Overview

This contract defines the behavioral and structural requirements for all 26 CLI commands in clnrm.

## Universal Requirements (ALL commands MUST)

### 1. Return Type
```rust
pub fn command_name(args: Args) -> Result<Output, CleanroomError>
```
- MUST return `Result<T, CleanroomError>` (never panic in production code)
- MUST NOT use `unwrap()` or `expect()` in production paths
- Exemption: Tests (`#[test]`, `tests/`, `benches/`) MAY use `unwrap()`

### 2. OTEL Instrumentation
```rust
use tracing::instrument;

#[instrument(skip_all, fields(param1 = %arg1))]
pub fn command_name(arg1: String) -> Result<Output, CleanroomError> {
    tracing::info!("Starting {command_name} with param: {}", arg1);
    // ... implementation
    tracing::info!("✅ {command_name} completed successfully");
    Ok(output)
}
```
- MUST have `#[instrument]` attribute on public functions
- MUST log start and completion via `tracing::info!`
- MUST log errors via `tracing::error!`

### 3. Help Text
- MUST provide one-line description (< 80 chars)
- MUST include usage example in `--help` output
- MUST document all arguments and options
- SHOULD include environment variable support (for NounVerb commands)

### 4. Error Messages
```rust
Err(CleanroomError::validation_error(
    format!("Invalid path '{}': file does not exist", path.display())
))
```
- MUST be actionable (tell user how to fix)
- MUST include context (what operation failed, why)
- MUST use appropriate CleanroomError variant

### 5. Timeouts
- MUST have bounded execution time (no infinite loops)
- SHOULD respect `CLNRM_TIMEOUT` environment variable
- MUST timeout long-running operations with clear error

## Architecture-Specific Requirements

### Legacy Clap Commands (24 commands)

**File structure**:
```
crates/clnrm-cli/src/
├── commands.rs                 # Implementation functions
└── cmds/{command_name}.rs      # Command module
```

**Command definition** (in main.rs or cli.rs):
```rust
#[derive(Parser)]
enum Commands {
    /// Execute test specifications
    Run {
        /// Path to test file or directory
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    // ... 23 more
}
```

**Implementation pattern**:
```rust
// In commands.rs
pub fn run(path: &Path) -> Result<TestExecutionResult, CleanroomError> {
    tracing::info!("Executing tests from: {:?}", path);

    if !path.exists() {
        return Err(CleanroomError::validation_error(
            format!("Path does not exist: {}", path.display())
        ));
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CleanroomError::runtime_error(e.to_string()))?;

    rt.block_on(async {
        clnrm_core::executor::run_tests(path).await
    })
}
```

### NounVerb Commands (2 commands: services, collector)

**File structure**:
```
crates/clnrm-cli/src/
└── cmds/services.rs            # Self-contained with #[noun] and #[verb] macros
```

**Command definition**:
```rust
use clap_noun_verb::{noun, verb, linkme, CnvResult};
use linkme::distributed_slice;

#[distributed_slice(clap_noun_verb::NOUNS)]
static NOUN: clap_noun_verb::Noun = noun! {
    name: "services",
    help: "Service lifecycle management commands",
    verbs: [start_verb, stop_verb, status_verb, list_verb, restart_verb, logs_verb, health_verb]
};

#[verb(output(json), output(msgpack), env("CLNRM_SERVICE_NAME"))]
pub fn start(
    /// Service name to start
    name: String,
) -> CnvResult<ServiceStatus> {
    tracing::info!("Starting service: {}", name);

    let service = ServiceManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to create service manager: {}", e))?;

    service.start(&name)
        .map_err(|e| anyhow::anyhow!("Failed to start service '{}': {}", name, e))
}
```

**Registration**: Automatic via `linkme` distributed slice (no manual enum entry)

## Command-Specific Contracts

### Test Execution Category

#### `run` Command
**Purpose**: Execute test specifications from TOML files

**Input**:
```rust
pub fn run(path: &Path) -> Result<TestExecutionResult, CleanroomError>
```

**Output**:
```rust
pub struct TestExecutionResult {
    pub tests_run: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: std::time::Duration,
    pub failures: Vec<TestFailure>,
}
```

**Behavior**:
1. Validate path exists (error if not)
2. Load TOML specification(s)
3. Execute tests via `clnrm_core::executor`
4. Emit OTEL spans for each test
5. Return aggregate results

**Exit codes**:
- `0`: All tests passed
- `1`: One or more tests failed
- `2`: Invalid arguments
- `3`: Docker/runtime error

**Test contract**:
```rust
#[test]
fn test_run_valid_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.clnrm.toml");
    std::fs::write(&test_file, "[[tests]]\nname = \"test\"\nimage = \"ubuntu:latest\"\n").unwrap();

    let result = run(&test_file);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().tests_run, 1);
}

#[test]
fn test_run_nonexistent_path() {
    let result = run(Path::new("/nonexistent/path"));
    assert!(matches!(result, Err(CleanroomError::ValidationError(_))));
}
```

#### `dry-run` Command
**Purpose**: Validate configuration without execution

**Input/Output**: Same as `run`, but `tests_run == 0`

**Behavior**:
1. Load and parse TOML
2. Validate schema
3. Check Docker image availability
4. Return validation result (NO execution)

**Test contract**:
```rust
#[test]
fn test_dry_run_no_execution() {
    // Test must verify Docker container NOT started
    let test_file = create_valid_test_file();
    let result = dry_run(&test_file).unwrap();

    assert_eq!(result.tests_run, 0);  // No execution
    assert!(result.validation_passed);
}
```

### Configuration Category

#### `init` Command
**Purpose**: Initialize new test specification

**Input**:
```rust
pub fn init(path: &Path, template: Option<&str>) -> Result<InitResult, CleanroomError>
```

**Output**:
```rust
pub struct InitResult {
    pub file_path: PathBuf,
    pub template_used: String,
}
```

**Behavior**:
1. Check if file already exists (error if exists)
2. Load template (built-in or custom)
3. Write TOML to path
4. Return created file path

**Test contract**:
```rust
#[test]
fn test_init_creates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("new-test.clnrm.toml");

    let result = init(&test_file, None).unwrap();

    assert!(test_file.exists());
    assert_eq!(result.file_path, test_file);
}

#[test]
fn test_init_existing_file_errors() {
    let temp_file = create_temp_file();
    let result = init(temp_file.path(), None);

    assert!(matches!(result, Err(CleanroomError::ValidationError(_))));
}
```

### System Management Category

#### `services` Command (NounVerb)
**Purpose**: Service lifecycle management

**Verbs**: start, stop, restart, status, list, logs, health

**start verb**:
```rust
#[verb(output(json), output(msgpack), env("CLNRM_SERVICE_NAME"))]
pub fn start(name: String) -> CnvResult<ServiceStatus>
```

**Output**:
```rust
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,  // Running, Stopped, Failed
    pub pid: Option<u32>,
    pub uptime: Option<Duration>,
}
```

**Behavior**:
1. Check if service already running (error if yes)
2. Start service process
3. Wait for health check (timeout 30s)
4. Return status

**Test contract**:
```rust
#[test]
fn test_services_start() {
    let service_name = "test-service";

    // Arrange: Service not running
    assert!(!is_service_running(service_name));

    // Act: Start service
    let result = start(service_name.to_string()).unwrap();

    // Assert: Service is running
    assert_eq!(result.state, ServiceState::Running);
    assert!(is_service_running(service_name));

    // Cleanup
    stop(service_name.to_string()).unwrap();
}
```

## Environment Variable Specifications

### Universal Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CLNRM_TIMEOUT` | u64 | 300 | Global timeout in seconds |
| `CLNRM_LOG_LEVEL` | String | "info" | Tracing log level (trace, debug, info, warn, error) |
| `CLNRM_OTLP_ENDPOINT` | String | None | OTLP exporter endpoint |

### Command-Specific Environment Variables

**services command**:
- `CLNRM_SERVICE_NAME`: Default service name for operations
- `CLNRM_SERVICE_TIMEOUT`: Service startup timeout (default: 30s)

**collector command**:
- `CLNRM_COLLECTOR_PORT`: Port for telemetry collector (default: 4317)
- `CLNRM_COLLECTOR_BACKEND`: Storage backend (jaeger, zipkin, stdout)

## Exit Code Contract

All commands MUST use standard exit codes:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | All tests passed, operation completed |
| 1 | Failure | Tests failed, operation failed |
| 2 | Invalid arguments | Missing required arg, invalid path |
| 3 | Runtime error | Docker daemon unavailable, timeout |
| 4 | Configuration error | Invalid TOML, missing config |

**Implementation**:
```rust
impl From<CleanroomError> for i32 {
    fn from(err: CleanroomError) -> i32 {
        match err {
            CleanroomError::ValidationError(_) => 2,
            CleanroomError::RuntimeError(_) => 3,
            CleanroomError::ConfigError(_) => 4,
            _ => 1,
        }
    }
}
```

## Testing Requirements

### Unit Tests
Each command MUST have:
- ✅ Happy path test (valid inputs, expected success)
- ✅ Invalid input test (malformed args, errors gracefully)
- ✅ Missing dependency test (Docker unavailable, config missing)
- ✅ Timeout test (long-running operations timeout correctly)

### Integration Tests
Each command MUST have:
- ✅ End-to-end CLI invocation test (via `std::process::Command`)
- ✅ Help text validation (`--help` output includes command)
- ✅ Version output test (`--version` matches Cargo.toml)

### Contract Tests
Each command MUST validate:
- ✅ Output schema matches contract (JSON/YAML structure)
- ✅ OTEL spans emitted correctly (trace validation)
- ✅ Exit codes match specification
- ✅ Environment variables honored

**Example contract test**:
```rust
#[test]
fn test_run_output_contract() {
    let result = run(Path::new("tests/integration.clnrm.toml")).unwrap();

    // Verify output structure
    assert!(result.tests_run > 0);
    assert_eq!(result.passed + result.failed, result.tests_run);
    assert!(result.duration.as_secs() < 300);  // Respect timeout

    // Verify can serialize to JSON
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("tests_run"));
}
```

## Backward Compatibility Requirements

### During Hybrid Migration (Current State: 2/26 NounVerb)

**Legacy commands** (`run`, `dry-run`, etc.):
- MUST continue working with existing syntax
- MUST NOT require migration for users
- SHOULD emit deprecation warnings if migration path exists

**NounVerb commands** (`services`, `collector`):
- MUST support new noun-verb syntax
- SHOULD support legacy syntax with deprecation warning (if applicable)

**Fallback behavior** (in lib.rs):
```rust
pub fn run_cli() -> Result<()> {
    // Try clap-noun-verb first
    if let Some(result) = clap_noun_verb::run() {
        return result.map_err(CleanroomError::from);
    }

    // Fall back to legacy clap
    let matches = Cli::parse();
    match matches.command {
        Commands::Run { path } => commands::run(&path),
        // ... 23 more legacy commands
    }
}
```

### Post-Migration (Target State: 26/26 NounVerb)

**All commands**:
- MUST use noun-verb syntax
- SHOULD support legacy syntax for 1 major version
- MUST emit deprecation warning: `eprintln!("DEPRECATED: Use 'clnrm test run' instead of 'clnrm run'")`

## Compliance Validation

### Pre-Commit Checks
```bash
# All commands must have tests
cargo test --package clnrm-cli

# All commands must pass clippy
cargo make lint

# All commands must be formatted
cargo make fmt
```

### CI/CD Checks
```bash
# Full validation pipeline
cargo make validate

# Contract tests (OTEL spans, exit codes)
cargo test --package clnrm-core --test telemetry_validation
cargo test --package clnrm-core --test exit_codes
```

## Summary

This contract ensures:
- **26 commands** follow consistent patterns
- **Zero unwrap** in production code
- **OTEL instrumentation** on all commands
- **Chicago TDD** with unit, integration, and contract tests
- **Backward compatibility** during migration (Hybrid → NounVerb)
- **Clear error messages** with actionable guidance
- **Standard exit codes** for shell script integration
- **Environment variable support** (especially NounVerb commands)

All commands MUST pass contract validation before merge.
