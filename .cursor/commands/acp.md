# Add, Commit, Push (ACP) - Multi-Step Workflow

## Purpose

This command guides agents through the complete workflow of staging changes, validating code quality, committing, and pushing. It breaks down the complex process into clear, sequential steps with validation checkpoints.

## Workflow Overview

```
Step 1: Pre-Validation → Step 2: Fix Issues → Step 3: Re-Validation → Step 4: Stage → Step 5: Commit → Step 6: Push
```

## Step-by-Step Instructions

### Step 1: Pre-Validation Checkpoint

**Action**: Run pre-commit validation to identify all issues before proceeding.

```bash
cargo make pre-commit
```

**What this does**:
- Formats code (`cargo make fmt`)
- Lints code (`cargo make lint`)
- Runs tests (`cargo make test-unit`)
- Checks for dead code

**Expected Result**: All checks pass (exit code 0)

**CRITICAL**: Pre-commit checks MUST pass before committing. CI/CD will block commits
with compilation errors. Always run `cargo make pre-commit` before committing.

**Root Cause Prevention**: This step prevents compilation errors from being committed.
If this step fails, DO NOT commit - fix errors first.

**If this step fails**: Proceed to Step 2 (Fix Issues)

**If this step succeeds**: Skip Step 2, proceed to Step 4 (Stage Changes)

---

### Step 2: Fix Issues (Only if Step 1 Failed)

**Action**: Systematically fix all issues found in Step 1.

#### 2.1: Fix Compilation Errors

**Check**: Look for `error[E...]` patterns in output

**Action**: 
```bash
cargo make check
```

**Fix**:
- Read error messages carefully
- Fix compilation errors in source files
- Re-run `cargo make check` until it passes

**Validation**: `cargo make check` exits with code 0

#### 2.2: Fix Formatting Issues

**Check**: Look for formatting warnings

**Action**:
```bash
cargo make fmt
```

**Fix**: Formatting is automatic - just run the command

**Validation**: `cargo make fmt` completes without errors

#### 2.3: Fix Linting Errors

**Check**: Look for clippy warnings/errors

**Action**:
```bash
cargo make lint
```

**Fix**:
- Read clippy suggestions
- Apply fixes or add `#[allow(...)]` attributes where appropriate
- Re-run `cargo make lint` until it passes

**Validation**: `cargo make lint` exits with code 0

#### 2.4: Fix Test Failures

**Check**: Look for `test ... FAILED` patterns

**Action**:
```bash
cargo make test
```

**Fix**:
- Identify failing tests from output
- Read test failure messages
- Fix test code or implementation
- Add failing tests to todo list if not immediately fixable
- Re-run `cargo make test` until all tests pass

**Validation**: `cargo make test` exits with code 0

**Reference**: See [verify-tests.md](./verify-tests.md) for detailed test failure handling

#### 2.5: Fix Git Hook Issues

**Check**: Look for hook validation failures

**Common Issues**:
- `unwrap()`/`expect()` in production code
- `unimplemented!()` placeholders
- TODO/FUTURE comments (on main branch)

**Fix**:
- Replace `unwrap()`/`expect()` with proper error handling
- Remove or convert placeholders
- Remove or document TODOs appropriately
- Install hooks if missing: `./scripts/install-git-hooks.sh`

**Validation**: Git hooks pass

---

### Step 3: Re-Validation Checkpoint

**Action**: Re-run pre-commit validation to ensure all issues are fixed.

```bash
cargo make pre-commit
```

**Expected Result**: All checks pass (exit code 0)

**If this step fails**: Return to Step 2, identify remaining issues, fix them

**If this step succeeds**: Proceed to Step 4

**CRITICAL**: Do not proceed to Step 4 until Step 3 passes completely.

---

### Step 4: Stage Changes

**Action**: Stage all changes for commit.

```bash
git add -A
```

**Verify**: Check what will be committed
```bash
git status
```

**Expected Result**: All intended changes are staged

**If unexpected files are staged**: Review and unstage if needed (`git reset <file>`)

---

### Step 5: Generate Commit Message

**Action**: Generate commit message based on changes.

**Pattern**: Analyze staged changes and generate appropriate message:

**Documentation changes** (`README.md`, `docs/**/*.md`):
```
docs: update documentation
```

**Rust code changes** (`crates/**/src/**/*.rs`, `tests/**/*.rs`):
- New features: `feat: <description>`
- Bug fixes: `fix: <description>`
- Refactoring: `refactor: <description>`
- Tests: `test: <description>`

**Dependencies** (`Cargo.toml`, `Cargo.lock`):
```
chore: update dependencies
```

**Configuration** (`Makefile.toml`, `.cursor/**`):
```
chore: update configuration
```

**Test configuration** (`tests/**/*.toml`):
```
test: update test configuration
```

**Multiple types**: Use most significant change type, or combine:
```
feat: add new feature and update docs
```

**Default**: If unclear, use descriptive message:
```
chore: update project files
```

**Format**: Use conventional commit format: `<type>: <description>`

---

### Step 6: Commit Changes

**Action**: Commit staged changes with generated message.

```bash
git commit -m "<generated-message>"
```

**Expected Result**: Commit succeeds

**If commit fails**: 
- Check for uncommitted changes: `git status`
- Verify message format
- Retry commit

---

### Step 7: Push to Remote

**Action**: Push committed changes to remote repository.

```bash
git push
```

**Expected Result**: Push succeeds

**If push fails**:
- Check remote connection: `git remote -v`
- Check branch tracking: `git branch -vv`
- Resolve conflicts if any
- Retry push

---

## Complete Workflow Example

```bash
# Step 1: Pre-Validation
cargo make pre-commit
# Output: Some tests failed

# Step 2: Fix Issues
cargo make test
# Identify failing tests
# Fix test code
cargo make test
# All tests pass

# Step 3: Re-Validation
cargo make pre-commit
# All checks pass ✅

# Step 4: Stage
git add -A
git status  # Verify

# Step 5: Generate Message
# Analyze: Changed docs/V2_0_0_ARCHITECTURE.md
# Message: "docs: update architecture documentation"

# Step 6: Commit
git commit -m "docs: update architecture documentation"

# Step 7: Push
git push
```

## Error Handling

### If Pre-Validation Fails Multiple Times

**After 3 attempts**: 
- Document remaining issues in todo list
- Commit with `WIP:` prefix if critical
- Create issue for remaining problems
- Do not commit broken code to main branch

### If Tests Are Flaky

**Action**:
- Add flaky test to todo list
- Document in commit message: `test: fix flaky test (WIP)`
- Create follow-up task to investigate

### If Git Push Fails

**Common Causes**:
- Remote branch diverged
- No upstream branch set
- Authentication issues

**Fix**:
```bash
# Check remote
git remote -v

# Set upstream if needed
git push --set-upstream origin <branch>

# Pull and merge if diverged (NEVER use --rebase)
git pull
git push
```

**Why Merge Instead of Rebase:**

Rebasing is an anti-pattern that should never be used because it:

- **Rewrites History**: Rebasing rewrites commit SHAs, breaking all references to those commits in CI/CD pipelines, issue trackers, deployment systems, and debugging tools. This creates a cascade of failures across the entire development infrastructure.

- **Requires Force-Push**: Rebasing requires force-pushing (`git push --force`), which disrupts all other developers who have already pulled the branch. This creates a destructive workflow that breaks collaboration.

- **Loses Context**: Merge commits preserve the complete development timeline, showing when features were developed and integrated. Rebasing destroys this historical accuracy, making debugging and auditing impossible.

- **Breaks CI/CD**: Automated systems that reference commit hashes (build artifacts, deployment pipelines, test results) break when commits are rebased. Merge operations maintain stable commit references that CI/CD systems depend on.

- **Creates Unnecessary Conflicts**: When multiple developers work on the same branch, rebasing creates unnecessary conflicts as each developer must rebase their work on top of others' rebased commits. Merging handles parallel development naturally without requiring force-pushes.

- **Destroys Audit Trail**: Merge commits provide a clear audit trail showing when features were integrated and by whom. This is essential for compliance, debugging, and understanding project evolution. Rebasing makes this impossible.

- **Makes Debugging Harder**: Stable commit references (SHAs) allow debugging tools to reliably reference specific commits. Rebasing changes commit hashes, breaking these critical integrations and making it impossible to track down issues.

**Always use `git pull` (which merges by default) - never use `git pull --rebase` or any rebase operations.**

## Best Practices

1. **Always run Step 1 first** - Never skip validation
2. **Fix issues immediately** - Don't accumulate technical debt
3. **Verify after fixes** - Always run Step 3 before committing
4. **Meaningful commit messages** - Use conventional commit format
5. **Small, focused commits** - One logical change per commit
6. **Never skip validation** - Broken code breaks the build

## Documentation References

- **[Build System Practices](../rules/build-system-practices.mdc)** - Build commands and workflows
- **[Verify Tests Command](./verify-tests.md)** - Detailed test failure handling
- **[Chicago TDD Standards](../rules/chicago-tdd-standards.mdc)** - Testing standards
- **[Chicago TDD Guide](../../docs/testing/chicago-tdd-guide.md)** - Complete testing guide with patterns

## Quick Reference

```bash
# Full workflow (run sequentially)
cargo make pre-commit          # Step 1: Validate
# Fix issues if needed          # Step 2: Fix
cargo make pre-commit          # Step 3: Re-validate
git add -A                      # Step 4: Stage
git commit -m "<message>"       # Step 5-6: Commit
git push                        # Step 7: Push
```
