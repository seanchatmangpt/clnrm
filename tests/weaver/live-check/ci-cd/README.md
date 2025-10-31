# CI/CD Integration Examples for Weaver Live-Check

This directory contains production-ready CI/CD pipeline configurations demonstrating how to integrate Weaver `registry live-check` validation into continuous integration workflows.

## Overview

Weaver live-check provides **runtime schema validation** for OpenTelemetry telemetry, ensuring that your application's actual telemetry output matches your declared schemas. This is the **ONLY** source of truth for validating that features work as intended - tests can have false positives, but schema validation cannot.

## Available Integrations

### 1. GitHub Actions (`github-actions.yml`)

**Features:**
- Automated validation on push and pull requests
- Service containers for OTLP Collector
- Artifact upload for validation reports
- PR comments with results
- Strict error handling (fail on schema violations)

**Usage:**
```bash
# Copy to your repository
cp github-actions.yml .github/workflows/weaver-validation.yml

# Commit and push
git add .github/workflows/weaver-validation.yml
git commit -m "Add Weaver live-check validation"
git push
```

**Key Configuration:**
- Runs on Ubuntu Latest
- 10-minute timeout
- Fails build on schema errors
- Warns on schema warnings

---

### 2. GitLab CI/CD (`gitlab-ci.yml`)

**Features:**
- Multi-stage pipeline (build → test → validate)
- Service integration for OTLP Collector
- JUnit test report format
- Artifact retention (30 days)
- Automatic failure on violations

**Usage:**
```bash
# Copy to your repository root
cp gitlab-ci.yml .gitlab-ci.yml

# Commit and push
git add .gitlab-ci.yml
git commit -m "Add Weaver validation stage"
git push
```

**Pipeline Stages:**
1. **Build** - Compile application with OTEL features
2. **Test** - Run unit and integration tests
3. **Validate** - Weaver live-check validation (quality gate)

---

### 3. Jenkins Pipeline (`jenkins-pipeline.groovy`)

**Features:**
- Declarative pipeline syntax
- Docker-based OTLP Collector
- Build status integration (FAILURE/UNSTABLE/SUCCESS)
- Email notifications on failure
- HTML report publishing

**Usage:**
```groovy
// Add to Jenkinsfile in repository root
@Library('shared-pipeline-library') _

// Include Weaver validation stage
weaverValidation()
```

**Setup:**
1. Install required plugins:
   - Docker Pipeline
   - Pipeline Utility Steps
   - HTML Publisher
2. Configure Weaver binary on Jenkins nodes
3. Add pipeline to job configuration

---

### 4. Azure DevOps Pipelines (`azure-pipelines.yml`)

**Features:**
- Multi-stage YAML pipeline
- Service containers for OTLP
- Test results publishing
- Build artifacts for reports
- Azure DevOps logging integration

**Usage:**
```bash
# Add to repository root
cp azure-pipelines.yml azure-pipelines.yml

# Configure in Azure DevOps:
# Pipelines → New Pipeline → Existing Azure Pipelines YAML file
```

**Azure-Specific Features:**
- `##vso[task.logissue]` for error/warning annotations
- Automatic test result parsing
- Build artifact storage

---

## Common Configuration

### Environment Variables

All pipelines use these standard OTLP environment variables:

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318  # HTTP endpoint
OTEL_SERVICE_NAME=clnrm-ci                         # Service identifier
```

### Weaver Installation

Each pipeline installs Weaver from official releases:

```bash
curl -sSL https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux-amd64 \
    -o /usr/local/bin/weaver
chmod +x /usr/local/bin/weaver
```

### Validation Command

Standard validation invocation:

```bash
weaver registry live-check \
    --registry registry/ \
    --otlp-http http://localhost:4318 \
    --timeout 60s \
    --output json > weaver-results.json
```

### Result Analysis

All pipelines parse JSON output for decision-making:

```bash
violations=$(jq -r '.violations | length' weaver-results.json)
errors=$(jq -r '[.violations[] | select(.severity == "error")] | length' weaver-results.json)
warnings=$(jq -r '[.violations[] | select(.severity == "warning")] | length' weaver-results.json)

# Fail build if errors detected
if [ "${errors}" -gt 0 ]; then
    exit 1
fi
```

---

## Quality Gate Strategy

### Recommended Approach

```
┌─────────────────────────────────────────────────────────────────┐
│ CI/CD Quality Gates                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ 1. Build & Compilation ──────────────► MUST PASS               │
│    - cargo build --release --features otel                      │
│                                                                 │
│ 2. Unit Tests ───────────────────────► MUST PASS               │
│    - cargo test --lib                                           │
│                                                                 │
│ 3. Integration Tests ────────────────► MUST PASS               │
│    - cargo test --test '*'                                      │
│                                                                 │
│ 4. Weaver Schema Validation ─────────► MUST PASS (NO ERRORS)   │
│    - weaver registry live-check                                 │
│    - Errors: FAIL BUILD                                         │
│    - Warnings: WARN (but allow)                                 │
│                                                                 │
│ 5. Deployment ───────────────────────► ONLY IF ALL PASS        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Severity Handling

| Severity | Action | Rationale |
|----------|--------|-----------|
| **Error** | ❌ Fail build | Schema contract violated, feature broken |
| **Warning** | ⚠️ Warn but pass | Deprecated or non-critical issues |
| **Info** | ℹ️ Log only | Informational, no action needed |

---

## Testing Your Integration

### Local Testing

Before committing CI/CD configuration, test locally:

```bash
# Start OTLP Collector
docker run -d -p 4318:4318 otel/opentelemetry-collector:latest

# Build with OTEL features
cargo build --release --features otel

# Run application with telemetry
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
    ./target/release/clnrm self-test --suite otel &

# Run Weaver validation
weaver registry live-check \
    --registry registry/ \
    --otlp-http http://localhost:4318 \
    --timeout 60s \
    --output json

# Verify results
jq '.violations' weaver-results.json
```

### Expected Results

**Successful validation:**
```json
{
  "violations": [],
  "coverage": {
    "total_schemas": 14,
    "used_schemas": 12,
    "coverage_percent": 85.7
  },
  "summary": {
    "total_spans": 47,
    "total_metrics": 23,
    "total_logs": 8
  }
}
```

**Failed validation (violations present):**
```json
{
  "violations": [
    {
      "severity": "error",
      "message": "Missing required attribute 'test.result'",
      "span_name": "test.execution",
      "advisor": "missing_attribute"
    }
  ],
  "summary": {
    "total_violations": 1,
    "error_count": 1
  }
}
```

---

## Troubleshooting

### Common Issues

**1. OTLP Collector not reachable**
```
Error: connection refused on localhost:4318
```

**Solution:** Verify collector is running and health check passes:
```bash
curl -f http://localhost:13133/
```

---

**2. No telemetry received by Weaver**
```
Warning: timeout after 60s with no telemetry
```

**Solution:** Check application OTLP configuration:
```bash
# Verify env vars are set
echo $OTEL_EXPORTER_OTLP_ENDPOINT
echo $OTEL_SERVICE_NAME

# Check application logs for OTLP export errors
```

---

**3. Invalid JSON output from Weaver**
```
Error: parse error in weaver-results.json
```

**Solution:** Check Weaver stderr for errors:
```bash
weaver registry live-check ... 2> weaver-errors.log
cat weaver-errors.log
```

---

**4. False violations detected**
```
Error: violations detected but telemetry looks correct
```

**Solution:** Update registry schemas to match actual telemetry:
```bash
# Validate schema syntax
weaver registry check -r registry/

# Check for schema version mismatches
grep "version:" registry/**/*.yaml
```

---

## Best Practices

### 1. Always validate in CI/CD
- Never merge without passing Weaver validation
- Schema validation is the source of truth (not tests)

### 2. Separate stages
- Build → Test → Validate (in sequence)
- Don't skip validation even if tests pass

### 3. Archive reports
- Store validation results as artifacts
- Track trends over time (coverage, violations)

### 4. Fast feedback
- Use timeouts (60s recommended)
- Fail fast on errors

### 5. Clear messaging
- Annotate PRs with validation results
- Include violation details in failure messages

---

## Integration Checklist

- [ ] OTLP Collector configured as service
- [ ] Weaver binary installed in pipeline
- [ ] Application built with `otel` feature
- [ ] OTLP environment variables set
- [ ] Validation runs after application start
- [ ] Results parsed and evaluated
- [ ] Build fails on schema errors
- [ ] Validation report archived as artifact
- [ ] PR/MR comments added (if applicable)
- [ ] Local testing completed successfully

---

## Additional Resources

- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)
- [clnrm Weaver Integration](../../../../docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md)

---

**Remember:** Weaver validation is not optional. It's the ONLY way to prove your telemetry matches your schema declarations. Tests can lie; schemas don't.
