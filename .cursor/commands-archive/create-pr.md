# Create Pull Request - Complete PR Workflow

Create a pull request with proper testing, documentation, and quality checks.

## What This Does

Complete workflow for creating a production-ready pull request:
1. **Quality checks** - Run full CI locally
2. **Commit changes** - Create atomic commits
3. **Push to remote** - Update remote branch
4. **Create PR** - Generate PR with description
5. **Verify CI** - Ensure GitHub Actions pass

## Prerequisites

- ✅ All changes committed
- ✅ Tests passing locally
- ✅ Code formatted and linted
- ✅ Documentation updated

## Step-by-Step Workflow

### 1. Run Full CI Locally

```bash
# Run complete validation
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release --all-features
clnrm self-test --suite otel --otel-exporter stdout
```

### 2. Check Git Status

```bash
# Verify branch and changes
git status
git diff --stat origin/master...HEAD
git log origin/master..HEAD --oneline
```

### 3. Create Commit (if needed)

```bash
# Stage changes
git add .

# Create commit with message
git commit -m "feat: Add new service plugin for XYZ

- Implement XServicePlugin with ServicePlugin trait
- Add integration tests with 90% coverage
- Update documentation and examples
- Add TOML configuration support

Resolves #123

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 4. Push to Remote

```bash
# Push branch
git push -u origin feature/your-branch-name
```

### 5. Create Pull Request

```bash
# Using GitHub CLI
gh pr create \
  --title "feat: Add new service plugin for XYZ" \
  --body "$(cat <<'EOF'
## Summary
- Add XServicePlugin for testing XYZ containers
- Implement proper error handling and health checks
- Add comprehensive test suite with 90% coverage

## Changes
- **New**: `crates/clnrm-core/src/services/x_service.rs` - XService plugin implementation
- **Modified**: `crates/clnrm-core/src/services/mod.rs` - Register new plugin
- **New**: `tests/integration/x_service_test.rs` - Integration tests
- **Modified**: `docs/PLUGINS.md` - Add XService documentation

## Testing
- ✅ All unit tests pass
- ✅ Integration tests pass (90% coverage)
- ✅ Self-tests pass with OTEL validation
- ✅ Manual testing completed

## Checklist
- [x] Code follows clnrm coding standards
- [x] No `.unwrap()` or `.expect()` in production code
- [x] All trait methods are sync (no async)
- [x] Tests follow AAA pattern
- [x] Documentation updated
- [x] Self-tests pass
- [x] No clippy warnings
- [x] Code formatted

## Related Issues
Resolves #123

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

## PR Description Template

Use this template for your PR description:

```markdown
## Summary
[Brief description of changes - 1-3 bullet points]

## Changes
- **New**: [New files added]
- **Modified**: [Modified files]
- **Removed**: [Removed files]

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Self-tests pass
- [ ] Manual testing completed

## Test Plan
[Specific steps to test these changes]

## Checklist
- [ ] Code follows clnrm standards
- [ ] No `.unwrap()` or `.expect()` in production
- [ ] All traits remain `dyn` compatible
- [ ] Tests follow AAA pattern
- [ ] Documentation updated
- [ ] Self-tests pass
- [ ] No clippy warnings
- [ ] Code formatted

## Related Issues
Resolves #[issue number]

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

## Definition of Done

Before creating PR, verify ALL are true:

- [ ] `cargo build --release --features otel` succeeds with zero warnings
- [ ] `cargo test` passes completely
- [ ] `cargo clippy -- -D warnings` shows zero issues
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] All traits remain `dyn` compatible (no async trait methods)
- [ ] Proper `Result<T, CleanroomError>` error handling
- [ ] Tests follow AAA pattern with descriptive names
- [ ] No `println!` in production code (use `tracing` macros)
- [ ] No fake `Ok(())` returns from incomplete implementations
- [ ] **Homebrew installation validates the feature** (`brew install clnrm && clnrm self-test`)
- [ ] Feature tested via production installation path, not `cargo run`

## After PR Created

1. **Verify CI passes** - Check GitHub Actions
2. **Request review** - Tag appropriate reviewers
3. **Address feedback** - Respond to PR comments
4. **Rebase if needed** - Keep PR up to date with master

## When to Use
- Before creating any pull request
- After completing a feature
- When ready for code review
