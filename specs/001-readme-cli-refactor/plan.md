# Implementation Plan: Complete README v2.1.0 and Partial CLI Refactor Migration

**Branch**: `001-readme-cli-refactor` | **Date**: 2025-12-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/Users/sac/clnrm/specs/001-readme-cli-refactor/spec.md`

## Summary

This feature updates README.md to reflect version 2.1.0 with accurate CLI command documentation and constitutional principles, while documenting the current partial clap-noun-verb migration status (2/26 commands migrated). The README will adopt a Hub-and-Spoke pattern with feature-driven command categorization (5 categories) and a 5-minute quick-start guide, establishing it as the primary entry point for new users and contributors.

**Technical approach**: Update README.md using Hub-and-Spoke documentation pattern with auto-populated version badges from crates.io. Document all 26 CLI commands across 5 functional categories (Test Execution, Configuration, Observation, System Management, Development). Reference constitutional principles with links to constitution.md. Create migration status indicators showing current Hybrid architecture mode (2 noun-verb, 24 legacy clap). No code changes required - this is a documentation-only feature.

**Migration context**: This feature documents the current partial migration state. Full CLI refactor (migrating remaining 24 commands to clap-noun-verb) is tracked as separate feature (002-complete-cli-migration).

## Technical Context

**Language/Version**: Rust 1.75+ (workspace edition 2021, as defined in Cargo.toml)
**Primary Dependencies**:
- clap v4.5.49 (derive features) for legacy 24 commands
- clap-noun-verb v5.3.2 (inferred from Cargo.lock) for 2 migrated commands
- linkme v0.3.35 for distributed slice registration
- OpenTelemetry 0.31.0 suite (trace, metrics, logs) for all CLI instrumentation

**Storage**: N/A (documentation-only feature)
**Testing**: README validation scripts, CLI help text integration tests, version consistency tests
**Target Platform**:
- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)
- Docker Desktop required (for container testing functionality)

**Project Type**: Single Rust workspace with 6 crates (clnrm, clnrm-cli, clnrm-core, clnrm-shared, clnrm-template, evidence-graph)
**Performance Goals**:
- Command discovery: < 1 minute (down from 10+ minutes)
- First success: < 5 minutes (new quick-start guide)
- Help text generation: < 5 seconds (`clnrm --help`)

**Constraints**:
- No hardcoded versions in README text (use crates.io badges only)
- Maintain backward compatibility (document Hybrid architecture mode)
- Constitutional compliance (all 5 principles must be referenced)
- Hub-and-Spoke pattern (5-10KB main README, link to detailed docs)

**Scale/Scope**:
- 26 CLI commands to document
- 5 functional categories to organize commands
- 5 constitutional principles to reference
- 1 quick-start guide (<5 minutes to first success)
- 2 migrated commands (services, collector) vs 24 legacy commands to distinguish

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Gate 1: Cargo Make Rule (ABSOLUTE)
**Status**: ✅ PASS (N/A - documentation only)

**Rationale**: This feature is documentation-only (README.md update). No build, test, or validation operations required. All validation scripts (e.g., validate_readme.sh) are BASH scripts, not cargo commands.

### Gate 2: Error Handling Rule (PRODUCTION CODE)
**Status**: ✅ PASS (N/A - documentation only)

**Rationale**: No production code changes. Documentation examples demonstrate proper `Result<T, CleanroomError>` patterns for illustrative purposes only.

### Gate 3: Chicago TDD Rule (Arrange-Act-Assert)
**Status**: ✅ PASS

**Verification**:
- README validation script tests (AAA pattern):
  - Arrange: Create README with test content
  - Act: Run validation script
  - Assert: Check exit code, verify required sections present

**Example test** (from contracts/README_CONTRACT.md):
```bash
#[test]
fn test_readme_version_accuracy() {
    // Arrange: README with version references
    let readme_content = std::fs::read_to_string("README.md").unwrap();

    // Act: Extract version numbers
    let cargo_version = env!("CARGO_PKG_VERSION");
    let badge_present = readme_content.contains("img.shields.io/crates/v/clnrm");

    // Assert: Version badge exists, no hardcoded versions
    assert!(badge_present, "README must have crates.io version badge");
    assert!(!readme_content.contains("v2.1.0") || badge_present);
}
```

### Gate 4: Andon Signal Rule (Stop the Line)
**Status**: ✅ PASS

**Workflow**:
- **RED signal**: Validation script exits non-zero (missing required sections, broken links)
  - **Action**: STOP - Fix README before commit
- **YELLOW signal**: Manual checklist incomplete, outdated examples
  - **Action**: Investigate - Update before PR
- **GREEN signal**: All validation checks pass
  - **Action**: Continue - Safe to merge

**Validation command**:
```bash
bash specs/001-readme-cli-refactor/contracts/validate_readme.sh
# Exit 0 = GREEN, Exit 1 = RED
```

### Gate 5: Concurrent Execution Rule (1 Message = All Operations)
**Status**: ✅ PASS

**Compliance**: All Phase 1 artifacts generated in single message batch:
- research.md (Phase 0 consolidated findings)
- data-model.md (CLI command structure)
- contracts/README_CONTRACT.md (validation rules)
- contracts/CLI_COMMAND_CONTRACT.md (command specifications)
- quickstart.md (5-minute onboarding guide)
- plan.md (this file - implementation plan)

**Execution pattern**: Agents spawned concurrently for research, then consolidated results written atomically.

### Summary: All Gates PASSED ✅

**Post-Phase 1 Re-check**: No design changes required (documentation-only feature). All gates remain PASS.

## Project Structure

### Documentation (this feature)

```text
specs/001-readme-cli-refactor/
├── spec.md                   # Feature specification (user stories, requirements)
├── plan.md                   # This file (/speckit.plan command output)
├── research.md               # Phase 0 output (clap-noun-verb + README research)
├── data-model.md             # Phase 1 output (CLI command entity model)
├── quickstart.md             # Phase 1 output (5-minute onboarding guide)
├── contracts/                # Phase 1 output (validation contracts)
│   ├── README_CONTRACT.md    # README structure and content rules
│   └── CLI_COMMAND_CONTRACT.md  # Command behavior specifications
└── tasks.md                  # Phase 2 output (/speckit.tasks - NOT YET CREATED)
```

### Source Code (repository root)

**Note**: This feature is **documentation-only**. No source code changes required. Future feature (002-complete-cli-migration) will modify source files.

```text
clnrm/                        # Repository root
├── README.md                 # PRIMARY DELIVERABLE (updated to v2.1.0)
├── Cargo.toml                # Workspace manifest (version 2.1.0 - single source of truth)
├── CLAUDE.md                 # Development guidance (references constitution)
├── .specify/memory/
│   └── constitution.md       # Constitution v1.0.0 (ratified 2025-12-13)
│
├── crates/                   # 6-crate workspace
│   ├── clnrm/                # Main library
│   ├── clnrm-cli/            # CLI interface (26 commands)
│   │   ├── src/
│   │   │   ├── lib.rs        # Hybrid mode: tries noun-verb, falls back to legacy
│   │   │   ├── commands.rs   # Implementation functions (28KB, 100+ lines)
│   │   │   └── cmds/         # 26 command modules
│   │   │       ├── services.rs      # ✅ NounVerb (7 verbs)
│   │   │       ├── collector.rs     # ✅ NounVerb (5 verbs)
│   │   │       ├── run.rs           # 🔄 Legacy
│   │   │       ├── dry_run.rs       # 🔄 Legacy
│   │   │       └── ... (22 more legacy)
│   │   └── Cargo.toml
│   ├── clnrm-core/           # Core engine (42 modules)
│   ├── clnrm-shared/         # Shared types + CleanroomError
│   ├── clnrm-template/       # Code generation (experimental)
│   └── evidence-graph/       # Evidence tracing
│
├── tests/                    # Workspace integration tests
│   ├── telemetry_validation/ # OTEL span validation
│   └── exit_codes/           # Exit code behavior tests
│
├── docs/                     # Documentation (Hub-and-Spoke links)
│   ├── CLAP_NOUN_VERB_RESEARCH.md  # 35KB migration research (from agent 1)
│   ├── README_RESEARCH_INDEX.md    # Navigation hub (from agent 2)
│   ├── README_BEST_PRACTICES_RESEARCH.md  # 30-min analysis
│   ├── COMMAND_CATEGORIZATION_REFERENCE.md  # Command mapping
│   └── CODE_STANDARDS.md     # Detailed standards (linked from README)
│
└── Makefile.toml             # Cargo make task configuration
```

**Structure Decision**:

Single-project Rust workspace (Option 1 from template) with 6 crates. This is an **existing structure** - no changes required for this feature.

**Rationale**:
- **Documentation target**: README.md at repository root
- **Version source**: Cargo.toml `[workspace.package] version = "2.1.0"`
- **Command source**: 26 command modules in `crates/clnrm-cli/src/cmds/`
- **Constitution source**: `.specify/memory/constitution.md`
- **No structural changes**: This feature only updates documentation files

**CLI Architecture Status**:
- **Hybrid mode** (current): 2 noun-verb (services, collector) + 24 legacy (run, dry-run, etc.)
- **Target mode** (future): 26 noun-verb (separate feature: 002-complete-cli-migration)

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**N/A** - No constitutional violations. All gates passed.

## Implementation Details

### Phase 0: Research & Analysis (COMPLETED ✅)

**Deliverable**: `research.md` with resolved technical unknowns

**Research findings**:
1. **Hub-and-Spoke README pattern** confirmed as best practice (used by Cargo, Rustup, Ripgrep)
2. **clap-noun-verb v5.3.2** verified in Cargo.lock, migration viable for future feature
3. **Feature-driven categorization** (5 categories) proven more discoverable than alphabetical
4. **Version automation** via crates.io badges eliminates manual sync burden
5. **Constitutional principles** must be visible in main README (not buried in separate doc)

**Agents used**:
- Agent 1: clap-noun-verb research (35KB, 1,187 lines) - `/Users/sac/clnrm/docs/CLAP_NOUN_VERB_RESEARCH.md`
- Agent 2: README best practices (5 documents, 30+ pages) - `/Users/sac/clnrm/docs/README_*`

### Phase 1: Design & Contracts (COMPLETED ✅)

**Deliverables**:
- ✅ `data-model.md` - CLI command entity model (26 commands, 5 categories, architecture patterns)
- ✅ `contracts/README_CONTRACT.md` - README structure validation rules
- ✅ `contracts/CLI_COMMAND_CONTRACT.md` - Command behavior specifications
- ✅ `quickstart.md` - 5-minute onboarding guide

**Entity model**:
- ClnrmCLI (root entity): version, 26 commands, 5 categories, architecture mode
- Command: name, category, architecture (Legacy/NounVerb), description, verbs, env vars
- Category: type, display_name, commands, priority (1-5)
- Verb: name, description, args, output_format (JSON/Msgpack)
- ConstitutionalPrinciple: 5 governance rules

**Contracts defined**:
- README must have 6 required sections (header, constitutional principles, quick start, command reference, code standards, troubleshooting)
- All 26 commands must be documented with one-line descriptions
- Version badges auto-populate from crates.io (no hardcoded versions)
- Validation script enforces structure (exit 0 = GREEN, exit 1 = RED)

### Phase 2: Task Breakdown (NOT YET RUN - use `/speckit.tasks`)

**Next command**: `/speckit.tasks` to generate `tasks.md`

**Expected task categories**:
1. **Setup**: N/A (no build setup needed)
2. **Tests**: Create README validation script, CLI help text tests
3. **Core**: Update README.md with 6 required sections
4. **Integration**: Link README to constitution.md, docs/, CLAUDE.md
5. **Polish**: Manual review checklist, version badge verification

### Phase 3: Implementation (NOT YET RUN - use `/speckit.implement`)

**Next command**: `/speckit.implement` to execute tasks

**Expected changes**:
- README.md: Complete rewrite following Hub-and-Spoke pattern
- specs/001-readme-cli-refactor/contracts/validate_readme.sh: Validation script
- tests/cli/help_text_test.rs: Integration test for command discovery

## Success Metrics

### Measurable Outcomes (from spec.md)

**SC-001**: README version accuracy
- **Metric**: `grep -c "img.shields.io/crates/v/clnrm" README.md` >= 1 AND no hardcoded "2.1.0"
- **Current**: Unknown (README not yet updated)
- **Target**: 100% version consistency

**SC-002**: Command discoverability
- **Metric**: Time for new user to find relevant command via `clnrm --help` or README
- **Current**: 10+ minutes (no categorization, alphabetical noise)
- **Target**: <1 minute (feature-driven categories)

**SC-003**: Quick-start completion
- **Metric**: Time to first passing test following quickstart.md
- **Current**: Unknown (no quick-start exists)
- **Target**: <5 minutes

**SC-004**: Command documentation completeness
- **Metric**: All 26 commands in README with one-line description
- **Current**: 0/26 (README has minimal command list)
- **Target**: 26/26 (100% coverage across 5 categories)

**SC-005**: Help text self-documentation
- **Metric**: User survey - "Did you need README for basic usage?"
- **Current**: Unknown
- **Target**: 90% "No, --help was sufficient"

**SC-006**: Constitutional principles visibility
- **Metric**: All 5 principles in README with links to constitution.md
- **Current**: 0/5 (principles exist in constitution.md but not README)
- **Target**: 5/5 (100% presence)

**SC-007**: Troubleshooting effectiveness
- **Metric**: Support questions reduced
- **Current**: Baseline (unknown)
- **Target**: 50% reduction post-release

**SC-008**: Migration transparency
- **Metric**: README documents noun-verb status (2/26 complete)
- **Current**: No documentation of partial migration
- **Target**: Clear migration status with ✅ NounVerb / 🔄 Legacy indicators

### Validation Tests

**Automated validation**:
```bash
# Run validation script (from contracts/)
bash specs/001-readme-cli-refactor/contracts/validate_readme.sh

# Expected output:
# ✅ All required sections present
# ✅ No hardcoded versions
# ✅ All 26 commands documented
# ✅ All 5 constitutional principles referenced
# ✅ Constitution.md linked 5+ times
# ✅ All internal links valid
# ✅ README.md validation PASSED
```

**Integration tests**:
```bash
# Help text includes all commands
cargo test --test cli_help_text

# Version consistency
cargo test --test version_consistency

# All commands executable
cargo make test
```

## Risk Assessment

### Low Risks (Documentation-only)

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Outdated examples | Medium | Low | Link to live code, not copy-paste |
| Broken links | Medium | Low | Validation script checks all links |
| Version drift | Low | Medium | Use auto-populated badges only |

### Assumptions Validation

From spec.md assumptions:

1. ✅ **Partial Refactor State**: Confirmed 2/26 migrated (services, collector use clap-noun-verb)
2. ✅ **Full Migration Goal**: Documented as separate feature (002-complete-cli-migration)
3. ✅ **Version Single Source of Truth**: Cargo.toml workspace version verified as authoritative
4. ✅ **26 Commands Total**: Verified via `ls crates/clnrm-cli/src/cmds/ | wc -l` = 26
5. ✅ **Constitutional Authority**: Constitution v1.0.0 ratified 2025-12-13, all 5 principles documented

## Timeline Estimate

**Phase 1 (Design)**: ✅ COMPLETE (2 hours - research agents + contract generation)

**Phase 2 (Task Breakdown)**: NOT YET RUN (~30 minutes - run `/speckit.tasks`)

**Phase 3 (Implementation)**: NOT YET RUN (~4 hours estimated):
- Week 1 (HIGH priority): 3.5 hours
  - Header & badges: 10 min
  - Quick Start: 30 min
  - Constitutional Principles: 45 min
  - Common Workflows: 30 min
  - Command Reference: 20 min
  - Troubleshooting: 60 min
- Week 2 (MEDIUM priority): 2.5 hours
  - Detailed reference links
  - Code standards section
  - Project structure diagram
- Week 3 (LOW priority): 5 min
  - Version automation verification
  - Link validation

**Total estimate**: 6.5 hours (not including review cycles)

## Next Actions

### Immediate (Ready to Execute)

1. **Run `/speckit.tasks`** to generate task breakdown
   - Input: This plan.md + research.md + data-model.md + contracts/
   - Output: tasks.md with dependency-ordered implementation tasks

2. **Run `/speckit.implement`** to execute task plan
   - Input: tasks.md
   - Output: Updated README.md + validation scripts + tests

### After Implementation

1. **Validate README**:
   ```bash
   bash specs/001-readme-cli-refactor/contracts/validate_readme.sh
   ```

2. **Test CLI help text**:
   ```bash
   clnrm --help | grep -c "Commands:"
   # Should output 26
   ```

3. **Verify version consistency**:
   ```bash
   clnrm --version  # Should show: clnrm 2.1.0
   grep "2.1.0" README.md  # Should ONLY appear in badge URLs
   ```

4. **Manual review checklist**:
   - [ ] Quick Start completes in < 5 minutes
   - [ ] All code examples are copy-pasteable
   - [ ] Command categories make sense
   - [ ] Troubleshooting covers common issues
   - [ ] Links to constitution.md work

### Future Work (Separate Specs)

**002-complete-cli-migration**: Migrate remaining 24 commands to clap-noun-verb
- Input: research.md (clap-noun-verb patterns from this spec)
- Scope: 24 legacy commands → noun-verb architecture
- Timeline: 4-month phased migration (7 commands/month)

## Artifacts Summary

### Generated by This Plan

| Artifact | Status | Location | Size |
|----------|--------|----------|------|
| research.md | ✅ Complete | specs/001-readme-cli-refactor/ | 20KB |
| data-model.md | ✅ Complete | specs/001-readme-cli-refactor/ | 15KB |
| quickstart.md | ✅ Complete | specs/001-readme-cli-refactor/ | 8KB |
| README_CONTRACT.md | ✅ Complete | specs/001-readme-cli-refactor/contracts/ | 10KB |
| CLI_COMMAND_CONTRACT.md | ✅ Complete | specs/001-readme-cli-refactor/contracts/ | 12KB |
| plan.md | ✅ Complete | specs/001-readme-cli-refactor/ | 18KB (this file) |

### To Be Generated

| Artifact | Command | Expected Output |
|----------|---------|-----------------|
| tasks.md | `/speckit.tasks` | Dependency-ordered task list |
| README.md (updated) | `/speckit.implement` | Hub-and-Spoke documentation |
| validate_readme.sh | `/speckit.implement` | Validation script |

---

**Plan Status**: ✅ COMPLETE - Ready for `/speckit.tasks`

**Branch**: `001-readme-cli-refactor`
**Feature Directory**: `/Users/sac/clnrm/specs/001-readme-cli-refactor/`
**Next Command**: `/speckit.tasks`
