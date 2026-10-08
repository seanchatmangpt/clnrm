# docs/ Index

Directory map for `docs/`. Subdirectory READMEs take precedence over this file.

- `architecture/` - architecture notes and diagrams.
- `design/` - design documents.
- `examples/` - documented examples.
- `validated/` - validated documentation snapshots.
- `jira/` - per-campaign work orders (e.g. `v26.9.19/`); current campaign at top.

## Key entries (top level)

- `MIGRATION_GUIDE_3.0.md` - v2.x to v3.0 upgrade (backend change).
- `V2_0_0_ARCHITECTURE.md`, `V2_0_0_CONFIG_REFERENCE.md`, `V2_0_0_MIGRATION_GUIDE.md` - v2 surface.
- `GVISOR_README.md` + `GVISOR_*.md` - gVisor backend docs (read `GVISOR_README.md` first).
- `OCI_GVISOR_*.md` - OCI + runsc integration.
- `TESTING.md`, `DOCTEST_GUIDE.md`, `GALL_TESTING_SPEC.md`, `FMEA_TESTING_AUDIT.md` - testing discipline.
- `CODE_STANDARDS.md`, `CODE_QUALITY_ANALYSIS_REPORT.md` - code standards.
- `CI_CD.md`, `GIT_HOOKS_ADVANCED.md` - CI and hooks.
- `DEVELOPMENT.md`, `SETUP.md`, `COMMAND_CATEGORIZATION_REFERENCE.md`, `CLAP_NOUN_VERB_RESEARCH.md` - development and CLI.
- `POKA_YOKE_*.md` - mistake-proofing abstraction layers.

## See Also

- [Fleet Doc Map](../../ggen-marketplace/docs/reference/FLEET-DOC-MAP.md) - cross-repo
  documentation map for the fleet.
- Repo root [README](../README.md).
