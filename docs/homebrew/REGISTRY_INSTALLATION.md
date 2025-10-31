# Homebrew Registry Installation - clnrm v1.2.1

## The Registry Path Bug Fix

**Problem:** The registry directory was not being installed by Homebrew, causing `clnrm self-test` to fail with registry path errors.

**Root Cause:** The original Homebrew formula only installed the binary (`bin.install "target/release/clnrm"`) and did not include the registry directory.

**Solution:** Update the formula to install the registry to `share/clnrm/registry/`.

## Updated Installation Section

```ruby
def install
  # Build with OTEL features enabled
  system "cargo", "build", "--release", "--features", "otel"
  bin.install "target/release/clnrm"

  # Install registry to share/clnrm/registry
  # The registry contains OpenTelemetry Weaver schemas that validate
  # the framework's telemetry output at runtime. This is critical for
  # the framework's false-positive detection capabilities.
  #
  # Installation layout:
  #   #{share}/clnrm/registry/registry_manifest.yaml  (registry metadata)
  #   #{share}/clnrm/registry/core/                   (core span schemas)
  #   #{share}/clnrm/registry/metrics/                (metric definitions)
  #   #{share}/clnrm/registry/events/                 (event definitions)
  #   #{share}/clnrm/registry/cli/                    (CLI operation schemas)
  (share/"clnrm/registry").mkpath
  (share/"clnrm/registry").install Dir["registry/*"]
end
```

## Installation Layout

After `brew install clnrm`, the registry will be installed at:

```
/opt/homebrew/share/clnrm/registry/          (Apple Silicon)
/usr/local/share/clnrm/registry/             (Intel Macs)
/home/linuxbrew/.linuxbrew/share/clnrm/registry/  (Linux)
```

### Directory Structure

```
share/clnrm/registry/
├── registry_manifest.yaml     # Registry metadata and configuration
├── validate.sh                # Validation script
├── INDEX.md                   # Quick reference guide
├── README.md                  # Complete documentation
├── SCHEMA_SUMMARY.md          # Implementation summary
├── VALIDATION_STRATEGY.md     # Validation methodology
├── core/                      # Core span schemas
│   ├── test_execution.yaml
│   ├── container_lifecycle.yaml
│   └── plugin_system.yaml
├── metrics/                   # Metric definitions
│   └── test_metrics.yaml
├── events/                    # Event definitions
│   └── test_events.yaml
└── cli/                       # CLI operation schemas
    ├── initialization.yaml
    ├── health_check.yaml
    ├── plugin_operations.yaml
    ├── service_management.yaml
    ├── project_operations.yaml
    ├── image_operations.yaml
    └── tdd_workflow.yaml
```

## Registry Validation Tests

The updated formula includes comprehensive tests to verify registry installation:

```ruby
def test
  # Test that registry was installed correctly
  assert_predicate share/"clnrm/registry/registry_manifest.yaml", :exist?,
                   "Registry manifest not found - registry installation failed"
  assert_predicate share/"clnrm/registry/core", :directory?,
                   "Registry core schemas not found"
  assert_predicate share/"clnrm/registry/metrics", :directory?,
                   "Registry metrics not found"
  assert_predicate share/"clnrm/registry/events", :directory?,
                   "Registry events not found"

  # Test that self-test can find the registry
  # This validates the fix for the registry path bug
  system "#{bin}/clnrm", "self-test", "--suite", "basic"
end
```

## Why Registry Installation Matters

The registry is **critical** to clnrm's false-positive detection:

1. **Schema-First Validation**: Runtime telemetry must match declared schemas
2. **No False Positives**: Tests can pass when features are broken; schemas can't
3. **Source of Truth**: Weaver validation proves features work, not tests
4. **Production Readiness**: Without registry, `clnrm self-test` cannot validate Weaver integration

## Verification Commands

After installation, verify the registry is present:

```bash
# Check registry location
ls -la $(brew --prefix)/share/clnrm/registry/

# Validate registry schemas
cd $(brew --prefix)/share/clnrm/registry
./validate.sh

# Run self-test (validates registry integration)
clnrm self-test --suite basic
clnrm self-test --suite otel --otel-exporter stdout
```

## Impact on Users

### Before (v1.2.0)
```bash
$ brew install clnrm
$ clnrm self-test
Error: Registry path not found: /opt/homebrew/share/clnrm/registry
```

### After (v1.2.1)
```bash
$ brew install clnrm
$ clnrm self-test
✅ Registry validation: PASSED
✅ Self-test suite: PASSED
```

## Formula Locations

Two formula files were updated:

1. **`homebrew/Formula/clnrm.rb`**
   Used for local development and testing (`brew install --build-from-source .`)

2. **`homebrew/homebrew-core-formula/clnrm.rb`**
   Template for submitting to Homebrew core (`homebrew/homebrew-core`)

Both files are now identical in their registry installation logic.

## Release Process

When releasing v1.2.1:

1. Update version numbers in both formula files
2. Update tarball URLs and SHA256 checksums
3. Test locally: `brew install --build-from-source .`
4. Verify registry installation: `ls $(brew --prefix)/share/clnrm/registry/`
5. Run self-test: `clnrm self-test`
6. Submit PR to `homebrew/homebrew-core` with updated formula

## Technical Details

### Why `share/` Directory?

Homebrew conventions:
- `bin/` - Executables
- `lib/` - Libraries
- `share/` - **Data files** (schemas, configs, documentation)
- `etc/` - Configuration files

The registry is **data**, not code, so it belongs in `share/`.

### Why `Dir["registry/*"]`?

This glob pattern installs all files and subdirectories from the source `registry/` directory into the Homebrew `share/clnrm/registry/` directory.

The pattern:
- Includes all YAML schemas
- Includes all markdown documentation
- Includes the validation script
- Preserves directory structure

### Build vs Install Commands

**Previous (broken):**
```ruby
cd "crates/clnrm" do
  system "cargo", "install", *std_cargo_args
end
```

- Only installs the binary
- No control over build features
- Cannot install additional files

**New (fixed):**
```ruby
system "cargo", "build", "--release", "--features", "otel"
bin.install "target/release/clnrm"
(share/"clnrm/registry").install Dir["registry/*"]
```

- Explicit build with OTEL features
- Explicit binary installation
- **Explicit registry installation**

## Related Issues

- Registry path bug: clnrm expects registry at `share/clnrm/registry`
- Weaver validation requires schemas at runtime
- Self-test suite validates registry presence

## Testing Checklist

Before releasing v1.2.1, verify:

- [ ] Formula builds successfully (`brew install --build-from-source .`)
- [ ] Binary installed: `which clnrm`
- [ ] Registry installed: `ls $(brew --prefix)/share/clnrm/registry/`
- [ ] Manifest present: `cat $(brew --prefix)/share/clnrm/registry/registry_manifest.yaml`
- [ ] Schemas present: `ls $(brew --prefix)/share/clnrm/registry/core/`
- [ ] Validation passes: `cd $(brew --prefix)/share/clnrm/registry && ./validate.sh`
- [ ] Self-test passes: `clnrm self-test --suite basic`
- [ ] Weaver check passes: `weaver registry check -r $(brew --prefix)/share/clnrm/registry/`

## References

- Homebrew formula documentation: https://docs.brew.sh/Formula-Cookbook
- clnrm registry documentation: `/registry/INDEX.md`
- Weaver validation guide: `/docs/WEAVER_VALIDATION_GUIDE.md`
