# Contract: README.md Structure

**Feature**: Complete README v2.1.0 and Partial CLI Refactor Migration
**Contract Type**: Documentation Structure
**Version**: 1.0.0

## Contract Overview

This contract defines the required structure, content, and validation rules for the README.md file.

## Required Sections (MUST)

### 1. Header Section
**Location**: Top of file
**Requirements**:
- Title: "# clnrm - Hermetic Container Testing Framework"
- Version badge: `![Version](https://img.shields.io/crates/v/clnrm.svg)` (auto-populated)
- Core purpose: One-line description
- Tech stack: Rust version, philosophy keywords
- NO hardcoded version numbers in text

**Validation**:
```bash
grep -q "# clnrm - Hermetic Container Testing Framework" README.md
grep -q "img.shields.io/crates/v/clnrm" README.md
! grep -E "v?[0-9]+\.[0-9]+\.[0-9]+" README.md || echo "ERROR: Hardcoded version found"
```

### 2. Constitutional Principles ("THE VITAL FEW")
**Location**: After header, before Quick Start
**Requirements**:
- Section title: "## 🎯 THE VITAL FEW (20% that matters)"
- All 5 principles with code examples
- Links to constitution.md for each principle
- Subsections: Cargo Make, Error Handling, Chicago TDD, Andon Signals, Concurrent Execution

**Validation**:
```bash
grep -q "## 🎯 THE VITAL FEW" README.md
grep -q "Cargo Make Rule" README.md
grep -q "Error Handling Rule" README.md
grep -q "Chicago TDD Rule" README.md
grep -q "Andon Signal Rule" README.md
grep -q "Concurrent Execution Rule" README.md
grep -c "constitution.md" README.md | grep -q "5"  # Must link to constitution 5 times
```

### 3. Quick Start Section
**Location**: After constitutional principles
**Requirements**:
- Section title: "## 🚀 Quick Start"
- 4-5 commands demonstrating core workflows
- Comments explaining each command
- Time-to-success < 5 minutes (documented)
- Prerequisites check (Docker running, etc.)

**Validation**:
```bash
grep -q "## 🚀 Quick Start" README.md
grep -q "clnrm run" README.md
grep -q "clnrm validate" README.md
```

### 4. Command Reference Section
**Location**: Mid-document
**Requirements**:
- Section title: "## 📋 Command Reference"
- 5 subsections (categories): Test Execution, Configuration, Observation, System Management, Development
- Each category MUST list commands with one-line descriptions
- Top 8 commands MUST have usage examples
- Migration status indicators (✅ Noun-Verb, 🔄 Legacy) for transparency

**Validation**:
```bash
command_sections=$(grep -c "^###.*Commands$" README.md)
[ "$command_sections" -eq 5 ] || echo "ERROR: Must have 5 command categories"

# Verify all 26 commands present
for cmd in run dry-run record repro stress self-test init validate lint fmt render spans report graph health live-check services collector plugins pull dev template diff analyze; do
    grep -q "$cmd" README.md || echo "ERROR: Missing command $cmd"
done
```

### 5. Code Standards Section
**Location**: After command reference
**Requirements**:
- Section title: "## ⚡ Code Standards (Zero Tolerance)"
- Bulleted list of 7+ standards
- Link to constitution.md
- Examples of prohibited patterns

**Validation**:
```bash
grep -q "## ⚡ Code Standards" README.md
grep -q "No unwrap/expect in production" README.md
grep -q "80%+ test coverage" README.md
```

### 6. Troubleshooting Section
**Location**: Near end of document
**Requirements**:
- Section title: "## 🔧 Troubleshooting"
- Organized by symptom (not alphabetically)
- Common issues: Docker not running, Rust toolchain, file permissions, timeout errors
- Each issue with cause and solution

**Validation**:
```bash
grep -q "## 🔧 Troubleshooting" README.md
grep -q "Docker" README.md
```

## Optional Sections (SHOULD)

### 7. Project Structure
**Requirements**:
- Tree diagram showing workspace layout
- Comments explaining key directories

### 8. SLOs (Service Level Objectives)
**Requirements**:
- Build times: first, incremental
- Test execution times
- Container startup times
- Reproducibility guarantees

### 9. Definition of Done
**Requirements**:
- Checklist of commands to run before merge
- All must pass (RED signal handling)

## Prohibited Content (MUST NOT)

1. **Hardcoded version numbers** in prose (use badges only)
2. **Installation instructions for Docker/Rust** (link to official docs)
3. **Complete command documentation** (Hub-and-Spoke pattern - link to separate docs)
4. **Internal implementation details** (architecture docs go elsewhere)
5. **TODOs or FIXMEs** (use GitHub issues instead)
6. **Broken links** (all links must resolve)

## Content Quality Requirements

### Tone
- **Concise**: Short sentences, active voice
- **Actionable**: Commands users can copy-paste
- **Accurate**: No outdated information
- **Accessible**: Avoids jargon without explanation

### Formatting
- **Markdown**: GitHub-flavored markdown
- **Code blocks**: Use ```bash for shell commands, ```rust for Rust code
- **Emoji**: Use sparingly for section headers only (🎯, 🚀, ⚡, 🔧, 📋)
- **Tables**: Align columns consistently

### Links
- **Internal**: Use relative paths (e.g., `docs/CODE_STANDARDS.md`)
- **External**: Use HTTPS
- **Constitution**: Link to `.specify/memory/constitution.md` (5 times minimum)

## Validation Tests

### Automated Checks
```bash
#!/bin/bash
# File: specs/001-readme-cli-refactor/contracts/validate_readme.sh

set -e

echo "Validating README.md structure..."

# Check required sections exist
sections=(
    "# clnrm - Hermetic Container Testing Framework"
    "## 🎯 THE VITAL FEW"
    "## 🚀 Quick Start"
    "## 📋 Command Reference"
    "## ⚡ Code Standards"
    "## 🔧 Troubleshooting"
)

for section in "${sections[@]}"; do
    if ! grep -q "$section" README.md; then
        echo "❌ Missing section: $section"
        exit 1
    fi
done
echo "✅ All required sections present"

# Check no hardcoded versions (allow in badge URLs)
if grep -v "img.shields.io" README.md | grep -E "[^/]v?[0-9]+\.[0-9]+\.[0-9]+"; then
    echo "❌ Hardcoded version found (use badges only)"
    exit 1
fi
echo "✅ No hardcoded versions"

# Check all 26 commands documented
commands=(run dry-run record repro stress self-test init validate lint fmt render spans report graph health live-check services collector plugins pull dev template diff analyze redgreen)
missing_commands=()

for cmd in "${commands[@]}"; do
    if ! grep -q "\`$cmd\`" README.md; then
        missing_commands+=("$cmd")
    fi
done

if [ ${#missing_commands[@]} -ne 0 ]; then
    echo "❌ Missing commands: ${missing_commands[*]}"
    exit 1
fi
echo "✅ All 26 commands documented"

# Check constitutional principles referenced
principles=("Cargo Make" "Error Handling" "Chicago TDD" "Andon Signal" "Concurrent Execution")
for principle in "${principles[@]}"; do
    if ! grep -q "$principle" README.md; then
        echo "❌ Missing principle: $principle"
        exit 1
    fi
done
echo "✅ All 5 constitutional principles referenced"

# Check constitution.md links (minimum 5)
constitution_links=$(grep -c "constitution.md" README.md)
if [ "$constitution_links" -lt 5 ]; then
    echo "❌ Need at least 5 constitution.md links (found $constitution_links)"
    exit 1
fi
echo "✅ Constitution.md linked $constitution_links times"

# Check for broken internal links
echo "Checking internal links..."
grep -oE '\[.*\]\(([^h].*\.md.*?)\)' README.md | sed -E 's/.*\(([^)]+)\).*/\1/' | while read -r link; do
    # Remove anchor (#section)
    file_path=$(echo "$link" | cut -d'#' -f1)

    if [ -n "$file_path" ] && [ ! -f "$file_path" ]; then
        echo "❌ Broken link: $file_path"
        exit 1
    fi
done
echo "✅ All internal links valid"

echo ""
echo "✅ README.md validation PASSED"
```

### Manual Review Checklist
- [ ] Quick Start can be completed in under 5 minutes
- [ ] All code examples are copy-pasteable
- [ ] No installation steps for Docker/Rust (linked to official docs instead)
- [ ] Command categories make sense (feature-driven, not alphabetical)
- [ ] Troubleshooting covers 90% of common issues
- [ ] Links to constitution.md work correctly
- [ ] Version badge displays current version from crates.io
- [ ] Tone is concise and actionable

## Example Content Templates

### Command Category Template
```markdown
### Test Execution Commands

Execute and validate container lifecycle tests.

| Command | Status | Description | Example |
|---------|--------|-------------|---------|
| `run` | 🔄 Legacy | Execute test specifications | `clnrm run tests/` |
| `dry-run` | 🔄 Legacy | Validate without execution | `clnrm dry-run tests/integration.clnrm.toml` |
| `self-test` | 🔄 Legacy | Framework self-tests | `clnrm self-test` |
| `record` | 🔄 Legacy | Record test evidence | `clnrm record tests/ --output report.json` |
| `repro` | 🔄 Legacy | Reproduce test failures | `clnrm repro failed-test.clnrm.toml` |
| `stress` | 🔄 Legacy | Stress testing with Docker | `clnrm stress --concurrency 10` |
```

### Troubleshooting Entry Template
```markdown
#### Error: "Cannot connect to Docker daemon"

**Symptom**: Test execution fails immediately with connection error.

**Cause**: Docker daemon not running or insufficient permissions.

**Solution**:
1. Check Docker status: `docker ps`
2. Start Docker Desktop (macOS/Windows) or `sudo systemctl start docker` (Linux)
3. Verify permissions: Add user to docker group (`sudo usermod -aG docker $USER`)
4. Retry test: `clnrm run tests/`
```

### Constitutional Principle Template
```markdown
### 1. CARGO MAKE RULE
```bash
# ✅ CORRECT: Always use cargo make
cargo make check     # Compilation check
cargo make test      # Run all tests
cargo make lint      # Clippy validation

# ❌ WRONG: Never direct cargo
cargo test
cargo clippy
```

**Why**: Cargo make enforces timeouts, integrates hooks, provides Andon signals.

See [Constitution: Cargo Make Rule](.specify/memory/constitution.md#i-cargo-make-rule-absolute) for full rationale.
```

## Success Criteria

**README is compliant when**:
1. All validation tests pass (automated script exits 0)
2. Manual review checklist is 100% complete
3. New user can complete Quick Start in < 5 minutes
4. All 26 commands are documented with descriptions
5. All 5 constitutional principles are referenced
6. Version information auto-populates from crates.io
7. No hardcoded version numbers in text content
8. Troubleshooting section resolves 90% of common issues

**Metrics**:
- Validation script exit code: `0`
- Command coverage: `26/26` (100%)
- Constitutional principles: `5/5` (100%)
- Constitution links: `>= 5`
- Internal broken links: `0`
- Time to first success: `< 5 minutes`
