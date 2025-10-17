# AI Feature Flag Implementation (Issues #12-13 Fix)

## Summary

Fixed confusing AI feature UX by properly using Cargo feature flags to hide experimental AI commands unless explicitly enabled.

## Changes Made

### 1. `/crates/clnrm-core/src/cli/types.rs`
- Added `#[cfg(feature = "ai")]` to 6 AI command variants in the `Commands` enum:
  - `AiOrchestrate`
  - `AiPredict`
  - `AiOptimize`
  - `AiReal`
  - `AiMonitor`
- Added `#[cfg(feature = "ai")]` to `ServiceCommands::AiManage`
- Updated command descriptions to include `[EXPERIMENTAL - requires 'ai' feature]`

### 2. `/crates/clnrm-core/src/cli/mod.rs`
- Added `#[cfg(feature = "ai")]` to all 6 AI command handlers to match the enum variants
- Commands now only compile when the `ai` feature is enabled

### 3. `/crates/clnrm-core/Cargo.toml`
- Added `ai = []` marker feature (no dependencies to avoid circular dependency with clnrm-ai crate)
- Updated `default = ["tracing-subscriber", "reqwest"]` to include reqwest

### 4. `/crates/clnrm/Cargo.toml`
- Added `[features]` section with:
  - `default = []`
  - `ai = ["clnrm-ai"]` - Enables AI features and includes clnrm-ai dependency
  - `otel = ["clnrm-core/otel"]` - OTEL features
- Made `clnrm-ai` dependency optional with `optional = true`

### 5. `/FALSE_README.md`
- Added comprehensive "Experimental AI Features" section explaining:
  - How to enable AI features via Cargo and Homebrew
  - List of all AI commands with descriptions
  - Clear warning that commands won't appear in help without the feature
  - Status and requirements (SurrealDB, Ollama)

## Result

**Without `--features ai`:**
- AI commands do NOT appear in `clnrm --help`
- Clean, production-focused CLI experience
- No confusing experimental features cluttering the interface

**With `--features ai`:**
```bash
cargo install clnrm --features ai
# OR
cargo build --release --features ai
```
- All 6 AI commands become available:
  - `clnrm ai-orchestrate` - AI test orchestration
  - `clnrm ai-predict` - Predictive analytics
  - `clnrm ai-optimize` - Test optimization
  - `clnrm ai-real` - Real AI with SurrealDB + Ollama
  - `clnrm ai-monitor` - Autonomous monitoring
  - `clnrm services ai-manage` - AI service management

## Technical Details

### Why No Circular Dependency?

The key insight is that `clnrm-core` (the library) doesn't actually need to depend on `clnrm-ai`. Only the top-level `clnrm` binary needs the optional dependency:

```
clnrm (binary)
├─ clnrm-core (library) [ai feature is just a marker]
└─ clnrm-ai (optional, only when ai feature enabled)
   └─ clnrm-core (library)
```

This avoids the circular dependency:
- `clnrm-core` defines AI command *types* with `#[cfg(feature = "ai")]`
- `clnrm` binary optionally depends on `clnrm-ai` for implementations
- No cycle!

### Feature Flag Best Practices Applied

1. **Opt-in by default** - Experimental features hidden unless requested
2. **Clear documentation** - README explains how to enable
3. **Proper Cargo.toml structure** - Optional dependencies, marker features
4. **Compile-time gating** - `#[cfg(feature = "ai")]` ensures zero runtime cost
5. **Consistent UX** - Commands simply don't exist without the feature

## Testing

To verify the fix works:

```bash
# Build without AI features (default)
cargo build --release
./target/release/clnrm --help  # AI commands should NOT appear

# Build with AI features
cargo build --release --features ai
./target/release/clnrm --help  # AI commands SHOULD appear
```

## Related Issues

- Fixes #12: AI feature UX confusion
- Fixes #13: AI commands appearing in help when not functional
