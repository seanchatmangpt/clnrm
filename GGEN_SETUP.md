# GGEN Setup for CLNRM

This document describes the ggen (ontology-driven code generation) infrastructure set up for the CLNRM project.

## Overview

**ggen** is a code generator that uses RDF ontologies and SPARQL queries to drive deterministic code generation. This setup enables:

- **Single Source of Truth**: Domain model defined once in RDF
- **Multi-Language Code Generation**: Generate Rust, TypeScript, Python, and more
- **Semantic Validation**: Use OWL constraints to validate the domain
- **Inference**: SPARQL CONSTRUCT queries to materialize implicit relationships
- **Documentation**: Auto-generate API docs and guides from the ontology

## Project Structure

```
clnrm/
├── ggen.toml                      # ggen configuration
├── schema/
│   ├── clnrm-ontology.ttl        # Core ontology (classes, properties, relationships)
│   └── clnrm-instances.ttl       # Instance data (actual project metadata)
├── templates/
│   ├── README-template.md.tera    # Generate README from ontology
│   ├── ONTOLOGY-REFERENCE.md.tera # Generate ontology reference docs
│   ├── API-REFERENCE.md.tera      # Generate API docs
│   └── rust-types.rs.tera         # Generate Rust type definitions
├── generated/                      # Output directory (auto-created)
└── GGEN_SETUP.md                  # This file
```

## Configuration

The `ggen.toml` file configures the generation process:

```toml
[project]
name = "clnrm"
version = "2.0.0"
description = "Cleanroom Testing Framework"

[generation]
ontology_dir = "schema/"        # Where ontologies are stored
templates_dir = "templates/"    # Where Tera templates are stored
output_dir = "generated/"       # Where generated files go
```

## Ontology Structure

### Core Ontology (`schema/clnrm-ontology.ttl`)

Defines the formal domain model:

**Classes**:
- `CleanroomEnvironment` - Main testing environment
- `Container` - Docker container abstraction
- `Service` - Service/plugin abstraction
- `Test` - Test case definition
- `Assertion` - Test assertion
- `HealthCheck` - Service health check
- `Metrics` - Performance metrics
- `Crate` - Rust workspace crate

**Properties**:
- Object properties: `hasContainer`, `hasService`, `hasTest`, etc.
- Data properties: `containerName`, `testCommand`, `executionTime`, etc.

### Instance Data (`schema/clnrm-instances.ttl`)

Contains concrete data about the CLNRM project:

- Project metadata (crates, versions, descriptions)
- Example tests and containers
- Dependency relationships
- Sample configuration data

## Templates

### README Template (`templates/README-template.md.tera`)

Generates a project README with:
- Project metadata (name, version, authors)
- Class documentation
- Quick start guide
- Link to crates

**Usage**:
```bash
ggen sync --template README-template.md.tera
```

### API Reference Template (`templates/API-REFERENCE.md.tera`)

Generates Rust API documentation with:
- All classes/structs
- Field documentation
- Method signatures
- Usage examples
- Trait implementations

### Ontology Reference Template (`templates/ONTOLOGY-REFERENCE.md.tera`)

Generates formal ontology documentation with:
- Class hierarchy
- Property definitions
- Domain and range constraints
- Inference rules
- Semantic validation rules

### Rust Types Template (`templates/rust-types.rs.tera`)

Generates Rust source code:
- Struct definitions with `#[derive]` attributes
- Constructor methods
- Builder pattern implementations
- Default trait implementations

Example output:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanroomEnvironment {
    pub session_id: String,
    pub containers: Vec<Container>,
}

impl Default for CleanroomEnvironment { ... }

pub struct CleanroomEnvironmentBuilder { ... }
```

## Usage

### Initial Setup

The ggen infrastructure is already configured. Files are in place:

- ✅ `ggen.toml` - Configuration
- ✅ `schema/` - Ontologies
- ✅ `templates/` - Tera templates

### Running Generation

```bash
# Install ggen (Homebrew, Cargo, or Docker)
cargo install ggen-cli-lib

# Or via Homebrew
brew install seanchatmangpt/ggen/ggen

# Or via Docker
docker run --rm -v $(pwd):/workspace seanchatman/ggen:5.0.2 sync
```

### Generate All Artifacts

```bash
ggen sync
```

Outputs:
- `generated/README.md` - Project README
- `generated/API-REFERENCE.md` - API documentation
- `generated/ONTOLOGY-REFERENCE.md` - Ontology documentation
- `generated/types.rs` - Rust type definitions

### Generate Specific Output

```bash
# Dry-run (preview without writing)
ggen sync --dry-run

# Verify mode (CI/CD validation)
ggen sync --mode verify

# Incremental (preserve manual edits)
ggen sync --mode incremental
```

### Docker Usage

```bash
docker run --rm -v $(pwd):/workspace seanchatman/ggen:5.0.2 sync
```

### CI/CD Integration

**GitHub Actions** (`.github/workflows/codegen.yml`):

```yaml
name: Code Generation
on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install ggen
      - run: ggen sync --mode verify
      - name: Check generated code is up to date
        run: git diff --exit-code generated/
```

## Extending the Ontology

### Add a New Class

Edit `schema/clnrm-ontology.ttl`:

```turtle
clnrm:NewConcept a rdfs:Class ;
    rdfs:label "New Concept" ;
    rdfs:comment "Description of the concept" ;
    rdfs:isDefinedBy clnrm:Ontology .
```

### Add a Property

```turtle
clnrm:newProperty a rdf:Property ;
    rdfs:label "new property" ;
    rdfs:comment "Property description" ;
    rdfs:domain clnrm:MyClass ;
    rdfs:range xsd:string .
```

### Add Instance Data

Edit `schema/clnrm-instances.ttl`:

```turtle
ex:MyInstance a clnrm:MyClass ;
    clnrm:newProperty "value" ;
    rdfs:label "My Instance" .
```

### Create a New Template

Create `templates/my-template.output.tera`:

```jinja2
{% for class in classes %}
/* {{ class.name }} - {{ class.comment }} */
{% endfor %}
```

Then regenerate:

```bash
ggen sync
```

## Supported Output Formats

The template engine supports multiple output formats:

- `.md.tera` → Markdown documentation
- `.rs.tera` → Rust source code
- `.ts.tera` → TypeScript definitions
- `.py.tera` → Python classes
- `.toml.tera` → TOML configuration
- `.json.tera` → JSON schemas

## Integration with CLNRM

The ggen setup for CLNRM serves multiple purposes:

1. **Documentation Generation** - Auto-generate API docs from the domain model
2. **Type Safety** - Generate Rust types from ontology
3. **Configuration** - Generate configuration files from instances
4. **Examples** - Generate example code from instance data
5. **Validation** - Ensure ontology consistency

## Example: Adding a New Test Framework Feature

1. **Define in ontology** (`schema/clnrm-ontology.ttl`):
   ```turtle
   clnrm:ParallelExecution a rdfs:Class ;
       rdfs:comment "Parallel test execution configuration" .
   ```

2. **Add instance** (`schema/clnrm-instances.ttl`):
   ```turtle
   ex:ParallelConfig a clnrm:ParallelExecution ;
       clnrm:maxConcurrency 4 .
   ```

3. **Create template** (`templates/parallel-config.rs.tera`):
   ```rust
   pub struct ParallelConfig {
       pub max_concurrency: usize,
   }
   ```

4. **Run generation**:
   ```bash
   ggen sync
   ```

## Troubleshooting

### "ontology not found" error

```bash
# Check schema directory exists
ls schema/

# Verify files are valid Turtle RDF
cat schema/clnrm-ontology.ttl
```

### Template not rendering

```bash
# Check template syntax
ggen sync --verbose

# Validate Tera template
# (Check for unmatched {% %} blocks)
```

### Generated code has issues

```bash
# Use dry-run to preview
ggen sync --dry-run

# Check output format
ls generated/
```

## Resources

- **ggen Documentation**: https://docs.ggen.io
- **RDF Turtle Format**: https://www.w3.org/TR/turtle/
- **SPARQL 1.1**: https://www.w3.org/TR/sparql11-query/
- **Tera Template Engine**: https://keats.github.io/tera/
- **OWL 2 Specification**: https://www.w3.org/TR/owl2-overview/

## Next Steps

1. ✅ **Install ggen**: `cargo install ggen` or `brew install seanchatmangpt/ggen/ggen`
2. ✅ **Verify setup**: `ggen sync --dry-run`
3. ✅ **Generate docs**: `ggen sync`
4. ✅ **Review output**: Check `generated/` directory
5. 📝 **Extend ontology**: Add new classes/properties as needed
6. 🚀 **Integrate with CI/CD**: Add ggen sync to build pipeline

## Contributing

When updating the ontology or templates:

1. Edit the appropriate file (`schema/*.ttl` or `templates/*.tera`)
2. Run `ggen sync --dry-run` to preview changes
3. Review generated output in `generated/`
4. Run tests: `cargo test`
5. Commit with clear message: `feat(ontology): add new class for X`

---

**Generated as part of ggen setup for CLNRM v2.0.0**
