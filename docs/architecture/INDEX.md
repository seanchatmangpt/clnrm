# CLNRM Architecture Documentation Index

**Last Updated**: 2025-10-16
**Current Version**: v0.6.0 (Design Phase)

## Overview

This directory contains all architectural documentation for the CLNRM (Cleanroom Testing Framework) project, including:
- Architecture Decision Records (ADRs)
- Version-specific design documents
- Migration guides
- Interface specifications
- Test strategies

## v0.6.0 Architecture Documents

### 1. Architecture Decision Records (ADRs)

#### [ADR-001: Tera Templating Integration](./ADR-001-tera-templating-integration.md)
**Status**: APPROVED
**Decision**: Extend existing template system with comprehensive fake data generators

**Key Points**:
- Template detection via content scanning (not file extension)
- Pure function approach for all generators
- Transparent rendering before TOML parsing
- Rich error messages with line/column context
- <50ms overhead for 1000-iteration templates

**New Functions**: `fake_uuid()`, `fake_name()`, `fake_email()`, `fake_ipv4()`, `fake_int()`, `fake_string()`, `fake_bool()`, `property_range()`, `random_choice()`

**Timeline**: 3 weeks (6 phases)

---

#### [ADR-002: OTEL Validation Enhancement](./ADR-002-otel-validation-enhancement.md)
**Status**: APPROVED
**Decision**: Complete OTEL validation with in-memory span collector

**Key Points**:
- In-memory span collector with `Arc<Mutex<Vec<SpanData>>>`
- Implement all validation methods (were `unimplemented!()`)
- Test utilities for easy OTEL setup in tests
- Custom exporter variant for test environments
- <10ms validation overhead

**New Components**: `InMemorySpanCollector`, `validate_span()`, `validate_trace()`, `validate_export()`, `setup_otel_for_testing()`

**Timeline**: 3 weeks (3 phases)

---

### 2. Planning & Strategy Documents

#### [v0.6.0 Migration Plan](./v0.6.0-migration-plan.md)
**Type**: Migration Guide
**Audience**: End Users, Plugin Developers, Contributors

**Contents**:
- Migration impact assessment (ZERO breaking changes)
- Feature comparison table (v0.4.x vs v0.6.0)
- Step-by-step migration instructions
- Rollback procedures
- Testing and validation steps
- Timeline and support resources

**Key Takeaway**: Migration is risk-free and completely backward compatible

---

#### [v0.6.0 Interface Specifications](./v0.6.0-interface-specifications.md)
**Type**: API Documentation
**Audience**: Contributors, Plugin Developers

**Contents**:
- Complete API surface for v0.6.0
- Template system interfaces
- OTEL validation interfaces
- Enhanced config loading interface
- Error handling interfaces
- Type aliases and re-exports
- Semantic versioning compliance
- Interface stability guarantees

**Key Sections**:
- TemplateRenderer API with all methods
- OtelValidator API with all validation methods
- Function signatures (Tera callable)
- Test utilities

---

#### [v0.6.0 Test Strategy (London TDD)](./v0.6.0-test-strategy-london-tdd.md)
**Type**: Testing Methodology
**Audience**: Contributors, QA Engineers

**Contents**:
- London School TDD principles
- Test pyramid structure (70% unit, 20% integration, 10% E2E)
- AAA pattern (Arrange, Act, Assert)
- Test doubles and mocking strategies
- Property-based testing with `proptest`
- Coverage requirements (90% minimum)
- CI/CD pipeline configuration

**Test Distribution**:
- Unit Tests: ~200 tests (95% coverage for functions)
- Integration Tests: ~60 tests (85% coverage)
- E2E Tests: ~30 tests (80% coverage)
- **Total**: ~290 tests

---

#### [v0.6.0 Architecture Coordination Summary](./v0.6.0-architecture-coordination-summary.md)
**Type**: Coordination Report
**Audience**: Stakeholders, Project Managers, Contributors

**Contents**:
- Executive summary of architecture work
- Team coordination and deliverables
- Risk assessment and mitigation
- Success criteria and metrics
- Implementation roadmap
- Handoff instructions

**Team Deliverables**:
- Requirements Analyst: Requirement analysis and gap identification
- System Designer: Architecture design and integration strategy
- API Architect: Interface specifications and API design

---

### 3. Existing Architecture Documents (v0.4.x)

#### [Tera Templating Architecture](./tera-templating-architecture.md)
**Status**: DESIGN PHASE - NOT IMPLEMENTED (v0.4.x)
**Type**: System Architecture Design

**Contents**:
- Full architecture diagrams (2,500+ lines)
- Component interaction diagrams
- 50+ custom functions/filters specifications
- 5 complete template TOML examples
- Implementation plan with 6 phases
- Error handling strategy
- Performance considerations

**Note**: This was the original design document. v0.6.0 ADR-001 supersedes this with refined decisions.

---

#### [Tera Templating Implementation Summary](./tera-templating-implementation-summary.md)
**Status**: DESIGN COMPLETE - READY FOR IMPLEMENTATION (v0.4.x)
**Type**: Summary Document

**Contents**:
- High-level overview of template system
- Key achievements and design decisions
- Function categories and examples
- Use cases and benefits
- Dependencies and file structure

**Note**: Summarizes the original architecture design. v0.6.0 documents provide updated specifications.

---

#### [Tera Templating Quick Reference](./tera-templating-quick-reference.md)
**Type**: Developer Guide
**Audience**: End Users

**Contents**:
- Quick start guide (3 steps)
- Function cheat sheet
- Common patterns (6 patterns)
- Template debugging commands
- Error examples and troubleshooting

**Note**: User-facing guide for template usage. Still applicable to v0.6.0 with additional functions.

---

## Document Relationships

```
Architecture Hierarchy:

├── ADRs (Decision Records)
│   ├── ADR-001: Tera Templating Integration ← Core decisions
│   └── ADR-002: OTEL Validation Enhancement  ← Core decisions
│
├── Planning Documents
│   ├── v0.6.0 Migration Plan                 ← References ADRs
│   ├── v0.6.0 Interface Specifications       ← Implements ADRs
│   └── v0.6.0 Test Strategy                  ← Tests ADRs
│
├── Coordination Summary
│   └── v0.6.0 Arch Coordination Summary      ← References all above
│
└── Legacy/Reference (v0.4.x)
    ├── Tera Templating Architecture          ← Original design
    ├── Tera Templating Implementation Summary
    └── Tera Templating Quick Reference
```

## Reading Path for Different Roles

### For Stakeholders
1. Start: [v0.6.0 Architecture Coordination Summary](./v0.6.0-architecture-coordination-summary.md)
2. Details: [ADR-001](./ADR-001-tera-templating-integration.md) and [ADR-002](./ADR-002-otel-validation-enhancement.md)
3. Migration: [v0.6.0 Migration Plan](./v0.6.0-migration-plan.md)

### For Contributors/Developers
1. Start: [v0.6.0 Interface Specifications](./v0.6.0-interface-specifications.md)
2. Design: [ADR-001](./ADR-001-tera-templating-integration.md) and [ADR-002](./ADR-002-otel-validation-enhancement.md)
3. Testing: [v0.6.0 Test Strategy (London TDD)](./v0.6.0-test-strategy-london-tdd.md)
4. Implementation: [v0.6.0 Architecture Coordination Summary](./v0.6.0-architecture-coordination-summary.md) (Roadmap section)

### For End Users
1. Start: [v0.6.0 Migration Plan](./v0.6.0-migration-plan.md)
2. Templates: [Tera Templating Quick Reference](./tera-templating-quick-reference.md)
3. Validation: [ADR-002 OTEL Validation](./ADR-002-otel-validation-enhancement.md) (Examples section)

### For QA Engineers
1. Start: [v0.6.0 Test Strategy (London TDD)](./v0.6.0-test-strategy-london-tdd.md)
2. Validation: [ADR-002 OTEL Validation](./ADR-002-otel-validation-enhancement.md)
3. Coverage: [v0.6.0 Architecture Coordination Summary](./v0.6.0-architecture-coordination-summary.md) (Success Criteria)

## Document Status Legend

| Status | Meaning |
|--------|---------|
| APPROVED | Architecture decision approved for implementation |
| DESIGN COMPLETE | Design work finished, ready for implementation |
| DESIGN PHASE | Design in progress, not yet implemented |
| NOT IMPLEMENTED | Design exists but no code written |
| PENDING APPROVAL | Awaiting stakeholder approval |

## Key Decisions Summary

### Template System (ADR-001)
- ✅ Content-based template detection (not extension-based)
- ✅ Pure functions with no side effects
- ✅ Transparent rendering before TOML parsing
- ✅ 13 new template functions
- ✅ 3-week implementation timeline

### OTEL Validation (ADR-002)
- ✅ In-memory span collector for validation
- ✅ Complete all `unimplemented!()` methods
- ✅ Test utilities for easy setup
- ✅ Custom exporter variant
- ✅ 3-week implementation timeline

### Migration (v0.6.0)
- ✅ Zero breaking changes
- ✅ 100% backward compatible
- ✅ Optional feature adoption
- ✅ Risk-free migration

### Quality (Test Strategy)
- ✅ 90% minimum coverage
- ✅ London TDD methodology
- ✅ 70/20/10 test pyramid
- ✅ ~290 total tests

## Timeline Overview

```
Week 1: Template Functions + OTEL Span Collector
├── Template: fake_uuid(), fake_name(), fake_email(), etc.
├── OTEL: InMemorySpanCollector implementation
└── Tests: Unit tests for all new functions

Week 2: Integration
├── Template: Config loading enhancement
├── OTEL: validate_span(), validate_trace(), validate_export()
└── Tests: Integration tests

Week 3: Testing & Documentation
├── E2E tests with 100+ generated scenarios
├── Property-based tests
├── Documentation and guides
└── Performance benchmarks
```

## Version History

### v0.6.0 (Design Phase - Current)
**Date**: 2025-10-16
**Documents**: 6 new architecture documents
**Status**: READY FOR IMPLEMENTATION

**Major Changes**:
- Comprehensive template system design
- Complete OTEL validation specification
- Migration plan and interface specs
- Test strategy with London TDD

### v0.4.x (Implemented)
**Documents**: 3 reference documents
**Status**: PRODUCTION

**Features**:
- Basic template support (env, now_rfc3339, sha256, toml_encode)
- OTEL validation stubs (unimplemented)
- Core framework functionality

## Contributing to Architecture

### Adding a New ADR

1. **Create file**: `docs/architecture/ADR-XXX-short-title.md`
2. **Use template**:
   ```markdown
   # ADR-XXX: [Title]

   **Status**: PROPOSED/APPROVED/DEPRECATED
   **Date**: YYYY-MM-DD
   **Deciders**: [List]
   **Technical Story**: [Description]

   ## Context and Problem Statement
   ## Decision Drivers
   ## Considered Options
   ## Decision Outcome
   ## Consequences
   ## Implementation Roadmap
   ## Links
   ```
3. **Update this index**
4. **Link related documents**

### Updating Existing Documents

1. **Update document** with changes
2. **Update "Last Updated"** date
3. **Add version note** if significant
4. **Update this index** if needed
5. **Create new ADR** if decision changes

## Support

### Documentation Issues
- File issue: https://github.com/seanchatmangpt/clnrm/issues
- Label: `documentation`

### Architecture Questions
- GitHub Discussions: https://github.com/seanchatmangpt/clnrm/discussions
- Category: `Architecture`

### Implementation Help
- Read: [v0.6.0 Architecture Coordination Summary](./v0.6.0-architecture-coordination-summary.md)
- Section: "Handoff to Implementation Team"

---

**Maintained by**: Architecture Sub-Coordinator
**Last Review**: 2025-10-16
**Next Review**: Upon v0.6.0 implementation start
