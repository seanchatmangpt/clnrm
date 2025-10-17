//! Diff command for trace comparison
//!
//! Compares two OpenTelemetry traces to detect regressions.

use crate::error::{CleanroomError, Result};
use std::path::Path;

/// Result of trace comparison
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Number of spans added
    pub added_count: usize,
    /// Number of spans removed
    pub removed_count: usize,
    /// Number of spans modified
    pub modified_count: usize,
    /// Added spans
    pub added: Vec<String>,
    /// Removed spans
    pub removed: Vec<String>,
    /// Modified spans
    pub modified: Vec<String>,
}

/// Compare two traces
pub fn diff_traces(
    baseline: &Path,
    current: &Path,
    format: &str,
    only_changes: bool,
) -> Result<DiffResult> {
    // Read baseline and current traces
    let baseline_content = std::fs::read_to_string(baseline)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read baseline: {}", e)))?;

    let current_content = std::fs::read_to_string(current)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read current: {}", e)))?;

    // Parse JSON traces
    let baseline_json: serde_json::Value =
        serde_json::from_str(&baseline_content).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse baseline JSON: {}", e))
        })?;

    let current_json: serde_json::Value = serde_json::from_str(&current_content).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to parse current JSON: {}", e))
    })?;

    // Extract span names
    let baseline_spans = extract_span_names(&baseline_json);
    let current_spans = extract_span_names(&current_json);

    // Compute differences
    let added: Vec<String> = current_spans
        .iter()
        .filter(|s| !baseline_spans.contains(s))
        .cloned()
        .collect();

    let removed: Vec<String> = baseline_spans
        .iter()
        .filter(|s| !current_spans.contains(s))
        .cloned()
        .collect();

    // For now, we don't detect modifications (would need deeper analysis)
    let modified = Vec::new();

    let result = DiffResult {
        added_count: added.len(),
        removed_count: removed.len(),
        modified_count: modified.len(),
        added,
        removed,
        modified,
    };

    // Display results
    match format {
        "json" => {
            let json = serde_json::json!({
                "added_count": result.added_count,
                "removed_count": result.removed_count,
                "modified_count": result.modified_count,
                "added": result.added,
                "removed": result.removed,
                "modified": result.modified,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json).map_err(|e| {
                    CleanroomError::serialization_error(format!("Failed to serialize JSON: {}", e))
                })?
            );
        }
        _ => {
            // Human-readable format
            if result.added_count > 0 {
                println!("Added spans ({}):", result.added_count);
                for span in &result.added {
                    println!("  + {}", span);
                }
            }

            if result.removed_count > 0 {
                println!("Removed spans ({}):", result.removed_count);
                for span in &result.removed {
                    println!("  - {}", span);
                }
            }

            if result.modified_count > 0 {
                println!("Modified spans ({}):", result.modified_count);
                for span in &result.modified {
                    println!("  ~ {}", span);
                }
            }

            if !only_changes
                || result.added_count + result.removed_count + result.modified_count == 0
            {
                println!(
                    "\nSummary: {} added, {} removed, {} modified",
                    result.added_count, result.removed_count, result.modified_count
                );
            }
        }
    }

    Ok(result)
}

/// Extract span names from JSON trace
fn extract_span_names(json: &serde_json::Value) -> Vec<String> {
    let mut spans = Vec::new();

    if let Some(array) = json.as_array() {
        for item in array {
            if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                spans.push(name.to_string());
            }
        }
    } else if let Some(obj) = json.as_object() {
        if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
            spans.push(name.to_string());
        }

        // Recursively extract from nested structures
        for (_, value) in obj {
            spans.extend(extract_span_names(value));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_diff_identical_traces() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

        let baseline_file = temp_dir.path().join("baseline.json");
        let current_file = temp_dir.path().join("current.json");

        let trace_json = r#"[
            {"name": "span1"},
            {"name": "span2"}
        ]"#;

        fs::write(&baseline_file, trace_json)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;
        fs::write(&current_file, trace_json)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

        // Act
        let result = diff_traces(
            baseline_file.as_path(),
            current_file.as_path(),
            "text",
            false,
        )?;

        // Assert
        assert_eq!(result.added_count, 0);
        assert_eq!(result.removed_count, 0);
        assert_eq!(result.modified_count, 0);

        Ok(())
    }

    #[test]
    fn test_diff_detects_added_span() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

        let baseline_file = temp_dir.path().join("baseline.json");
        let current_file = temp_dir.path().join("current.json");

        fs::write(&baseline_file, r#"[{"name": "span1"}]"#)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;
        fs::write(&current_file, r#"[{"name": "span1"}, {"name": "span2"}]"#)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

        // Act
        let result = diff_traces(
            baseline_file.as_path(),
            current_file.as_path(),
            "text",
            false,
        )?;

        // Assert
        assert_eq!(result.added_count, 1);
        assert_eq!(result.removed_count, 0);
        assert!(result.added.contains(&"span2".to_string()));

        Ok(())
    }
}
