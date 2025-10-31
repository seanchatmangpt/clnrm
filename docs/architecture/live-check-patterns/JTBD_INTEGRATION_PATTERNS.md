# Live-Check Integration Patterns by JTBD

**Document Version:** 1.0.0
**Date:** 2025-10-30
**Status:** Architecture Design

## Overview

This document defines integration patterns for OpenTelemetry Weaver `registry live-check` across all Job-To-Be-Done (JTBD) scenarios. Each pattern includes complete implementation examples, configuration, and operational guidance.

## Pattern Architecture Principles

1. **Schema-First Validation**: Live-check validates runtime telemetry against declared schemas
2. **Fast Feedback**: Patterns designed for <5 second feedback cycles
3. **Zero Configuration**: Sensible defaults, explicit configuration when needed
4. **Observable Integration**: All patterns emit telemetry about validation itself
5. **Fail-Fast, Fail-Clear**: Immediate visibility of violations with actionable errors

---

## JTBD 1: Local Development Debugging

**Job Statement:** *When I'm developing new features, I need to validate telemetry in real-time so I can fix issues before commit.*

### Pattern: Interactive Streaming Validation

```bash
#!/bin/bash
# scripts/dev_live_check.sh
# Usage: ./scripts/dev_live_check.sh [--debug]

set -euo pipefail

DEBUG=${1:-""}
REGISTRY_PATH="registry/"
OTLP_PORT=4317
OUTPUT_DIR=".weaver/dev_$(date +%s)"

mkdir -p "$OUTPUT_DIR"

# Start live-check with streaming output
echo "🔍 Starting Weaver live-check on port $OTLP_PORT"
echo "📁 Output: $OUTPUT_DIR"
echo ""

if [ "$DEBUG" = "--debug" ]; then
    # Debug mode: verbose output
    weaver registry live-check \
        --registry "$REGISTRY_PATH" \
        --otlp-grpc-port "$OTLP_PORT" \
        --format json \
        --output "$OUTPUT_DIR" \
        --verbose
else
    # Normal mode: structured output
    weaver registry live-check \
        --registry "$REGISTRY_PATH" \
        --otlp-grpc-port "$OTLP_PORT" \
        --format json \
        --output "$OUTPUT_DIR" \
        2>&1 | while IFS= read -r line; do
            # Parse and colorize output
            if echo "$line" | grep -q "VIOLATION"; then
                echo "❌ $line"
            elif echo "$line" | grep -q "WARNING"; then
                echo "⚠️  $line"
            elif echo "$line" | grep -q "INFO"; then
                echo "ℹ️  $line"
            else
                echo "$line"
            fi
        done
fi
```

### Development Workflow

**Terminal 1: Start Live-Check**
```bash
./scripts/dev_live_check.sh

# Output:
# 🔍 Starting Weaver live-check on port 4317
# 📁 Output: .weaver/dev_1730246800
#
# ℹ️  Registry loaded: 14 schemas
# ℹ️  Listening on gRPC port 4317
# ℹ️  Press Ctrl+C to stop and generate report
```

**Terminal 2: Run Code with Breakpoints**
```bash
# Set OTLP endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Run with cargo watch for hot reload
cargo watch -x 'run --features otel'

# Or run specific test
cargo test test_container_lifecycle --features otel -- --nocapture
```

**Terminal 3: Monitor Violations**
```bash
# Watch for new violations in real-time
watch -n 1 'jq -r ".violations[]? | \"\(.level): \(.message)\"" .weaver/dev_*/live_check.json 2>/dev/null'
```

### IDE Integration (VS Code)

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Start Weaver Live-Check",
      "type": "shell",
      "command": "./scripts/dev_live_check.sh",
      "isBackground": true,
      "problemMatcher": {
        "pattern": [
          {
            "regexp": "^❌ VIOLATION: (.+)$",
            "message": 1
          }
        ],
        "background": {
          "activeOnStart": true,
          "beginsPattern": "^🔍 Starting Weaver",
          "endsPattern": "^Press Ctrl\\+C"
        }
      },
      "presentation": {
        "reveal": "always",
        "panel": "dedicated"
      }
    },
    {
      "label": "Run Tests with Telemetry",
      "type": "shell",
      "command": "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo test --features otel",
      "dependsOn": ["Start Weaver Live-Check"],
      "problemMatcher": []
    }
  ]
}
```

### Quick Validation Script

```bash
#!/bin/bash
# scripts/quick_validate.sh
# Usage: ./scripts/quick_validate.sh "cargo test test_name"

COMMAND=$1
OUTPUT_DIR=".weaver/quick_$(date +%s)"

# Start live-check in background
weaver registry live-check \
    --registry registry/ \
    --otlp-grpc-port 4317 \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout 10 &
PID=$!

sleep 2

# Run command
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    bash -c "$COMMAND"

# Stop live-check gracefully
kill -HUP $PID 2>/dev/null
wait $PID
EXIT_CODE=$?

# Show summary
echo ""
echo "📊 Validation Summary:"
jq '{
    violations: .statistics.advice_level_counts.violation // 0,
    warnings: .statistics.advice_level_counts.warning // 0,
    coverage: (.statistics.registry_coverage * 100 | round) + "%"
}' "$OUTPUT_DIR/live_check.json"

exit $EXIT_CODE
```

---

## JTBD 2: CI/CD Quality Gate

**Job Statement:** *When code is pushed or PR created, I need automated telemetry validation to block broken changes from merging.*

### Pattern: Automated Validation Pipeline

```yaml
# .github/workflows/telemetry-validation.yml
name: Telemetry Validation

on:
  push:
    branches: [ master, main ]
  pull_request:
    branches: [ master, main ]

env:
  WEAVER_VERSION: "0.11.0"
  OTLP_PORT: 4317
  VALIDATION_TIMEOUT: 300

jobs:
  validate-telemetry:
    name: Validate OpenTelemetry Schemas
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Weaver CLI
        run: |
          cargo install weaver-cli --version ${{ env.WEAVER_VERSION }} --locked
          weaver --version

      - name: Create Output Directory
        run: mkdir -p validation_output

      - name: Start Weaver Live-Check
        run: |
          weaver registry live-check \
            --registry registry/ \
            --otlp-grpc-port ${{ env.OTLP_PORT }} \
            --format json \
            --output validation_output/ \
            --inactivity-timeout ${{ env.VALIDATION_TIMEOUT }} &
          echo $! > weaver.pid

          # Wait for server to start
          for i in {1..10}; do
            if lsof -i :${{ env.OTLP_PORT }} >/dev/null 2>&1; then
              echo "✅ Weaver live-check started on port ${{ env.OTLP_PORT }}"
              exit 0
            fi
            echo "⏳ Waiting for Weaver to start (attempt $i/10)..."
            sleep 2
          done

          echo "❌ Failed to start Weaver live-check"
          exit 1

      - name: Build Project
        env:
          OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:${{ env.OTLP_PORT }}
        run: cargo build --release --features otel

      - name: Run Tests with Telemetry
        env:
          OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:${{ env.OTLP_PORT }}
          RUST_LOG: info
        run: |
          cargo test --features otel --all -- --nocapture

      - name: Stop Weaver Live-Check
        if: always()
        run: |
          PID=$(cat weaver.pid)
          echo "🛑 Stopping Weaver (PID: $PID)"

          # Send SIGHUP to trigger report generation
          kill -HUP $PID

          # Wait for graceful shutdown
          for i in {1..30}; do
            if ! kill -0 $PID 2>/dev/null; then
              echo "✅ Weaver stopped gracefully"
              exit 0
            fi
            sleep 1
          done

          # Force kill if still running
          echo "⚠️  Force killing Weaver"
          kill -9 $PID 2>/dev/null || true

      - name: Parse Validation Results
        if: always()
        id: results
        run: |
          if [ ! -f validation_output/live_check.json ]; then
            echo "❌ No validation output found"
            exit 1
          fi

          # Extract statistics
          VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' validation_output/live_check.json)
          WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' validation_output/live_check.json)
          COVERAGE=$(jq -r '.statistics.registry_coverage // 0' validation_output/live_check.json)
          COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc | cut -d. -f1)

          echo "violations=$VIOLATIONS" >> $GITHUB_OUTPUT
          echo "warnings=$WARNINGS" >> $GITHUB_OUTPUT
          echo "coverage=$COVERAGE_PCT" >> $GITHUB_OUTPUT

          # Pretty print summary
          echo "## 📊 Telemetry Validation Summary" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "| Metric | Value |" >> $GITHUB_STEP_SUMMARY
          echo "|--------|-------|" >> $GITHUB_STEP_SUMMARY
          echo "| ❌ Violations | $VIOLATIONS |" >> $GITHUB_STEP_SUMMARY
          echo "| ⚠️ Warnings | $WARNINGS |" >> $GITHUB_STEP_SUMMARY
          echo "| 📈 Coverage | $COVERAGE_PCT% |" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY

          # Show violations if any
          if [ "$VIOLATIONS" -gt 0 ]; then
            echo "### ❌ Violations Found" >> $GITHUB_STEP_SUMMARY
            echo "" >> $GITHUB_STEP_SUMMARY
            echo "\`\`\`json" >> $GITHUB_STEP_SUMMARY
            jq -r '.violations[]? | "- \(.level): \(.message)"' validation_output/live_check.json >> $GITHUB_STEP_SUMMARY
            echo "\`\`\`" >> $GITHUB_STEP_SUMMARY
          fi

      - name: Upload Validation Artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: telemetry-validation-${{ github.sha }}
          path: validation_output/
          retention-days: 30

      - name: Check Violation Threshold
        if: always()
        run: |
          VIOLATIONS=${{ steps.results.outputs.violations }}
          COVERAGE=${{ steps.results.outputs.coverage }}

          if [ "$VIOLATIONS" -gt 0 ]; then
            echo "❌ Quality gate failed: $VIOLATIONS violations found"
            echo "   Telemetry MUST conform to registry schemas"
            exit 1
          fi

          if [ "$COVERAGE" -lt 80 ]; then
            echo "⚠️  Warning: Registry coverage is ${COVERAGE}% (target: 80%)"
            echo "   Consider adding more schema coverage"
            # Warning only, don't fail
          fi

          echo "✅ Quality gate passed: Zero violations"

      - name: Comment on PR
        if: always() && github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const violations = ${{ steps.results.outputs.violations }};
            const warnings = ${{ steps.results.outputs.warnings }};
            const coverage = ${{ steps.results.outputs.coverage }};

            const status = violations === 0 ? '✅ PASSED' : '❌ FAILED';
            const emoji = violations === 0 ? '🎉' : '🚨';

            const comment = `
            ## ${emoji} Telemetry Validation ${status}

            | Metric | Value |
            |--------|-------|
            | Violations | ${violations} |
            | Warnings | ${warnings} |
            | Coverage | ${coverage}% |

            ${violations === 0
              ? '**All telemetry conforms to registry schemas!**'
              : '**Violations found!** See workflow artifacts for details.'}

            ---
            <sub>Generated by Weaver live-check | [View Full Report](https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }})</sub>
            `;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
```

### Branch Protection Rule

```yaml
# .github/branch-protection.yml
# Apply with: gh api repos/:owner/:repo/branches/main/protection -X PUT -d @.github/branch-protection.yml

required_status_checks:
  strict: true
  contexts:
    - "Validate OpenTelemetry Schemas"
enforce_admins: true
required_pull_request_reviews:
  required_approving_review_count: 1
restrictions: null
```

---

## JTBD 3: Pre-Commit Hook

**Job Statement:** *When I commit code, I need fast telemetry validation to catch issues before pushing.*

### Pattern: Fast Pre-Commit Validation

```bash
#!/bin/bash
# .git/hooks/pre-commit
# Install: cp scripts/pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

set -euo pipefail

# Configuration
REGISTRY_PATH="registry/"
OTLP_PORT=4317
OUTPUT_DIR="/tmp/weaver_precommit_$$"
INACTIVITY_TIMEOUT=30
MAX_WAIT=60

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 Running Weaver telemetry validation...${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Start live-check
weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --otlp-grpc-port "$OTLP_PORT" \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout "$INACTIVITY_TIMEOUT" \
    2>&1 | tee "$OUTPUT_DIR/weaver.log" &
PID=$!

# Wait for server to start
for i in {1..10}; do
    if lsof -i :"$OTLP_PORT" >/dev/null 2>&1; then
        echo "✅ Live-check started"
        break
    fi
    sleep 1
done

# Get list of changed test files
CHANGED_TESTS=$(git diff --cached --name-only --diff-filter=ACMR | grep -E '(tests?/.*\.rs$|src/.*\.rs$)' || true)

if [ -z "$CHANGED_TESTS" ]; then
    echo "ℹ️  No test files changed, running quick smoke test"
    OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
        cargo test --features otel --lib -- --test-threads=1 >/dev/null 2>&1 || true
else
    echo "🧪 Running tests for changed files:"
    echo "$CHANGED_TESTS" | sed 's/^/  - /'

    # Run only affected tests
    for test_file in $CHANGED_TESTS; do
        # Extract module path from file path
        MODULE=$(echo "$test_file" | sed 's/src\///' | sed 's/\.rs$//' | sed 's/\//::/')

        OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
            cargo test --features otel "$MODULE" -- --test-threads=1 >/dev/null 2>&1 || true
    done
fi

# Wait for telemetry to be processed
sleep 2

# Stop live-check gracefully
echo "🛑 Stopping live-check..."
kill -HUP $PID 2>/dev/null
wait $PID 2>/dev/null || true

# Check if output was generated
if [ ! -f "$OUTPUT_DIR/live_check.json" ]; then
    echo -e "${RED}❌ No validation output generated${NC}"
    echo "   Check $OUTPUT_DIR/weaver.log for errors"
    exit 1
fi

# Parse results
VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' "$OUTPUT_DIR/live_check.json")
WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE=$(jq -r '.statistics.registry_coverage // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc | cut -d. -f1)

# Show summary
echo ""
echo "📊 Validation Summary:"
echo "   Violations: $VIOLATIONS"
echo "   Warnings: $WARNINGS"
echo "   Coverage: $COVERAGE_PCT%"

# Check for violations
if [ "$VIOLATIONS" -gt 0 ]; then
    echo -e "${RED}❌ Telemetry validation failed: $VIOLATIONS violations found${NC}"
    echo ""
    echo "Violations:"
    jq -r '.violations[]? | "  - \(.level): \(.message)"' "$OUTPUT_DIR/live_check.json"
    echo ""
    echo "Full report: $OUTPUT_DIR/live_check.json"
    echo ""
    echo "To bypass (not recommended): git commit --no-verify"
    exit 1
fi

# Warn on low coverage
if [ "$COVERAGE_PCT" -lt 50 ]; then
    echo -e "${YELLOW}⚠️  Low registry coverage: $COVERAGE_PCT% (target: 80%)${NC}"
fi

echo -e "${GREEN}✅ Telemetry validation passed${NC}"

# Cleanup old validation outputs
find /tmp -name "weaver_precommit_*" -type d -mtime +1 -exec rm -rf {} + 2>/dev/null || true

exit 0
```

### Installation Script

```bash
#!/bin/bash
# scripts/install_hooks.sh
# Usage: ./scripts/install_hooks.sh

set -euo pipefail

HOOK_SRC="scripts/pre-commit.sh"
HOOK_DEST=".git/hooks/pre-commit"

if [ ! -d ".git" ]; then
    echo "❌ Not in a git repository"
    exit 1
fi

if [ ! -f "$HOOK_SRC" ]; then
    echo "❌ Hook source not found: $HOOK_SRC"
    exit 1
fi

# Backup existing hook
if [ -f "$HOOK_DEST" ]; then
    echo "📦 Backing up existing hook to ${HOOK_DEST}.backup"
    cp "$HOOK_DEST" "${HOOK_DEST}.backup"
fi

# Install hook
cp "$HOOK_SRC" "$HOOK_DEST"
chmod +x "$HOOK_DEST"

echo "✅ Pre-commit hook installed"
echo "   Hook will run Weaver validation on commit"
echo "   Bypass with: git commit --no-verify"
```

---

## JTBD 4: Coverage Tracking

**Job Statement:** *When testing changes, I need to track registry coverage over time to ensure comprehensive validation.*

### Pattern: Historical Coverage Tracking

```bash
#!/bin/bash
# scripts/track_coverage.sh
# Usage: ./scripts/track_coverage.sh [--upload] [--baseline PERCENT]

set -euo pipefail

# Configuration
REGISTRY_PATH="registry/"
OTLP_PORT=4317
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR=".weaver/coverage/$TIMESTAMP"
HISTORY_DIR=".weaver/coverage/history"
BASELINE=${BASELINE:-80}
UPLOAD=${1:-""}

mkdir -p "$OUTPUT_DIR" "$HISTORY_DIR"

echo "📊 Running coverage analysis..."

# Start live-check
weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --otlp-grpc-port "$OTLP_PORT" \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout 60 &
PID=$!

sleep 2

# Run comprehensive test suite
echo "🧪 Running all tests..."
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
    cargo test --all --features otel -- --test-threads=4

# Stop live-check
kill -HUP $PID
wait $PID

# Extract metrics
COVERAGE=$(jq -r '.statistics.registry_coverage // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc -l | cut -d. -f1)
VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' "$OUTPUT_DIR/live_check.json")
WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' "$OUTPUT_DIR/live_check.json")

# Create history record
cat > "$HISTORY_DIR/$TIMESTAMP.json" <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "coverage_percent": $COVERAGE_PCT,
  "violations": $VIOLATIONS,
  "warnings": $WARNINGS,
  "git_commit": "$(git rev-parse HEAD)",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD)"
}
EOF

# Show summary
echo ""
echo "📊 Coverage Report:"
echo "   Coverage: $COVERAGE_PCT%"
echo "   Violations: $VIOLATIONS"
echo "   Warnings: $WARNINGS"
echo "   Git Commit: $(git rev-parse --short HEAD)"

# Check baseline
if [ "$COVERAGE_PCT" -lt "$BASELINE" ]; then
    echo "❌ Coverage below baseline: $COVERAGE_PCT% < $BASELINE%"
    exit 1
fi

# Generate trend chart
python3 - <<PYTHON
import json
import glob
from pathlib import Path

history_files = sorted(glob.glob("$HISTORY_DIR/*.json"))
if len(history_files) > 1:
    data = [json.load(open(f)) for f in history_files[-10:]]

    print("\n📈 Coverage Trend (last 10 runs):")
    for record in data:
        timestamp = record['timestamp'][:10]
        coverage = record['coverage_percent']
        bar = '█' * (coverage // 2)
        print(f"  {timestamp}: {bar} {coverage}%")
PYTHON

# Upload to metrics service (optional)
if [ "$UPLOAD" = "--upload" ]; then
    echo "📤 Uploading metrics..."
    curl -X POST "${METRICS_API_ENDPOINT}" \
        -H "Content-Type: application/json" \
        -d @"$HISTORY_DIR/$TIMESTAMP.json"
fi

echo "✅ Coverage tracking complete"
exit 0
```

### Coverage Dashboard Script

```python
#!/usr/bin/env python3
# scripts/coverage_dashboard.py
# Usage: python3 scripts/coverage_dashboard.py [--days N]

import json
import glob
import argparse
from datetime import datetime, timedelta
from pathlib import Path

def load_history(history_dir, days=30):
    """Load coverage history from JSON files."""
    cutoff = datetime.now() - timedelta(days=days)

    history_files = sorted(glob.glob(f"{history_dir}/*.json"))
    records = []

    for filepath in history_files:
        with open(filepath) as f:
            record = json.load(f)
            record_time = datetime.fromisoformat(record['timestamp'])

            if record_time >= cutoff:
                records.append(record)

    return records

def print_dashboard(records, baseline=80):
    """Print ASCII dashboard of coverage trends."""
    print("=" * 80)
    print("📊 TELEMETRY COVERAGE DASHBOARD")
    print("=" * 80)
    print()

    if not records:
        print("No coverage data found.")
        return

    # Current status
    latest = records[-1]
    coverage = latest['coverage_percent']
    violations = latest['violations']
    warnings = latest['warnings']

    status = "✅ PASS" if coverage >= baseline and violations == 0 else "❌ FAIL"

    print(f"Current Status: {status}")
    print(f"Coverage: {coverage}% (baseline: {baseline}%)")
    print(f"Violations: {violations}")
    print(f"Warnings: {warnings}")
    print(f"Last Updated: {latest['timestamp']}")
    print()

    # Trend chart
    print("📈 Coverage Trend (last 30 days):")
    print()

    for record in records[-30:]:
        date = datetime.fromisoformat(record['timestamp']).strftime('%Y-%m-%d')
        coverage = record['coverage_percent']
        violations = record['violations']

        bar_length = coverage
        bar = '█' * (bar_length // 2)

        violation_marker = f" ⚠️ {violations} violations" if violations > 0 else ""

        print(f"{date}  {bar:<50} {coverage:3d}%{violation_marker}")

    print()

    # Statistics
    coverages = [r['coverage_percent'] for r in records]
    avg_coverage = sum(coverages) / len(coverages)
    min_coverage = min(coverages)
    max_coverage = max(coverages)

    total_violations = sum(r['violations'] for r in records)

    print("📊 Statistics:")
    print(f"   Average Coverage: {avg_coverage:.1f}%")
    print(f"   Min Coverage: {min_coverage}%")
    print(f"   Max Coverage: {max_coverage}%")
    print(f"   Total Violations: {total_violations}")
    print()

    # Health check
    if avg_coverage >= baseline and total_violations == 0:
        print("✅ Telemetry health: EXCELLENT")
    elif avg_coverage >= baseline * 0.9:
        print("⚠️  Telemetry health: GOOD (some violations)")
    else:
        print("❌ Telemetry health: NEEDS IMPROVEMENT")

    print("=" * 80)

def main():
    parser = argparse.ArgumentParser(description='Display telemetry coverage dashboard')
    parser.add_argument('--days', type=int, default=30, help='Days of history to show')
    parser.add_argument('--history-dir', default='.weaver/coverage/history', help='History directory')
    parser.add_argument('--baseline', type=int, default=80, help='Coverage baseline percentage')

    args = parser.parse_args()

    records = load_history(args.history_dir, args.days)
    print_dashboard(records, args.baseline)

if __name__ == '__main__':
    main()
```

---

## JTBD 5: Production Monitoring

**Job Statement:** *When code runs in production, I need to validate telemetry samples to detect schema drift or violations.*

### Pattern: Production Telemetry Validation

```bash
#!/bin/bash
# scripts/validate_production_telemetry.sh
# Usage: ./scripts/validate_production_telemetry.sh [--source URL|FILE]

set -euo pipefail

# Configuration
SOURCE=${1:-"${PRODUCTION_OTLP_ENDPOINT}"}
REGISTRY_PATH="registry/"
OUTPUT_DIR=".weaver/production/$(date +%Y%m%d_%H%M%S)"
ALERT_WEBHOOK="${SLACK_WEBHOOK_URL}"

mkdir -p "$OUTPUT_DIR"

echo "🔍 Validating production telemetry..."
echo "   Source: $SOURCE"

# Collect telemetry sample
if [[ "$SOURCE" =~ ^http ]]; then
    echo "📥 Collecting sample from HTTP endpoint..."
    curl -s "$SOURCE/telemetry/sample" > "$OUTPUT_DIR/sample.json"
else
    echo "📥 Reading sample from file..."
    cp "$SOURCE" "$OUTPUT_DIR/sample.json"
fi

# Validate with Weaver
echo "⚙️  Running Weaver validation..."
weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --input-source "$OUTPUT_DIR/sample.json" \
    --format json \
    --output "$OUTPUT_DIR"

# Parse results
VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' "$OUTPUT_DIR/live_check.json")
WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE=$(jq -r '.statistics.registry_coverage // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc | cut -d. -f1)

# Create alert payload
if [ "$VIOLATIONS" -gt 0 ]; then
    SEVERITY="error"
    COLOR="#FF0000"
    TITLE="🚨 Production Telemetry Violations Detected"
else
    SEVERITY="info"
    COLOR="#00FF00"
    TITLE="✅ Production Telemetry Validation Passed"
fi

# Send alert to Slack
if [ -n "$ALERT_WEBHOOK" ]; then
    curl -X POST "$ALERT_WEBHOOK" \
        -H "Content-Type: application/json" \
        -d @- <<EOF
{
  "attachments": [
    {
      "color": "$COLOR",
      "title": "$TITLE",
      "fields": [
        {
          "title": "Violations",
          "value": "$VIOLATIONS",
          "short": true
        },
        {
          "title": "Warnings",
          "value": "$WARNINGS",
          "short": true
        },
        {
          "title": "Coverage",
          "value": "$COVERAGE_PCT%",
          "short": true
        },
        {
          "title": "Timestamp",
          "value": "$(date -Iseconds)",
          "short": true
        }
      ]
    }
  ]
}
EOF
fi

# Log to metrics
echo "{
  \"timestamp\": \"$(date -Iseconds)\",
  \"violations\": $VIOLATIONS,
  \"warnings\": $WARNINGS,
  \"coverage_percent\": $COVERAGE_PCT,
  \"environment\": \"production\"
}" >> .weaver/production/metrics.jsonl

# Exit with error if violations found
if [ "$VIOLATIONS" -gt 0 ]; then
    echo "❌ Production validation failed: $VIOLATIONS violations"
    jq -r '.violations[]? | "  - \(.level): \(.message)"' "$OUTPUT_DIR/live_check.json"
    exit 1
fi

echo "✅ Production validation passed"
exit 0
```

### Continuous Monitoring (Kubernetes CronJob)

```yaml
# k8s/telemetry-validation-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: telemetry-validation
  namespace: monitoring
spec:
  schedule: "*/15 * * * *"  # Every 15 minutes
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: validator
            image: clnrm/weaver-validator:latest
            env:
            - name: PRODUCTION_OTLP_ENDPOINT
              valueFrom:
                configMapKeyRef:
                  name: telemetry-config
                  key: otlp_endpoint
            - name: SLACK_WEBHOOK_URL
              valueFrom:
                secretKeyRef:
                  name: alerting-secrets
                  key: slack_webhook
            command:
            - /scripts/validate_production_telemetry.sh
            volumeMounts:
            - name: registry
              mountPath: /registry
              readOnly: true
            - name: output
              mountPath: /.weaver/production
          volumes:
          - name: registry
            configMap:
              name: otel-registry
          - name: output
            persistentVolumeClaim:
              claimName: validation-output
          restartPolicy: OnFailure
```

### Prometheus Metrics Exporter

```python
#!/usr/bin/env python3
# scripts/export_validation_metrics.py
# Usage: python3 scripts/export_validation_metrics.py --port 9090

from prometheus_client import start_http_server, Gauge, Counter
import json
import time
import argparse
from pathlib import Path

# Define metrics
telemetry_violations = Gauge('telemetry_violations', 'Number of telemetry violations', ['environment'])
telemetry_warnings = Gauge('telemetry_warnings', 'Number of telemetry warnings', ['environment'])
telemetry_coverage = Gauge('telemetry_coverage_percent', 'Telemetry registry coverage percentage', ['environment'])
validation_runs = Counter('telemetry_validation_runs_total', 'Total validation runs', ['environment', 'status'])

def read_latest_validation(output_dir, environment='production'):
    """Read latest validation result."""
    validation_files = sorted(Path(output_dir).glob('*/live_check.json'))

    if not validation_files:
        return None

    latest_file = validation_files[-1]

    with open(latest_file) as f:
        data = json.load(f)

    violations = data['statistics']['advice_level_counts'].get('violation', 0)
    warnings = data['statistics']['advice_level_counts'].get('warning', 0)
    coverage = data['statistics']['registry_coverage'] * 100

    return {
        'violations': violations,
        'warnings': warnings,
        'coverage': coverage,
        'status': 'pass' if violations == 0 else 'fail'
    }

def update_metrics(output_dir, environment='production'):
    """Update Prometheus metrics from validation results."""
    result = read_latest_validation(output_dir, environment)

    if result:
        telemetry_violations.labels(environment=environment).set(result['violations'])
        telemetry_warnings.labels(environment=environment).set(result['warnings'])
        telemetry_coverage.labels(environment=environment).set(result['coverage'])
        validation_runs.labels(environment=environment, status=result['status']).inc()

def main():
    parser = argparse.ArgumentParser(description='Export telemetry validation metrics to Prometheus')
    parser.add_argument('--port', type=int, default=9090, help='Metrics server port')
    parser.add_argument('--output-dir', default='.weaver/production', help='Validation output directory')
    parser.add_argument('--environment', default='production', help='Environment name')
    parser.add_argument('--interval', type=int, default=60, help='Update interval (seconds)')

    args = parser.parse_args()

    # Start Prometheus HTTP server
    start_http_server(args.port)
    print(f"📊 Metrics server started on port {args.port}")
    print(f"   Metrics: http://localhost:{args.port}/metrics")

    # Update metrics periodically
    while True:
        try:
            update_metrics(args.output_dir, args.environment)
            print(f"✅ Metrics updated at {time.strftime('%Y-%m-%d %H:%M:%S')}")
        except Exception as e:
            print(f"❌ Error updating metrics: {e}")

        time.sleep(args.interval)

if __name__ == '__main__':
    main()
```

---

## Cross-Pattern Integration

### Unified Configuration

```toml
# .weaver/config.toml
# Centralized configuration for all live-check patterns

[registry]
path = "registry/"

[live_check]
otlp_port = 4317
format = "json"
inactivity_timeout = 60

[coverage]
baseline_percent = 80
history_retention_days = 90

[ci_cd]
timeout_seconds = 300
fail_on_violations = true
warn_on_low_coverage = true

[production]
sample_interval_minutes = 15
alert_on_violations = true
alert_on_warnings = false

[alerts.slack]
webhook_url = "${SLACK_WEBHOOK_URL}"
mention_on_violations = "@here"

[alerts.pagerduty]
integration_key = "${PAGERDUTY_KEY}"
severity = "error"
```

### Makefile Targets

```makefile
# Makefile
# Usage: make validate-dev, make validate-ci, etc.

.PHONY: validate-dev validate-ci validate-pre-commit track-coverage validate-prod

validate-dev:
	@echo "🔍 Running development validation..."
	@./scripts/dev_live_check.sh

validate-quick:
	@echo "⚡ Quick validation..."
	@./scripts/quick_validate.sh "cargo test --features otel"

validate-ci:
	@echo "🤖 CI validation..."
	@./scripts/ci_validation.sh

install-hooks:
	@echo "📦 Installing pre-commit hooks..."
	@./scripts/install_hooks.sh

track-coverage:
	@echo "📊 Tracking coverage..."
	@./scripts/track_coverage.sh

coverage-dashboard:
	@echo "📈 Showing coverage dashboard..."
	@python3 scripts/coverage_dashboard.py

validate-prod:
	@echo "🚀 Validating production telemetry..."
	@./scripts/validate_production_telemetry.sh

export-metrics:
	@echo "📊 Exporting Prometheus metrics..."
	@python3 scripts/export_validation_metrics.py

help:
	@echo "Available targets:"
	@echo "  validate-dev       - Interactive development validation"
	@echo "  validate-quick     - Fast validation for current changes"
	@echo "  validate-ci        - Full CI validation"
	@echo "  install-hooks      - Install pre-commit hooks"
	@echo "  track-coverage     - Track coverage over time"
	@echo "  coverage-dashboard - Show coverage trends"
	@echo "  validate-prod      - Validate production telemetry"
	@echo "  export-metrics     - Export metrics to Prometheus"
```

---

## Pattern Selection Guide

| JTBD | When to Use | Response Time | Scope | Cost |
|------|-------------|---------------|-------|------|
| **Local Dev** | During feature development | Real-time | Changed files | Low |
| **CI/CD** | On push/PR | 2-5 minutes | All tests | Medium |
| **Pre-Commit** | Before commit | 10-30 seconds | Changed files | Low |
| **Coverage** | Weekly/sprint | 5-10 minutes | Full suite | Medium |
| **Production** | Continuous | 1-5 minutes | Samples | High |

---

## Summary

This document provides 5 complete integration patterns for Weaver `registry live-check`:

1. **Local Development**: Real-time streaming validation with IDE integration
2. **CI/CD**: Automated quality gates with GitHub Actions
3. **Pre-Commit**: Fast validation before commits
4. **Coverage Tracking**: Historical metrics and trends
5. **Production Monitoring**: Continuous validation of live telemetry

All patterns are production-ready and follow best practices for observability, error handling, and operational excellence.
