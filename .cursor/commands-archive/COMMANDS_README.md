# clnrm Custom Commands Reference

This directory contains custom Cursor commands for streamlined clnrm development workflows.

## Available Commands

### 🚀 Production & Release

| Command | Purpose | Usage |
|---------|---------|-------|
| `/production-validate` | Run comprehensive production readiness validation | Pre-deployment checks |
| `/release-prep` | Prepare for release (version, changelog, validation) | Before publishing to crates.io |

### 🔧 Development Workflow

| Command | Purpose | Usage |
|---------|---------|-------|
| `/pre-commit` | Run pre-commit validation (fmt, clippy, tests) | Before every commit |
| `/dev-workflow` | Quick development iteration cycle | Rapid testing during development |
| `/create-test` | Generate new test following AAA pattern | When writing new tests |
| `/add-service-plugin` | Create new service plugin | Adding database/service integrations |

### 🛡️ Quality & Standards

| Command | Purpose | Usage |
|---------|---------|-------|
| `/fix-core-standards` | Fix core team standard violations | When validation finds `.unwrap()` |
| `/code-review-checklist` | Comprehensive code review checklist | Before PR review |
| `/enforce-best-practices` | Check and enforce best practices | Periodic codebase health checks |

### 🐛 Debugging & Testing

| Command | Purpose | Usage |
|---------|---------|-------|
| `/debug-test-failure` | Systematically debug failing tests | When tests fail |
| `/run-all-tests-and-fix` | Run full test suite and fix failures | Comprehensive test validation |
| `/framework-self-test` | Run framework dogfooding tests | Validate framework with itself |

### 📊 Performance & Monitoring

| Command | Purpose | Usage |
|---------|---------|-------|
| `/benchmark-performance` | Run performance benchmarks and profiling | Performance optimization |
| `/security-audit` | Comprehensive security audit | Security validation |

### 📚 Documentation & Maintenance

| Command | Purpose | Usage |
|---------|---------|-------|
| `/generate-docs` | Build and validate documentation | Documentation updates |
| `/project-maintenance` | Regular maintenance tasks | Periodic project health checks |

## Command Categories

### Production-Ready Commands (New from ggen)

These commands were adapted from the ggen project and enforce production-grade standards:

- **`/production-validate`** - Full production readiness suite
  - Prerequisites check (Docker, Cargo)
  - Core team standards enforcement (NO `.unwrap()` or `.expect()`)
  - Test suite execution (unit, integration, cleanroom)
  - Performance benchmarks (build < 120s, CLI < 2s)
  - Security audits

- **`/fix-core-standards`** - Automated standard violation fixes
  - Scans for `.unwrap()` and `.expect()` in production code
  - Detects async trait methods (breaks `dyn` compatibility)
  - Provides fix suggestions and auto-fixes

- **`/create-test`** - Test generation with AAA pattern
  - Enforces Arrange-Act-Assert structure
  - Descriptive test naming convention
  - Proper error handling with `Result<()>`

- **`/add-service-plugin`** - Plugin architecture scaffold
  - ServicePlugin trait implementation
  - Sync method requirements for `dyn` compatibility
  - Proper error handling patterns

### Development Commands

- **`/pre-commit`** - Fast validation before commits
  - Format check
  - Clippy with zero warnings
  - Quick test suite

- **`/benchmark-performance`** - Performance analysis
  - Flamegraph generation
  - SLO validation
  - Memory profiling

### Testing Commands

- **`/debug-test-failure`** - Systematic debugging
  - Docker troubleshooting
  - Logging configuration
  - Common failure patterns

## Quick Start Examples

### Before Every Commit
```
/pre-commit
```
Runs: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --lib`

### Before Production Deployment
```
/production-validate
```
Runs: Full validation suite with reports

### Creating a New Test
```
/create-test for CleanroomEnvironment container cleanup when service fails
```
Generates test scaffold following AAA pattern

### Adding a Service Plugin
```
/add-service-plugin for PostgreSQL
```
Creates plugin template with all boilerplate

### Fixing Standard Violations
```
/fix-core-standards
```
Scans and fixes `.unwrap()` and `.expect()` usage

### Debugging Failed Test
```
/debug-test-failure test_container_creation_fails
```
Provides systematic debugging steps

## Core Team Standards Enforced

All commands enforce these mandatory standards:

1. **Error Handling**
   - ❌ NO `.unwrap()` in production code
   - ❌ NO `.expect()` in production code
   - ✅ Always use `Result<T, CleanroomError>`

2. **Async/Sync Rules**
   - ❌ NO async trait methods (breaks `dyn`)
   - ✅ Use `tokio::task::block_in_place` for async in traits

3. **Testing Standards**
   - ✅ AAA pattern (Arrange, Act, Assert)
   - ✅ Descriptive test names
   - ✅ No fake `Ok(())` - use `unimplemented!()`

4. **Quality Gates**
   - ✅ Clippy with `-D warnings` (ZERO tolerance)
   - ✅ All tests must pass
   - ✅ Documentation must build

## Integration with Makefile.toml

Commands map to cargo-make tasks:

| Command | Makefile.toml Task |
|---------|-------------------|
| `/production-validate` | `cargo make production-readiness-full` |
| `/pre-commit` | `cargo make pre-commit-full` |
| `/release-prep` | `cargo make release-validation` |
| `/benchmark-performance` | `cargo make benchmark-performance` |

## Command Development

### Creating New Commands

1. Create `.md` file in `.cursor/commands/`
2. Use descriptive filename (e.g., `my-workflow.md`)
3. Follow structure:
   - **What This Does** - Clear explanation
   - **Commands to Execute** - Exact bash commands
   - **Checklist** - Validation steps
   - **Examples** - Usage examples

### Command Template

```markdown
# Command Title

Brief description of what this command does.

## What This Does

1. Step one
2. Step two
3. Step three

## Commands to Execute

```bash
# Command examples
cargo test
```

## Checklist

- [ ] Item 1
- [ ] Item 2

## Expected Output

What success looks like.

## Time Estimate

How long this takes.
```

## Tips for Using Commands

1. **Type `/` in chat** - See all available commands
2. **Add context after command** - `/production-validate for clnrm-core only`
3. **Chain commands** - `/pre-commit then /production-validate`
4. **Use in CI/CD** - Commands map to cargo-make tasks

## Command Maintenance

Commands are version-controlled with the project. To update:

1. Edit `.md` file in `.cursor/commands/`
2. Test command works as documented
3. Update this README if adding new commands
4. Commit changes with descriptive message

## Related Documentation

- **Makefile.toml** - Task automation (see Section 22: Cleanroom Validation)
- **scripts/** - Validation scripts (`validate-crate.sh`, etc.)
- **docs/GGEN_ADAPTATION.md** - Implementation details
- **.cursorrules** - Core team standards

---

**Total Commands:** 19
**Production Commands:** 8 (adapted from ggen)
**Development Commands:** 11 (original)
