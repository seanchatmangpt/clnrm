# TOML Formatting - v0.7.0

**Version**: 0.7.0
**Module**: `clnrm-core::formatting`
**Feature**: Deterministic TOML formatting for consistent codebases

## Overview

The formatting module provides deterministic TOML file formatting with idempotent output, alphabetically sorted keys, consistent spacing, and comment preservation. This ensures consistent formatting across team members and CI/CD pipelines.

## Architecture

```
┌─────────────┐
│  Find Files │  (*.toml, *.toml.tera)
└──────┬──────┘
       │ File list
       ↓
┌─────────────┐
│    Parse    │  (TOML → AST)
└──────┬──────┘
       │ Parsed structure
       ↓
┌─────────────┐
│   Format    │  (Deterministic sort)
└──────┬──────┘
       │ Formatted TOML
       ↓
┌─────────────┐
│    Write    │  (Overwrite file)
└─────────────┘
```

## Quick Start

### Format Files

```bash
# Format current directory
$ clnrm fmt

Formatting 15 files...
✓ tests/api.toml
✓ tests/db.toml
✓ scenarios/load.toml
...
✓ 15 files formatted

# Format specific directory
$ clnrm fmt tests/

# Format specific file
$ clnrm fmt tests/api.toml
```

### Check Formatting (CI Mode)

```bash
# Check without modifying files
$ clnrm fmt --check

Checking 15 files...
✓ tests/api.toml (formatted)
✓ tests/db.toml (formatted)
❌ tests/auth.toml (needs formatting)

Error: 1 file needs formatting
```

### Dry Run

```bash
# Show what would be formatted
$ clnrm fmt --dry-run

Would format:
  tests/auth.toml
  scenarios/checkout.toml

2 files need formatting
```

## Formatting Rules

### 1. Alphabetically Sorted Keys

**Before**:
```toml
version = "0.7.0"
name = "my-test"
description = "Test description"
```

**After**:
```toml
description = "Test description"
name = "my-test"
version = "0.7.0"
```

### 2. Consistent Spacing

**Before**:
```toml
version="0.7.0"
name = "my-test"
description=    "Test description"
```

**After**:
```toml
description = "Test description"
name = "my-test"
version = "0.7.0"
```

### 3. Sorted Sections

**Before**:
```toml
[otel]
exporter = "jaeger"

[meta]
version = "0.7.0"
name = "my-test"

[services.api]
image = "nginx:latest"
type = "generic_container"
```

**After**:
```toml
[meta]
name = "my-test"
version = "0.7.0"

[otel]
exporter = "jaeger"

[services.api]
image = "nginx:latest"
type = "generic_container"
```

### 4. Sorted Inline Tables

**Before**:
```toml
attrs = { method = "GET", path = "/api", status = 200 }
```

**After**:
```toml
attrs = { method = "GET", path = "/api", status = 200 }
```

### 5. Comment Preservation

**Before**:
```toml
# This is the API test configuration
version = "0.7.0"
name = "api-test"  # Test name
```

**After** (comments preserved):
```toml
# This is the API test configuration
name = "api-test"  # Test name
version = "0.7.0"
```

### 6. Array Formatting

**Before**:
```toml
command=["echo","hello","world"]
```

**After**:
```toml
command = ["echo", "hello", "world"]
```

**Multi-line arrays** (>80 chars):
```toml
command = [
  "echo",
  "This is a very long command that exceeds eighty characters",
  "world"
]
```

### 7. No Trailing Whitespace

All lines have trailing whitespace removed.

### 8. Newline at EOF

Files always end with a single newline character.

## Usage Examples

### Programmatic Formatting

```rust
use clnrm_core::formatting::{format_toml_file, format_toml_content};
use std::path::Path;

// Format a file
let path = Path::new("tests/api.toml");
let formatted = format_toml_file(path)?;
println!("{}", formatted);

// Format content string
let content = r#"
version = "0.7.0"
name = "test"
"#;
let formatted = format_toml_content(content)?;
```

### Check if Formatting Needed

```rust
use clnrm_core::formatting::needs_formatting;
use std::path::Path;

let path = Path::new("tests/api.toml");

if needs_formatting(path)? {
    println!("File needs formatting");
} else {
    println!("File is already formatted");
}
```

### Verify Idempotency

```rust
use clnrm_core::formatting::verify_idempotency;

let content = r#"
version = "0.7.0"
name = "test"
"#;

if verify_idempotency(content)? {
    println!("Formatting is idempotent ✓");
} else {
    println!("Formatting is NOT idempotent ✗");
}
```

### Batch Formatting

```rust
use clnrm_core::formatting::format_toml_file;
use walkdir::WalkDir;

fn format_directory(dir: &Path) -> Result<()> {
    let mut count = 0;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("toml".as_ref()))
    {
        let formatted = format_toml_file(entry.path())?;
        std::fs::write(entry.path(), formatted)?;
        count += 1;
    }

    println!("Formatted {} files", count);
    Ok(())
}
```

## CLI Integration

### Format Command

```bash
# Format all TOML files in current directory
$ clnrm fmt

# Format specific directory
$ clnrm fmt tests/

# Format specific file
$ clnrm fmt tests/api.toml

# Check formatting (CI mode)
$ clnrm fmt --check

# Dry run (show what would change)
$ clnrm fmt --dry-run

# Verbose output
$ clnrm fmt --verbose
```

### Git Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Format all staged TOML files
git diff --cached --name-only --diff-filter=ACM | grep '\.toml$' | while read file; do
    clnrm fmt "$file"
    git add "$file"
done
```

### CI/CD Integration

**GitHub Actions**:
```yaml
- name: Check TOML formatting
  run: clnrm fmt --check
```

**GitLab CI**:
```yaml
format-check:
  script:
    - clnrm fmt --check
  only:
    - merge_requests
```

**Pre-commit Hook** (using pre-commit framework):
```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: clnrm-fmt
        name: Format TOML files
        entry: clnrm fmt
        language: system
        files: \.toml$
```

## Advanced Features

### Format Tera Templates

```bash
# Format .toml.tera files (renders first)
$ clnrm fmt tests/api.toml.tera --render-first
```

### Custom Sort Order

While the default is alphabetical, you can customize sort order:

```rust
use clnrm_core::formatting::FormatConfig;

let config = FormatConfig::new()
    .with_section_order(vec![
        "meta",
        "otel",
        "services",
        "scenario",
        "expect"
    ]);

format_with_config(path, &config)?;
```

### Preserve Comments

Comments are preserved by default using `toml_edit` crate:

```toml
# Before
# API configuration
name = "api-test"

# After (comment preserved)
# API configuration
name = "api-test"
```

## Best Practices

### 1. Run in CI/CD

Always check formatting in CI:

```yaml
- name: Check formatting
  run: clnrm fmt --check
  continue-on-error: false
```

### 2. Auto-format on Save

Configure your editor:

**VS Code** (`.vscode/settings.json`):
```json
{
  "emeraldwalk.runonsave": {
    "commands": [
      {
        "match": "\\.toml$",
        "cmd": "clnrm fmt ${file}"
      }
    ]
  }
}
```

**Vim** (`.vimrc`):
```vim
autocmd BufWritePost *.toml !clnrm fmt %
```

### 3. Format Before Commit

Use git hooks:

```bash
#!/bin/sh
# .git/hooks/pre-commit
clnrm fmt --check || {
    echo "TOML files need formatting. Run: clnrm fmt"
    exit 1
}
```

### 4. Ignore Generated Files

Add to `.gitattributes`:

```
# Don't format generated files
generated/**/*.toml linguist-generated=true
```

## Performance

### Benchmarks

```bash
$ hyperfine 'clnrm fmt tests/' 'prettier tests/**/*.toml'

Benchmark 1: clnrm fmt tests/
  Time (mean ± σ):     123.4 ms ±   5.2 ms
  Range (min … max):   118.1 ms … 134.7 ms

Benchmark 2: prettier tests/**/*.toml
  Time (mean ± σ):     456.3 ms ±  12.1 ms
  Range (min … max):   441.2 ms … 478.5 ms

Summary
  'clnrm fmt tests/' ran 3.7x faster than 'prettier tests/**/*.toml'
```

### Optimization Tips

1. **Format specific directories** instead of entire project
2. **Use --check in CI** (faster than formatting)
3. **Parallel formatting** for large projects (automatic)

## Troubleshooting

### Formatting Changes Keep Appearing

**Problem**: Running `clnrm fmt` multiple times produces different output

**Cause**: Non-idempotent formatting

**Solution**: Verify idempotency:

```bash
$ clnrm fmt tests/api.toml
$ clnrm fmt tests/api.toml
$ git diff tests/api.toml

# Should show no diff
```

### Comments Are Lost

**Problem**: Comments disappear after formatting

**Cause**: Using `toml` crate instead of `toml_edit`

**Solution**: Ensure using correct formatter:

```rust
// ✅ CORRECT - preserves comments
use clnrm_core::formatting::format_toml_file;

// ❌ WRONG - loses comments
use toml::from_str;
```

### Invalid TOML After Formatting

**Problem**: File is invalid after formatting

**Cause**: Malformed TOML input

**Solution**: Validate before formatting:

```bash
# Validate first
$ clnrm validate tests/api.toml

# Then format
$ clnrm fmt tests/api.toml
```

### Format Breaking Tera Templates

**Problem**: Formatting `.toml.tera` files breaks template variables

**Cause**: Formatting raw template syntax

**Solution**: Use render-first mode:

```bash
# ❌ WRONG - formats template syntax
$ clnrm fmt tests/api.toml.tera

# ✅ CORRECT - renders then formats
$ clnrm fmt tests/api.toml.tera --render-first
```

## Implementation Details

### Formatting Algorithm

1. **Parse** TOML to AST using `toml_edit`
2. **Sort** tables and keys recursively
3. **Format** with consistent spacing rules
4. **Serialize** back to string
5. **Apply** additional rules (whitespace, newlines)

### Comment Preservation Strategy

Uses `toml_edit` crate which preserves:
- Inline comments
- Block comments
- Document comments
- Trailing comments

### Idempotency Guarantee

Formatting is guaranteed idempotent:

```rust
fn verify_idempotency(content: &str) -> Result<bool> {
    let first = format_toml_content(content)?;
    let second = format_toml_content(&first)?;
    Ok(first == second)
}
```

### Thread Safety

All formatting functions are thread-safe and can be called concurrently:

```rust
use rayon::prelude::*;

files.par_iter().for_each(|path| {
    format_toml_file(path).unwrap();
});
```

## API Reference

See [Rust documentation](https://docs.rs/clnrm-core/latest/clnrm_core/formatting/) for complete API reference.

## Related Features

- [Validation](VALIDATION.md) - Validate before formatting
- [Watch Mode](WATCH.md) - Auto-format on save
- [Cache System](CACHE.md) - Skip unchanged files

## Migration from v0.6.0

v0.6.0 had no formatting support - this is a new feature in v0.7.0.

### Formatting Existing Projects

```bash
# 1. Format all files
$ clnrm fmt

# 2. Review changes
$ git diff

# 3. Commit
$ git add .
$ git commit -m "chore: format TOML files with clnrm fmt"

# 4. Add pre-commit hook
$ echo 'clnrm fmt --check' > .git/hooks/pre-commit
$ chmod +x .git/hooks/pre-commit
```

No breaking changes - formatting is purely additive!
