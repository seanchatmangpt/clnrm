# clnrm Cursor Commands - 80/20 Unified System

**Principle:** Cargo-make is the engine, Cursor commands are the interface.

## Available Commands (6 Essential)

Type `/` in Cursor chat to access:

| Command | Maps To | Purpose |
|---------|---------|---------|
| `/dev` | `cargo make dev` | Quick dev iteration |
| `/test` | `cargo make test-all` | Run all tests |
| `/validate` | `cargo make validate` | Production validation |
| `/fix` | `cargo make fix` | Auto-fix issues |
| `/release` | `cargo make release-validation` | Release prep |
| `/help` | - | Show help |

## System Architecture

```
┌──────────────────┐
│ Cursor Commands  │  ← Simple delegators (6 commands)
│   /dev, /test    │
└────────┬─────────┘
         │ delegates to
         ▼
┌──────────────────┐
│  Cargo-Make      │  ← Single source of truth
│  Makefile.toml   │  ← All logic here (125 tasks)
└────────┬─────────┘
         │ inline scripts
         ▼
┌──────────────────┐
│ Embedded Logic   │  ← No external scripts needed
│   (inline)       │
└──────────────────┘
```

## Why This Design?

### Before (Complex)
- 35 cursor commands (overwhelming)
- 125 cargo-make tasks (duplicated logic)
- 25 shell scripts (maintenance burden)
- **Total:** 185 things to maintain

### After (Simple)
- 6 cursor commands (80/20 essential)
- 125 cargo-make tasks (single source of truth)
- 0 shell scripts (logic embedded)
- **Total:** 131 things to maintain (29% reduction)

## Benefits

1. **Single Source of Truth** - All logic in Makefile.toml
2. **Consistency** - Same commands work in CI, local, and Cursor
3. **Discoverability** - `cargo make --list-all-steps`
4. **Maintainability** - Update once, works everywhere
5. **Testability** - Easy to test cargo-make tasks

## Quick Start

### Most Common Workflows

**Daily Development:**
```bash
/dev              # Format, lint, quick test (30s)
```

**Before Commit:**
```bash
cargo make pre-commit    # Full pre-commit validation (2-3m)
```

**Before PR:**
```bash
/test             # All tests (1-2m)
/validate         # Production validation (5-10m)
```

**Before Release:**
```bash
/release          # Release validation (10-15m)
```

## Cursor Commands Are Delegators

Each cursor command is intentionally minimal - it just delegates to cargo-make:

```markdown
# /dev

Quick development iteration.

## Command
cargo make dev

## What It Does
- Format code
- Lint with clippy
- Run quick tests
```

This ensures:
- ✅ No duplicated logic
- ✅ Cursor and CLI always in sync
- ✅ Easy to maintain
- ✅ Clear single responsibility

## Finding Commands

### In Cursor
Type `/` to see all 6 commands

### In Terminal
```bash
cargo make --list-all-steps    # All 125 tasks
cargo make help-categories     # Organized by category
```

### Most Used Tasks

```bash
# Development
cargo make dev, quick, watch, pre-commit

# Testing
cargo make test, test-all, test-cleanroom, test-proptest

# Quality
cargo make fmt, clippy, check, audit

# Validation
cargo make validate, validate-crate, production-ready

# Build
cargo make build, build-release, clean
```

## Archived Commands

Old cursor commands are in `.cursor/commands-archive/` for reference.

They were consolidated into the 6 essential commands to reduce complexity and maintenance burden.

## Documentation

- **This README** - Cursor command reference
- **Makefile.toml** - All task definitions
- **docs/SYSTEM_CONSOLIDATION_ANALYSIS.md** - Design rationale
- **.cursorrules** - Core team standards

## Support

Questions? Check:
1. `/help` command
2. `cargo make --list-all-steps`
3. `docs/GGEN_ADAPTATION.md`

