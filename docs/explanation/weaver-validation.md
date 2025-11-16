# Understanding Weaver Validation

**The Core Insight**: Schema-first validation catches fake-green tests that traditional testing misses.

## The False-Positive Problem

### Traditional Testing Only Checks Exit Codes

```bash
#!/bin/bash
# This "test" passes!
echo "Test completed"
exit 0
```

Why is this a problem?
- ✅ Exit code = 0 (success)
- ❌ No actual testing happened
- ❌ Can't distinguish between "feature works" and "test is broken"

### Real-World Example: API Testing

```toml
[service.api]
plugin = "generic_container"
image = "my-api:latest"

[[scenario]]
name = "test_api"
service = "api"
run = "curl http://localhost:8080/api"

[expect.output]
stdout = ""  # Empty! This always passes!
```

**This test passes if:**
- ✅ API starts correctly
- ✅ API crashes on startup
- ✅ Endpoint doesn't exist
- ✅ Database is down
- ✅ curl isn't installed
- **All equally "pass"!**

---

## Why Behavior Validation Matters

### Testing is About Proving Behavior

**Traditional approach**: Assume exit code = correct behavior
**Problem**: Assumption can be wrong (test is fake-green)

**Behavior validation approach**: **Prove** execution through telemetry
**Advantage**: Telemetry doesn't lie

### Telemetry as Proof

When code runs, it emits OpenTelemetry spans:

```
API HTTP request → Server span emitted
  ↓
Database query → DB span emitted
  ↓
Cache lookup → Cache span emitted
  ↓
Response sent → Response span emitted
```

**These spans ARE the proof that code executed.**

---

## How Weaver Works

### Schema-First Validation

1. **Define schema** — What telemetry should exist
   ```yaml
   groups:
     - id: http.request
       attributes:
         - id: method
           required: true
   ```

2. **Code emits telemetry** — As it executes
   ```
   HTTP server span with method="GET"
   ```

3. **Weaver validates** — Runtime telemetry matches schema
   ```
   ✅ Span exists
   ✅ Method attribute required
   ✅ All required attributes present
   ```

4. **Test result** — Proof of correct behavior
   ```
   ✅ TEST PASSES (with proof!)
   ```

### Key Insight: Schema Validation Catches Lies

```
Without schema:
  Test passes ✅
  (But did it actually work?)

With schema:
  Telemetry doesn't match ❌
  (Code didn't execute as expected)
```

---

## Why This Matters: Real Examples

### Example 1: Database Not Queried

```toml
[[expect.span]]
name = "db.query"
attrs.all = { "db.operation" = "SELECT" }
```

Without schema validation:
- ❌ Database down
- ✅ Test passes (exit code 0)
- **FALSE POSITIVE!**

With schema validation:
- ❌ Database down
- ❌ No db.query span emitted
- ❌ Test fails
- **CAUGHT!**

### Example 2: Wrong Database Queried

```toml
[[expect.span]]
name = "db.query"
attrs.all = { "db.system" = "postgresql" }
```

Without schema:
- ✅ Queries work (different database)
- ✅ Test passes
- **FALSE POSITIVE!**

With schema:
- ❌ db.system = "mysql" (not postgresql)
- ❌ Attribute doesn't match
- ❌ Test fails
- **CAUGHT!**

### Example 3: Race Condition

```toml
[expect.order]
must_precede = [["auth.check", "db.query"]]
```

Without schema:
- ✅ Eventually works
- ✅ Test passes most times
- **FLAKY!**

With schema:
- ❌ db.query before auth.check sometimes
- ❌ Temporal order violated
- ❌ Test fails
- **CAUGHT!**

---

## The Proof is in Telemetry

### What Telemetry Proves

For each span, Weaver validates:

1. **Existence** — Did the operation happen?
   ```
   Expected span "http.server.request"
   ✅ Found at 10:00:00.123
   ```

2. **Attributes** — Did it happen correctly?
   ```
   Expected http.method = "GET"
   ✅ Found http.method = "GET"
   ```

3. **Structure** — Did operations relate correctly?
   ```
   Expected parent-child: request → query
   ✅ Found parent-child relationship
   ```

4. **Timing** — Did things happen in right order?
   ```
   Expected: auth before database
   ✅ Auth at 123ms, DB at 145ms (correct order)
   ```

5. **Performance** — Did it take reasonable time?
   ```
   Expected duration: 10-1000ms
   ✅ Actual: 245ms (within bounds)
   ```

---

## Semantic Conventions Matter

Weaver uses **OpenTelemetry Semantic Conventions** — standard names for spans and attributes.

### Why Standards Matter

Without standards:
```
"http.server.request" (my app)
"http_server_request" (your app)
"server-http-request" (their app)
```
**Same thing, 3 different names!**

With standards:
```
"http.server.request" (everyone)
```
**Consistent, comparable, testable!**

### Common Semantic Conventions

| Convention | Meaning | Example |
|-----------|---------|---------|
| `http.server.request` | Server-side HTTP request | API handler |
| `db.client.call` | Database query | SELECT statement |
| `cache.get` | Cache lookup | Redis GET |
| `auth.check` | Authentication | Token validation |
| `rpc.call` | RPC invocation | gRPC call |

---

## Design Philosophy

### Problem: Tests Can Lie

```
Test passes ✅
├─ Exit code = 0 ✅
├─ (But did we actually test anything?)
└─ Unknown!
```

### Solution: Prove Behavior

```
Test passes ✅
├─ Exit code = 0 ✅
├─ HTTP span emitted ✅
├─ Database queried ✅
├─ Attributes correct ✅
└─ Behavior PROVEN!
```

### Why Weaver is Different

**Traditional tools**: Test exit codes, mocks, assertions
**Problem**: Can all lie, code might not execute

**Weaver approach**: Validate actual runtime telemetry
**Advantage**: Real execution produces real telemetry, telemetry doesn't lie

---

## Integration with clnrm

### clnrm + Weaver = Behavior-Driven Testing

1. **clnrm** — Orchestrates test execution in isolated containers
2. **OTEL instrumentation** — Code emits telemetry during execution
3. **Weaver** — Validates telemetry against schema
4. **Result** — Proof that test actually worked

### Test Execution Flow

```
Test TOML
  ↓
clnrm parses config
  ↓
[weaver] section enables validation
  ↓
Container starts (with instrumentation)
  ↓
Code runs, emits telemetry
  ↓
Weaver validates telemetry
  ↓
Results: ✅ PASS (behavior proven)
         ❌ FAIL (behavior violated)
```

---

## Benefits

### 1. Catch Real Bugs

Bugs that traditional testing misses:
- API not actually handling request
- Database never queried
- Services not communicating
- Operations in wrong order

### 2. Confidence in Tests

**Know** your tests are actually testing what you think.

### 3. Deterministic Results

No flaky tests from race conditions (you catch them).

### 4. Documentation

Schema documents expected behavior explicitly.

---

## See Also

- [Tutorial 3: Weaver Validation](../tutorials/03-weaver-validation/)
- [How-To: Weaver Schemas](../how-to/weaver-schemas.md)
- [OpenTelemetry Documentation](https://opentelemetry.io)
- [Semantic Conventions](https://opentelemetry.io/docs/reference/specification/protocol/exporter/)
