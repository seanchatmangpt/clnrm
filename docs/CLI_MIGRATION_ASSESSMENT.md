# CLI Migration Assessment: clap → clap-noun-verb

## Executive Summary

**Current State**: The main clnrm CLI uses standard `clap` with a large enum-based command structure (~30+ commands)

**Target State**: Migrate to `clap-noun-verb` framework for better command composition

**Complexity**: **MODERATE to HIGH** - Requires hybrid approach (not all commands fit noun-verb pattern)

## Current CLI Structure

### Commands That Fit Noun-Verb Pattern ✅

1. **Services** (already partially implemented)
   - `services status`
   - `services logs <service>`
   - `services restart <service>`
   - `services ai-manage` (feature-gated)

2. **Collector** (already partially implemented)
   - `collector up`
   - `collector down`
   - `collector status`
   - `collector logs`

### Commands That Don't Fit Noun-Verb Pattern ❌

These are standalone verbs or complex commands:

1. **Core Commands** (15+ commands)
   - `run` - Complex with many flags (paths, parallel, jobs, fail_fast, watch, force, shard, digest, report_junit, validate, otel_exporter, otel_endpoint)
   - `init` - Standalone verb
   - `validate` - Standalone verb
   - `template` - Standalone verb
   - `plugins` - Standalone noun (no verbs)
   - `report` - Standalone verb
   - `self-test` - Standalone verb
   - `health` - Standalone verb
   - `dev` - Standalone verb
   - `fmt` - Standalone verb
   - `dry-run` - Standalone verb
   - `lint` - Standalone verb
   - `diff` - Standalone verb
   - `record` - Standalone verb
   - `pull` - Standalone verb
   - `graph` - Standalone verb
   - `repro` - Standalone verb
   - `red-green` - Standalone verb
   - `render` - Standalone verb
   - `spans` - Standalone verb
   - `analyze` - Standalone verb

2. **AI Commands** (5 commands, feature-gated)
   - `ai-orchestrate`
   - `ai-predict`
   - `ai-optimize`
   - `ai-real`
   - `ai-monitor`

## Migration Challenges

### 1. **Hybrid CLI Architecture**

**Challenge**: Only ~8% of commands (2/25 core commands) naturally fit noun-verb pattern

**Solution**: Need hybrid approach:
- Use `clap-noun-verb` for noun-verb commands (`services`, `collector`)
- Keep standard `clap` for standalone verbs (`run`, `init`, `validate`, etc.)
- Or: Create top-level "verbs" that delegate to noun-verb when appropriate

### 2. **Argument Parsing Complexity**

**Current**: Commands use structured clap derives:
```rust
Commands::Run {
    paths: Option<Vec<PathBuf>>,
    parallel: bool,
    jobs: usize,
    // ... 12+ typed fields
}
```

**clap-noun-verb**: Uses `VerbArgs` with raw `ArgMatches`:
```rust
verb!("run", "Run tests", |args: &VerbArgs| {
    let paths = args.matches.get_many::<PathBuf>("paths")?; // Manual parsing
    let parallel = args.matches.get_flag("parallel");
    // ... manual extraction for all 12+ fields
})
```

**Impact**: 
- Lose type safety
- Manual argument extraction required
- More verbose and error-prone
- No compile-time validation of flags

### 3. **Global Arguments**

**Current**: Global args (verbose, format, config) are on `Cli` struct:
```rust
pub struct Cli {
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    pub config: Option<PathBuf>,
    pub format: OutputFormat,
    pub command: Commands,
}
```

**clap-noun-verb**: Need to handle via:
- `CliBuilder::global_args()` - but requires manual handling in verb handlers
- Pass through `VerbContext` data

**Impact**: Verbose argument passing throughout

### 4. **Complex Commands with Many Flags**

**Example**: `run` command has 12+ flags:
- paths, parallel, jobs, fail_fast, watch, force, shard, digest, report_junit, validate, otel_exporter, otel_endpoint

**Challenge**: Converting to `clap-noun-verb` would require:
1. Defining all args manually in `additional_args()`
2. Extracting all args manually from `VerbArgs.matches`
3. Losing type safety and validation

### 5. **Backward Compatibility**

**Current CLI**: 
- `clnrm run tests/`
- `clnrm validate file.toml`
- `clnrm services status`

**If Migrated**: Could potentially maintain same interface, but:
- Implementation would be fundamentally different
- All tests would need updates
- Documentation needs updates
- Risk of breaking existing scripts

## Migration Options

### Option 1: **Full Migration to clap-noun-verb** ⚠️

**Approach**: Convert all commands to noun-verb pattern

**Pros**:
- Consistent architecture
- Better composability
- Framework provides structure

**Cons**:
- **Major refactoring** (~30+ command handlers)
- Loss of type safety for complex commands
- Verbose argument extraction code
- Many commands don't fit noun-verb naturally
- **High risk** of introducing bugs
- **High effort** (estimated 2-3 weeks)

**Estimated Effort**: 
- Code changes: ~2,000-3,000 lines
- Test updates: ~500-800 lines
- Documentation: ~200-300 lines
- **Total: 3-4 weeks**

### Option 2: **Hybrid Migration** ✅ (Recommended)

**Approach**: Use clap-noun-verb only for commands that naturally fit (services, collector), keep standard clap for others

**Implementation**:
```rust
pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse(); // Standard clap for top level
    
    match cli.command {
        // Use noun-verb for appropriate commands
        Commands::Services { .. } | Commands::Collector { .. } => {
            run_noun_verb_cli().await?;
        }
        // Keep standard clap for others
        Commands::Run { .. } => { /* current implementation */ }
        // ... rest of commands
    }
}
```

**Pros**:
- **Minimal changes** - only migrate 2-4 commands
- **Low risk** - most code unchanged
- **Best fit** - only use noun-verb where it makes sense
- Maintains type safety for complex commands
- **Fast implementation** (1-2 days)

**Cons**:
- Inconsistent architecture (two patterns)
- Doesn't fully leverage clap-noun-verb framework

**Estimated Effort**: 
- Code changes: ~200-300 lines
- Test updates: ~100-150 lines
- Documentation: ~50-100 lines
- **Total: 2-3 days**

### Option 3: **Improve Existing Noun-Verb Implementation** ✅✅ (Best)

**Approach**: Fix the existing `services_noun_verb` and `collector_noun_verb` to work properly, then integrate into main CLI

**Current Issues**:
1. Noun-verb handlers don't extract arguments properly (hardcoded values)
2. `run_noun_verb_cli()` exists but is never called
3. Need to wire up argument extraction from `VerbArgs.matches`

**Fixes Needed**:

```rust
// In services_noun_verb.rs - Fix argument extraction
verb!("logs", "Show logs for a service", |args: &VerbArgs| {
    let service = args.matches.get_one::<String>("service")
        .ok_or_else(|| NounVerbError::missing_argument("service"))?;
    let lines: usize = args.matches.get_one::<String>("lines")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            show_service_logs(service, lines).await
                .map_err(|e| NounVerbError::ExecutionError { message: e.to_string() })
        })
    })
})
```

**Then integrate into main CLI**:
```rust
pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Services { .. } | Commands::Collector { .. } => {
            // Route to noun-verb implementation
            return run_noun_verb_cli().await;
        }
        // ... rest of commands stay as-is
    }
    
    // Current implementation for other commands
    // ...
}
```

**Pros**:
- **Minimal risk** - only fixes existing code
- **Low effort** - ~100-200 lines of changes
- Makes noun-verb actually work
- Can be done incrementally
- Maintains backward compatibility

**Estimated Effort**: 
- Code changes: ~150-250 lines
- Test updates: ~50-100 lines
- **Total: 1-2 days**

## Recommended Path Forward

### Phase 1: Fix Existing Noun-Verb Implementation (1-2 days)
1. Fix argument extraction in `services_noun_verb.rs`
2. Fix argument extraction in `collector_noun_verb.rs`
3. Wire `run_noun_verb_cli()` into main CLI flow
4. Add proper argument definitions to verbs
5. Test services/collector commands work end-to-end

### Phase 2: Evaluate Full Migration (Future)
1. Monitor usage of noun-verb commands
2. If successful, consider migrating more commands
3. Consider creating helper macros for argument extraction

## Technical Details

### Argument Extraction Helper

Would need to create helper functions:

```rust
// Helper to extract typed arguments
fn get_arg<T: FromStr>(args: &VerbArgs, name: &str) -> Result<T, NounVerbError> {
    args.matches.get_one::<String>(name)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| NounVerbError::missing_argument(name))
}

// Helper to extract optional arguments
fn get_opt_arg<T: FromStr>(args: &VerbArgs, name: &str) -> Option<T> {
    args.matches.get_one::<String>(name)
        .and_then(|s| s.parse().ok())
}

// Helper to extract Vec arguments
fn get_many_args<T: FromStr>(args: &VerbArgs, name: &str) -> Result<Vec<T>, NounVerbError> {
    args.matches.get_many::<String>(name)
        .map(|iter| iter.map(|s| s.parse().ok()).collect::<Option<Vec<_>>>())
        .flatten()
        .ok_or_else(|| NounVerbError::missing_argument(name))
}
```

### Verb Argument Definition

Need to update verb macro to support arguments:

```rust
verb!("logs", "Show logs for a service", |args: &VerbArgs| {
    // Extract arguments
    let service = get_arg::<String>(args, "service")?;
    let lines = get_opt_arg::<usize>(args, "lines").unwrap_or(50);
    
    // Execute
    show_service_logs(&service, lines).await?;
    Ok(())
})
.with_args(vec![
    Arg::new("service").required(true),
    Arg::new("lines").short('n').long("lines").default_value("50"),
])
```

## Conclusion

**Recommended**: **Option 3 - Fix Existing Noun-Verb Implementation**

This provides:
- ✅ **Lowest risk** (only fixes existing code)
- ✅ **Fastest implementation** (1-2 days)
- ✅ **Immediate value** (makes noun-verb actually work)
- ✅ **Foundation for future** (can migrate more commands later if desired)

**Full migration (Option 1) is not recommended** because:
- ❌ Too many commands don't fit noun-verb pattern naturally
- ❌ Loss of type safety for complex commands
- ❌ High risk and effort for minimal benefit
- ❌ Would require major architectural changes

