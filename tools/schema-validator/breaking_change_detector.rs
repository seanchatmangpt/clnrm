/// Breaking Change Detector
/// Detects schema changes that would break existing code or telemetry
///
/// PURPOSE: Prevent schema evolution from invalidating existing instrumentation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttributeInfo {
    pub name: String,
    pub attr_type: String,
    pub requirement_level: String,
    pub is_enum: bool,
    pub enum_values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaInfo {
    pub id: String,
    pub schema_type: String,
    pub stability: String,
    pub attributes: HashMap<String, AttributeInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BreakingChange {
    pub kind: BreakingChangeKind,
    pub schema_id: String,
    pub attribute: Option<String>,
    pub impact: String,
    pub severity: Severity,
    pub migration_guide: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BreakingChangeKind {
    RemovedSchema,
    RemovedRequiredAttribute,
    RemovedRecommendedAttribute,
    ChangedAttributeType,
    ChangedEnumValues,
    ChangedRequirementLevel,
    ChangedStability,
    RemovedEnumMember,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Severity {
    Critical,  // Breaks existing code
    High,      // Breaks existing telemetry
    Medium,    // Breaks best practices
    Low,       // Minor compatibility issue
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BreakingChangeReport {
    pub old_version: String,
    pub new_version: String,
    pub timestamp: String,
    pub breaking_changes: Vec<BreakingChange>,
    pub critical_count: usize,
    pub high_count: usize,
    pub safe_to_upgrade: bool,
}

pub struct BreakingChangeDetector;

impl BreakingChangeDetector {
    /// Compare two schema versions and detect breaking changes
    pub fn check_schema_breaking_changes(
        old_schema: &SchemaInfo,
        new_schema: &SchemaInfo,
    ) -> Vec<BreakingChange> {
        let mut breaking = vec![];

        // Check for removed required attributes (CRITICAL)
        for (attr_name, attr_info) in &old_schema.attributes {
            if attr_info.requirement_level == "required" {
                if !new_schema.attributes.contains_key(attr_name) {
                    breaking.push(BreakingChange {
                        kind: BreakingChangeKind::RemovedRequiredAttribute,
                        schema_id: old_schema.id.clone(),
                        attribute: Some(attr_name.clone()),
                        impact: format!(
                            "Existing telemetry will fail validation. Code expecting '{}' will break.",
                            attr_name
                        ),
                        severity: Severity::Critical,
                        migration_guide: format!(
                            "Update all instrumentation to remove references to '{}' or keep old schema version.",
                            attr_name
                        ),
                    });
                }
            }
        }

        // Check for type changes (CRITICAL)
        for (attr_name, old_attr) in &old_schema.attributes {
            if let Some(new_attr) = new_schema.attributes.get(attr_name) {
                if old_attr.attr_type != new_attr.attr_type {
                    breaking.push(BreakingChange {
                        kind: BreakingChangeKind::ChangedAttributeType,
                        schema_id: old_schema.id.clone(),
                        attribute: Some(attr_name.clone()),
                        impact: format!(
                            "Type changed from {} to {}. Existing code will not compile.",
                            old_attr.attr_type, new_attr.attr_type
                        ),
                        severity: Severity::Critical,
                        migration_guide: format!(
                            "Update all code setting '{}' to use {} type instead of {}.",
                            attr_name, new_attr.attr_type, old_attr.attr_type
                        ),
                    });
                }
            }
        }

        // Check for enum value changes (HIGH)
        for (attr_name, old_attr) in &old_schema.attributes {
            if old_attr.is_enum {
                if let Some(new_attr) = new_schema.attributes.get(attr_name) {
                    if new_attr.is_enum {
                        // Check for removed enum values
                        for old_value in &old_attr.enum_values {
                            if !new_attr.enum_values.contains(old_value) {
                                breaking.push(BreakingChange {
                                    kind: BreakingChangeKind::RemovedEnumMember,
                                    schema_id: old_schema.id.clone(),
                                    attribute: Some(format!("{}.{}", attr_name, old_value)),
                                    impact: format!(
                                        "Code using enum value '{}' will fail validation.",
                                        old_value
                                    ),
                                    severity: Severity::High,
                                    migration_guide: format!(
                                        "Replace all uses of '{}' with one of: {:?}",
                                        old_value, new_attr.enum_values
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check for requirement level changes (MEDIUM/HIGH)
        for (attr_name, old_attr) in &old_schema.attributes {
            if let Some(new_attr) = new_schema.attributes.get(attr_name) {
                if old_attr.requirement_level != new_attr.requirement_level {
                    let severity = if new_attr.requirement_level == "required" {
                        Severity::High  // Making optional -> required breaks existing code
                    } else {
                        Severity::Medium  // Making required -> optional is less severe
                    };

                    breaking.push(BreakingChange {
                        kind: BreakingChangeKind::ChangedRequirementLevel,
                        schema_id: old_schema.id.clone(),
                        attribute: Some(attr_name.clone()),
                        impact: format!(
                            "Requirement level changed from {} to {}. May break validation.",
                            old_attr.requirement_level, new_attr.requirement_level
                        ),
                        severity,
                        migration_guide: if new_attr.requirement_level == "required" {
                            format!("Ensure all code sets '{}' attribute", attr_name)
                        } else {
                            format!("'{}' is now optional, can be removed if not needed", attr_name)
                        },
                    });
                }
            }
        }

        // Check for stability changes (MEDIUM)
        if old_schema.stability != new_schema.stability {
            let severity = match (old_schema.stability.as_str(), new_schema.stability.as_str()) {
                ("stable", "experimental") => Severity::High,
                ("stable", "deprecated") => Severity::Medium,
                _ => Severity::Low,
            };

            breaking.push(BreakingChange {
                kind: BreakingChangeKind::ChangedStability,
                schema_id: old_schema.id.clone(),
                attribute: None,
                impact: format!(
                    "Stability changed from {} to {}",
                    old_schema.stability, new_schema.stability
                ),
                severity,
                migration_guide: if new_schema.stability == "deprecated" {
                    "Migrate to replacement schema before removal".to_string()
                } else {
                    "Review stability implications for production use".to_string()
                },
            });
        }

        breaking
    }

    /// Check all schemas for breaking changes
    pub fn check_all_schemas(
        old_schemas: &HashMap<String, SchemaInfo>,
        new_schemas: &HashMap<String, SchemaInfo>,
    ) -> Vec<BreakingChange> {
        let mut all_breaking = vec![];

        // Check for removed schemas
        for (schema_id, old_schema) in old_schemas {
            if !new_schemas.contains_key(schema_id) {
                all_breaking.push(BreakingChange {
                    kind: BreakingChangeKind::RemovedSchema,
                    schema_id: schema_id.clone(),
                    attribute: None,
                    impact: format!("Schema '{}' was removed. All instrumentation using it will break.", schema_id),
                    severity: Severity::Critical,
                    migration_guide: format!("Migrate to replacement schema or keep old schema version for '{}'", schema_id),
                });
            }
        }

        // Check each schema for changes
        for (schema_id, old_schema) in old_schemas {
            if let Some(new_schema) = new_schemas.get(schema_id) {
                let changes = Self::check_schema_breaking_changes(old_schema, new_schema);
                all_breaking.extend(changes);
            }
        }

        all_breaking
    }

    /// Generate breaking change report
    pub fn generate_report(
        old_schemas: &HashMap<String, SchemaInfo>,
        new_schemas: &HashMap<String, SchemaInfo>,
        old_version: &str,
        new_version: &str,
    ) -> BreakingChangeReport {
        let breaking_changes = Self::check_all_schemas(old_schemas, new_schemas);

        let critical_count = breaking_changes.iter()
            .filter(|c| matches!(c.severity, Severity::Critical))
            .count();

        let high_count = breaking_changes.iter()
            .filter(|c| matches!(c.severity, Severity::High))
            .count();

        let safe_to_upgrade = critical_count == 0;

        BreakingChangeReport {
            old_version: old_version.to_string(),
            new_version: new_version.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            breaking_changes,
            critical_count,
            high_count,
            safe_to_upgrade,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaking_change_detection() {
        // Create old schema
        let mut old_attrs = HashMap::new();
        old_attrs.insert(
            "container.id".to_string(),
            AttributeInfo {
                name: "container.id".to_string(),
                attr_type: "string".to_string(),
                requirement_level: "required".to_string(),
                is_enum: false,
                enum_values: vec![],
            },
        );
        old_attrs.insert(
            "test.result".to_string(),
            AttributeInfo {
                name: "test.result".to_string(),
                attr_type: "enum".to_string(),
                requirement_level: "required".to_string(),
                is_enum: true,
                enum_values: vec!["pass".to_string(), "fail".to_string(), "error".to_string()],
            },
        );

        let old_schema = SchemaInfo {
            id: "span.clnrm.test_execution".to_string(),
            schema_type: "span".to_string(),
            stability: "stable".to_string(),
            attributes: old_attrs,
        };

        // Create new schema with breaking changes
        let mut new_attrs = HashMap::new();
        // Removed container.id (BREAKING!)
        new_attrs.insert(
            "test.result".to_string(),
            AttributeInfo {
                name: "test.result".to_string(),
                attr_type: "enum".to_string(),
                requirement_level: "required".to_string(),
                is_enum: true,
                enum_values: vec!["pass".to_string(), "fail".to_string()],  // Removed "error" (BREAKING!)
            },
        );

        let new_schema = SchemaInfo {
            id: "span.clnrm.test_execution".to_string(),
            schema_type: "span".to_string(),
            stability: "stable".to_string(),
            attributes: new_attrs,
        };

        let changes = BreakingChangeDetector::check_schema_breaking_changes(&old_schema, &new_schema);

        println!("Breaking Changes Detected: {}", changes.len());
        for change in &changes {
            println!("  {:?}: {}", change.kind, change.impact);
            println!("    Migration: {}", change.migration_guide);
        }

        assert!(changes.len() >= 2, "Should detect removed attribute and removed enum value");
    }
}
