# Git Hooks Quick Reference

## Setup (One Time)

```bash
./scripts/setup-git-hooks.sh
```

## Hooks Overview

| Hook | When | Duration | Checks |
|------|------|----------|--------|
| **pre-commit** | Before commit | ~30s | TOML, format, clippy, common issues |
| **pre-push** | Before push | ~60-120s | Tests, Weaver, integration, production build |

## Quick Commands

### Enable Hooks
```bash
./scripts/setup-git-hooks.sh
```

### Disable Hooks
```bash
git config --local --unset core.hooksPath
```

### Skip Once (Emergency Only)
```bash
git commit --no-verify -m "msg"
git push --no-verify origin branch
```

### Test Hooks Manually
```bash
./.githooks/pre-commit
./.githooks/pre-push
```

## Common Fixes

### Format Issues
```bash
cargo fmt --all
```

### Clippy Issues
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --fix  # Auto-fix (review changes!)
```

### TOML Issues
```bash
bash scripts/doc-validation/validate-toml-examples.sh
```

### Test Failures
```bash
cargo test --workspace --all-features
```

## Hook Checks

### pre-commit (Fast)
1. ✓ TOML validation
2. ✓ Code formatting
3. ✓ Clippy linting
4. ✓ Build check
5. ✓ Common issues (unwrap, println)

### pre-push (Comprehensive)
1. ✓ Full test suite
2. ✓ Weaver schema validation
3. ✓ Integration tests
4. ✓ Production build
5. ✓ Documentation build
6. ✓ Branch protection

## Performance Tips

- **First run:** ~30s (cold cache)
- **Subsequent:** ~10-15s (warm cache)
- **No changes:** ~5s (validation only)

Keep cache warm:
```bash
cargo build  # Keeps incremental compilation cache
```

## Troubleshooting

### Hook Not Running?
```bash
git config --local core.hooksPath
# Should output: .githooks
```

### Permission Denied?
```bash
chmod +x .githooks/*
```

### Weaver Not Found?
```bash
cargo install weaver
```

## Best Practices

✅ **DO:**
- Let hooks run (they catch issues early)
- Fix issues rather than skip
- Run `cargo test` before push
- Keep commits small

❌ **DON'T:**
- Skip hooks regularly
- Ignore hook failures
- Push without local testing
- Disable hooks permanently

## Documentation

Full documentation: `docs/GIT_HOOKS.md`

## Support

- Hook issues: See `docs/GIT_HOOKS.md`
- Build issues: See `CLAUDE.md`
- Test issues: See `docs/TESTING.md`
