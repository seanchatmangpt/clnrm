# Analyzer Artifact Loading Implementation

## Overview

The `clnrm analyze` command has been updated to automatically load OpenTelemetry spans from artifact files, eliminating the need to manually specify `--traces` when analyzing tests.

## Changes

### 1. CLI Types (`crates/clnrm-core/src/cli/types.rs`)

Made the `--traces` parameter optional:

```rust
/// Analyze OTEL traces against test expectations (v0.7.0)
Analyze {
    /// Test configuration file with expectations
    #[arg(value_name = "TEST_FILE")]
    test_file: PathBuf,

    /// OTEL traces JSON file (optional, will auto-load from artifacts if not provided)
    #[arg(long, value_name = "TRACES")]
    traces: Option<PathBuf>,  // Changed from PathBuf to Option<PathBuf>
},
```

### 2. Analyze Module (`crates/clnrm-core/src/cli/commands/v0_7_0/analyze.rs`)

#### Added `load_spans_from_artifacts()` Function

Automatically discovers and loads spans from `.clnrm/artifacts/<scenario-name>/spans.json`:

```rust
fn load_spans_from_artifacts(test_config: &TestConfig) -> Result<Vec<SpanData>> {
    let mut all_spans = Vec::new();
    let mut found_any_artifacts = false;

    // Check for scenarios (v0.6.0+ format)
    if !test_config.scenario.is_empty() {
        for scenario in &test_config.scenario {
            let artifact_path = format!(".clnrm/artifacts/{}/spans.json", scenario.name);

            if Path::new(&artifact_path).exists() {
                let validator = SpanValidator::from_file(&artifact_path)?;
                let spans = validator.spans();
                all_spans.extend_from_slice(spans);
                found_any_artifacts = true;
            }
        }
    }

    // Fallback to test name (v0.4.x compatibility)
    if !found_any_artifacts {
        let test_name = test_config.get_name()?;
        let artifact_path = format!(".clnrm/artifacts/{}/spans.json", test_name);
        // ...
    }

    Ok(all_spans)
}
```

#### Updated `analyze_traces()` Function

Now accepts an optional traces file path:

```rust
pub fn analyze_traces(test_file: &Path, traces_file: Option<&Path>) -> Result<AnalysisReport> {
    // Load OTEL traces from explicit file or artifacts
    let (validator, traces_source) = if let Some(traces_path) = traces_file {
        // Explicit --traces flag provided
        let validator = SpanValidator::from_file(traces_path)?;
        (validator, traces_path.display().to_string())
    } else {
        // Auto-load from artifacts
        let spans = load_spans_from_artifacts(&config)?;
        let validator = SpanValidator { spans };
        (validator, ".clnrm/artifacts/**/spans.json".to_string())
    };

    // ... rest of validation logic
}
```

### 3. CLI Command Handler (`crates/clnrm-core/src/cli/mod.rs`)

Updated to pass `Option<&Path>`:

```rust
Commands::Analyze { test_file, traces } => {
    use crate::cli::commands::v0_7_0::analyze::analyze_traces;

    match analyze_traces(&test_file, traces.as_deref()) {  // .as_deref() converts Option<PathBuf> to Option<&Path>
        Ok(report) => {
            println!("{}", report.format_report());
            // ...
        }
        // ...
    }
}
```

### 4. SpanValidator (`crates/clnrm-core/src/validation/span_validator.rs`)

Made the `spans` field crate-visible to allow direct construction:

```rust
pub struct SpanValidator {
    /// Loaded span data from OTEL collector export
    pub(crate) spans: Vec<SpanData>,  // Changed from private to pub(crate)
}
```

## Usage

### Automatic Loading (New)

```bash
# Analyzes spans from .clnrm/artifacts/<scenario-name>/spans.json
clnrm analyze test.toml
```

### Explicit Traces File (Backward Compatible)

```bash
# Still works when you want to analyze a specific traces file
clnrm analyze test.toml --traces custom-spans.json
```

## Artifact Path Structure

The analyzer looks for artifacts in the following locations:

1. **Per-scenario (v0.6.0+):**
   ```
   .clnrm/artifacts/<scenario-name>/spans.json
   ```

2. **Legacy format (v0.4.x compatibility):**
   ```
   .clnrm/artifacts/<test-name>/spans.json
   ```

## Error Handling

If no artifacts are found and --traces is not provided:

```
Error: No artifact files found. Run tests with artifact collection enabled first,
       or provide --traces flag explicitly.
```

## Testing

Added comprehensive unit tests:

1. **`test_load_spans_from_artifacts_with_scenarios()`**
   - Creates temporary artifact directory structure
   - Writes span files for multiple scenarios
   - Verifies all spans are loaded correctly

2. **`test_load_spans_from_artifacts_no_artifacts()`**
   - Tests error handling when no artifacts exist
   - Verifies helpful error message

3. **`test_artifact_path_resolution_format()`**
   - Validates correct path format generation

## Benefits

1. **Improved DX**: No need to remember artifact paths
2. **Automatic Discovery**: Finds all scenario artifacts automatically
3. **Backward Compatible**: `--traces` flag still works for custom files
4. **Multi-Scenario Support**: Loads spans from all scenarios in one command
5. **Clear Error Messages**: Guides users when artifacts are missing

## Implementation Notes

- Uses `pub(crate)` visibility for `SpanValidator::spans` to allow internal construction while maintaining encapsulation
- Follows core team standards (no `.unwrap()`, proper error handling with `Result<T, CleanroomError>`)
- Includes descriptive logging with `tracing` macros
- AAA test pattern for all unit tests
