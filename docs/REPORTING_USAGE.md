# Report Generation Usage Guide

## Overview

The `clnrm-core` reporting module provides multi-format report generation for validation results:
- **JSON**: Structured data for programmatic access
- **JUnit XML**: CI/CD integration (Jenkins, GitHub Actions, etc.)
- **SHA-256 Digest**: Reproducibility verification

## Quick Start

```rust
use clnrm_core::{
    generate_reports, ReportConfig, ValidationReport,
    JsonReporter, JunitReporter, DigestReporter
};

// Create validation report
let mut report = ValidationReport::new();
report.add_pass("graph_topology");
report.add_pass("span_counts");
report.add_fail("hermeticity", "Cross-container communication detected".to_string());

// Generate all formats
let config = ReportConfig::new()
    .with_json("results/report.json")
    .with_junit("results/junit.xml")
    .with_digest("results/digest.txt");

let spans_json = r#"{"spans": [...]}"#;
generate_reports(&config, &report, spans_json)?;
```

## Individual Reporters

### JSON Reporter

```rust
use clnrm_core::{JsonReporter, ValidationReport};
use std::path::Path;

let mut report = ValidationReport::new();
report.add_pass("test1");
report.add_fail("test2", "Expected 2 but got 1".to_string());

JsonReporter::write(Path::new("report.json"), &report)?;
```

**Output** (`report.json`):
```json
{
  "passed": false,
  "total_passes": 1,
  "total_failures": 1,
  "passes": [
    "test1"
  ],
  "failures": [
    {
      "name": "test2",
      "error": "Expected 2 but got 1"
    }
  ]
}
```

### JUnit XML Reporter

```rust
use clnrm_core::{JunitReporter, ValidationReport};
use std::path::Path;

let mut report = ValidationReport::new();
report.add_pass("test1");
report.add_fail("test2", "Missing span".to_string());

JunitReporter::write(Path::new("junit.xml"), &report)?;
```

**Output** (`junit.xml`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="clnrm" tests="2" failures="1" errors="0">
  <testcase name="test1" />
  <testcase name="test2">
    <failure message="Missing span" />
  </testcase>
</testsuite>
```

### Digest Reporter

```rust
use clnrm_core::{DigestReporter};
use std::path::Path;

let spans_json = r#"{"spans": [{"name": "root"}]}"#;

DigestReporter::write(Path::new("digest.txt"), spans_json)?;
```

**Output** (`digest.txt`):
```
a1b2c3d4e5f6...  (64-character SHA-256 hash)
```

**Use Cases**:
- Verify test output hasn't changed between runs
- Detect configuration drift
- Ensure reproducible test results
- Version control for span data

## Configuration Builder

```rust
use clnrm_core::ReportConfig;

// Build configuration incrementally
let config = ReportConfig::new()
    .with_json("output/report.json")
    .with_junit("output/junit.xml")
    .with_digest("output/digest.txt");

// Or selectively enable formats
let json_only = ReportConfig::new()
    .with_json("report.json");

// Empty config (no reports generated)
let no_reports = ReportConfig::new();
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Tests
on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run tests
        run: cargo test -- --format junit > junit.xml
      - name: Publish test results
        uses: EnricoMi/publish-unit-test-result-action@v2
        if: always()
        with:
          files: junit.xml
```

### Jenkins

```groovy
pipeline {
    stages {
        stage('Test') {
            steps {
                sh 'cargo test'
            }
            post {
                always {
                    junit 'junit.xml'
                }
            }
        }
    }
}
```

## Error Handling

All reporters follow core team standards:

```rust
use clnrm_core::{Result, CleanroomError};

fn generate_test_report() -> Result<()> {
    let report = ValidationReport::new();

    // Proper error propagation (no unwrap/expect)
    JsonReporter::write(Path::new("report.json"), &report)
        .map_err(|e| {
            CleanroomError::report_error(
                format!("Failed to generate JSON report: {}", e)
            )
        })?;

    Ok(())
}
```

## Advanced Usage

### Digest Comparison

```rust
use clnrm_core::DigestReporter;

// Generate digest for baseline
let baseline_json = std::fs::read_to_string("baseline.json")?;
let baseline_digest = DigestReporter::compute_digest(&baseline_json);

// Compare with current run
let current_json = std::fs::read_to_string("current.json")?;
let current_digest = DigestReporter::compute_digest(&current_json);

if baseline_digest != current_digest {
    eprintln!("Warning: Span data has changed!");
    eprintln!("Baseline: {}", baseline_digest);
    eprintln!("Current:  {}", current_digest);
}
```

### Parallel Report Generation

```rust
use clnrm_core::{JsonReporter, JunitReporter, DigestReporter};
use rayon::prelude::*;

// Generate all reports in parallel
let report = ValidationReport::new();
let spans_json = "...";

vec![
    || JsonReporter::write(Path::new("report.json"), &report),
    || JunitReporter::write(Path::new("junit.xml"), &report),
    || DigestReporter::write(Path::new("digest.txt"), spans_json),
]
.par_iter()
.for_each(|f| {
    if let Err(e) = f() {
        eprintln!("Report generation failed: {}", e);
    }
});
```

## Testing

All reporters include comprehensive tests:

```bash
# Run all reporting tests
cargo test -p clnrm-core reporting

# Run specific reporter tests
cargo test -p clnrm-core json_reporter
cargo test -p clnrm-core junit_reporter
cargo test -p clnrm-core digest_reporter
```

## Features

- ✅ Zero `unwrap()`/`expect()` - proper error handling
- ✅ Comprehensive test coverage (22 tests)
- ✅ XML/JSON escaping for special characters
- ✅ Deterministic SHA-256 hashing
- ✅ Builder pattern for configuration
- ✅ CI/CD ready output formats
- ✅ Production-ready code quality

## Module Structure

```
src/reporting/
├── mod.rs           # Main module with generate_reports()
├── json.rs          # JSON report generator
├── junit.rs         # JUnit XML generator
└── digest.rs        # SHA-256 digest generator
```

## API Reference

### `generate_reports(config, report, spans_json) -> Result<()>`

Generate all configured report formats.

**Parameters**:
- `config: &ReportConfig` - Report configuration
- `report: &ValidationReport` - Validation results
- `spans_json: &str` - Raw JSON spans for digest

**Returns**: `Result<()>`

### `JsonReporter::write(path, report) -> Result<()>`

Generate JSON report.

### `JunitReporter::write(path, report) -> Result<()>`

Generate JUnit XML report.

### `DigestReporter::write(path, spans_json) -> Result<()>`

Generate SHA-256 digest file.

### `DigestReporter::compute_digest(spans_json) -> String`

Compute SHA-256 hash without writing to file.

## See Also

- [OTEL_PRD.md](../OTEL-PRD.md) - OTEL validation specification
- [TESTING.md](TESTING.md) - Testing guide
- [CLI_GUIDE.md](CLI_GUIDE.md) - CLI documentation
