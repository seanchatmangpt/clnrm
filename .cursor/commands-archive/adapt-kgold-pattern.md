# Adapt KGold Pattern - Reuse Verified Patterns

Systematically adapt patterns from the kgold repository analysis into clnrm.

## What This Does

Provides step-by-step guidance for adapting verified kgold patterns:
1. **Identify pattern** - Choose from kgold analysis
2. **Assess compatibility** - Verify fit for clnrm
3. **Copy and customize** - Adapt for clnrm needs
4. **Test integration** - Verify it works
5. **Document changes** - Update docs

## Available Patterns (from KGOLD_REPOSITORY_ANALYSIS.md)

### High-Value Scripts (Reusability 90%+)

| Pattern | Source | Reusability | Adaptation Effort |
|---------|--------|-------------|-------------------|
| **Security Config** | `deny.toml` | 95% | Low (copy & customize) |
| **Coverage Script** | `scripts/coverage.sh` | 95% | Low (adjust thresholds) |
| **Quick Check** | `scripts/local-dev/quick-check.sh` | 95% | Low (remove kgold-specific) |
| **Security Audit** | `scripts/security/security-audit.sh` | 95% | Low (universal pattern) |
| **Fake Detection** | `scripts/scan_fakes.sh` | 95% | Low (40+ patterns) |

### Build System Patterns

| Pattern | Source | Complexity | Benefits |
|---------|--------|------------|----------|
| **Makefile.toml** | `Makefile.toml` | Medium | Advanced task automation |
| **Workspace Deps** | `Cargo.toml` | Low | Centralized version management |
| **Feature Flags** | `crates/*/Cargo.toml` | Low | Optional capabilities |

### Configuration Patterns

| Pattern | Source | Use Case |
|---------|--------|----------|
| **Environment Config** | `kgold-otel/src/config.rs` | Environment-based configuration |
| **Docker Compose** | `observability/docker-compose.yml` | Observability stack |
| **CI Pipeline** | `.github/workflows/ci.yml` | GitHub Actions workflows |

## Step-by-Step Adaptation Process

### 1. Choose Pattern to Adapt

Review `/Users/sac/clnrm/docs/KGOLD_REPOSITORY_ANALYSIS.md` and select a pattern.

Example: Let's adapt the security configuration (`deny.toml`)

### 2. Verify Pattern Works

Check `/Users/sac/clnrm/docs/KGOLD_VERIFICATION_REPORT.md` to confirm the pattern is verified.

**Status for deny.toml**: ✅ Verified - Valid configuration

### 3. Copy Source Files

```bash
# Copy deny.toml from kgold
cp /Users/sac/dev/kgold/deny.toml /Users/sac/clnrm/deny.toml

# Or for scripts
cp /Users/sac/dev/kgold/scripts/security/security-audit.sh /Users/sac/clnrm/scripts/
```

### 4. Customize for clnrm

#### Example: Adapting deny.toml

```toml
# Original kgold deny.toml has:
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

# ✅ Keep as-is for clnrm (universal security)

[licenses]
allow = [
  "MIT",
  "Apache-2.0",
  # ... kgold list
]

# ✅ Keep as-is or add clnrm-specific licenses if needed

[sources]
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = [
  # kgold has OTEL repos
  "https://github.com/open-telemetry/opentelemetry-rust",
]

# ⚠️ Customize: Remove kgold-specific, add clnrm-specific
allow-git = [
  # Add any clnrm-specific git dependencies
]
```

#### Example: Adapting coverage.sh

```bash
# Original kgold has:
CRITICAL_FILES=("lib.rs" "api.rs" "validators.rs")  # 80%
HIGH_PRIORITY_FILES=("mttr.rs" "metrics.rs")         # 70%
MEDIUM_PRIORITY_FILES=("governance.rs" "security.rs") # 60%

# Customize for clnrm:
CRITICAL_FILES=("lib.rs" "cleanroom.rs" "backend/testcontainer.rs")  # 80%
HIGH_PRIORITY_FILES=("services/mod.rs" "config.rs")                   # 70%
MEDIUM_PRIORITY_FILES=("cli/mod.rs" "error.rs")                       # 60%
```

#### Example: Adapting quick-check.sh

```bash
# Original kgold script:
#!/bin/bash
set -euo pipefail

# Colors (keep as-is)
GREEN='\033[0;32m'
# ...

echo "🚀 Quick Check - KGold"  # Change to clnrm
echo "Running fast validation..."

# Commands - keep structure, customize output:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --lib  # Keep this

# kgold-specific removed:
# cargo make owl-lint  # REMOVE - clnrm doesn't have OWL

echo "✅ Quick check passed!"
```

### 5. Test Adapted Pattern

```bash
# For deny.toml
cargo install cargo-deny
cargo deny check

# For coverage.sh
chmod +x scripts/coverage.sh
./scripts/coverage.sh

# For quick-check.sh
chmod +x scripts/quick-check.sh
./scripts/quick-check.sh
```

### 6. Update Documentation

Add to relevant clnrm docs:

```markdown
## Security Configuration

We use `deny.toml` for supply chain security, adapted from the kgold repository.

See: `/Users/sac/clnrm/docs/KGOLD_REPOSITORY_ANALYSIS.md` for original pattern.

### Usage

\`\`\`bash
cargo deny check
\`\`\`
```

### 7. Commit Changes

```bash
git add deny.toml
git commit -m "feat: Add security configuration adapted from kgold

- Add deny.toml for supply chain security
- Configure vulnerability scanning
- Enforce license compliance
- Prohibit wildcard versions

Adapted from kgold repository analysis (95% reusable)

🤖 Generated with [Claude Code](https://claude.com/claude-code)"
```

## Quick Reference: Common Adaptations

### Security Configuration (deny.toml)

```bash
# 1. Copy
cp /Users/sac/dev/kgold/deny.toml /Users/sac/clnrm/

# 2. Customize [sources.allow-git] section

# 3. Test
cargo install cargo-deny
cargo deny check
```

### Coverage Script (coverage.sh)

```bash
# 1. Copy
cp /Users/sac/dev/kgold/scripts/coverage.sh /Users/sac/clnrm/scripts/

# 2. Edit file tiers: CRITICAL_FILES, HIGH_PRIORITY_FILES, MEDIUM_PRIORITY_FILES

# 3. Test
chmod +x scripts/coverage.sh
cargo install cargo-llvm-cov
./scripts/coverage.sh
```

### Docker Compose Stack

```bash
# 1. Copy
mkdir -p /Users/sac/clnrm/observability
cp /Users/sac/dev/kgold/observability/docker-compose.yml /Users/sac/clnrm/observability/

# 2. Customize service names (kgold → clnrm)

# 3. Test
cd observability
docker-compose up -d
```

### GitHub Actions CI

```bash
# 1. Copy
cp /Users/sac/dev/kgold/.github/workflows/ci.yml /Users/sac/clnrm/.github/workflows/

# 2. Remove kgold-specific jobs (owl-validation, ontology-compliance)

# 3. Update for clnrm (clnrm self-test commands)

# 4. Push and verify on GitHub
```

## Customization Checklist

When adapting any pattern, check:

- [ ] Remove kgold-specific references
- [ ] Update paths (`/Users/sac/dev/kgold` → `/Users/sac/clnrm`)
- [ ] Update crate names (`kgold-*` → `clnrm-*`)
- [ ] Update service names
- [ ] Adjust file lists (coverage tiers, etc.)
- [ ] Remove OWL/ontology validation (clnrm doesn't use this)
- [ ] Keep security patterns as-is (universal)
- [ ] Test before committing

## Anti-Patterns to Avoid

❌ **Don't blindly copy** - Understand what you're copying
❌ **Don't keep kgold-specific code** - OWL validation, ZIO ontology, etc.
❌ **Don't ignore verification** - Check KGOLD_VERIFICATION_REPORT.md first
❌ **Don't skip testing** - Always test adapted patterns
❌ **Don't forget documentation** - Update clnrm docs

## Pattern Categories

### ✅ Universal (Copy As-Is)

- Security configuration (deny.toml)
- Security audit scripts
- Fake code detection patterns
- Format/lint rules (rustfmt.toml)
- License enforcement

### ⚠️ Requires Customization

- Coverage scripts (file lists)
- Build automation (Makefile.toml tasks)
- CI workflows (remove OWL validation)
- Docker Compose (service names)
- Environment configs (variable names)

### ❌ Not Applicable to clnrm

- OWL ontology validation
- SHACL shape validation
- SPARQL query validation
- Weaver code generation (unless clnrm adds contracts)
- ZIO ontology compliance

## When to Use

- Implementing features kgold already has
- Adding security/quality tooling
- Setting up CI/CD pipeline
- Creating observability stack
- Establishing code standards

## Reference Documents

- **Analysis**: `/Users/sac/clnrm/docs/KGOLD_REPOSITORY_ANALYSIS.md`
- **Verification**: `/Users/sac/clnrm/docs/KGOLD_VERIFICATION_REPORT.md`
- **Source Repo**: `/Users/sac/dev/kgold`
