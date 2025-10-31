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
|--------|---------|-------------|
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

---

## 📋 Pre-Commit Checklist

Before committing documentation changes:

### 1. Compilation Check
```bash
cargo build --release
cargo test
```

### 2. Command Verification
```bash
clnrm --version
clnrm --help
clnrm init
clnrm run examples/basic.clnrm.toml
clnrm validate examples/basic.clnrm.toml
clnrm self-test
```

### 3. Feature Status Audit
```bash
cargo test --test readme_validation_complete
```

---

See full guide content in archived `docs/archive/completion-reports/VALIDATION_GUIDE.md` for complete rules, examples, and best practices.

---

**For telemetry/code validation, see:** [Telemetry Validation Guide](VALIDATION_GUIDE.md)

