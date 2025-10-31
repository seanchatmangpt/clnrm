# Documentation Validation Guide

**Purpose:** Prevent false claims in clnrm documentation
**Audience:** Contributors and maintainers
**Last Updated:** 2025-10-29

---

## 🎯 The Problem We're Solving

Previous versions of the clnrm README had a **68% false positive rate** in feature claims. This guide ensures that never happens again.

---

## ✅ The Golden Rules

### Rule 1: Code is Truth, Documentation is Commentary

**Never claim a feature works until:**
1. ✅ Code is written and compiles
2. ✅ Tests pass
3. ✅ You've run the command yourself
4. ✅ Binary is installed and tested

**Example of Violation:**
```markdown
❌ BAD: "clnrm self-test ✅ Working"
         (when tests call unimplemented!())

✅ GOOD: "clnrm self-test 🚧 Partial - Core tests work,
         container validation incomplete"
```

### Rule 2: Use Accurate Status Indicators

| Symbol | Meaning | Requirements |
|--------|---------|--------------|
| ✅ Working | Feature is complete and tested | - All code paths implemented<br>- Tests passing<br>- Command works in installed binary<br>- No known bugs |
| 🚧 Partial | Feature partially implemented | - Core functionality works<br>- Some limitations exist<br>- Clearly document what doesn't work<br>- Link to tracking issue |
| ❌ Not Implemented | Feature doesn't exist | - No code written<br>- May be planned<br>- Roadmap item only |

### Rule 3: Examples Must Run

Every code example in documentation **must be executable**.

**Before Adding an Example:**
1. Copy the exact code into a file
2. Run it with current clnrm version
3. Verify output matches what's shown
4. Add to automated validation tests

**Example Checklist:**
```markdown
## ✅ Example Validation

- [ ] Created `examples/your_example.clnrm.toml`
- [ ] Ran: `clnrm run examples/your_example.clnrm.toml`
- [ ] Output matches documentation
- [ ] Added test to `tests/readme_validation_complete.rs`
```

### Rule 4: Version Claims Must Be Consistent

**Never have multiple version numbers** in the same document.

**Checklist Before Release:**
- [ ] README header shows correct version
- [ ] Cargo.toml matches README version
- [ ] All feature descriptions reference same version
- [ ] Roadmap section updated or removed
- [ ] No contradictory "current version" claims

---

## 📋 Pre-Commit Checklist

Before committing documentation changes:

### 1. Compilation Check
```bash
# README claims "from source installation" must work
cargo build --release
cargo test

# If this fails, you CANNOT claim:
# - "PRODUCTION READY"
# - "Complete Implementation"
# - "cargo build works"
```

### 2. Command Verification
```bash
# Every command in README must work
clnrm --version
clnrm --help
clnrm init
clnrm run examples/basic.clnrm.toml
clnrm validate examples/basic.clnrm.toml
clnrm self-test
clnrm plugins

# If ANY command fails or doesn't exist:
# - Mark it as ❌ Not Implemented
# - Remove it from "✅ Working" section
# - Update status indicator
```

### 3. Feature Status Audit
```bash
# Run automated validation
cargo test --test readme_validation_complete

# Check results:
# - All tests passing = documentation accurate
# - Any test failing = documentation claim is false
```

### 4. Cross-Reference Check

For each claimed feature:
1. **Find the source code** that implements it
2. **Read the implementation** to verify it works
3. **Check for `unimplemented!()`** macro calls
4. **Verify tests exist** and pass
5. **Link documentation to source** with file:line references

---

## 🔍 Validation Commands

### Automated Validation

```bash
# Run full validation suite
cargo test --test readme_validation_complete

# Test specific claim
cargo test test_cli_version_command
cargo test test_self_test_exists
cargo test test_container_execution

# Validate README examples
cargo test test_readme_example_
```

### Manual Validation

```bash
# Install binary and test
cargo install --path crates/clnrm

# Test each README claim
clnrm --version  # Should show version
clnrm init       # Should create .clnrm.toml
clnrm run test.clnrm.toml  # Should execute

# Test examples
cd examples
for file in *.clnrm.toml; do
    echo "Testing: $file"
    clnrm run "$file" || echo "FAILED: $file"
done
```

---

## 📝 Writing Honest Documentation

### Template for New Features

```markdown
## Feature Name

**Status:** [✅ Working / 🚧 Partial / ❌ Not Implemented]

**Description:** [What it does in one sentence]

**Implementation:**
- Source: `path/to/file.rs:line`
- Tests: `path/to/test.rs:line`
- CLI: `clnrm command --flag`

**Example:**
\`\`\`toml
# Working example that has been tested
[test.config]
name = "example"
\`\`\`

**Limitations:** [If partial, list what doesn't work]

**Verification:**
\`\`\`bash
# Command users can run to verify this works
clnrm command --flag
\`\`\`
```

### Example: Honest vs Dishonest Claims

**❌ Dishonest (Don't Do This):**
```markdown
## Container Execution ✅ Working

clnrm executes all tests in isolated Docker containers with complete
hermetic isolation. Each test gets a fresh container ensuring no state
leakage between tests.

[No source links, no evidence, no limitations mentioned]
```

**✅ Honest (Do This):**
```markdown
## Container Execution 🚧 Partial

**Status:** Core execution works, validation incomplete

**What Works:**
- Execute commands in fresh Docker containers
- Per-test isolation (one container per test step)
- Automatic cleanup after test completion
- Source: `crates/clnrm-core/src/cleanroom.rs:724-818`

**Limitations:**
- Advanced container options not exposed
- No support for custom networks yet
- Volume mounting limited to basic cases

**Example:**
\`\`\`bash
# This works:
clnrm run tests/container_test.clnrm.toml

# This works:
clnrm self-test --suite container
\`\`\`

**Verification:**
\`\`\`bash
# Run this to verify container execution works:
clnrm self-test --suite container
# Expected: 3 tests pass
\`\`\`
```

---

## 🚨 Red Flags to Avoid

### 1. Vague Claims
❌ "clnrm provides comprehensive testing"
✅ "clnrm executes TOML-defined tests in Docker containers with regex output validation"

### 2. Weasel Words
❌ "Near-instant test execution"
✅ "Tests complete in <2 seconds for basic cases (measured with hyperfine)"

### 3. Contradictory Status
❌ "Feature X ✅ Working" in one section, "Feature X planned for v2.0" in another
✅ Choose ONE status per feature

### 4. Missing Evidence
❌ "Self-test validates all core functionality"
✅ "Self-test runs 32 tests across 5 suites (framework, container, plugin, CLI, OTEL)"

### 5. Outdated Examples
❌ Examples from v0.3 in v1.0 documentation
✅ Test every example with current version before documenting

---

## 🔄 Regular Maintenance

### Monthly Audit

```bash
# 1. Check compilation
cargo build --release

# 2. Run full test suite
cargo test

# 3. Run validation tests
cargo test --test readme_validation_complete

# 4. Verify all examples
./scripts/validate_examples.sh

# 5. Check version consistency
grep -r "version.*1.0" README.md Cargo.toml

# 6. Review status indicators
grep "✅\|🚧\|❌" README.md > status_review.txt
```

### Before Each Release

- [ ] All examples tested with new version
- [ ] README version updated everywhere
- [ ] Feature matrix reflects actual code
- [ ] Compilation succeeds
- [ ] All tests pass
- [ ] Validation suite passes
- [ ] No contradictory claims
- [ ] Status indicators accurate

---

## 📊 Measuring Documentation Quality

### Metrics to Track

1. **Validation Pass Rate**
   ```bash
   cargo test --test readme_validation_complete 2>&1 | \
       grep "test result" | \
       awk '{print "Pass rate:", $4/$2}'
   ```
   - Target: 100% pass rate

2. **Claim Accuracy Rate**
   - Count: Number of ✅ Working claims
   - Verify: Number that actually work
   - Calculate: (Working / Claims) × 100
   - Target: 100%

3. **Example Success Rate**
   - Count: Number of code examples
   - Test: Number that execute successfully
   - Calculate: (Success / Total) × 100
   - Target: 100%

---

## 🛠️ Tools and Scripts

### Validation Script

Create `scripts/validate_docs.sh`:

```bash
#!/bin/bash
set -e

echo "🔍 Validating clnrm Documentation..."

# 1. Check compilation
echo "Building project..."
cargo build --release

# 2. Run tests
echo "Running test suite..."
cargo test

# 3. Run validation tests
echo "Running README validation..."
cargo test --test readme_validation_complete

# 4. Check version consistency
echo "Checking version consistency..."
README_VER=$(grep -m 1 "version-.*-blue" README.md | sed 's/.*version-\(.*\)-blue.*/\1/')
CARGO_VER=$(grep -m 1 "^version" Cargo.toml | cut -d'"' -f2)

if [ "$README_VER" != "$CARGO_VER" ]; then
    echo "❌ Version mismatch: README=$README_VER, Cargo=$CARGO_VER"
    exit 1
fi

# 5. Test examples
echo "Testing examples..."
for file in examples/*.clnrm.toml; do
    echo "Testing: $file"
    clnrm run "$file" || {
        echo "❌ Example failed: $file"
        exit 1
    }
done

echo "✅ All validation checks passed!"
```

---

## 📚 Resources

- **Validation Specification:** `docs/validation/CLNRM_CLAIMS_VALIDATION_SPEC.md`
- **Honest Feature Status:** `docs/HONEST_FEATURE_STATUS.md`
- **Discrepancy Report:** `docs/validation/CLNRM_DISCREPANCIES.md`
- **Test Suite:** `tests/readme_validation_complete.rs`

---

## 💡 Best Practices Summary

1. **Code First, Docs Second** - Only document what exists
2. **Test Every Claim** - Automated validation prevents drift
3. **Link to Source** - Provide evidence for claims
4. **Be Specific** - Avoid vague language
5. **Update Examples** - Test with every release
6. **Honest Status** - Use ✅🚧❌ accurately
7. **Single Version** - Consistency throughout
8. **Verify Before Commit** - Run validation script

---

**Remember:** Users trust documentation. Breaking that trust is worse than admitting limitations.

*"Under-promise and over-deliver" beats "over-promise and under-deliver" every time.*
