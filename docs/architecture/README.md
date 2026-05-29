# clnrm Architecture Documentation

**Index of architecture guides and technical decisions**

---

## Recent Studies & Strategic Initiatives

### ggen Marketplace Integration (Dec 2025)

**Status**: Study Phase Complete ✅ | Marketplace Explored ✅ | Next: Prototyping

An investigation into using ggen (ontology-driven code generation) to refactor the clnrm CLI from 26 hand-coded command files into a single RDF ontology + Tera templates system.

**Why This Matters**:
- Current CLI: Hybrid (2 noun-verb commands, 24 legacy clap)
- Proposed: Ontology-driven (1 RDF file + 4 templates)
- Result: Maintenance burden drops 60%, documentation auto-syncs, reproducible builds
- **ggen 4.0.0**: Production-ready with full marketplace system (not vaporware!)

**Key Documents** (recommended reading order):

1. **[GGEN_QUICK_START.md](./GGEN_QUICK_START.md)** ⭐ START HERE
   - 30-minute primer on ggen fundamentals
   - Core concepts: RDF, SPARQL, Tera templates
   - Working example (copy-paste ready)
   - Decision checklist
   - **Read time**: 30 minutes
   - **Best for**: Quick understanding, seeing if approach is viable

2. **[GGEN_MARKETPLACE_EXPLORATION.md](./GGEN_MARKETPLACE_EXPLORATION.md)** 🔍 PRACTICAL GUIDE
   - Actual ggen 4.0.0 CLI commands (tested)
   - Real marketplace system implementation
   - Generation pipeline (RDF → SPARQL → Tera → code)
   - Publishing and discovery workflow
   - Success criteria for prototyping
   - Implementation path for clnrm
   - **Read time**: 45-60 minutes
   - **Best for**: Understanding actual capabilities, planning implementation

3. **[GGEN_MARKETPLACE_APPROACH.md](./GGEN_MARKETPLACE_APPROACH.md)** 📋 DETAILED STRATEGY
   - Full technical strategy for clnrm CLI refactor
   - 4-week implementation plan (phase breakdown)
   - Ontology design patterns
   - SPARQL query examples
   - Tera template structures
   - Integration into build system
   - Marketplace publishing process
   - Risk mitigation
   - Cost-benefit analysis
   - **Read time**: 1-2 hours
   - **Best for**: Team members implementing the approach

**Task List**:
- See `TodoWrite` task list (10 actionable items)
- Organized from study phase → marketplace publication

**Decision Point**: End of prototyping phase (1-2 weeks)
- ✅ Proceed with 4-week implementation plan
- ⚠️  Continue prototyping for more confidence
- ❌ Abandon approach, maintain CLI manually

---

## Architecture Decision Records (ADRs)

*To be added as major decisions are made*

### Planned ADRs
- [ ] ADR-001: CLI architecture (hand-coded vs ontology-driven)
- [ ] ADR-002: Version management strategy
- [ ] ADR-003: Constitutional principles governance

---

## System Components

### clnrm-cli (Command-Line Interface)

**Current State**:
- 26 commands across 5 categories
- Partial noun-verb refactor (services, collector)
- 24 commands using legacy clap

**Architecture**: See [GGEN_QUICK_START.md](./GGEN_QUICK_START.md) for proposed refactor

**Key Files**:
- `crates/clnrm-cli/src/lib.rs` - Entry point
- `crates/clnrm-cli/src/cmds/` - Command implementations
- `crates/clnrm-cli/Cargo.toml` - Dependencies (clap-noun-verb v5.3.2)

### clnrm-core (Core Engine)

**Responsibility**: RDF/SPARQL engine, test execution, observability

**Key Files**:
- `crates/clnrm-core/src/` - Core implementation
- `crates/clnrm-core/src/otel/` - OpenTelemetry instrumentation

### clnrm-shared (Shared Types)

**Responsibility**: Common error types, traits, constants

---

## Constitutional Principles

**Source**: [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md) (v1.0.0)

### 5 Core Principles

1. **Cargo Make Rule** - All build operations via `cargo make`, never direct cargo
2. **Error Handling Rule** - `Result<T, E>` in production, `unwrap()` only in tests
3. **Chicago TDD Rule** - State-based testing with real collaborators, no mocks
4. **Andon Signal Rule** - RED/YELLOW/GREEN discipline for quality gates
5. **Concurrent Execution Rule** - Batch operations in single messages for atomicity

---

## Documentation Navigation

### For Different Audiences

**New Contributors**:
1. Read this README (5 min)
2. Read [GGEN_QUICK_START.md](./GGEN_QUICK_START.md) (30 min)
3. Skim [GGEN_MARKETPLACE_APPROACH.md](./GGEN_MARKETPLACE_APPROACH.md) (20 min)
4. See [../BUILD_SYSTEM_SUMMARY.md](../BUILD_SYSTEM_SUMMARY.md) for build details

**Team Leads**:
1. Read [GGEN_MARKETPLACE_APPROACH.md](./GGEN_MARKETPLACE_APPROACH.md) (1 hour)
2. Review cost-benefit section
3. Make decision on prototyping timeline

**CLI Developers**:
1. Read [GGEN_QUICK_START.md](./GGEN_QUICK_START.md)
2. Follow task list in TodoWrite
3. Execute 4-week plan if approved

---

## Key Metrics

| Metric | Current | Proposed (ggen) | Improvement |
|--------|---------|-----------------|-------------|
| Files to maintain | 26+ | 1 (ontology) | -96% |
| Code templates | None | 4 | N/A |
| Time to add command | 1-2 hours | 5 minutes | 12-24x |
| Documentation sync | Manual | Auto | ∞ |
| Type safety | Runtime | Compile-time | Better |
| Reproducibility | Manual | Guaranteed | Better |

---

## Timeline

### Study Phase (Week 1)
- ✅ Complete (Dec 13, 2025)
- Deliverables: 2 architecture documents, 10 task items

### Prototyping Phase (Week 2)
- ⏳ Ready to start
- Validation of ggen approach on sample commands
- Decision gate: Proceed or abandon

### Implementation Phase (Weeks 3-6)
- 📋 Planned (pending prototype approval)
- Week 1: Ontology design + SPARQL
- Week 2: Templates + validation
- Week 3: Full generation + integration
- Week 4: Marketplace + docs

### Publication Phase (Week 7)
- 🎯 Target for marketplace release

---

## Related Resources

### External
- **ggen Repository**: https://github.com/seanchatmangpt/ggen
- **ggen Documentation**: https://ggen.io/docs
- **RDF Concepts**: https://www.w3.org/TR/rdf-concepts/
- **SPARQL Tutorial**: https://www.w3.org/TR/sparql11-query/

### Internal
- **Constitution**: [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md)
- **Build System**: [`../BUILD_SYSTEM_SUMMARY.md`](../BUILD_SYSTEM_SUMMARY.md)
- **Makefile.toml**: [`../../Makefile.toml`](../../Makefile.toml)

---

## Contact & Questions

**For ggen marketplace questions**: See architecture documents or open GitHub issue
**For CLI architecture decisions**: See constitutional principles
**For build system questions**: See `BUILD_SYSTEM_SUMMARY.md`

---

**Last Updated**: 2025-12-13
**Owner**: clnrm maintainers
**Status**: Active (study phase complete, prototyping pending)
