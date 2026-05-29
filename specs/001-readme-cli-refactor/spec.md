# Feature Specification: Complete README v2.1.0 and Partial CLI Refactor Migration

**Feature Branch**: `001-readme-cli-refactor`
**Created**: 2025-12-13
**Status**: Draft
**Input**: Update README to v2.1.0 reflecting actual CLI capabilities with completed noun-verb refactor for all commands

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Update README to v2.1.0 (Priority: P1)

As a user or maintainer of clnrm, I want the README to reflect version 2.1.0 so that documentation matches the installed software and build instructions work correctly.

**Why this priority**: Stale documentation causes user confusion, failed builds, and support burden. Version accuracy is non-negotiable.

**Independent Test**: Can be fully tested by verifying all version references in README (title, build section, dependencies) match Cargo.toml v2.1.0 exactly.

**Acceptance Scenarios**:

1. **Given** README is displayed, **When** user sees version references, **Then** all show "2.1.0" matching Cargo.toml
2. **Given** user compiles clnrm with build instructions in README, **When** build completes, **Then** binary reports `clnrm --version` as 2.1.0
3. **Given** user checks Tech Stack and dependencies, **When** they read README, **Then** all version numbers are current and accurate

---

### User Story 2 - Document Current CLI Commands (Priority: P1)

As a new user of clnrm, I want to discover all 26 available CLI commands and understand their purposes so that I can find what I need without trial-and-error or external documentation.

**Why this priority**: CLI discoverability is the primary entry point for all users. Poor command documentation directly impacts adoption.

**Independent Test**: Can be fully tested by running `clnrm --help` showing all commands and README containing categorized command reference with examples for top 8 commands.

**Acceptance Scenarios**:

1. **Given** user runs `clnrm --help`, **When** output is displayed, **Then** all 26 commands are listed with one-line descriptions
2. **Given** user reads README CLI Reference section, **When** they look for a command, **Then** they find it categorized by function with usage example
3. **Given** user runs `clnrm [command] --help`, **When** they need details, **Then** help text shows parameters, options, and a concrete usage example

---

### User Story 3 - Migrate Commands to Noun-Verb Pattern (Priority: P2)

As a clnrm maintainer, I want to complete the partial refactor and migrate all 26 commands to the clap-noun-verb pattern (currently only `services` and `collector` are migrated) so that the CLI has consistent architecture and better discoverability.

**Why this priority**: Currently services and collector use noun-verb but other 24 commands use legacy clap. Full migration enables better help text, environment variable support, and agent compatibility.

**Independent Test**: Can be fully tested by verifying all 26 commands are discovered by clap-noun-verb::run(), execute correctly with `[command] [verb]` syntax, and generate proper help via `--help`.

**Acceptance Scenarios**:

1. **Given** the CLI is refactored, **When** user runs any command with `--help`, **Then** help output shows verb options, environment variables, and dependency information
2. **Given** user runs a command without arguments, **When** help is generated, **Then** it lists all available verbs for that command (e.g., `clnrm run help` shows start, stop, etc.)
3. **Given** integration with agents/MCP, **When** they query available commands, **Then** full command tree is introspectable and machine-readable

---

### User Story 4 - Provide Clear Quick-Start Guide (Priority: P2)

As a new user, I want a simple 5-minute quick-start that gets me running a test without complex configuration so that I can validate clnrm works before diving deeper.

**Why this priority**: Reduces time-to-first-success from 30+ minutes to <5 minutes, improving conversion rate for new users.

**Independent Test**: Can be fully tested by following README quick-start step-by-step and having a test pass with Docker already running.

**Acceptance Scenarios**:

1. **Given** user has Docker running, **When** they follow the Quick Start steps, **Then** they can run a basic test in under 5 minutes
2. **Given** user follows the example, **When** they check output, **Then** they see clear success/failure indication
3. **Given** user wants to try before reading full docs, **When** they execute quick-start, **Then** they understand the core workflow (create test file, run, see results)

---

### User Story 5 - Document Constitutional Principles (Priority: P2)

As a developer or contributor, I want the README to clearly state the 5 constitutional principles that govern clnrm development so that I understand the project values and constraints.

**Why this priority**: Constitutional alignment ensures consistent code review and maintains project quality standards.

**Independent Test**: Can be fully tested by verifying README "Code Standards" section lists all 5 principles with proper links to constitution.md.

**Acceptance Scenarios**:

1. **Given** contributor reads README, **When** they review Code Standards section, **Then** they see the 5 principles: Cargo Make, Error Handling, Chicago TDD, Andon Signals, Concurrent Execution
2. **Given** contributor wants to understand a principle, **When** they follow a link, **Then** they reach constitution.md with detailed rationale
3. **Given** contributor submits a PR, **When** reviewer checks constitutional compliance, **Then** the README serves as quick reference for enforcement

### Edge Cases

- What happens when a new command is added? → README command reference MUST be updated as part of PR
- How are users with older versions guided to upgrade? → README should link to release notes and changelog
- What if noun-verb refactor introduces breaking changes? → README should document migration guide for users with scripts
- [NEEDS CLARIFICATION: Should README document the 24 legacy commands separately from 2 noun-verb commands during partial refactor phase?]

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: README MUST display version 2.1.0 in all locations (header, install instructions, version references)
- **FR-002**: README MUST list all 26 CLI commands with one-line descriptions organized by function (Test Execution, Analysis, Configuration, Advanced, Troubleshooting)
- **FR-003**: README MUST include at least one usage example for each of the 8 most-used commands (`run`, `dry-run`, `live-check`, `init`, `lint`, `fmt`, `analyze`, `report`)
- **FR-004**: README MUST provide a Quick Start section that enables running a test in under 5 minutes
- **FR-005**: README MUST document system prerequisites (Docker version, Rust 1.75+, system resources, etc.)
- **FR-006**: README MUST include a "Code Standards" section referencing all 5 constitutional principles with links to constitution.md
- **FR-007**: README MUST document how `clnrm --help` works and how to use subcommand help
- **FR-008**: Cargo.toml MUST be the single source of truth for version (no hardcoded versions in docs)
- **FR-009**: README MUST document the partial noun-verb migration status (which commands use new pattern vs legacy clap)
- **FR-010**: README MUST include troubleshooting section for common failures (Docker not running, Rust toolchain, file permissions, etc.)

### Key Entities

- **clnrm CLI**: Command-line interface with 26 subcommands across Test Execution, Analysis, Configuration, and Advanced categories
- **Version**: v2.1.0 (single source of truth defined in Cargo.toml workspace)
- **Commands by Architecture**:
  - Noun-Verb (clap-noun-verb v5.3.2): `services`, `collector` (2 commands)
  - Legacy Clap: `run`, `dry-run`, `live-check`, `dev`, `init`, `lint`, `fmt`, `plugins`, `analyze`, `spans`, `report`, `health`, `pull`, `render`, `graph`, `validate`, `record`, `stress`, `template`, `self-test`, `repro`, `diff` (24 commands)
- **Constitutional Principles**: 5 governance rules (Cargo Make, Error Handling, Chicago TDD, Andon Signals, Concurrent Execution)

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: README version and Cargo.toml version match exactly (both v2.1.0) with no stale references
- **SC-002**: Users can discover all 26 commands by running `clnrm --help` in under 5 seconds
- **SC-003**: New users can complete Quick Start section and have a passing test in under 5 minutes (with Docker running)
- **SC-004**: All 26 CLI commands appear in README with categorization, one-line description, and usage example for top 8 commands
- **SC-005**: CLI help output is self-documenting - users don't need README for basic command usage
- **SC-006**: "Code Standards" section accurately reflects all 5 constitutional principles with working links to constitution.md
- **SC-007**: Troubleshooting section resolves 90% of common setup failures (Docker, Rust, permissions, file format issues)
- **SC-008**: README clearly documents which commands use noun-verb pattern vs legacy clap, with migration guidance

## Assumptions

The following assumptions are made to fill in unspecified details:

1. **Partial Refactor State**: Only 2 commands (`services`, `collector`) are currently refactored to clap-noun-verb v5.3.2 with linkme integration. Remaining 24 commands use legacy clap.

2. **Full Migration Goal**: Feature assumes all 26 commands will be fully migrated to noun-verb pattern (separate task, not blocking README update).

3. **Version Single Source of Truth**: Cargo.toml workspace version is the authoritative version number for all documentation.

4. **26 Commands Total**: Actual audit found 26 distinct command modules in crates/clnrm-cli/src/cmds/ (not 15 as originally assumed).

5. **Constitutional Authority**: Constitution v1.0.0 (ratified 2025-12-13) governs all 5 principles referenced in code standards section.

6. **README Entry Point**: README should be primary discoverable entry point before CLAUDE.md, which provides detailed development guidance.
