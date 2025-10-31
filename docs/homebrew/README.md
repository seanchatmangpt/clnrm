# Homebrew Formula Documentation

This directory contains documentation for the clnrm Homebrew formula and registry installation.

## Quick Links

- **[REGISTRY_INSTALLATION.md](REGISTRY_INSTALLATION.md)** - Complete guide to registry installation, verification, and testing
- **[FORMULA_UPDATE_v1.2.1.md](FORMULA_UPDATE_v1.2.1.md)** - Summary of v1.2.1 formula changes and registry fix

## Formula Files

The repository contains two formula files:

1. **`/homebrew/Formula/clnrm.rb`**
   - Used for local development and testing
   - Install with: `brew install --build-from-source .`

2. **`/homebrew/homebrew-core-formula/clnrm.rb`**
   - Template for submission to `homebrew/homebrew-core`
   - Identical to Formula/clnrm.rb but may have different version/SHA

## Key Changes in v1.2.1

### The Registry Path Bug (FIXED)

**Problem:** Formula didn't install registry directory, causing `clnrm self-test` to fail.

**Solution:** Added registry installation to formula:

```ruby
# Install registry to share/clnrm/registry
(share/"clnrm/registry").mkpath
(share/"clnrm/registry").install Dir["registry/*"]
```

### Installation Layout

After `brew install clnrm`:

```
$(brew --prefix)/
├── bin/clnrm                    # Binary executable
└── share/clnrm/registry/        # Registry schemas (NEW)
    ├── registry_manifest.yaml
    ├── core/
    ├── metrics/
    ├── events/
    └── cli/
```

## Testing Locally

```bash
# Build and install from source
brew install --build-from-source .

# Verify installation
which clnrm                      # Should show Homebrew path
clnrm --version                  # Should show version

# Verify registry installation
ls $(brew --prefix)/share/clnrm/registry/
cat $(brew --prefix)/share/clnrm/registry/registry_manifest.yaml

# Validate schemas
cd $(brew --prefix)/share/clnrm/registry
./validate.sh
weaver registry check -r .

# Run self-test (validates registry integration)
clnrm self-test --suite basic
clnrm self-test --suite otel --otel-exporter stdout
```

## Release Checklist

Before releasing a new version:

1. **Update version numbers** in both formula files
2. **Update URLs** to point to new release tarball
3. **Calculate SHA256** checksums:
   ```bash
   curl -L https://github.com/seanchatmangpt/clnrm/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
   ```
4. **Test locally:**
   ```bash
   brew install --build-from-source .
   clnrm self-test
   ```
5. **Verify registry:**
   ```bash
   ls $(brew --prefix)/share/clnrm/registry/
   cd $(brew --prefix)/share/clnrm/registry && ./validate.sh
   ```
6. **Submit PR** to `homebrew/homebrew-core` (if applicable)

## Why Registry Installation Matters

The registry is **critical** for clnrm's core value proposition:

### The False Positive Problem

Traditional testing:
```
Test passes ✅ → Assume feature works → FALSE POSITIVE
```

clnrm with Weaver validation:
```
Schema validation passes ✅ → Feature PROVEN to work → TRUE POSITIVE
```

### Without Registry

- ❌ `clnrm self-test` fails (cannot find schemas)
- ❌ Weaver validation cannot run
- ❌ No proof that telemetry is correct
- ❌ Back to traditional testing with false positives

### With Registry

- ✅ `clnrm self-test` validates full system
- ✅ Weaver validation proves schema conformance
- ✅ Runtime telemetry verified against schemas
- ✅ No false positives - validation proves it works

## Homebrew Conventions

Why we use `share/clnrm/registry/`:

- **`bin/`** - Executables (clnrm binary goes here)
- **`lib/`** - Libraries (not applicable for clnrm)
- **`share/`** - **Data files** (registry schemas belong here)
- **`etc/`** - Configuration files (not applicable for clnrm)

The registry is **data** (YAML schemas), not code, so it belongs in `share/`.

## Common Issues

### "Registry path not found"

**Symptom:**
```bash
$ clnrm self-test
Error: Registry path not found: /opt/homebrew/share/clnrm/registry
```

**Cause:** Formula didn't install registry directory (v1.2.0 and earlier)

**Solution:** Upgrade to v1.2.1 or later:
```bash
brew upgrade clnrm
ls $(brew --prefix)/share/clnrm/registry/  # Verify it exists
```

### "Weaver validation failed"

**Symptom:**
```bash
$ weaver registry check -r $(brew --prefix)/share/clnrm/registry/
Error: Registry not found
```

**Cause:** Registry not installed or installed to wrong location

**Solution:**
```bash
# Verify registry location
ls -la $(brew --prefix)/share/clnrm/

# Reinstall if needed
brew reinstall clnrm
```

### "Self-test fails with registry errors"

**Symptom:**
```bash
$ clnrm self-test
Registry validation: FAILED
```

**Cause:** Registry schemas may be invalid or incomplete

**Solution:**
```bash
# Validate schemas manually
cd $(brew --prefix)/share/clnrm/registry
./validate.sh

# Check for missing files
ls -la core/ metrics/ events/ cli/

# Reinstall if incomplete
brew reinstall clnrm
```

## Development Workflow

### Making Formula Changes

1. Edit `homebrew/Formula/clnrm.rb`
2. Test locally:
   ```bash
   brew uninstall clnrm  # if already installed
   brew install --build-from-source .
   ```
3. Verify installation:
   ```bash
   clnrm --version
   ls $(brew --prefix)/share/clnrm/registry/
   clnrm self-test
   ```
4. Update `homebrew/homebrew-core-formula/clnrm.rb` to match
5. Document changes in this directory

### Testing Registry Installation

```bash
# After installing from source
REGISTRY_PATH=$(brew --prefix)/share/clnrm/registry

# Check structure
tree $REGISTRY_PATH

# Validate all schemas
cd $REGISTRY_PATH
./validate.sh

# Run Weaver validation
weaver registry check -r $REGISTRY_PATH
weaver registry live-check --registry $REGISTRY_PATH

# Test clnrm can find registry
clnrm self-test --suite basic
```

## References

- Homebrew Formula Cookbook: https://docs.brew.sh/Formula-Cookbook
- clnrm Registry Documentation: `/registry/INDEX.md`
- Weaver Documentation: https://github.com/open-telemetry/weaver
- OpenTelemetry Semantic Conventions: https://opentelemetry.io/docs/specs/semconv/

## Support

For issues with Homebrew installation:
1. Check this documentation first
2. Verify registry installation: `ls $(brew --prefix)/share/clnrm/registry/`
3. Run self-test: `clnrm self-test`
4. File issue: https://github.com/seanchatmangpt/clnrm/issues

Include in issue report:
- Homebrew version: `brew --version`
- clnrm version: `clnrm --version`
- Installation method: `brew install clnrm` or `brew install --build-from-source .`
- Registry location: `ls -la $(brew --prefix)/share/clnrm/`
- Self-test output: `clnrm self-test 2>&1`
