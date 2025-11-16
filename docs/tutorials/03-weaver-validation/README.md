# Tutorial 3: Weaver Validation (15 minutes)

**⏱ Estimated Time**: 15 minutes
**📋 Prerequisites**: Completed Tutorial 1
**🎯 Learning Objectives**: Catch false-positive tests using schema validation

## What You'll Learn

By the end of this tutorial, you'll:
- ✅ Understand the false-positive problem in testing
- ✅ Install and configure Weaver
- ✅ Define a validation schema
- ✅ Enable live-checking in your test
- ✅ See how Weaver catches broken tests

---

## The Problem: False Positives in Testing

Consider this "test":

```bash
#!/bin/bash
echo "✅ Test passed"
exit 0
```

**This test passes!** But did it actually test anything?

No. It just prints a message and exits successfully. Traditional testing only checks exit codes. A test can "pass" while doing absolutely nothing.

### Real-World Example

```toml
[service.api]
plugin = "generic_container"
image = "my-api:latest"

[[scenario]]
name = "api_responds"
service = "api"
run = "curl http://localhost:8080/api/users"

[expect.output]
stdout = ""  # Empty expectation!
```

This test:
- ✅ **Passes** even if API never starts
- ✅ **Passes** even if API crashes
- ✅ **Passes** if curl command doesn't run
- ✅ **Passes** if database isn't queried
- **This is a fake-green test!**

---

## The Solution: Weaver Schema Validation

Instead of trusting exit codes, **validate actual behavior through telemetry**:

```
Traditional Testing:
  ✅ Exit code = 0 → PASS
  ❌ (But did the code actually execute?)

Schema Validation:
  Code must emit correct telemetry span
  ❌ No HTTP span → Test FAILS (API didn't run)
  ❌ No DB query span → Test FAILS (Database wasn't queried)
  ✅ All spans present and correct → Test PASSES
```

OpenTelemetry proves what actually happened.

---

## Step 1: Install Weaver (2 minutes)

Install the Weaver CLI:

```bash
# Using cargo
cargo install weaver

# Verify installation
weaver --version
# Output: weaver 0.X.X
```

---

## Step 2: Create a Schema Registry (3 minutes)

Create a registry directory structure:

```bash
cd my-clnrm-tests

# Create registry directories
mkdir -p registry/schemas

# Create a simple schema file
cat > registry/schemas/http.yaml << 'EOF'
groups:
  - id: http.request
    prefix: http
    brief: HTTP request attributes
    attributes:
      - id: method
        type: string
        required: true
        brief: HTTP request method (GET, POST, etc.)
        examples: ["GET", "POST"]

      - id: status_code
        type: int
        brief: HTTP response status code
        examples: [200, 404, 500]

      - id: url
        type: string
        brief: Full HTTP URL
        examples: ["http://localhost:8080/api/users"]
EOF
```

This schema defines:
- `http.request` span group
- Required attribute: `method` (string)
- Optional attribute: `status_code` (int)
- Optional attribute: `url` (string)

---

## Step 3: Create a Test with Telemetry (5 minutes)

Update your test to expect OpenTelemetry spans:

```toml
[meta]
name = "http_request_test"
description = "Test HTTP request with telemetry validation"

# Enable Weaver validation
[weaver]
enabled = true
registry_path = "registry"
otlp_port = 0                    # Auto-select port
admin_port = 0

# Configure OTEL export
[otel]
exporter = "otlp-http"
sample_ratio = 1.0
resources = {
  "service.name" = "test-api",
  "deployment.environment" = "test"
}

[service.api]
plugin = "generic_container"
image = "my-instrumented-api:latest"

[[scenario]]
name = "api_handles_request"
service = "api"
run = "curl http://localhost:8080/api/users"
timeout_ms = 5000

# Validate HTTP span exists
[[expect.span]]
name = "http.request"
kind = "server"
attrs.all = {
  "http.method" = "GET",
  "http.status_code" = 200,
  "http.url" = "http://localhost:8080/api/users"
}

# Validate span duration is reasonable
duration_ms = { min = 10.0, max = 1000.0 }
```

### What This Does

1. **Enables Weaver** — `[weaver]` section
2. **Configures OTEL export** — `[otel]` section
3. **Defines span expectations** — `[expect.span]` section
4. **Validates attributes** — `attrs.all` requires all attributes to match
5. **Validates duration** — Ensures span took reasonable time

---

## Step 4: Run with Live-Checking (3 minutes)

Run the test with Weaver validation:

```bash
clnrm run --live-check --registry registry/
```

### Three Possible Outcomes

**✅ Outcome 1: All Valid**
```
Testing http_request_test...
Validating against schema: http.request
  ✅ Span 'http.request' found
  ✅ Attribute 'http.method' = 'GET' (valid)
  ✅ Attribute 'http.status_code' = 200 (valid)
  ✅ Attribute 'http.url' matches schema
  ✅ Duration 145ms is within bounds [10ms, 1000ms]

Result: ✅ PASSED
```

**❌ Outcome 2: Missing Span**
```
Testing http_request_test...
Validating against schema: http.request
  ❌ Span 'http.request' NOT FOUND

Reason: API didn't emit HTTP span
(API probably crashed or didn't start)

Result: ❌ FAILED
```

**❌ Outcome 3: Invalid Attribute**
```
Testing http_request_test...
Validating against schema: http.request
  ✅ Span 'http.request' found
  ❌ Attribute 'http.status_code' = 500 (expected 200)

Reason: Attribute doesn't match schema

Result: ❌ FAILED
```

---

## Step 5: Debug Failures (2 minutes)

When validation fails, Weaver tells you exactly what's wrong:

```bash
# Run with verbose output for debugging
clnrm run --live-check --registry registry/ --verbose

# Output shows:
# - Which spans were emitted
# - Which attributes each span has
# - Which attributes don't match schema
# - Recommendations for fixing
```

---

## Understanding Span Validation

### Span Attributes

Every OpenTelemetry span has attributes:

```json
{
  "name": "http.request",
  "kind": "server",
  "attributes": {
    "http.method": "GET",
    "http.status_code": 200,
    "http.url": "http://localhost:8080/api/users"
  },
  "duration_ms": 145
}
```

### Validation Rules

```toml
[[expect.span]]
name = "http.request"           # Span name (required)
kind = "server"                # Span kind: internal, client, server, producer, consumer

# All attributes must match
attrs.all = {
  "http.method" = "GET",       # Must be exactly "GET"
  "http.status_code" = 200     # Must be exactly 200
}

# At least one must match
attrs.any = {
  "http.status_code" = 200,    # Could be 200 OR
  "http.status_code" = 201     # 201
}

# Duration must be in range
duration_ms = { min = 10.0, max = 1000.0 }
```

---

## Why Weaver Matters

### Without Weaver
```
Test passes ✅
├─ Exit code = 0
└─ (But did we actually test anything? Unknown!)
```

### With Weaver
```
Test passes ✅
├─ Exit code = 0 ✅
├─ HTTP span emitted ✅
├─ All attributes correct ✅
├─ Duration reasonable ✅
└─ We KNOW code executed correctly!
```

Weaver proves behavior, not just exit codes.

---

## Key Concepts

### Schema-First Validation
- Code must emit correct telemetry
- Schema defines expected structure
- Weaver validates runtime against schema
- Test fails if telemetry doesn't match

### OpenTelemetry Semantic Conventions
- Standard attribute names (http.method, http.status_code, etc.)
- Standard span names (http.server.request, db.client.call, etc.)
- Weaver uses these conventions to validate

### Live-Checking
- `--live-check` flag enables validation during test execution
- Weaver validates each span as it's emitted
- Immediate feedback on correctness

---

## Common Mistakes

### ❌ Mistake 1: Empty Expectations
```toml
[[expect.span]]
name = "http.request"
# No attrs.all or attrs.any = test always passes!
```

**Fix**: Add attribute expectations:
```toml
[[expect.span]]
name = "http.request"
attrs.all = { "http.method" = "GET" }
```

### ❌ Mistake 2: Wrong Attribute Names
```toml
attrs.all = {
  "method" = "GET"  # Wrong! Should be "http.method"
}
```

**Fix**: Use semantic convention names:
```toml
attrs.all = {
  "http.method" = "GET"  # Correct!
}
```

### ❌ Mistake 3: Unrealistic Duration Bounds
```toml
duration_ms = { min = 1.0, max = 10.0 }  # Too tight!
```

**Fix**: Use realistic bounds:
```toml
duration_ms = { min = 10.0, max = 5000.0 }  # Better
```

---

## Summary

You now know:
- ✅ **The false-positive problem** — Tests can pass while doing nothing
- ✅ **Why Weaver matters** — Proves behavior through telemetry
- ✅ **How to define schemas** — YAML schema files
- ✅ **How to enable validation** — `--live-check` flag
- ✅ **How to debug failures** — Weaver tells you exactly what's wrong

---

## Next Steps

### Want to extend clnrm with custom services?
→ [Tutorial 4: Custom Plugins](../04-custom-plugins/)

### Want to add observability?
→ [Tutorial 5: OTEL Integration](../05-otel-integration/)

### Want to understand Weaver deeply?
→ [Explanation: Weaver Validation](../../explanation/weaver-validation.md)

### Want practical examples?
→ [How-To: Weaver Schemas](../../how-to/weaver-schemas.md)

---

**Congratulations!** You can now validate that tests actually work, not just assume! ✅

Next: [Tutorial 4: Custom Plugins](../04-custom-plugins/)
