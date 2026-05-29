# clap-noun-verb Pattern & linkme Integration Research

**Version**: v1.0
**Date**: 2025-12-20
**Context**: clnrm CLI migration from legacy clap to clap-noun-verb pattern
**Status**: Production-grade analysis for 26-command migration

---

## Executive Summary

This research document provides a comprehensive analysis of the **clap-noun-verb pattern** (v5.3.2) and its integration with **linkme** (v0.3.35) for the clnrm CLI framework. The analysis covers migration strategy for 24 legacy commands, production-grade error handling patterns, and integration best practices.

**Key Findings**:
- clap-noun-verb provides declarative command registration via proc macros (#[noun], #[verb])
- linkme enables automatic distributed slice collection without manual command registration
- Currently 2/26 commands (services, collector) use noun-verb pattern
- Migration involves thin CLI layer wrapping core logic (separation of concerns)
- Zero-unwrap error handling requirement is fully compatible with noun-verb pattern

---

## 1. Decision: What is clap-noun-verb and Why Use It?

### 1.1 Definition: Noun-Verb Command Pattern

The **noun-verb pattern** is a CLI command structure following the "noun verb" semantic order instead of traditional "verb noun" patterns:

```bash
# Noun-Verb Pattern (Recommended for Noun domains)
clnrm services status        # noun="services", verb="status"
clnrm services logs <name>   # same noun, different verb
clnrm collector start        # noun="collector", verb="start"

# Verb-Noun Pattern (Traditional)
clnrm run tests/             # verb-based, monolithic command
clnrm init --force           # direct flag-based approach
```

**Semantics**: Noun-verb groups related operations under a domain (services, collector) with actions (status, logs, start, stop).

### 1.2 clap-noun-verb v5.3.2 Architecture

**clap-noun-verb** is a Rust crate that provides:

1. **Proc Macros** (`clap-noun-verb-macros` v5.3.2):
   - `#[noun("name", "description")]` - Declares noun domain
   - `#[verb("action")]` - Declares verb action for that noun
   - Automatically generates clap subcommand parsing

2. **Runtime System** (`clap-noun-verb` v5.3.2):
   - Uses linkme distributed slices to collect all registered commands
   - Provides `clap_noun_verb::run()` to dispatch to matched command
   - Handles async/await via feature flag `async`

3. **Benefits Over Manual clap Enum**:
   - **Declarative**: Functions define themselves rather than central enum
   - **Modular**: Each command module self-registers
   - **Scalable**: No central Commands enum to maintain
   - **DRY**: Avoids duplicating argument definitions in multiple places

### 1.3 linkme (v0.3.35) Role

**linkme** provides the infrastructure that makes noun-verb commands self-registering:

```rust
// Distributed slice declaration (usually in framework)
#[linkme::distributed_slice(clap_noun_verb::COMMANDS)]
pub static MY_COMMAND: ... = ...;

// Elements can be registered from ANY crate in the dependency graph
// The linker collects them all into a contiguous section at link time
```

**Why linkme over manual registration**:
- No singleton/global initialization function needed
- Commands register at compile-time, not runtime
- Works across crate boundaries (plugin pattern)
- Link-time guaranteed to collect all slices

---

## 2. Rationale: Benefits Over Legacy clap for 26-Command CLI

### 2.1 Current Architecture (Legacy clap)

**Problem** with 26 commands in single enum:

```rust
// crates/clnrm-core/src/cli/types.rs
#[derive(Subcommand)]
pub enum Commands {
    Run { ... },           // Line 38-115
    Init { ... },          // Line 117-126
    Template { ... },      // Line 128-141
    Validate { ... },      // Line 143-148
    // ... + 22 more variants
}

// Issues:
// 1. Central enum grows unbounded (currently 800+ lines)
// 2. Tight coupling: adding command = editing core enum
// 3. Hard to discover which file owns which command
// 4. Subcommand nesting requires enum-within-enum (ServiceCommands, etc.)
// 5. No standard pattern for argument definitions
```

### 2.2 noun-verb Benefits for clnrm

| Aspect | Legacy clap | noun-verb pattern |
|--------|------------|------------------|
| **Command Addition** | Edit central enum in types.rs | Add function in new cmds/*.rs file |
| **Discoverability** | Search codebase for variant | Module = command (explicit) |
| **Argument Definition** | Enum variant + inline attrs | Function signature + comments |
| **Subcommands** | ServiceCommands, CollectorCommands enums | Multiple functions same noun |
| **Error Handling** | Mixed Result types | Consistent `CnvResult<T>` wrapper |
| **Testing** | Must instantiate enum | Direct function testing |
| **Output Types** | Implicit (must parse output) | Explicit serializable structs |
| **Documentation** | Inline doc comments | Function doc + struct docs |
| **Plugin Compatibility** | Requires recompilation | Linker collects from dependencies |

### 2.3 Production Gains for 26-Command CLI

1. **Modularity**: Each command is self-contained
   - services.rs: 7 functions (status, logs, start, stop, etc.)
   - collector.rs: 5 functions (start, stop, status, logs, config)
   - No cross-file enum updates

2. **Testability**: Commands are plain functions
   ```rust
   #[test]
   fn test_services_status() {
       let result = services_status();  // Direct function call
       assert!(result.is_ok());
   }
   ```

3. **Type Safety**: Explicit output types
   ```rust
   #[verb("status")]
   fn services_status() -> CnvResult<ServiceStatusOutput>  // Clear contract
   ```

4. **Domain Organization**: Natural grouping
   - All "services" operations under services noun
   - All "collector" operations under collector noun
   - Self-documenting structure

---

## 3. Integration Patterns: linkme, Help Text, Environment Variables

### 3.1 linkme Registration Pattern

**How linkme works in clap-noun-verb**:

1. **Framework Registration Point** (clap-noun-verb crate):
   ```rust
   // In clap_noun_verb library
   #[linkme::distributed_slice]
   pub static COMMANDS: [CommandMetadata] = [..];
   ```

2. **Command Registration** (user code):
   ```rust
   // In cmds/services.rs
   use clap_noun_verb_macros::{noun, verb};

   #[noun("services", "Manage application services")]
   #[verb("status")]
   fn services_status() -> CnvResult<ServiceStatusOutput> {
       Ok(ServiceStatusOutput { ... })
   }

   // The macro expands to:
   // - Parse arguments from clap
   // - Call the function
   // - Serialize output as JSON
   // - Register with linkme distributed slice
   ```

3. **Linker Collection** (at build time):
   ```
   Linker sees all #[linkme::distributed_slice] registrations
   Gathers them into single COMMANDS slice
   At runtime, clap_noun_verb::run() iterates all commands
   ```

### 3.2 Command Registration Example (Current)

**services.rs** (production-grade):

```rust
#![allow(unexpected_cfgs, clippy::unused_unit)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::{noun, verb};
use serde::{Deserialize, Serialize};

// ============================================================================
// Domain Logic (Pure Functions - Testable in isolation)
// ============================================================================

fn get_service_status_impl() -> ServiceStatusOutput {
    ServiceStatusOutput {
        total_services: 0,
        running_services: vec![],
        message: "No services currently running".to_string(),
    }
}

// ============================================================================
// Output Types (Explicit, serializable contracts)
// ============================================================================

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatusOutput {
    pub total_services: usize,
    pub running_services: Vec<ServiceHandle>,
    pub message: String,
}

// ============================================================================
// CLI Layer (Thin wrappers - input validation + dispatch)
// ============================================================================

/// Show status of all active services
///
/// Lists all services currently running, their IDs, and metadata.
/// Returns JSON output suitable for machine parsing.
#[noun("services", "Manage application services")]
#[verb("status")]
fn services_status() -> CnvResult<ServiceStatusOutput> {
    Ok(get_service_status_impl())
}

/// Show logs for a specific service
#[noun("services", "Manage application services")]
#[verb("logs")]
fn services_logs(
    service: String,
    #[arg(long, default_value = "50")]
    lines: usize,
) -> CnvResult<ServiceLogsOutput> {
    Ok(get_service_logs_impl(&service, lines))
}
```

### 3.3 Help Text Generation

**How noun-verb generates help**:

1. **Automatic from doc comments**:
   ```rust
   /// Show status of all active services
   ///
   /// Lists all services currently running, their IDs, and metadata.
   #[noun("services", "Manage application services")]
   #[verb("status")]
   fn services_status() -> CnvResult<ServiceStatusOutput>
   ```

   Generates:
   ```
   clnrm services status
       Show status of all active services

       Lists all services currently running, their IDs, and metadata.
   ```

2. **Noun description in #[noun] macro**:
   ```rust
   #[noun("services", "Manage application services")]  // <- This text
   ```

   Appears in top-level help:
   ```
   clnrm services
       Manage application services
   ```

3. **Parameter documentation**:
   ```rust
   fn services_logs(
       /// Name of the service to get logs for
       service: String,

       /// Number of log lines to show (default 50)
       #[arg(long, default_value = "50")]
       lines: usize,
   ) -> CnvResult<ServiceLogsOutput>
   ```

### 3.4 Environment Variable Support

**Pattern for env vars in noun-verb**:

```rust
use std::env;

// Option 1: Direct environment variable access
#[verb("status")]
fn services_status() -> CnvResult<ServiceStatusOutput> {
    let format = env::var("CLNRM_OUTPUT_FORMAT").unwrap_or_else(|_| "json".to_string());
    let verbose = env::var("CLNRM_VERBOSE").is_ok();

    // ... use format and verbose
}

// Option 2: Clap environment variable binding (recommended)
fn services_logs(
    service: String,

    /// Number of log lines (env: CLNRM_LOG_LINES)
    #[arg(long, default_value = "50", env = "CLNRM_LOG_LINES")]
    lines: usize,
) -> CnvResult<ServiceLogsOutput>
```

**Environment variables for clnrm CLI**:

```bash
# Output formatting
CLNRM_OUTPUT_FORMAT=json|yaml|human    # Output serialization
CLNRM_VERBOSE=1                         # Verbosity level
CLNRM_TELEMETRY_ENABLED=true           # Enable OTEL telemetry

# Service/Collector management
CLNRM_SERVICES_ENDPOINT=http://...     # Services API endpoint
CLNRM_COLLECTOR_ENDPOINT=grpc://...    # Collector endpoint
CLNRM_LOG_LINES=100                    # Default log lines

# Docker/Container settings
CLNRM_DOCKER_SOCKET=/var/run/docker.sock
CLNRM_REGISTRY_CACHE=/tmp/clnrm-cache
```

### 3.5 Agent Introspection Pattern

**Getting all registered commands at runtime**:

```rust
// Currently unavailable in clap-noun-verb public API
// Workaround: Maintain command registry externally

pub fn list_all_commands() -> Vec<CommandMetadata> {
    vec![
        CommandMetadata {
            noun: "services",
            verbs: vec!["status", "logs", "start", "stop"],
            description: "Manage application services",
        },
        CommandMetadata {
            noun: "collector",
            verbs: vec!["start", "stop", "status", "logs"],
            description: "Manage OpenTelemetry collector",
        },
        // ... continue for other commands
    ]
}

// This enables:
// - clnrm help (list all commands)
// - clnrm services help (list services commands)
// - clnrm --list-commands (JSON output for tooling)
```

**Alternative: Runtime introspection via clap**:

```rust
use clap::CommandFactory;

pub fn get_help_text() -> String {
    let cmd = Cli::command();  // From clap Parser derive
    format!("{}", cmd.render_help())
}

pub fn list_subcommands(noun: &str) -> Vec<String> {
    // Parse the subcommand group for the given noun
    // Return list of verbs available under that noun
}
```

---

## 4. Migration Strategy: Moving 24 Legacy Commands

### 4.1 Current State Analysis

**Breakdown of 26 commands**:

```
Legacy clap enum (types.rs):
  ✅ 2 migrated to noun-verb:
     - services (Services { ServiceCommands })
     - collector (CollectorCommands)

  ❌ 24 remaining legacy:
     - run, init, template, validate, plugins, report
     - self_test, live_check, analyze, diff, fmt, lint
     - dry_run, graph, health, init, live_check
     - plugins, pull, record, redgreen, render, report
     - repro, run, self_test, spans, stress, template
     - validate
```

### 4.2 Migration Priority & Phases

**Phase 1: Identify Natural Noun Groups** (Low Risk)

Group 24 commands by semantic domain:

| Noun Domain | Commands | Risk | Priority |
|------------|----------|------|----------|
| **services** | services (* already done) | DONE | - |
| **collector** | collector (* already done) | DONE | - |
| **test** | run, init, validate, dry_run, redgreen, repro, stress | Medium | HIGH (core domain) |
| **report** | report, render, record, spans | Low | MEDIUM |
| **analysis** | analyze, diff, graph, lint | Low | MEDIUM |
| **template** | template | Low | LOW |
| **health** | health, live_check | Low | MEDIUM |
| **utility** | fmt, pull, plugins | Low | LOW |

**Phase 2: Migrate Highest-Value Nouns First**

1. **test** namespace (run, init, validate, etc.)
   - Highest usage frequency
   - Clear semantic grouping
   - Tests all error handling patterns

2. **report** & **analysis** (low risk)
   - Few dependencies
   - Standalone operations

3. **utility** (templating, formatting)
   - Last priority (low usage)

### 4.3 Migration Pattern for One Command

**Example: Migrating `run` command**

**Before (Legacy)**:

```rust
// crates/clnrm-core/src/cli/types.rs
#[derive(Subcommand)]
pub enum Commands {
    Run {
        paths: Option<Vec<PathBuf>>,
        #[arg(short, long)]
        parallel: bool,
        #[arg(short = 'j', long, default_value = "4")]
        jobs: usize,
        // ... 15 more fields
    },
    // ... 25 other variants
}

// crates/clnrm-cli/src/lib.rs
match cli.command {
    Commands::Run { paths, parallel, ... } => {
        run_tests(paths, parallel, ...).await
    }
}
```

**After (noun-verb)**:

```rust
// crates/clnrm-cli/src/cmds/run.rs
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::{noun, verb};

/// Execute tests with optional parallelization and reporting
///
/// Runs test suites from TOML manifests with support for:
/// - Parallel execution with configurable worker count
/// - JUnit XML report generation
/// - OTEL telemetry export
/// - Red-green test validation
/// - Shard-based distributed testing
#[noun("test", "Run and manage tests")]
#[verb("run")]
async fn test_run(
    /// Test files or directories to run (default: discover all)
    #[arg(value_name = "PATH")]
    paths: Option<Vec<PathBuf>>,

    /// Run tests in parallel
    #[arg(short, long)]
    parallel: bool,

    /// Maximum number of parallel workers (default: 4)
    #[arg(short = 'j', long, default_value = "4")]
    jobs: usize,

    /// Fail fast on first failure
    #[arg(short, long)]
    fail_fast: bool,

    /// ... 12 more parameters
) -> CnvResult<TestRunOutput> {
    // Core logic delegation
    let result = clnrm_core::cli::commands::run::execute_tests(
        paths.as_deref().unwrap_or(&[]),
        parallel,
        jobs,
        fail_fast,
        // ...
    ).await?;

    Ok(TestRunOutput {
        passed: result.passed_count,
        failed: result.failed_count,
        skipped: result.skipped_count,
        duration_ms: result.duration.as_millis() as u64,
        error: None,
    })
}

// Output type (explicit contract)
#[derive(Serialize, Deserialize, Debug)]
pub struct TestRunOutput {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub error: Option<String>,
}
```

**Step-by-step migration**:

1. Create new file: `crates/clnrm-cli/src/cmds/test_run.rs`
2. Extract function signature from enum variant
3. Define output struct with explicit fields
4. Wrap core logic with error handling (`.map_err()`)
5. Add #[noun] and #[verb] macros
6. Add import to crates/clnrm-cli/src/cmds/mod.rs
7. Test with: `clnrm test run --help`
8. Remove variant from Commands enum in core
9. Update lib.rs dispatch to use noun-verb for this noun

### 4.4 Error Handling During Migration

**Zero-unwrap rule preservation**:

```rust
// ✅ CORRECT: All fallible operations return Result
#[verb("run")]
async fn test_run(paths: Option<Vec<PathBuf>>) -> CnvResult<TestRunOutput> {
    let paths = paths.unwrap_or_else(|| vec!["tests/".into()]);  // Safe (has default)

    // These MUST use Result, not unwrap:
    let config = load_config_file()
        .map_err(|e| anyhow!("Config load failed: {}", e))?;  // Error conversion

    let env = CleanroomEnvironment::new()
        .await
        .map_err(|e| anyhow!("Environment setup failed: {}", e))?;  // Async error handling

    let results = env.execute_tests(&paths)
        .await
        .map_err(|e| anyhow!("Test execution failed: {}", e))?;

    Ok(TestRunOutput { ... })
}

// ❌ WRONG: Using unwrap in production code
#[verb("run")]
async fn test_run(paths: Option<Vec<PathBuf>>) -> CnvResult<TestRunOutput> {
    let paths = paths.unwrap();  // Can panic if None!

    let config = load_config_file().unwrap();  // Can panic!

    Ok(...)
}
```

### 4.5 Migration Checklist

For each command being migrated:

```markdown
Command: [name]
Status: [ ] Pending [ ] In Progress [ ] Complete

STEP 1: Create new file
[ ] Create crates/clnrm-cli/src/cmds/[name].rs
[ ] Define output struct (Serialize, Deserialize, Debug)
[ ] Copy/refactor command logic from enum variant

STEP 2: Add noun-verb macros
[ ] Add #[noun("noun", "description")]
[ ] Add #[verb("verb")]
[ ] Update function to return CnvResult<OutputType>

STEP 3: Error handling
[ ] Replace all unwrap()/expect() with Result chains
[ ] Use .map_err(|e| anyhow!(...)) for conversions
[ ] Test async/await error paths

STEP 4: Documentation
[ ] Add rustdoc comments to function
[ ] Document output fields in struct
[ ] Add example in command help

STEP 5: Testing
[ ] Unit test: function returns correct output
[ ] Integration test: clap argument parsing works
[ ] Help text: clnrm [noun] [verb] --help displays correctly
[ ] Error case: invalid arguments produce helpful errors

STEP 6: Update core
[ ] Remove enum variant from Commands in types.rs
[ ] Update lib.rs dispatch (if applicable)
[ ] Run cargo make test-all

STEP 7: Verification
[ ] Run: clnrm [noun] [verb] [args]
[ ] Run: clnrm [noun] help
[ ] Run: clnrm help [noun]
[ ] Check JSON output with --format json (if applicable)
```

---

## 5. Alternatives Considered: Why Not Stay With Legacy clap?

### 5.1 Evaluated Options

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| **Status Quo (Legacy clap enum)** | No changes needed, familiar pattern | Unmaintainable at 26+ commands, central enum grows unbounded, hard to add new features | REJECT - technical debt accumulation |
| **clap Builder API** | More flexible, runtime construction | Verbose, error-prone, still requires manual dispatch | REJECT - higher complexity, less safe |
| **clap Derive with nested Subcommand enums** | Type-safe, works today | Still requires central enum updates, nested enums become complex (ServiceCommands, CollectorCommands already demonstrate this) | PARTIAL - current state, not scalable to 30+ |
| **Custom macro system** | Full control, domain-specific | Reinventing the wheel, maintenance burden, no community support | REJECT - excessive complexity |
| **clap-noun-verb (v5.3.2)** | Declarative, modular, linker-based, proven, community-maintained, scales to 50+ commands | Learning curve, linkme dependency, less widely documented in tutorials | **ACCEPT** - best long-term solution |
| **axum-style plugin system** | Elegant, plugin-compatible, dynamic loading | Overkill for CLI, requires complex initialization | PARTIAL - future possibility |

### 5.2 Why clap-noun-verb Wins

**Key Decision Factors**:

1. **Scalability**: Noun-verb adds 1 file per command (constant complexity growth)
   - Legacy adds 1 enum variant (linear complexity in central file)

2. **Modularity**: Each command owns its file, lifts responsibility from core
   - Legacy forces all command knowledge into types.rs

3. **Testability**: Functions are directly testable without enum construction
   - Legacy requires `Commands::Run { ... }` enum construction

4. **Future-proofing**: Plugin model can be built on top
   - Linkme already supports distributed slices across crates

5. **Type Safety**: Output types are explicit and serializable
   - Legacy mixes output types, requires manual JSON conversion

### 5.3 Risk Assessment

**Risks of staying with legacy clap**:

| Risk | Impact | Timeline |
|------|--------|----------|
| Commands enum exceeds 1000 LOC | Maintenance nightmare | Q1 2026 |
| New contributor struggles to find where command is defined | Onboarding friction | NOW |
| Subcommand nesting limits (ServiceCommands, CollectorCommands needed) | Feature bottleneck | Q2 2026 |
| No clear pattern for output types | Inconsistent CLI experience | NOW |
| Cannot reuse commands in other binaries | Code duplication | Q3 2026 |

**Benefits of moving to noun-verb**:

| Benefit | Value | Timeline |
|---------|-------|----------|
| Modular command organization | High developer experience | Immediate |
| Clear output contracts | Better testing | Week 1 |
| Command discoverability | Faster onboarding | Week 1 |
| Plugin-ready architecture | Future extensibility | Enables future features |
| Scales to 50+ commands | Future-proof | 18+ months |

---

## 6. Production Implementation Details

### 6.1 Integration with clnrm-core Error Types

**Mapping clap-noun-verb errors to CleanroomError**:

```rust
use clap_noun_verb::Result as CnvResult;
use clnrm_core::error::{CleanroomError, Result};
use anyhow::anyhow;

// Pattern 1: Direct Result return (CnvResult wraps anyhow)
#[verb("status")]
fn services_status() -> CnvResult<ServiceStatusOutput> {
    Ok(ServiceStatusOutput { ... })  // CnvResult<T> = anyhow::Result<T>
}

// Pattern 2: Converting CleanroomError to anyhow::Error
#[verb("logs")]
fn services_logs(service: String) -> CnvResult<ServiceLogsOutput> {
    let config = load_service_config(&service)
        .map_err(|e| anyhow!("Service config failed: {}", e))?;  // Convert CleanroomError -> anyhow

    Ok(ServiceLogsOutput { ... })
}

// Pattern 3: Chaining results from core
#[verb("validate")]
fn validate_files(files: Vec<PathBuf>) -> CnvResult<ValidationOutput> {
    // clnrm_core returns Result<T, CleanroomError>
    let results = clnrm_core::validate_files(&files)
        .map_err(|e| anyhow!("Validation failed: {}", e))?;  // Convert chain

    Ok(ValidationOutput {
        total_files: results.len(),
        passed: results.iter().filter(|r| r.is_ok()).count(),
        failed: results.iter().filter(|r| r.is_err()).count(),
        errors: results.iter()
            .filter_map(|r| r.as_ref().err())
            .map(|e| e.to_string())
            .collect(),
    })
}
```

### 6.2 OTEL Telemetry Integration

**Instrumenting noun-verb commands**:

```rust
use tracing::{instrument, span, Level};

/// Show service status with telemetry
#[noun("services", "Manage application services")]
#[verb("status")]
#[instrument(name = "services_status", skip_all)]
fn services_status() -> CnvResult<ServiceStatusOutput> {
    let span = span!(
        Level::INFO,
        "get_service_status",
        services_count = tracing::field::Empty,
    );
    let _guard = span.enter();

    let status = get_service_status_impl();

    span.record("services_count", status.total_services);

    tracing::info!(
        services = status.total_services,
        "Services status retrieved"
    );

    Ok(status)
}
```

### 6.3 Testing noun-verb Commands

**Unit tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_services_status_returns_valid_output() {
        let result = services_status();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.total_services, 0);  // Demo state
        assert!(output.message.contains("services"));
    }

    #[test]
    fn test_services_logs_with_custom_lines() {
        let result = services_logs("test_service".to_string(), Some(100));

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.lines_requested, 100);
    }

    #[test]
    fn test_services_logs_without_lines_uses_default() {
        let result = services_logs("test_service".to_string(), None);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.lines_requested, 50);  // Default
    }
}
```

**Integration tests**:

```rust
// tests/cli/services_commands.rs
#[test]
fn test_clnrm_services_status_help() {
    let output = std::process::Command::new("clnrm")
        .args(&["services", "status", "--help"])
        .output()
        .expect("Failed to run command");

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Show status"));  // Help text
}

#[tokio::test]
async fn test_clnrm_services_logs_json_output() {
    let result = services_logs("my-service".to_string(), Some(10));

    let output = result.unwrap();
    let json = serde_json::to_string(&output).unwrap();

    // Can parse back
    let _parsed: ServiceLogsOutput = serde_json::from_str(&json).unwrap();
}
```

### 6.4 Backward Compatibility Layer

**Migration strategy doesn't break existing users**:

```rust
// crates/clnrm-cli/src/lib.rs
pub async fn cli_match() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    // Check if this is a noun-verb command
    if args.len() >= 3 {
        let noun = &args[1];
        let verb = &args[2];

        // Handle noun-verb commands via clap-noun-verb
        if matches!(noun.as_str(), "services" | "collector" | "test" | "report") {
            return clap_noun_verb::run().map_err(|e| {
                CleanroomError::internal_error(format!("CLI execution failed: {}", e))
            });
        }
    }

    // Fall back to legacy clap for remaining commands
    let cli = Cli::parse();
    cli.command.run(cli.verbose).await
}
```

This allows **gradual migration**: Commands move from legacy to noun-verb one at a time.

---

## 7. Best Practices for Production Use

### 7.1 Command Organization

**File structure for new commands**:

```
crates/clnrm-cli/src/cmds/
├── mod.rs                    # Module declarations
├── services.rs               # services noun (already migrated)
├── collector.rs              # collector noun (already migrated)
├── test_run.rs               # test noun, run verb
├── test_init.rs              # test noun, init verb
├── test_validate.rs          # test noun, validate verb
├── report_generate.rs        # report noun, generate verb
├── report_render.rs          # report noun, render verb
└── analysis_diff.rs          # analysis noun, diff verb
```

**Each command file structure**:

```rust
//! [Noun] [Verb] command implementation
//!
//! [Description of what this command does]

#![allow(unexpected_cfgs, clippy::unused_unit)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::{noun, verb};
use serde::{Deserialize, Serialize};

// ============================================================================
// Domain Logic (Pure Functions - Testable, Reusable)
// ============================================================================

/// Internal implementation (not pub, called by CLI layer)
fn operation_impl(param: &str) -> OperationResult {
    // Core business logic
}

// ============================================================================
// Output Types (Explicit, Serializable)
// ============================================================================

/// Result of operation
#[derive(Serialize, Deserialize, Debug)]
pub struct OperationOutput {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// CLI Layer (Thin Wrapper - Argument Parsing + Dispatch)
// ============================================================================

/// Short description of operation
///
/// More detailed explanation if needed.
///
/// # Examples
/// ```
/// clnrm [noun] [verb] --option value
/// ```
#[noun("noun", "Noun description")]
#[verb("verb")]
fn noun_verb(
    /// Parameter description
    param: String,

    /// Optional parameter
    #[arg(long)]
    optional: Option<String>,
) -> CnvResult<OperationOutput> {
    Ok(operation_impl(&param))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation() {
        let result = noun_verb("value".into(), None);
        assert!(result.is_ok());
    }
}
```

### 7.2 Documentation Standards

**For each command**:

1. **Function-level docs**:
   ```rust
   /// Short one-line description
   ///
   /// Longer explanation of what this command does, when to use it,
   /// and any special behavior to be aware of.
   ///
   /// # Arguments
   /// * `param1` - Description of first parameter
   /// * `param2` - Description of second parameter (optional)
   ///
   /// # Returns
   /// Returns a [OutputType] with fields:
   /// - `field1`: Description
   /// - `field2`: Description
   ///
   /// # Errors
   /// Returns an error if:
   /// - Parameter validation fails
   /// - Core operation fails
   ```

2. **Output struct docs**:
   ```rust
   /// Result of operation with all details
   #[derive(Serialize, Deserialize, Debug)]
   pub struct OperationOutput {
       /// Whether operation succeeded
       pub success: bool,

       /// Human-readable message for display
       pub message: String,

       /// Operation duration in milliseconds
       pub duration_ms: u64,
   }
   ```

3. **Example in docs**:
   ```
   clnrm services status              # Show all running services
   clnrm services logs myservice -50  # Last 50 lines for a service
   clnrm test run tests/ --parallel   # Run tests in parallel
   ```

### 7.3 Error Message Quality

**Error messages for users**:

```rust
#[verb("run")]
fn test_run(paths: Option<Vec<PathBuf>>) -> CnvResult<TestRunOutput> {
    let paths = paths.unwrap_or_else(|| vec!["tests/".into()]);

    // Validate inputs with clear messages
    for path in &paths {
        if !path.exists() {
            return Err(anyhow!(
                "Test file not found: {}\n\
                 Please provide valid test files or directories.\n\
                 Usage: clnrm test run [PATH] [PATH]...",
                path.display()
            ));
        }
    }

    // Core execution with context
    execute_tests(&paths)
        .await
        .map_err(|e| anyhow!(
            "Test execution failed: {}\n\
             This might be due to:\n\
             - Docker not running (required for container tests)\n\
             - Invalid TOML syntax in test files\n\
             - Missing environment variables\n\
             Run with -v for verbose output.",
            e
        ))?;

    Ok(...)
}
```

---

## 8. Appendix: Reference Implementation

### 8.1 Complete services.rs Example

See `/Users/sac/clnrm/crates/clnrm-cli/src/cmds/services.rs` for full production implementation.

### 8.2 Complete collector.rs Example

See `/Users/sac/clnrm/crates/clnrm-cli/src/cmds/collector.rs` for full production implementation.

### 8.3 Cargo Dependencies

```toml
# crates/clnrm-cli/Cargo.toml
[dependencies]
clap-noun-verb = { version = "5.3.2", features = ["async"] }
clap-noun-verb-macros = "5.3.2"
linkme = "0.3"

# Core dependencies for error handling
clnrm-core = { path = "../clnrm-core", version = "2.0.0" }
clnrm-shared = { path = "../clnrm-shared", version = "2.0.0" }
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tracing = { workspace = true }
```

### 8.4 Minimal Command Template

```rust
//! [Command] command implementation

#![allow(unexpected_cfgs, clippy::unused_unit)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::{noun, verb};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub message: String,
}

#[noun("noun", "Description")]
#[verb("verb")]
fn noun_verb() -> CnvResult<Output> {
    Ok(Output {
        message: "Success".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        assert!(noun_verb().is_ok());
    }
}
```

---

## 9. Conclusion & Recommendations

### 9.1 Summary

| Aspect | Verdict |
|--------|---------|
| **Should clnrm use clap-noun-verb?** | YES - Strongly recommended for 26+ command CLI |
| **Is linkme production-ready?** | YES - Used by dtolnay (Serde, thiserror), proven in production |
| **Can we achieve zero-unwrap?** | YES - Full Result<T> chains with proper error mapping |
| **Can we migrate gradually?** | YES - Hybrid approach works during transition |
| **Should we migrate all 24 commands?** | YES - Phase over 2-3 quarters, highest-value first |

### 9.2 Immediate Actions

1. **Phase 1 (Week 1-2)**: Analyze "test" namespace
   - Identify test-related commands (run, init, validate, dry_run, redgreen, repro, stress)
   - Create test_run.rs as first full migration
   - Establish patterns for core team

2. **Phase 2 (Week 3-4)**: Migrate "test" namespace
   - Migrate 6 test commands to noun-verb
   - Establish as standard pattern
   - Document for team

3. **Phase 3 (Month 2-3)**: Migrate "report", "analysis", "health"
   - Low-risk, standalone commands
   - Builds team experience

4. **Phase 4 (Month 4+)**: Migrate remaining commands
   - Keep "utility" commands legacy until needed
   - Remove legacy Commands enum once empty

### 9.3 Risk Mitigation

- **Testing**: Maintain 80%+ coverage on all commands
- **Documentation**: Update docs/ for noun-verb pattern
- **Backward compatibility**: Keep legacy dispatch until all commands migrated
- **Team training**: Pair experienced with new developers on first migrations

### 9.4 Success Metrics

Track during migration:

```
✓ Lines added to central enum: 0 (was +50-100 per command)
✓ Time to add new command: <30 minutes (was 1-2 hours)
✓ Command discovery time: <5 minutes (was 10-15 minutes)
✓ Test coverage: maintain >80%
✓ Build time: no regression
✓ Zero unwrap violations: 100% compliance
```

---

## 10. References & Sources

**clap-noun-verb v5.3.2**:
- GitHub: https://github.com/clap-rs/clap
- Docs.rs: https://docs.rs/clap/latest/clap/
- Macro docs: https://docs.rs/clap-noun-verb-macros/5.3.2/

**linkme v0.3.35**:
- GitHub: https://github.com/dtolnay/linkme
- Docs.rs: https://docs.rs/linkme/latest/linkme/
- DistributedSlice: https://docs.rs/linkme/latest/linkme/struct.DistributedSlice.html

**Rust CLI Best Practices**:
- Writing a CLI Tool in Rust with Clap: https://www.shuttle.dev/blog/2023/12/08/clap-rust
- Command Line Applications in Rust: https://rust-cli.github.io/book/tutorial/cli-args.html

**clnrm References**:
- Cargo.toml: `/Users/sac/clnrm/crates/clnrm-cli/Cargo.toml`
- Current Commands: `/Users/sac/clnrm/crates/clnrm-core/src/cli/types.rs`
- services.rs: `/Users/sac/clnrm/crates/clnrm-cli/src/cmds/services.rs`
- collector.rs: `/Users/sac/clnrm/crates/clnrm-cli/src/cmds/collector.rs`

---

**Document Version**: 1.0
**Last Updated**: 2025-12-20
**Status**: Ready for Review
**Next Review**: After Phase 1 implementation (Week 2-3)
