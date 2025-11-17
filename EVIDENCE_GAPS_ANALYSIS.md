#  Evidence Gap Analysis & Closure Strategy

## Identified Gaps

1. **C_CODE_AS_PROJECTION** - Code generation as projection from ontology
2. **C_MU_KERNEL_PHYSICS** - Timing kernel with formal bounds
3. **C_KNHK_GRAPH_PRIMARY** - Knowledge graph as primary source of truth
4. **C_DFLSS_FLOW** - Design for Lean Six Sigma optimization flow

## Root Cause Analysis

The concept matching rules were too restrictive or looking for terms not present in the actual codebase. Evidence exists but in different formulations:

### C_CODE_AS_PROJECTION

**Where it exists:**
- `crates/clnrm-core/src/telemetry/weaver_controller.rs` - Code generation from Weaver schemas
- `docs/weaver/*.md` - Weaver as code generation engine
- `crates/clnrm-template/` - Tera templating for code projection

**Why it missed:**
- Rules looked for "projection" OR "generated" in very specific contexts
- Weaver docs use "code generation" and "generated telemetry code" but pipeline didn't tokenize these properly

**Fix:**
- Add tokens: "code generation", "generated telemetry", "Weaver", "builder", "schema-driven"

### C_MU_KERNEL_PHYSICS

**Where it exists:**
- `crates/clnrm-core/src/timing/validator.rs` - μ-kernel timing receipts and validation
- `crates/clnrm-core/src/backend/engine.rs` - BackendType::MuKernel variant
- Documentation of timing bounds and guarantees

**Why it missed:**
- Token "patterns" is too generic and interferes with matching
- μ-kernel is referenced but in different contexts (comments, type names)
- ISA not explicitly mentioned, only implied through timing patterns

**Fix:**
- Add tokens: "timing validator", "timing receipt", "τ bounds", "latency band"
- Make must_include rules more specific

### C_KNHK_GRAPH_PRIMARY

**Where it exists:**
- `docs/archive/analysis/KGOLD_REPOSITORY_ANALYSIS.md` - Knowledge graph ontology implementation
- `registry/` - YAML schemas as ontology/knowledge representation
- Cross-references in ggen documentation

**Why it missed:**
- KNHK not explicitly named in clnrm codebase (external reference)
- "ontology" appears in contexts like "OWL ontology" but matching was too specific
- Knowledge graph terminology exists but scattered across docs

**Fix:**
- Add tokens: "OWL", "SHACL", "SPARQL", "semantic convention", "ontology validation"
- Add boost for "knowledge" + "graph" proximity

### C_DFLSS_FLOW

**Where it exists:**
- `docs/archive/analysis/KGOLD_REPOSITORY_ANALYSIS.md` - mentions "DFSS-R (Design for Lean Six Sigma Rust)"
- Optimization flows in phase implementations
- Continuous learning pipelines

**Why it missed:**
- DFLSS not explicitly spelled out in clnrm
- Documentation uses "DFSS-R" (variant with -R suffix) which was explicitly excluded
- Design patterns for optimization are implicit, not explicit

**Fix:**
- Allow "DFSS" with context (not just exclude it)
- Add tokens: "DFSS-R", "lean six sigma", "optimization", "continuous improvement"

## Gap Closure Plan

1. Enhance concept matching rules with more granular tokens
2. Create targeted documentation files that explicitly name missing concepts
3. Re-run pipeline with improved rules
4. Validate that all 13 expected concepts are found
