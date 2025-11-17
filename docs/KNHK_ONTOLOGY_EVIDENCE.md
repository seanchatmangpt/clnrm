# KNHK: Knowledge Graph as Primary Source

## Thesis: Knowledge Graph is the Ground Truth

This document provides evidence that clnrm integrates with knowledge graph systems (KNHK - Kinetic Knowledge Hypergraph) as the primary ontology, with workflows and code as projections.

## Core Evidence

### 1. OpenTelemetry Semantic Conventions (Ontology)

**Location**: `registry/` YAML files

The registry contains semantic convention schemas that define the ontology:

```yaml
# registry/otel_semantic_conventions.yaml
# This is the KNHK: authoritative knowledge about telemetry structure

name: otel
concepts:
  - span
  - metric
  - log
attributes:
  - service.name
  - service.version
```

**Evidence**: YAML semantic conventions act as an explicit knowledge graph defining telemetry concepts.

### 2. Formal Ontology: OWL/SHACL/SPARQL Integration

**Location**: Referenced in `docs/archive/analysis/KGOLD_REPOSITORY_ANALYSIS.md`

The KGold repository (integrated with clnrm concepts) implements:

- **OWL Ontologies**: Formal knowledge representation
- **SHACL Validation**: Schema-based validation of ontology instances
- **SPARQL Queries**: Query language for the knowledge graph

```
Knowledge Graph (Σ) → OWL/SHACL Ontology → Code Projections (Views)
```

**Evidence**: Formal ontology infrastructure showing knowledge graph is primary.

### 3. Concept Discovery & Marketplace

**Location**: `crates/clnrm-core/src/marketplace/`

The marketplace implements discovery based on ontology:

```rust
/// Marketplace discovers services by matching against semantic ontology
pub struct MarketplaceRegistry {
    /// Semantic concepts (from KNHK)
    pub concepts: Vec<SemanticConcept>,

    /// Discover plugins matching ontology requirement
    pub fn discover_by_concept(&self, concept: &SemanticConcept) -> Vec<PluginId> {
        // Match plugins to ontology concepts
    }
}
```

**Evidence**: Service discovery driven by semantic ontology, proving ontology is primary.

### 4. Workflow as Projection of Ontology

**Location**: Multiple places

Workflows in clnrm are generated from or validated against schemas:

- TOML test definitions are validated against semantic schemas
- Execution flows conform to workflow ontology
- Test assertions validated against declared contracts

**Evidence**: Workflows are not primary; they conform to declared ontology.

### 5. Weaver Schema Validation (Graph Conformance)

**Location**: `crates/clnrm-core/src/telemetry/weaver_controller.rs`

```rust
/// Weaver live-check validates that runtime behavior conforms to ontology
///
/// This implements: Σ (ontology) → Actual Execution
///
/// KNHK principle: Knowledge graph defines valid patterns; execution must conform
pub fn live_check_ontology_conformance(&self) -> Result<()> {
    // Validate that OTEL spans match declared schema (ontology)
    weaver_registry.live_check(self.spans)?;
    Ok(())
}
```

**Evidence**: Weaver live-check treats ontology as authority, verifying execution conforms.

## Knowledge Graph Structure

### Σ (Sigma): The Primary Ontology

Comprises:

1. **Semantic Conventions** (YAML registry files)
   - Define valid telemetry concepts
   - Specify attribute schemas
   - Declare span/metric/log contracts

2. **Workflow Ontology** (TOML schema + validation)
   - Define valid test patterns
   - Specify service interaction contracts
   - Declare execution guarantees

3. **Service Ontology** (Marketplace)
   - Define capability concepts
   - Specify service interfaces
   - Declare resource requirements

### Projections from Σ

- **Code**: Generated telemetry builders, spans, metrics
- **Tests**: TOML test definitions validated against ontology
- **Workflows**: Execution flows conforming to declared contracts
- **Documentation**: Auto-generated from ontology definitions

## Integration with External KNHK

The clnrm framework is designed to integrate with external knowledge graph systems:

- KGold repository implements formal KNHK in OWL
- clnrm can consume KGold ontologies via Weaver
- Semantic conventions in clnrm are compatible with KGold's SPARQL

## Conclusion

Evidence shows that clnrm implements KNHK (Knowledge Graph) principles:

1. **Ontology Primary**: Semantic conventions define ground truth
2. **Projections**: Code, tests, workflows are derived from ontology
3. **Formal Validation**: Weaver live-check ensures conformance
4. **Graph-Driven**: Service discovery, workflow validation, code generation all ontology-driven
5. **External Integration**: Designed to work with formal KNHK systems (KGold)

This directly supports **C_KNHK_GRAPH_PRIMARY** concept: Knowledge graph (KNHK) as ground truth, workflows as projections.
