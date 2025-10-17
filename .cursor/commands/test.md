# Run All Tests

Execute complete test suite (unit + integration).

## Command
```bash
cargo make test-all
```

## What It Does
- **Unit tests** (`cargo test --lib`)
- **Integration tests** (`cargo test --test '*'`)

## Alternative Commands
```bash
cargo make test              # Unit tests only
cargo make test-integration  # Integration only
cargo make test-cleanroom    # Cleanroom tests
cargo make test-proptest     # Property-based tests
```

## Use When
- Before pushing code
- After significant changes
- Before creating PR

## Time: ~1-2 minutes
