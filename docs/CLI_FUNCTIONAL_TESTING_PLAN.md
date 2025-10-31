# CLI Command Functional Testing Plan - Core Team Standards

**Date**: 2025-01-17  
**Status**: Planning  
**Approach**: Behavior-focused testing with AAA pattern

---

## Goal

Verify all 25 CLI commands work end-to-end using core team best practices:
- **AAA Pattern** (Arrange, Act, Assert)
- **Behavior Testing** (not implementation details)
- **Proper Async Handling** (`#[tokio::test]`)
- **No False Positives** (verify actual work, not just Ok() returns)
- **Descriptive Test Names** (explain what is being tested)

---

## Core Team Testing Standards

### Testing Philosophy
**Test behaviors, not implementation details. The best test suite is the smallest one that still catches all bugs.**

### Patterns to Follow

#### ✅ AAA Pattern (Arrange, Act, Assert)
```rust
#[tokio::test]
async fn test_fmt_formats_toml_files_and_writes_changes() -> Result<()> {
    // Arrange - Set up test data and dependencies
    let test_file = create_temp_file("unformatted.toml", unformatted_content)?;
    let original_content = fs::read_to_string(&test_file)?;
    
    // Act - Execute the code under test
    format_files(&[test_file.path()], false, false)?;
    
    // Assert - Verify the results
    let formatted = fs::read_to_string(&test_file)?;
    assert_ne!(original_content, formatted); // BEHAVIOR: File was modified
    assert!(verify_toml_syntax(&formatted)?); // BEHAVIOR: Result is valid TOML
    Ok(())
}
```

#### ✅ Descriptive Test Names
```rust
// ✅ Good: Explains what is being tested
test_container_creation_with_valid_image_succeeds()
test_container_creation_with_invalid_image_fails_with_proper_error()
test_concurrent_scenario_execution_maintains_deterministic_order()

// ❌ Bad: Vague or unclear
test_container()
test_scenario()
```

#### ✅ Proper Async Test Functions
```rust
// ✅ Good: Async test for async operations
#[tokio::test]
async fn test_record_creates_baseline_with_digest() -> Result<()> {
    let results = run_record(Some(test_files), None).await?;
    // ...
}

// ❌ Bad: Sync test for async operations
#[test]
fn test_record() {
    // Won't work for async operations
}
```

#### ✅ Verify Actual Behavior
```rust
// ✅ Good: Verifies actual work was done
#[tokio::test]
async fn test_pull_actually_pulls_docker_images() -> Result<()> {
    // Arrange
    let images = vec!["alpine:latest"];
    
    // Act
    pull_images(Some(paths), true, 1).await?;
    
    // Assert - Verify image actually exists locally
    assert!(docker_image_exists("alpine:latest")?); // BEHAVIOR: Image was pulled
    Ok(())
}

// ❌ Bad: Just checks Ok() return
#[test]
fn test_pull() -> Result<()> {
    pull_images(None, false, 1)?; // Doesn't verify anything!
    Ok(())
}
```

---

## Test Categories

### Category 1: File Operations
**Commands**: `fmt`, `lint`, `validate`, `dry-run`

**Test Approach**:
- Create real TOML files (formatted/unformatted)
- Execute commands with real inputs
- Verify files are actually modified/validated
- Verify error handling for invalid inputs

**Key Behaviors to Verify**:
- `fmt`: Files are formatted, idempotency works
- `lint`: Errors are detected and reported correctly
- `validate`: Invalid configs fail with proper errors
- `dry-run`: No containers started, validation occurs

### Category 2: Test Execution
**Commands**: `run`, `init`, `template`

**Test Approach**:
- Create valid test configurations
- Execute commands with real scenarios
- Verify files are created/executed

**Key Behaviors to Verify**:
- `run`: Tests actually execute, containers start, results produced
- `init`: Files created in correct structure
- `template`: Generated templates are valid Tera syntax

### Category 3: Trace Analysis
**Commands**: `analyze`, `graph`, `spans`, `diff`

**Test Approach**:
- Create sample trace files (JSON)
- Create test configs with expectations
- Verify trace processing and output generation

**Key Behaviors to Verify**:
- `analyze`: Traces loaded, validators run, violations detected
- `graph`: Visualizations generated (ASCII/DOT/JSON/Mermaid)
- `spans`: Filtering works, output format correct
- `diff`: Differences detected and reported

### Category 4: Baseline Management
**Commands**: `record`, `repro`, `red-green`

**Test Approach**:
- Create test files for baseline
- Record baseline, verify file creation
- Reproduce from baseline, verify matching

**Key Behaviors to Verify**:
- `record`: Baseline file created, digest computed, results included
- `repro`: Tests re-run, results compared, digest verified
- `red-green`: TDD history tracked, state transitions validated

### Category 5: Service Management
**Commands**: `plugins`, `services`, `health`, `collector`

**Test Approach**:
- Test with actual service states
- Verify container lifecycle operations
- Verify status reporting

**Key Behaviors to Verify**:
- `plugins`: Lists actual plugins with correct info
- `services`: Shows active services (if any)
- `health`: Checks actual system health
- `collector`: Container lifecycle works (start/stop/status/logs)

### Category 6: Reporting and Utilities
**Commands**: `report`, `self-test`, `pull`, `render`

**Test Approach**:
- Create test data for reporting
- Verify actual output generation
- Verify Docker image operations

**Key Behaviors to Verify**:
- `report`: Reports generated in requested format
- `self-test`: Framework tests execute
- `pull`: Docker images actually pulled
- `render`: Templates render with variable substitution

### Category 7: Development Tools
**Commands**: `dev`

**Test Approach**:
- Create test files
- Start dev mode
- Modify files and verify rerun

**Key Behaviors to Verify**:
- File watching works
- Tests rerun on file change
- Filtering/timeboxing works

---

## Test Directory Structure

```
crates/clnrm-core/tests/cli_functional/
├── mod.rs                          # Test module
├── helpers.rs                      # Test helpers (AAA utilities)
├── test_data/                     # Sample test files
│   ├── valid_test.clnrm.toml
│   ├── invalid_test.clnrm.toml
│   ├── trace.json
│   ├── template.toml.tera
│   └── baseline.json
├── file_ops/
│   ├── fmt_test.rs                # fmt command tests
│   ├── lint_test.rs               # lint command tests
│   ├── validate_test.rs           # validate command tests
│   └── dry_run_test.rs            # dry-run command tests
├── execution/
│   ├── run_test.rs
│   ├── init_test.rs
│   └── template_test.rs
├── trace_analysis/
│   ├── analyze_test.rs
│   ├── graph_test.rs
│   ├── spans_test.rs
│   └── diff_test.rs
├── baseline/
│   ├── record_test.rs
│   ├── repro_test.rs
│   └── redgreen_test.rs
├── services/
│   ├── plugins_test.rs
│   ├── services_test.rs
│   ├── health_test.rs
│   └── collector_test.rs
├── reporting/
│   ├── report_test.rs
│   ├── self_test_test.rs
│   ├── pull_test.rs
│   └── render_test.rs
└── dev/
    └── dev_test.rs
```

---

## Verification Checklist Per Command

For each command, verify:
- [ ] Command executes without panicking
- [ ] Returns proper `Result<T, CleanroomError>` type
- [ ] Produces expected output/files
- [ ] Handles errors gracefully (invalid inputs)
- [ ] Provides meaningful error messages with context
- [ ] **Actually performs the work** (not just returns Ok(()))
- [ ] Follows async/sync patterns correctly
- [ ] Uses tracing, not println
- [ ] No unwrap()/expect() causes panics in normal operation

---

## Success Criteria

### All Commands Must:
1. **Execute successfully** with valid inputs
2. **Produce expected outputs** (files, stdout, etc.)
3. **Handle errors gracefully** with meaningful messages
4. **Actually perform work** (verify behaviors, not just Ok() returns)
5. **Follow core team standards** (no unwrap, proper errors, async/sync)

### Test Suite Must:
1. **Follow AAA pattern** (Arrange, Act, Assert)
2. **Use descriptive names** explaining behavior
3. **Use proper async functions** (`#[tokio::test]`)
4. **Test behaviors**, not implementation details
5. **Catch real bugs**, not false positives

---

## Implementation Phases

### Phase 1: Test Infrastructure
- Create test directory structure
- Implement test helpers (AAA utilities)
- Create sample test data files
- Set up temp directory utilities

### Phase 2-8: Command Category Tests
- Test each category following AAA pattern
- Verify actual behaviors, not just returns
- Document failures

### Phase 9: Failure Documentation
- Categorize failures (critical/minor)
- Create fix priorities
- Document reproduction steps

### Phase 10: Report Generation
- Generate `CLI_COMMAND_FUNCTIONALITY_REPORT.md`
- Include test results for all commands
- Document working/broken status

---

## Output

**Report**: `docs/CLI_COMMAND_FUNCTIONALITY_REPORT.md`

**Contents**:
- Test results for all 25 commands
- Status (Working/Broken/Partial)
- Test scenarios used
- Output verification results
- Known issues and limitations
- Fix priorities

---

**Last Updated**: 2025-01-17

