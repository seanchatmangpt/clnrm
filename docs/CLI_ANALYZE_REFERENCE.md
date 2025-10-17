# CLI Analyze Command Reference

## Table of Contents

1. [Overview](#overview)
2. [Command Syntax](#command-syntax)
3. [Options and Flags](#options-and-flags)
4. [Output Formats](#output-formats)
5. [Exit Codes](#exit-codes)
6. [Examples](#examples)
7. [Troubleshooting](#troubleshooting)

---

## Overview

The `clnrm analyze` command validates OpenTelemetry traces against configured expectations to detect fake-green tests.

**Purpose:** Catch tests that report "PASS" but never actually executed code.

**How it works:**
1. Load test configuration (`.clnrm.toml`)
2. Load OTEL traces (JSON file)
3. Run 7 validation layers
4. Report pass/fail with detailed diagnostics

---

## Command Syntax

### Basic Usage

```bash
clnrm analyze <test-file> <traces-file>
```

### Arguments

#### `<test-file>` (required)

Path to test configuration file containing expectations.

**Format:** `.clnrm.toml` file with `[expect]` section

**Example:**
```bash
clnrm analyze tests/my-test.clnrm.toml traces.json
```

#### `<traces-file>` (required)

Path to JSON file containing OTEL traces.

**Format:** JSON file with OTEL span data

**Example:**
```bash
clnrm analyze test.toml /tmp/otel-traces-123.json
```

---

## Options and Flags

### `--verbose` / `-v`

Enable verbose output with detailed validation information.

**Usage:**
```bash
clnrm analyze test.toml traces.json --verbose
```

**Output:**
```
📊 OTEL Validation Report (Verbose)
===================================

Test: my_test
Traces File: traces.json
Span Count: 5
Event Count: 12

Validators:
  ✅ Span Expectations (2/2 passed)
     - Span 'clnrm.run' found with all required attributes
     - Span 'container.exec' found with lifecycle events

  ✅ Graph Structure (all 1 edges present)
     - Edge found: 'clnrm.run' -> 'container.exec'

  ✅ Counts (spans_total: 5)
     - Total spans: 5 (expected: ≥2) ✓
     - Error count: 0 (expected: 0) ✓

  ✅ Window Containment (all 1 windows satisfied)
     - Window 'clnrm.run' contains ['container.exec'] ✓

  ✅ Ordering (all constraints satisfied)
     - 'plugin.registry' precedes 'container.exec' ✓

  ✅ Status (all spans OK)
     - All 5 spans have status OK ✓

  ✅ Hermeticity (no external services detected)
     - No external network attributes found ✓

Result: PASS (7/7 validators passed)
Digest: sha256:a1b2c3d4... (recorded for reproduction)
```

### `--format <format>`

Specify output format.

**Valid values:**
- `human` (default): Human-readable report
- `json`: JSON output for programmatic parsing
- `junit`: JUnit XML format for CI/CD integration

**Examples:**

**Human format (default):**
```bash
clnrm analyze test.toml traces.json
```

**JSON format:**
```bash
clnrm analyze test.toml traces.json --format json
```

**JUnit format:**
```bash
clnrm analyze test.toml traces.json --format junit > results.xml
```

### `--output <file>` / `-o <file>`

Write output to file instead of stdout.

**Usage:**
```bash
clnrm analyze test.toml traces.json -o report.txt
clnrm analyze test.toml traces.json --format json -o report.json
```

### `--fail-fast`

Stop validation on first failure (don't run remaining validators).

**Usage:**
```bash
clnrm analyze test.toml traces.json --fail-fast
```

**Behavior:**
- Without flag: All 7 validators run regardless of failures
- With flag: First validator failure stops execution

### `--no-color`

Disable colored output (useful for CI/CD or file output).

**Usage:**
```bash
clnrm analyze test.toml traces.json --no-color
```

### `--quiet` / `-q`

Suppress all output except errors.

**Usage:**
```bash
clnrm analyze test.toml traces.json --quiet
echo $?  # Check exit code: 0 = pass, 1 = fail
```

### `--help` / `-h`

Show help information.

**Usage:**
```bash
clnrm analyze --help
```

---

## Output Formats

### Human Format (Default)

Human-readable report with visual indicators.

**Example:**
```
📊 OTEL Validation Report
========================

Test: api_integration_test
Traces: 5 spans, 12 events

Validators:
  ✅ Span Expectations (2/2 passed)
  ✅ Graph Structure (all 1 edges present)
  ✅ Counts (spans_total: 5)
  ✅ Window Containment (all 1 windows satisfied)
  ✅ Ordering (all constraints satisfied)
  ✅ Status (all spans OK)
  ✅ Hermeticity (no external services detected)

Result: PASS (7/7 validators passed)
Digest: sha256:abc123... (recorded for reproduction)
```

**Failure Example:**
```
📊 OTEL Validation Report
========================

Test: api_integration_test
Traces: 0 spans, 0 events

Validators:
  ❌ Span Expectations (Expected span 'http.request' not found)
  ❌ Graph Structure (FAIL: required edge not found)
  ❌ Counts (FAIL: expected at least 2 items, found 0)
  ⚠️  Window Containment (not evaluated due to prior failures)
  ⚠️  Ordering (not evaluated due to prior failures)
  ⚠️  Status (not evaluated due to prior failures)
  ⚠️  Hermeticity (not evaluated due to prior failures)

Result: FAIL (5/7 validators failed)
```

### JSON Format

Machine-readable JSON output.

**Schema:**
```json
{
  "test_name": "api_integration_test",
  "traces_file": "traces.json",
  "span_count": 5,
  "event_count": 12,
  "digest": "sha256:abc123...",
  "validators": [
    {
      "name": "Span Expectations",
      "passed": true,
      "details": "2/2 passed"
    },
    {
      "name": "Graph Structure",
      "passed": true,
      "details": "all 1 edges present"
    }
  ],
  "is_success": true,
  "pass_count": 7,
  "failure_count": 0
}
```

**Usage with `jq`:**
```bash
# Extract pass/fail status
clnrm analyze test.toml traces.json --format json | jq '.is_success'

# List failed validators
clnrm analyze test.toml traces.json --format json | \
  jq '.validators[] | select(.passed == false) | .name'

# Get failure count
clnrm analyze test.toml traces.json --format json | jq '.failure_count'
```

### JUnit XML Format

JUnit-compatible XML for CI/CD integration.

**Schema:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="OTEL Validation" tests="7" failures="0" errors="0" time="0.125">
    <testcase name="Span Expectations" classname="api_integration_test" time="0.010"/>
    <testcase name="Graph Structure" classname="api_integration_test" time="0.005"/>
    <testcase name="Counts" classname="api_integration_test" time="0.003"/>
    <testcase name="Window Containment" classname="api_integration_test" time="0.008"/>
    <testcase name="Ordering" classname="api_integration_test" time="0.006"/>
    <testcase name="Status" classname="api_integration_test" time="0.004"/>
    <testcase name="Hermeticity" classname="api_integration_test" time="0.002"/>
  </testsuite>
</testsuites>
```

**Failure Example:**
```xml
<testcase name="Span Expectations" classname="api_integration_test" time="0.010">
  <failure type="ValidationError" message="Expected span 'http.request' not found">
Expected span 'http.request' not found in trace
  </failure>
</testcase>
```

**Integration with CI/CD:**

**GitHub Actions:**
```yaml
- name: Run validation
  run: clnrm analyze test.toml traces.json --format junit > results.xml

- name: Publish test results
  uses: EnricoMi/publish-unit-test-result-action@v2
  with:
    files: results.xml
```

**Jenkins:**
```groovy
stage('Validate') {
  steps {
    sh 'clnrm analyze test.toml traces.json --format junit > results.xml'
    junit 'results.xml'
  }
}
```

---

## Exit Codes

The `analyze` command uses standard Unix exit codes:

| Exit Code | Meaning | Description |
|-----------|---------|-------------|
| `0` | Success | All validators passed |
| `1` | Validation Failure | One or more validators failed |
| `2` | Configuration Error | Invalid test config or traces file |
| `3` | File Not Found | Test file or traces file not found |
| `4` | Parse Error | Failed to parse TOML or JSON |
| `5` | Internal Error | Unexpected error in validator |

### Example Usage with Exit Codes

```bash
#!/bin/bash

# Run validation
clnrm analyze test.toml traces.json
exit_code=$?

case $exit_code in
  0)
    echo "✅ Validation passed"
    ;;
  1)
    echo "❌ Fake-green test detected!"
    exit 1
    ;;
  2)
    echo "⚠️  Configuration error"
    exit 2
    ;;
  3)
    echo "⚠️  File not found"
    exit 3
    ;;
  4)
    echo "⚠️  Parse error"
    exit 4
    ;;
  *)
    echo "⚠️  Unexpected error"
    exit 5
    ;;
esac
```

---

## Examples

### Example 1: Basic Validation

```bash
# Run validation with default output
clnrm analyze tests/integration.clnrm.toml traces.json
```

**Output:**
```
📊 OTEL Validation Report
========================

Test: integration_test
Traces: 5 spans, 12 events

Validators:
  ✅ Span Expectations (2/2 passed)
  ✅ Graph Structure (all 1 edges present)
  ✅ Counts (spans_total: 5)
  ✅ Status (all spans OK)

Result: PASS (4/4 validators passed)
```

### Example 2: Verbose Output

```bash
# Get detailed validation information
clnrm analyze tests/api.clnrm.toml traces.json --verbose
```

### Example 3: JSON Output for Parsing

```bash
# Generate JSON report
clnrm analyze test.toml traces.json --format json > report.json

# Extract specific information
jq '.validators[] | select(.passed == false)' report.json
```

### Example 4: CI/CD Integration

```bash
# Run in CI/CD pipeline
clnrm analyze test.toml traces.json \
  --format junit \
  --output results.xml \
  --no-color

# Check exit code
if [ $? -ne 0 ]; then
  echo "❌ Fake-green tests detected!"
  exit 1
fi
```

### Example 5: Quiet Mode for Scripting

```bash
# Only check pass/fail
clnrm analyze test.toml traces.json --quiet

if [ $? -eq 0 ]; then
  echo "PASS"
else
  echo "FAIL"
fi
```

### Example 6: Fail-Fast Mode

```bash
# Stop on first failure
clnrm analyze test.toml traces.json --fail-fast
```

**Output (stops after first failure):**
```
📊 OTEL Validation Report
========================

Test: my_test
Traces: 0 spans, 0 events

Validators:
  ❌ Span Expectations (Expected span 'container.exec' not found)

Result: FAIL (1 validator failed, 6 not evaluated)
```

### Example 7: Multiple Test Validation

```bash
#!/bin/bash

# Validate all tests in directory
for test_file in tests/*.clnrm.toml; do
  test_name=$(basename "$test_file" .clnrm.toml)
  traces_file="/tmp/traces-${test_name}.json"

  echo "Validating: $test_name"
  clnrm analyze "$test_file" "$traces_file" --quiet

  if [ $? -ne 0 ]; then
    echo "  ❌ FAIL"
    exit 1
  else
    echo "  ✅ PASS"
  fi
done

echo "All tests validated successfully!"
```

### Example 8: Debugging Failed Validation

```bash
# Run with verbose output and save to file
clnrm analyze test.toml traces.json \
  --verbose \
  --output debug-report.txt

# Also check the traces file
jq '.spans | length' traces.json
jq '.spans[].name' traces.json
```

---

## Troubleshooting

### Issue: "File not found"

**Error:**
```
Error: Failed to read test file tests/my-test.toml: No such file or directory
```

**Solution:**
```bash
# Check file exists
ls -la tests/my-test.toml

# Use absolute path
clnrm analyze /full/path/to/test.toml /full/path/to/traces.json
```

### Issue: "Failed to parse TOML"

**Error:**
```
Error: Failed to parse test TOML: expected '=', found ':'
```

**Solution:**
```bash
# Validate TOML syntax
cat test.toml | toml-lint

# Common issues:
# - Missing quotes around strings
# - Incorrect table syntax
# - Trailing commas
```

### Issue: "Invalid JSON traces"

**Error:**
```
Error: Failed to parse traces JSON: EOF while parsing an object
```

**Solution:**
```bash
# Validate JSON syntax
jq '.' traces.json

# Check file is not empty
wc -l traces.json

# Verify JSON structure
jq '.spans | type' traces.json  # Should be "array"
```

### Issue: "No spans collected"

**Output:**
```
Traces: 0 spans, 0 events
Result: FAIL (all validators failed)
```

**Solution:**
```bash
# 1. Check OTEL exporter is configured
cat test.toml | grep -A 3 "\[otel\]"

# 2. Verify OTEL collector is running
curl http://localhost:4318/v1/traces

# 3. Check instrumentation is enabled
RUST_LOG=trace clnrm run test.toml

# 4. Verify traces were written
ls -la /tmp/traces*.json
```

### Issue: "Validator unexpectedly failed"

**Output:**
```
❌ Graph Structure (FAIL: required edge not found)
```

**Solution:**
```bash
# Run with verbose output
clnrm analyze test.toml traces.json --verbose

# Inspect span relationships
jq '.spans[] | {name, parent: .parent_span_id}' traces.json

# Check expected edges in config
cat test.toml | grep -A 5 "\[expect.graph\]"
```

### Issue: "Exit code 5 (Internal Error)"

**Error:**
```
Internal error during validation
```

**Solution:**
```bash
# Enable debug logging
RUST_LOG=debug clnrm analyze test.toml traces.json

# Check for stack trace
RUST_BACKTRACE=1 clnrm analyze test.toml traces.json

# Report bug with:
# - Test config
# - Traces file
# - Full error output
```

---

## Advanced Usage

### Pipeline with Multiple Validators

```bash
#!/bin/bash

# Step 1: Run test and collect traces
clnrm run test.toml --otel-endpoint http://localhost:4318

# Step 2: Export traces to JSON
curl http://localhost:4318/v1/traces > traces.json

# Step 3: Validate traces
clnrm analyze test.toml traces.json --format json > validation.json

# Step 4: Check specific validators
passed_validators=$(jq '.pass_count' validation.json)
total_validators=$(jq '.validators | length' validation.json)

echo "Passed: $passed_validators / $total_validators validators"

if [ "$passed_validators" -ne "$total_validators" ]; then
  echo "Failed validators:"
  jq '.validators[] | select(.passed == false) | .name' validation.json
  exit 1
fi
```

### Custom Reporting Script

```bash
#!/bin/bash

# Run validation and generate custom report
result=$(clnrm analyze test.toml traces.json --format json)

# Extract data
test_name=$(echo "$result" | jq -r '.test_name')
span_count=$(echo "$result" | jq -r '.span_count')
is_success=$(echo "$result" | jq -r '.is_success')

# Generate custom report
cat > report.html <<EOF
<html>
<head><title>Validation Report: $test_name</title></head>
<body>
  <h1>Test: $test_name</h1>
  <p>Spans: $span_count</p>
  <p>Status: $([ "$is_success" == "true" ] && echo "✅ PASS" || echo "❌ FAIL")</p>
</body>
</html>
EOF

echo "Report generated: report.html"
```

---

## See Also

- [User Guide](FAKE_GREEN_DETECTION_USER_GUIDE.md) - Understanding fake-green detection
- [Developer Guide](FAKE_GREEN_DETECTION_DEV_GUIDE.md) - Extending validators
- [TOML Schema Reference](FAKE_GREEN_TOML_SCHEMA.md) - Configuration options

**Questions?** See [documentation](.) or file an issue.
