# Data Model: CLI Command Structure

**Feature**: Complete README v2.1.0 and Partial CLI Refactor Migration
**Branch**: `001-readme-cli-refactor`
**Date**: 2025-12-13

## Entity Overview

This data model defines the structure of CLI commands, their organization, and relationships for the clnrm hermetic container testing framework.

## Entities

### 1. ClnrmCLI

**Description**: Root CLI application with 26 subcommands organized into 5 functional categories.

**Attributes**:
| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| version | String | Yes | Semver (e.g., "2.1.0") | CLI version from Cargo.toml workspace |
| commands | Vec<Command> | Yes | length == 26 | All available CLI commands |
| categories | Vec<Category> | Yes | length == 5 | Functional command groupings |
| architecture_mode | ArchitectureMode | Yes | Enum variant | Legacy, NounVerb, or Hybrid |

**Invariants**:
- version MUST match Cargo.toml `[workspace.package] version`
- commands.len() == 26 (verified by command count test)
- All commands MUST appear in exactly one category

**State transitions**:
```
[Legacy] → [Hybrid] → [NounVerb]
  (0/26)    (2/26)     (26/26)
```

### 2. Command

**Description**: Individual CLI command with metadata, arguments, and execution handler.

**Attributes**:
| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| name | String | Yes | kebab-case, unique | Command name (e.g., "run", "services") |
| category | CategoryType | Yes | Enum variant | Functional category assignment |
| architecture | Architecture | Yes | Legacy \| NounVerb | Implementation pattern |
| description | String | Yes | non-empty | One-line command description |
| usage_example | Option<String> | No | valid shell syntax | Example invocation |
| verbs | Vec<Verb> | Conditional | required if NounVerb | Sub-actions (for noun-verb commands) |
| env_vars | Vec<EnvVar> | No | valid env var names | Supported environment variables |
| otel_instrumented | bool | Yes | always true | OTEL tracing requirement |

**Invariants**:
- If architecture == NounVerb, then verbs.len() > 0
- If architecture == Legacy, then verbs.is_empty()
- name MUST match file in `crates/clnrm-cli/src/cmds/{name}.rs`
- otel_instrumented MUST be true (constitutional requirement)

**Validation rules**:
```rust
impl Command {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Name must be kebab-case
        if !self.name.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric()) {
            return Err(ValidationError::InvalidCommandName(self.name.clone()));
        }

        // Noun-verb commands require verbs
        if self.architecture == Architecture::NounVerb && self.verbs.is_empty() {
            return Err(ValidationError::NounVerbMissingVerbs(self.name.clone()));
        }

        // OTEL instrumentation is mandatory
        if !self.otel_instrumented {
            return Err(ValidationError::MissingOtelInstrumentation(self.name.clone()));
        }

        Ok(())
    }
}
```

### 3. Category

**Description**: Functional grouping of related commands (e.g., "Test Execution").

**Attributes**:
| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| type | CategoryType | Yes | Enum variant | Category identifier |
| display_name | String | Yes | non-empty | Human-readable category name |
| description | String | Yes | non-empty | Category purpose description |
| commands | Vec<&Command> | Yes | len > 0 | Commands in this category |
| priority | u8 | Yes | 1-5 | Display order in README |

**CategoryType Enum**:
```rust
pub enum CategoryType {
    TestExecution,      // Priority 1 (most important)
    Configuration,      // Priority 2
    Observation,        // Priority 3
    SystemManagement,   // Priority 4
    Development,        // Priority 5
}
```

**Invariants**:
- Each category MUST contain at least 1 command
- Sum of commands across all categories == 26
- priority values 1-5 MUST be unique (no duplicates)

### 4. Verb

**Description**: Sub-action for noun-verb pattern commands (e.g., "start", "stop" for "services").

**Attributes**:
| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| name | String | Yes | lowercase, unique within noun | Verb name (e.g., "start") |
| description | String | Yes | non-empty | Verb action description |
| args | Vec<Arg> | No | - | Required/optional arguments |
| output_format | Vec<OutputFormat> | Yes | len > 0 | Supported output formats |
| env_vars | Vec<EnvVar> | No | - | Verb-specific env vars |

**OutputFormat Enum**:
```rust
pub enum OutputFormat {
    Json,        // #[verb(output(json))]
    Msgpack,     // #[verb(output(msgpack))]
    Text,        // Default human-readable
}
```

**Example** (from services.rs):
```rust
Verb {
    name: "start".to_string(),
    description: "Start a clnrm service".to_string(),
    args: vec![
        Arg { name: "name", type: "String", required: true }
    ],
    output_format: vec![OutputFormat::Json, OutputFormat::Msgpack],
    env_vars: vec![
        EnvVar { name: "CLNRM_SERVICE_NAME", description: "Default service name" }
    ],
}
```

### 5. EnvVar

**Description**: Environment variable support for commands/verbs.

**Attributes**:
| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| name | String | Yes | uppercase, CLNRM_ prefix | Environment variable name |
| description | String | Yes | non-empty | Variable purpose |
| default_value | Option<String> | No | - | Default if unset |
| required | bool | Yes | - | Whether variable is mandatory |

**Invariants**:
- name MUST start with "CLNRM_" prefix
- name MUST be uppercase with underscores
- If required == true, default_value MUST be None (cannot have both)

**Validation**:
```rust
impl EnvVar {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.name.starts_with("CLNRM_") {
            return Err(ValidationError::InvalidEnvVarPrefix(self.name.clone()));
        }

        if self.required && self.default_value.is_some() {
            return Err(ValidationError::RequiredEnvVarWithDefault(self.name.clone()));
        }

        Ok(())
    }
}
```

### 6. Architecture

**Description**: CLI command implementation architecture pattern.

**Enum**:
```rust
pub enum Architecture {
    Legacy,     // Traditional clap with Commands enum (24 commands)
    NounVerb,   // clap-noun-verb with linkme registration (2 commands: services, collector)
}
```

**Properties by Architecture**:

| Property | Legacy | NounVerb |
|----------|--------|----------|
| File structure | Central enum + match | Distributed (#[noun] macro) |
| Registration | Manual in main.rs | linkme distributed slice |
| Help text | clap derive macros | #[noun] description |
| Env vars | Manual std::env::var | #[arg(env = "...")] |
| Testing | Enum construction | Direct function call |
| Modularity | Low (central coupling) | High (self-contained) |

### 7. ArchitectureMode

**Description**: Overall CLI architecture mode during migration.

**Enum**:
```rust
pub enum ArchitectureMode {
    Legacy,    // All 26 commands use legacy clap
    Hybrid,    // Some commands use legacy, some noun-verb (current state: 2/26)
    NounVerb,  // All 26 commands use noun-verb (target state)
}
```

**Current state**: `Hybrid` (2 noun-verb, 24 legacy)
**Target state**: `NounVerb` (26 noun-verb, 0 legacy) - **SEPARATE SPEC**

### 8. ConstitutionalPrinciple

**Description**: Governance rule from constitution v1.0.0 that applies to CLI commands.

**Attributes**:
| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| id | u8 | Yes | 1-5 | Principle number (I-V) |
| name | String | Yes | non-empty | Principle name |
| description | String | Yes | non-empty | Brief explanation |
| link | String | Yes | valid path | Link to constitution.md section |

**Instances** (5 total):
```rust
const PRINCIPLES: [ConstitutionalPrinciple; 5] = [
    ConstitutionalPrinciple {
        id: 1,
        name: "Cargo Make Rule".to_string(),
        description: "All build/test operations via cargo make".to_string(),
        link: "/Users/sac/clnrm/.specify/memory/constitution.md#i-cargo-make-rule-absolute".to_string(),
    },
    ConstitutionalPrinciple {
        id: 2,
        name: "Error Handling Rule".to_string(),
        description: "Production code: Result<T, CleanroomError>, no unwrap/expect".to_string(),
        link: "/Users/sac/clnrm/.specify/memory/constitution.md#ii-error-handling-rule-production-code".to_string(),
    },
    ConstitutionalPrinciple {
        id: 3,
        name: "Chicago TDD Rule".to_string(),
        description: "State-based testing with real collaborators, AAA pattern".to_string(),
        link: "/Users/sac/clnrm/.specify/memory/constitution.md#iii-chicago-tdd-rule-arrange-act-assert".to_string(),
    },
    ConstitutionalPrinciple {
        id: 4,
        name: "Andon Signal Rule".to_string(),
        description: "RED/YELLOW/GREEN discipline, stop on errors".to_string(),
        link: "/Users/sac/clnrm/.specify/memory/constitution.md#iv-andon-signal-rule-stop-the-line".to_string(),
    },
    ConstitutionalPrinciple {
        id: 5,
        name: "Concurrent Execution Rule".to_string(),
        description: "Batch all operations in single message (1 Message = All Ops)".to_string(),
        link: "/Users/sac/clnrm/.specify/memory/constitution.md#v-concurrent-execution-rule-1-message--all-operations".to_string(),
    },
];
```

## Relationships

```
ClnrmCLI
├── has many → Category (5)
│   └── contains many → Command (26 total)
│       ├── has optional many → Verb (if NounVerb architecture)
│       │   ├── has many → Arg
│       │   └── has many → EnvVar
│       └── has many → EnvVar
└── governed by → ConstitutionalPrinciple (5)
```

**Cardinality**:
- ClnrmCLI `1 : 5` Category
- Category `1 : N` Command (where sum(N) = 26)
- Command `1 : 0..M` Verb (M > 0 if NounVerb, M = 0 if Legacy)
- Verb `1 : N` Arg
- Command/Verb `1 : N` EnvVar

## Data Integrity Constraints

### Global Constraints

1. **Command Count Invariant**:
   ```rust
   assert_eq!(cli.commands.len(), 26, "Must have exactly 26 commands");
   ```

2. **Category Coverage**:
   ```rust
   let categorized_count: usize = cli.categories.iter()
       .map(|cat| cat.commands.len())
       .sum();
   assert_eq!(categorized_count, 26, "All commands must be categorized");
   ```

3. **Unique Command Names**:
   ```rust
   let unique_names: HashSet<&str> = cli.commands.iter()
       .map(|cmd| cmd.name.as_str())
       .collect();
   assert_eq!(unique_names.len(), 26, "Command names must be unique");
   ```

4. **Version Synchronization**:
   ```rust
   let cargo_version = env!("CARGO_PKG_VERSION");
   assert_eq!(cli.version, cargo_version, "CLI version must match Cargo.toml");
   ```

### Category-Specific Constraints

**Test Execution** (6 commands):
- Commands: run, dry-run, record, repro, stress, self-test
- All must have OTEL instrumentation
- All must return Result<TestExecutionResult, CleanroomError>

**Configuration** (5 commands):
- Commands: init, validate, lint, fmt, render
- All must validate TOML files
- All must provide --check mode (read-only validation)

**Observation** (5 commands):
- Commands: spans, report, graph, health, live-check
- All must produce structured output (JSON/YAML)
- All must handle missing telemetry gracefully

**System Management** (4 commands):
- Commands: services, collector, plugins, pull
- services and collector MUST use NounVerb architecture
- All must handle service lifecycle states

**Development** (5 commands):
- Commands: dev, template, diff, analyze
- All must support hot-reload or incremental execution

## Migration State Tracking

### Current State (v2.1.0)

**Migrated to NounVerb** (2/26):
- `services` (7 verbs: start, stop, restart, status, list, logs, health)
- `collector` (5 verbs: start, stop, status, config, metrics)

**Remaining Legacy** (24/26):
- Test Execution: run, dry-run, record, repro, stress, self-test
- Configuration: init, validate, lint, fmt, render
- Observation: spans, report, graph, health, live-check
- System Mgmt: plugins, pull
- Development: dev, template, diff, analyze

### Target State (Future Spec: 002-complete-cli-migration)

**All NounVerb** (26/26):
- Test namespace: test {run, dry-run, record, repro, stress, self-test}
- Config namespace: config {init, validate, lint, fmt, render}
- Observe namespace: observe {spans, report, graph, health, live-check}
- System namespace: system {services, collector, plugins, pull}
- Dev namespace: dev {dev, template, diff, analyze}

**Backward compatibility**: Legacy syntax supported via deprecation warnings for 1 major version.

## Example Instance Data

```rust
// Example: services command (NounVerb architecture)
Command {
    name: "services".to_string(),
    category: CategoryType::SystemManagement,
    architecture: Architecture::NounVerb,
    description: "Service lifecycle management commands".to_string(),
    usage_example: Some("clnrm services start my-service".to_string()),
    verbs: vec![
        Verb {
            name: "start".to_string(),
            description: "Start a clnrm service".to_string(),
            args: vec![
                Arg { name: "name", arg_type: "String".to_string(), required: true }
            ],
            output_format: vec![OutputFormat::Json, OutputFormat::Msgpack],
            env_vars: vec![
                EnvVar {
                    name: "CLNRM_SERVICE_NAME".to_string(),
                    description: "Default service name".to_string(),
                    default_value: None,
                    required: false,
                }
            ],
        },
        // ... 6 more verbs
    ],
    env_vars: vec![],
    otel_instrumented: true,
}

// Example: run command (Legacy architecture)
Command {
    name: "run".to_string(),
    category: CategoryType::TestExecution,
    architecture: Architecture::Legacy,
    description: "Execute test specifications from TOML files".to_string(),
    usage_example: Some("clnrm run tests/integration.clnrm.toml".to_string()),
    verbs: vec![], // Legacy commands have no verbs
    env_vars: vec![
        EnvVar {
            name: "CLNRM_TIMEOUT".to_string(),
            description: "Test execution timeout in seconds".to_string(),
            default_value: Some("300".to_string()),
            required: false,
        }
    ],
    otel_instrumented: true,
}
```

## Testing Requirements

### Unit Tests

**Command validation**:
```rust
#[test]
fn test_command_name_validation() {
    let invalid = Command { name: "Invalid_Name".to_string(), /* ... */ };
    assert!(invalid.validate().is_err());

    let valid = Command { name: "valid-name".to_string(), /* ... */ };
    assert!(valid.validate().is_ok());
}
```

**Category coverage**:
```rust
#[test]
fn test_all_commands_categorized() {
    let cli = ClnrmCLI::new();
    let categorized: HashSet<&str> = cli.categories.iter()
        .flat_map(|cat| cat.commands.iter().map(|cmd| cmd.name.as_str()))
        .collect();

    assert_eq!(categorized.len(), 26);
}
```

**Architecture mode transitions**:
```rust
#[test]
fn test_architecture_mode_progression() {
    let modes = vec![
        ArchitectureMode::Legacy,
        ArchitectureMode::Hybrid,
        ArchitectureMode::NounVerb,
    ];

    // Hybrid state must have at least 1 legacy and 1 noun-verb
    let hybrid_cli = ClnrmCLI::with_mode(ArchitectureMode::Hybrid);
    let legacy_count = hybrid_cli.commands.iter()
        .filter(|cmd| matches!(cmd.architecture, Architecture::Legacy))
        .count();
    let nounverb_count = hybrid_cli.commands.iter()
        .filter(|cmd| matches!(cmd.architecture, Architecture::NounVerb))
        .count();

    assert!(legacy_count > 0 && nounverb_count > 0);
}
```

### Integration Tests

**Help text generation**:
```rust
#[test]
fn test_help_text_includes_all_commands() {
    let output = std::process::Command::new("clnrm")
        .arg("--help")
        .output()
        .unwrap();

    let help_text = String::from_utf8(output.stdout).unwrap();

    // All 26 commands must appear in help
    assert!(help_text.contains("run"));
    assert!(help_text.contains("services"));
    // ... verify all 26
}
```

**Version consistency**:
```rust
#[test]
fn test_version_matches_cargo_toml() {
    let output = std::process::Command::new("clnrm")
        .arg("--version")
        .output()
        .unwrap();

    let version_output = String::from_utf8(output.stdout).unwrap();
    let cargo_version = env!("CARGO_PKG_VERSION");

    assert!(version_output.contains(cargo_version));
}
```

## JSON Schema (for contracts validation)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ClnrmCLI",
  "type": "object",
  "required": ["version", "commands", "categories", "architecture_mode"],
  "properties": {
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$"
    },
    "commands": {
      "type": "array",
      "minItems": 26,
      "maxItems": 26,
      "items": { "$ref": "#/definitions/Command" }
    },
    "categories": {
      "type": "array",
      "minItems": 5,
      "maxItems": 5,
      "items": { "$ref": "#/definitions/Category" }
    },
    "architecture_mode": {
      "type": "string",
      "enum": ["Legacy", "Hybrid", "NounVerb"]
    }
  },
  "definitions": {
    "Command": {
      "type": "object",
      "required": ["name", "category", "architecture", "description", "otel_instrumented"],
      "properties": {
        "name": {
          "type": "string",
          "pattern": "^[a-z0-9-]+$"
        },
        "category": {
          "type": "string",
          "enum": ["TestExecution", "Configuration", "Observation", "SystemManagement", "Development"]
        },
        "architecture": {
          "type": "string",
          "enum": ["Legacy", "NounVerb"]
        },
        "description": { "type": "string", "minLength": 1 },
        "usage_example": { "type": "string" },
        "verbs": {
          "type": "array",
          "items": { "$ref": "#/definitions/Verb" }
        },
        "env_vars": {
          "type": "array",
          "items": { "$ref": "#/definitions/EnvVar" }
        },
        "otel_instrumented": { "type": "boolean", "const": true }
      }
    }
  }
}
```

## Summary

This data model defines:
- **26 CLI commands** organized into **5 functional categories**
- **2 architecture patterns**: Legacy (24 commands) and NounVerb (2 commands)
- **Hybrid architecture mode** during migration (current state)
- **5 constitutional principles** governing all commands
- **Validation rules** ensuring data integrity
- **Testing requirements** for command structure

All entities follow zero-unwrap error handling, OTEL instrumentation, and Chicago TDD patterns as required by the constitution.
