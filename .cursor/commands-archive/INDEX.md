# Cursor Commands Index for clnrm

Complete reference of all available Cursor commands for the clnrm project.

## Quick Reference

Type `/` in Cursor chat to see all available commands.

---

## Quality & Testing Commands

### `/run-quality-gates`
**Execute all quality gates before committing**
- Runs CI gate script with 7 quality checks
- Scans for fake implementations
- Validates Core Team Standards compliance
- **When to use**: Before every commit, PR creation

### `/run-determinism-tests`
**Verify hermetic isolation produces deterministic results**
- Runs 10 test functions with 5-iteration verification
- Ensures container execution repeatability
- Validates TOML parsing consistency
- **When to use**: After changing core isolation logic, service lifecycle

### `/validate-otel-integration`
**Verify OpenTelemetry actually emits traces**
- Starts real OTEL collector in Docker
- Validates trace emission to collector
- Checks span relationships and metrics
- **When to use**: After OTEL changes, before OTEL feature release

### `/run-framework-self-test`
**Execute clnrm self-tests with production binary**
- Uses Homebrew-installed binary (dogfooding)
- Runs basic, container, service, and OTEL test suites
- **When to use**: After installation, before release, daily validation

### `/run-all-tests-and-fix`
**Run complete test suite and fix failures**
- Unit, integration, and property-based tests
- Automated failure diagnosis
- **When to use**: Before PR, after major changes

---

## Development Workflow Commands

### `/create-new-service-plugin`
**Guide for implementing new service plugins**
- ServicePlugin trait implementation
- Core Team Standards compliance
- Testing requirements and examples
- **When to use**: Adding database, cache, or API service plugins

### `/review-pr-changes`
**Comprehensive PR review checklist**
- Core Team Standards verification
- Automated and manual review steps
- Approval criteria and post-merge tasks
- **When to use**: Before approving any PR, self-review before submission

### `/debug-test-failure`
**Systematic approach to debugging test failures**
- Container lifecycle debugging
- OTEL integration issues
- Non-determinism diagnosis
- **When to use**: When tests fail, investigating flaky tests

### `/dev-workflow`
**Development workflow and best practices**
- Feature branch workflow
- Commit conventions
- Testing requirements
- **When to use**: Starting new feature, onboarding

---

## Release & Deployment Commands

### `/prepare-release`
**Complete release preparation checklist**
- Version bump procedures
- Complete test suite validation
- Documentation updates
- Homebrew installation testing
- **When to use**: Creating new release (patch, minor, major)

### `/create-pr`
**Create pull request with proper formatting**
- PR template with all required sections
- Testing checklist
- Breaking changes documentation
- **When to use**: Ready to submit feature for review

### `/production-validate`
**Validate production-ready code**
- Zero warnings enforcement
- Fake implementation detection
- Core Team Standards compliance
- **When to use**: Before merging to main, pre-release

---

## Security & Code Quality Commands

### `/security-audit`
**Comprehensive security audit**
- Dependency vulnerability scanning
- Secret detection
- Container security validation
- OTEL security review
- **When to use**: Before release, quarterly audits, after dependency updates

### `/code-review-checklist`
**Detailed code review checklist**
- Error handling verification
- Testing standards compliance
- Documentation completeness
- **When to use**: During PR review, self-review

### `/enforce-best-practices`
**Enforce Core Team Standards**
- No unwrap/expect detection
- Proper error handling validation
- AAA test pattern verification
- **When to use**: Before committing, during code review

---

## Documentation & Onboarding Commands

### `/onboard-new-developer`
**Complete developer onboarding guide**
- Environment setup (Rust, Docker, Homebrew)
- Project structure familiarization
- First contribution walkthrough
- **When to use**: New team member joining, knowledge refresh

### `/generate-docs`
**Generate and update documentation**
- API documentation with cargo doc
- CLI guide updates
- TOML reference generation
- **When to use**: After API changes, before release

---

## Performance & Optimization Commands

### `/benchmark-performance`
**Run performance benchmarks**
- Criterion benchmarks
- Container startup profiling
- Service lifecycle timing
- **When to use**: Optimizing performance, before release, regression detection

### `/full-ci`
**Run complete CI pipeline locally**
- All CI jobs in parallel
- Same checks as GitHub Actions
- **When to use**: Before pushing to CI, validating complex changes

---

## Project Maintenance Commands

### `/project-maintenance`
**Routine project maintenance tasks**
- Dependency updates
- Documentation refresh
- Test suite cleanup
- **When to use**: Monthly maintenance, technical debt reduction

### `/fix-core-standards`
**Fix Core Team Standards violations**
- Convert unwrap/expect to proper error handling
- Fix fake implementations
- Update test patterns
- **When to use**: After quality gate failures, refactoring legacy code

### `/framework-self-test`
**Advanced self-test validation**
- Multiple test suite execution
- OTEL exporter validation
- Verbose output options
- **When to use**: Deep validation, troubleshooting

---

## Specialized Commands

### `/add-otel-integration`
**Add OpenTelemetry to new code**
- Proper feature gating
- Trace/metric/log integration
- Testing requirements
- **When to use**: Adding telemetry to new features

### `/add-service-plugin`
**Quick service plugin creation**
- Simplified plugin template
- Basic integration
- **When to use**: Simple service wrappers, rapid prototyping

### `/add-test-plugin`
**Add test plugin for validation**
- Test-specific plugin patterns
- Mock service creation
- **When to use**: Creating test fixtures, validation plugins

### `/adapt-kgold-pattern`
**Adapt kcura/kgold testing patterns**
- Extract battle-tested patterns
- Integrate best practices
- **When to use**: Learning from kcura, adopting proven patterns

---

## Testing Specific Commands

### `/create-test`
**Create new test following AAA pattern**
- Arrange-Act-Assert structure
- Proper error handling
- Descriptive naming
- **When to use**: Adding new test cases

### `/fix-test-failures`
**Systematic test failure resolution**
- Failure categorization
- Debugging strategies
- Fix patterns
- **When to use**: CI failures, local test issues

---

## Quick Actions

### `/quick-check`
**Fast quality check before commit**
- Clippy
- Format check
- Fake scanner
- **Duration**: ~30 seconds
- **When to use**: Rapid validation during development

### `/pre-commit`
**Pre-commit hook validation**
- All pre-commit checks
- Automated fixes where possible
- **When to use**: Before every commit (install as git hook)

---

## Usage Patterns

### Daily Development
1. `/quick-check` - Before each commit
2. `/create-test` - When adding features
3. `/debug-test-failure` - When tests break

### Before PR
1. `/run-quality-gates` - Comprehensive validation
2. `/run-all-tests-and-fix` - Full test suite
3. `/review-pr-changes` - Self-review
4. `/create-pr` - Submit for review

### Release Cycle
1. `/run-determinism-tests` - Verify isolation
2. `/validate-otel-integration` - OTEL validation
3. `/security-audit` - Security check
4. `/prepare-release` - Release checklist
5. `/production-validate` - Final validation

### Onboarding
1. `/onboard-new-developer` - Complete setup
2. `/dev-workflow` - Learn workflow
3. `/create-new-service-plugin` - First contribution

---

## Command Categories Summary

**Quality** (7 commands): quality-gates, determinism-tests, otel-integration, framework-self-test, run-all-tests, enforce-best-practices, production-validate

**Development** (8 commands): create-service-plugin, review-pr, debug-test, dev-workflow, add-otel, add-service-plugin, add-test-plugin, create-test

**Release** (5 commands): prepare-release, create-pr, generate-docs, release-prep, full-ci

**Security** (3 commands): security-audit, code-review-checklist, fix-core-standards

**Maintenance** (4 commands): project-maintenance, benchmark-performance, fix-test-failures, framework-self-test

**Onboarding** (2 commands): onboard-new-developer, adapt-kgold-pattern

**Quick** (2 commands): quick-check, pre-commit

**Total**: 34+ commands

---

## Recently Added (Tier 1 Adaptations)

These commands leverage the kcura adaptations:

- ✅ `/run-quality-gates` - Uses new CI gate script
- ✅ `/run-determinism-tests` - 5-iteration kcura pattern
- ✅ `/validate-otel-integration` - Real collector validation
- ✅ `/run-framework-self-test` - Dogfooding validation

---

## Tips

1. **Tab completion**: Type `/` and use arrow keys to navigate
2. **Parameters**: Add context after command, e.g., `/create-pr for DX-523`
3. **Help**: Most commands include troubleshooting sections
4. **Combine**: Chain commands for workflows (e.g., quality-gates → all-tests → create-pr)

---

## Contributing New Commands

To add a new command:

1. Create `.cursor/commands/your-command.md`
2. Write clear, actionable Markdown
3. Include examples and when to use
4. Test the command
5. Update this INDEX.md

**Command Template**:
```markdown
# Command Name

Brief description of what this command does.

## When to Use
Clear guidance on when to invoke this command.

## Steps
1. First step with code examples
2. Second step
...

## Expected Results
What success looks like.

## Troubleshooting
Common issues and solutions.
```

---

**Generated**: 2025-10-17
**Project**: clnrm v0.4.0
**Total Commands**: 34+
