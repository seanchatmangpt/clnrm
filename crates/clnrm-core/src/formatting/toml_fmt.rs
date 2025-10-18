//! TOML formatting module for Cleanroom v0.7.0
//!
//! Provides deterministic TOML formatting with:
//! - Alphabetically sorted keys within sections
//! - Comment preservation using toml_edit
//! - Consistent indentation and spacing
//! - Idempotent formatting (fmt(fmt(x)) == fmt(x))

use crate::error::{CleanroomError, Result};
use std::collections::BTreeMap;
use std::path::Path;
use toml_edit::{DocumentMut, Item};

/// Format a TOML file with deterministic rules
pub fn format_toml_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!("Failed to read file {}: {}", path.display(), e))
    })?;

    format_toml_content(&content)
}

/// Format TOML content string
pub fn format_toml_content(content: &str) -> Result<String> {
    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| CleanroomError::serialization_error(format!("Failed to parse TOML: {}", e)))?;

    // Sort all tables recursively
    sort_document(&mut doc)?;

    // Convert to string with proper formatting
    let formatted = doc.to_string();

    // Apply additional formatting rules
    apply_formatting_rules(&formatted)
}

/// Sort all tables in the document recursively
fn sort_document(doc: &mut DocumentMut) -> Result<()> {
    // Sort root table
    let root = doc.as_table_mut();
    sort_table_recursive(root)?;

    Ok(())
}

/// Sort a table and all nested tables recursively
fn sort_table_recursive(table: &mut toml_edit::Table) -> Result<()> {
    // Collect all keys and values
    let mut entries: Vec<(String, Item)> = table
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();

    // Sort entries by key
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Clear the table
    let keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();
    for key in keys {
        table.remove(&key);
    }

    // Re-insert in sorted order and recurse into nested tables
    for (key, mut value) in entries {
        // If value is a table, sort it recursively
        if let Some(nested_table) = value.as_table_mut() {
            sort_table_recursive(nested_table)?;
        } else if let Some(inline_table) = value.as_inline_table_mut() {
            sort_inline_table(inline_table)?;
        } else if let Some(array) = value.as_array_mut() {
            // Sort inline tables within arrays
            for item in array.iter_mut() {
                if let Some(inline_table) = item.as_inline_table_mut() {
                    sort_inline_table(inline_table)?;
                }
            }
        }

        table.insert(&key, value);
    }

    Ok(())
}

/// Sort an inline table
fn sort_inline_table(table: &mut toml_edit::InlineTable) -> Result<()> {
    let mut entries: BTreeMap<String, toml_edit::Value> = BTreeMap::new();

    // Collect entries
    for (key, value) in table.iter() {
        entries.insert(key.to_string(), value.clone());
    }

    // Clear and re-insert in sorted order
    table.clear();
    for (key, value) in entries {
        table.insert(&key, value);
    }

    Ok(())
}

/// Apply additional formatting rules
fn apply_formatting_rules(content: &str) -> Result<String> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Remove trailing whitespace from all lines
    for line in &mut lines {
        *line = line.trim_end().to_string();
    }

    // Ensure spacing around = in key-value pairs
    for line in &mut lines {
        if line.contains('=') && !line.trim_start().starts_with('#') {
            // Split on = and rejoin with proper spacing
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim_end();
                let value = parts[1].trim_start();
                *line = format!("{} = {}", key, value);
            }
        }
    }

    // Join lines back together
    let mut result = lines.join("\n");

    // Ensure file ends with newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

/// Check if a file needs formatting
pub fn needs_formatting(path: &Path) -> Result<bool> {
    let original = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!("Failed to read file {}: {}", path.display(), e))
    })?;

    let formatted = format_toml_content(&original)?;

    Ok(original != formatted)
}

/// Verify idempotency: formatting twice should produce same result
pub fn verify_idempotency(content: &str) -> Result<bool> {
    let first_pass = format_toml_content(content)?;
    let second_pass = format_toml_content(&first_pass)?;

    Ok(first_pass == second_pass)
}
