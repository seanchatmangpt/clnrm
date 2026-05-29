# Tasks: Complete README v2.1.0 and Partial CLI Refactor Migration

**Input**: Design documents from `/Users/sac/clnrm/specs/001-readme-cli-refactor/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: This feature is documentation-only. Tests validate README structure, not code functionality.

**Organization**: Tasks are grouped by user story (P1, P2) to enable independent implementation and testing of each documentation increment.

## Format: `- [ ] [ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- **File paths**: All paths are absolute or relative to repository root

## Path Conventions

**Documentation-only feature** - No source code changes:
- **README.md** - Primary deliverable (repository root)
- **specs/001-readme-cli-refactor/** - Feature specification and contracts
- **docs/** - Supporting documentation (Hub-and-Spoke pattern)
- **.specify/memory/constitution.md** - Constitutional reference

---

## Phase 1: Setup (Validation Infrastructure)

**Purpose**: Create validation scripts and test infrastructure for README verification

**Estimated time**: 30 minutes

- [X] T001 Create validation script directory at specs/001-readme-cli-refactor/scripts/
- [X] T002 [P] Create README validation script at specs/001-readme-cli-refactor/scripts/validate_readme.sh based on contracts/README_CONTRACT.md
- [X] T003 [P] Create version consistency test script at specs/001-readme-cli-refactor/scripts/check_version_consistency.sh
- [X] T004 Make validation scripts executable with chmod +x

**Checkpoint**: Validation infrastructure ready for README verification

---

## Phase 2: Foundational (Research Reference Documentation)

**Purpose**: Ensure all research documentation is accessible for reference during README writing

**⚠️ CRITICAL**: No user story work can begin until research artifacts are confirmed available

**Estimated time**: 5 minutes

- [X] T005 Verify research documentation exists at docs/CLAP_NOUN_VERB_RESEARCH.md (35KB, from planning phase)
- [X] T006 [P] Verify README research artifacts exist: docs/README_RESEARCH_INDEX.md, docs/README_BEST_PRACTICES_RESEARCH.md, docs/COMMAND_CATEGORIZATION_REFERENCE.md
- [X] T007 [P] Verify constitution.md is accessible at .specify/memory/constitution.md (v1.0.0)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Update README to v2.1.0 (Priority: P1) 🎯 MVP

**Goal**: README displays version 2.1.0 accurately with auto-populated badges, no hardcoded versions

**Independent Test**: Run `grep "2.1.0" README.md` (should ONLY appear in badge URLs), verify `clnrm --version` matches README version badge

**Estimated time**: 45 minutes

### Implementation for User Story 1

- [X] T008 [US1] Update README header section: Add crates.io version badge `![Version](https://img.shields.io/crates/v/clnrm.svg)` in README.md
- [X] T009 [US1] Remove all hardcoded version references (search for "2.1.0", "v2.1", etc.) and replace with badge references in README.md
- [X] T010 [US1] Update "Tech Stack" section with Rust 1.75+, Cargo workspace structure in README.md
- [X] T011 [US1] Add build badge `![Build](https://img.shields.io/github/actions/workflow/status/seanchatmangpt/clnrm/ci.yml)` if CI workflow exists in README.md
- [X] T012 [US1] Verify Cargo.toml workspace version is 2.1.0 (single source of truth) - no changes needed, just verification

**Checkpoint**: User Story 1 complete - README version accuracy can be independently tested with validation script

**Validation**:
```bash
bash specs/001-readme-cli-refactor/scripts/check_version_consistency.sh
# Expected: ✅ Version badge present, no hardcoded versions
```

---

## Phase 4: User Story 2 - Document Current CLI Commands (Priority: P1) 🎯 MVP Extension

**Goal**: All 26 CLI commands discoverable in README with feature-driven categorization (5 categories)

**Independent Test**: Run `clnrm --help` and verify all 26 commands listed; check README has 5 command categories with all commands documented

**Estimated time**: 1 hour 30 minutes

### Implementation for User Story 2

- [X] T013 [P] [US2] Create "Command Reference" section with 5 category subsections (Test Execution, Configuration, Observation, System Management, Development) in README.md
- [X] T014 [P] [US2] Document Test Execution commands (6 commands: run, dry-run, record, repro, stress, self-test) with migration status indicators (✅ NounVerb / 🔄 Legacy) in README.md
- [X] T015 [P] [US2] Document Configuration commands (5 commands: init, validate, lint, fmt, render) with one-line descriptions in README.md
- [X] T016 [P] [US2] Document Observation commands (5 commands: spans, report, graph, health, live-check) with one-line descriptions in README.md
- [X] T017 [P] [US2] Document System Management commands (4 commands: services, collector, plugins, pull) highlighting services and collector as NounVerb (✅) in README.md
- [X] T018 [P] [US2] Document Development commands (6 commands: dev, template, diff, analyze, redgreen, live-check) with one-line descriptions in README.md
- [X] T019 [US2] Add usage examples for top 8 commands (run, dry-run, live-check, init, lint, fmt, analyze, report) in README.md
- [X] T020 [US2] Add "How to use --help" subsection explaining `clnrm --help` and `clnrm [command] --help` patterns in README.md
- [X] T021 [US2] Create command reference table showing 26/26 commands with categories in README.md

**Checkpoint**: User Story 2 complete - Command discoverability can be independently tested by new users navigating README

**Validation**:
```bash
# Verify all 26 commands documented
grep -c '`run`\|`dry-run`\|`record`\|`repro`\|`stress`\|`self-test`\|`init`\|`validate`\|`lint`\|`fmt`\|`render`\|`spans`\|`report`\|`graph`\|`health`\|`live-check`\|`services`\|`collector`\|`plugins`\|`pull`\|`dev`\|`template`\|`diff`\|`analyze`\|`redgreen`' README.md
# Expected: >= 26
```

---

## Phase 5: User Story 4 - Provide Clear Quick-Start Guide (Priority: P2)

**Goal**: New users can run their first test in under 5 minutes following README Quick Start section

**Independent Test**: Follow README Quick Start step-by-step with Docker running; test should pass in <5 minutes

**Estimated time**: 30 minutes

### Implementation for User Story 4

- [X] T022 [US4] Create "🚀 Quick Start" section in README.md with step-by-step instructions based on quickstart.md
- [X] T023 [US4] Add prerequisite checklist (Docker running, Rust 1.75+, 5 minutes of time) in Quick Start section of README.md
- [X] T024 [US4] Include example TOML test specification (ubuntu echo test) in Quick Start section of README.md
- [X] T025 [US4] Add expected output example showing successful test execution in Quick Start section of README.md
- [X] T026 [US4] Link to quickstart.md for detailed 5-minute walkthrough from README.md Quick Start section

**Checkpoint**: User Story 4 complete - Quick Start section enables 5-minute first success

**Validation**:
```bash
# Manual test: Follow Quick Start in README
# Expected: Test passes in <5 minutes with Docker running
```

---

## Phase 6: User Story 5 - Document Constitutional Principles (Priority: P2)

**Goal**: README clearly states all 5 constitutional principles with links to constitution.md

**Independent Test**: Verify README "Code Standards" section lists all 5 principles with working links

**Estimated time**: 45 minutes

### Implementation for User Story 5

- [X] T027 [P] [US5] Create "🎯 THE VITAL FEW (20% that matters)" section in README.md with 5 subsections
- [X] T028 [P] [US5] Document Principle I: Cargo Make Rule with code example (✅ CORRECT: cargo make test, ❌ WRONG: cargo test) in README.md
- [X] T029 [P] [US5] Document Principle II: Error Handling Rule with Result<T, CleanroomError> pattern example in README.md
- [X] T030 [P] [US5] Document Principle III: Chicago TDD Rule with Arrange-Act-Assert test example in README.md
- [X] T031 [P] [US5] Document Principle IV: Andon Signal Rule with RED/YELLOW/GREEN table in README.md
- [X] T032 [P] [US5] Document Principle V: Concurrent Execution Rule with single-message batch example in README.md
- [X] T033 [US5] Add links to constitution.md for all 5 principles (minimum 5 links) in README.md
- [X] T034 [US5] Create "⚡ Code Standards (Zero Tolerance)" section summarizing standards with link to constitution.md in README.md

**Checkpoint**: User Story 5 complete - Constitutional principles are discoverable and linked

**Validation**:
```bash
# Verify 5 constitutional principles referenced
grep -c "Cargo Make\|Error Handling\|Chicago TDD\|Andon Signal\|Concurrent Execution" README.md
# Expected: >= 5

# Verify constitution.md linked at least 5 times
grep -c "constitution.md" README.md
# Expected: >= 5
```

---

## Phase 7: User Story 3 - Migrate Commands to Noun-Verb Pattern (Priority: P2) - DOCUMENTATION ONLY

**Goal**: README clearly documents current migration status (2/26 complete) and future migration plan

**Independent Test**: Verify README shows which commands use NounVerb (✅) vs Legacy (🔄) and links to migration research

**Estimated time**: 20 minutes

**Note**: This user story is about DOCUMENTING the migration, not performing it. Actual code migration is tracked in separate feature (002-complete-cli-migration).

### Implementation for User Story 3

- [X] T035 [US3] Add migration status section explaining Hybrid architecture mode (2 NounVerb, 24 Legacy) in README.md
- [X] T036 [US3] Add migration status indicators (✅ NounVerb, 🔄 Legacy) to all 26 commands in command reference tables in README.md
- [X] T037 [US3] Document which commands are migrated: services (7 verbs), collector (5 verbs) in README.md
- [X] T038 [US3] Add "Future Work" section linking to clap-noun-verb research (docs/CLAP_NOUN_VERB_RESEARCH.md) in README.md
- [X] T039 [US3] Explain benefits of noun-verb pattern (environment variables, help text, agent introspection) in README.md

**Checkpoint**: User Story 3 complete - Migration transparency achieved, future work clearly documented

**Validation**:
```bash
# Verify migration status documented
grep -q "Hybrid" README.md && grep -q "2/26" README.md
# Expected: Both present
```

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories, final validation, and Hub-and-Spoke links

**Estimated time**: 1 hour

- [X] T040 [P] Create "📁 Project Structure" section showing 6-crate workspace tree diagram in README.md
- [X] T041 [P] Create "🔧 Troubleshooting" section with common issues (Docker not running, Rust toolchain, file permissions, timeout errors) in README.md
- [X] T042 [P] Add "Definition of Done" section with pre-merge validation commands (cargo make check, test, lint) in README.md
- [X] T043 [P] Add "🚫 Prohibited Patterns" section listing anti-patterns (direct cargo, unwrap in production, skipping tests, etc.) in README.md
- [X] T044 [P] Create "🧪 Self-Testing Principle" section explaining dogfooding (framework tests itself) in README.md
- [X] T045 Add Hub-and-Spoke links: Link to docs/CODE_STANDARDS.md, docs/TEST_SPECIFICATIONS.md, docs/CLAP_NOUN_VERB_RESEARCH.md in README.md
- [X] T046 Add table of contents (optional, if README > 200 lines) in README.md
- [X] T047 Verify all internal links work (no 404s for docs/, constitution.md, etc.) in README.md
- [X] T048 Run validation script: bash specs/001-readme-cli-refactor/scripts/validate_readme.sh
- [X] T049 Run version consistency check: bash specs/001-readme-cli-refactor/scripts/check_version_consistency.sh
- [X] T050 Manual review: Verify Quick Start completes in <5 minutes with Docker running
- [X] T051 Manual review: Verify all code examples are copy-pasteable (no syntax errors)
- [X] T052 Verify README follows Hub-and-Spoke pattern (5-10KB main file, links to detailed docs)

**Final Checkpoint**: All user stories integrated, README production-ready

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion (validation scripts ready)
- **User Stories (Phases 3-7)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (if staffed) or sequentially in priority order
  - **MVP scope**: Phase 3 (US1) + Phase 4 (US2) = Version accuracy + Command discoverability
  - **Extended MVP**: Add Phase 5 (US4) = Quick Start guide
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1) - Version Accuracy**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1) - Command Documentation**: Can start after Foundational (Phase 2) - Independent of US1, but logically follows
- **User Story 4 (P2) - Quick Start**: Can start after Foundational (Phase 2) - Should reference commands from US2
- **User Story 5 (P2) - Constitutional Principles**: Can start after Foundational (Phase 2) - Independent of all other stories
- **User Story 3 (P2) - Migration Documentation**: Can start after US2 (needs command reference structure) - Documents current state

### Within Each User Story

**User Story 1** (Version Accuracy):
- T008 (badge) → T009 (remove hardcoded) → T010 (tech stack) → T011 (build badge) → T012 (verify)
- Tasks T008-T011 can run in parallel if multiple contributors

**User Story 2** (Command Documentation):
- T013 (create structure) MUST complete first
- T014-T018 (5 categories) can run in PARALLEL [P] - different sections
- T019-T021 (examples, help, table) run after categories complete

**User Story 4** (Quick Start):
- T022 (create section) → T023-T026 (content) in sequence
- T023-T025 can run in parallel [P] if Quick Start is split into subsections

**User Story 5** (Constitutional Principles):
- T027 (create section) MUST complete first
- T028-T032 (5 principles) can run in PARALLEL [P] - different subsections
- T033-T034 (links, summary) run after principles documented

**User Story 3** (Migration Documentation):
- T035 (migration status) → T036-T039 (indicators, documentation) in sequence
- Depends on US2 (T021) completing command reference table

### Parallel Opportunities

**Phase 1 (Setup)**: T002 and T003 can run in parallel [P]

**Phase 2 (Foundational)**: T006 and T007 can run in parallel [P]

**Phase 3 (US1)**: T008, T009, T010, T011 can run in parallel if README sections are independent

**Phase 4 (US2)**: T013 MUST complete first, then T014-T018 (5 categories) run in parallel [P]

**Phase 6 (US5)**: T027 MUST complete first, then T028-T032 (5 principles) run in parallel [P]

**Phase 8 (Polish)**: T040, T041, T042, T043, T044 can all run in parallel [P] - different README sections

---

## Parallel Example: User Story 2 (Command Documentation)

```bash
# After T013 (structure) completes, launch all category documentation together:

Task("Document Test Execution commands", "6 commands in README.md Test Execution section", "documenter")
Task("Document Configuration commands", "5 commands in README.md Configuration section", "documenter")
Task("Document Observation commands", "5 commands in README.md Observation section", "documenter")
Task("Document System Management commands", "4 commands in README.md System Management section", "documenter")
Task("Document Development commands", "6 commands in README.md Development section", "documenter")

# All 5 tasks run concurrently (different README sections, no file conflicts)
```

---

## Parallel Example: User Story 5 (Constitutional Principles)

```bash
# After T027 (section creation) completes, launch all principle documentation together:

Task("Document Cargo Make Rule", "Principle I with example in README.md", "documenter")
Task("Document Error Handling Rule", "Principle II with Result pattern in README.md", "documenter")
Task("Document Chicago TDD Rule", "Principle III with AAA pattern in README.md", "documenter")
Task("Document Andon Signal Rule", "Principle IV with RED/YELLOW/GREEN table in README.md", "documenter")
Task("Document Concurrent Execution Rule", "Principle V with batch example in README.md", "documenter")

# All 5 tasks run concurrently (different subsections, no conflicts)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

**Scope**: Version accuracy + Command discoverability (P1 priorities only)

1. Complete Phase 1: Setup (validation infrastructure) - 30 min
2. Complete Phase 2: Foundational (verify research docs) - 5 min
3. Complete Phase 3: User Story 1 (version accuracy) - 45 min
4. Complete Phase 4: User Story 2 (command documentation) - 1h 30min
5. **STOP and VALIDATE**: Run validation scripts
6. **MVP Ready**: README has accurate version + all 26 commands documented

**Total MVP time**: ~2 hours 50 minutes

**MVP validation**:
```bash
bash specs/001-readme-cli-refactor/scripts/validate_readme.sh
bash specs/001-readme-cli-refactor/scripts/check_version_consistency.sh
# Both should exit 0 (GREEN)
```

### Incremental Delivery (Add P2 Priorities)

1. **MVP** (US1 + US2) → Test independently → Commit → Tag as v2.1.0-docs-mvp
2. **Add US4** (Quick Start) → Test independently → Commit → Tag as v2.1.0-docs-quickstart
3. **Add US5** (Constitutional Principles) → Test independently → Commit → Tag as v2.1.0-docs-constitution
4. **Add US3** (Migration Documentation) → Test independently → Commit → Tag as v2.1.0-docs-migration
5. **Add Phase 8** (Polish) → Final validation → Commit → Tag as v2.1.0-docs-complete

**Each increment adds value without breaking previous work**

### Parallel Team Strategy

With 3 contributors:

1. **All team members**: Complete Setup + Foundational together (35 min)
2. **Once Foundational is done**:
   - **Contributor A**: User Story 1 (Version Accuracy) - 45 min
   - **Contributor B**: User Story 2 (Command Documentation) - 1h 30min
   - **Contributor C**: User Story 5 (Constitutional Principles) - 45 min
3. **After US2 completes**:
   - **Contributor A or B**: User Story 4 (Quick Start) - 30 min
   - **Contributor C**: User Story 3 (Migration Documentation) - 20 min
4. **All team members**: Phase 8 (Polish) - 1 hour
5. **Total parallel time**: ~2 hours 35 minutes (vs 6+ hours sequential)

---

## Task Summary

### Total Task Count: 52 tasks

**By Phase**:
- Phase 1 (Setup): 4 tasks
- Phase 2 (Foundational): 3 tasks
- Phase 3 (US1 - Version Accuracy): 5 tasks
- Phase 4 (US2 - Command Documentation): 9 tasks
- Phase 5 (US4 - Quick Start): 5 tasks
- Phase 6 (US5 - Constitutional Principles): 8 tasks
- Phase 7 (US3 - Migration Documentation): 5 tasks
- Phase 8 (Polish): 13 tasks

**By User Story**:
- US1 (P1): 5 tasks (~45 min)
- US2 (P1): 9 tasks (~1h 30min)
- US3 (P2): 5 tasks (~20 min)
- US4 (P2): 5 tasks (~30 min)
- US5 (P2): 8 tasks (~45 min)

**Parallel Opportunities**: 26 tasks marked [P] (50% parallelizable)

**MVP Scope** (P1 only): Phases 1-4 = 21 tasks (~2h 50min)

**Full Feature** (P1 + P2): All 52 tasks (~5-6 hours sequential, ~2-3 hours parallel)

---

## Independent Test Criteria

### User Story 1 (Version Accuracy)
**Test**: `bash specs/001-readme-cli-refactor/scripts/check_version_consistency.sh`
- ✅ Version badge present in README
- ✅ No hardcoded "2.1.0" in prose (only in badge URLs)
- ✅ `clnrm --version` matches Cargo.toml

### User Story 2 (Command Documentation)
**Test**: `bash specs/001-readme-cli-refactor/scripts/validate_readme.sh`
- ✅ All 26 commands present in README
- ✅ 5 command categories exist
- ✅ Top 8 commands have usage examples
- ✅ Migration status indicators (✅ NounVerb / 🔄 Legacy) present

### User Story 3 (Migration Documentation)
**Test**: Manual verification
- ✅ README documents Hybrid architecture mode
- ✅ services and collector marked as ✅ NounVerb
- ✅ 24 commands marked as 🔄 Legacy
- ✅ Link to CLAP_NOUN_VERB_RESEARCH.md present

### User Story 4 (Quick Start)
**Test**: Manual walkthrough
- ✅ Follow Quick Start step-by-step
- ✅ Test passes in <5 minutes (Docker running)
- ✅ Clear success/failure indication

### User Story 5 (Constitutional Principles)
**Test**: `grep -c "constitution.md" README.md` (should be >= 5)
- ✅ All 5 principles documented
- ✅ Links to constitution.md work
- ✅ Code examples present

---

## Validation Checklist (Before Merge)

Run these commands to validate README is production-ready:

```bash
# 1. Automated validation
bash specs/001-readme-cli-refactor/scripts/validate_readme.sh
# Expected: ✅ All required sections present, ✅ All 26 commands documented, ✅ All 5 principles referenced

# 2. Version consistency
bash specs/001-readme-cli-refactor/scripts/check_version_consistency.sh
# Expected: ✅ Version badge present, ✅ No hardcoded versions

# 3. Manual checks
# [ ] Quick Start completes in <5 minutes
# [ ] All code examples are copy-pasteable
# [ ] All internal links work (no 404s)
# [ ] README is 5-10KB (Hub-and-Spoke pattern)
# [ ] Constitutional principles all linked to constitution.md

# 4. Final validation
cargo make test  # Ensure no regressions
clnrm --help | grep -c "Commands:"  # Should show 26
clnrm --version  # Should show 2.1.0
```

**All checks MUST pass (GREEN) before merge**

---

## Notes

- **[P] tasks**: Different README sections, can run in parallel without file conflicts
- **[Story] labels**: Maps tasks to user stories for traceability (US1-US5)
- **Independent testing**: Each user story can be tested in isolation
- **No code changes**: This is documentation-only (no Rust code modified)
- **MVP definition**: US1 + US2 (version accuracy + command discoverability)
- **Hub-and-Spoke**: Main README links to detailed docs (constitution.md, research docs)
- **Validation-first**: Scripts enforce structure (RED/YELLOW/GREEN Andon signals)
- **Incremental delivery**: Each user story adds value independently

**Ready to proceed with `/speckit.implement`**
