//! Comprehensive tests for TOML formatting module
//!
//! Test coverage includes:
//! - Format correctness (alphabetical sorting, spacing, indentation)
//! - Format idempotency (formatting twice produces same result)
//! - Comment preservation
//! - Nested table handling
//! - Inline table sorting
//! - Array formatting
//! - Edge cases (empty files, malformed TOML, special characters)
//! - Performance (large files)

use clnrm_core::error::Result;
use clnrm_core::formatting::{
    format_toml_content, format_toml_file, needs_formatting, verify_idempotency,
};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Basic Formatting Tests
// ============================================================================

#[test]
fn test_format_empty_toml() -> Result<()> {
    // Arrange
    let input = "";

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert_eq!(formatted, "\n");
    Ok(())
}

#[test]
fn test_format_single_key_value() -> Result<()> {
    // Arrange
    let input = "name=\"test\"";

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("name = \"test\""));
    assert!(formatted.ends_with('\n'));
    Ok(())
}

#[test]
fn test_format_sorts_keys_alphabetically() -> Result<()> {
    // Arrange
    let input = r#"
z_key = "last"
a_key = "first"
m_key = "middle"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    let a_pos = formatted.find("a_key").unwrap();
    let m_pos = formatted.find("m_key").unwrap();
    let z_pos = formatted.find("z_key").unwrap();

    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
    Ok(())
}

#[test]
fn test_format_adds_spaces_around_equals() -> Result<()> {
    // Arrange
    let input = "name=\"test\"\nversion=\"1.0.0\"\n";

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("name = \"test\""));
    assert!(formatted.contains("version = \"1.0.0\""));
    Ok(())
}

#[test]
fn test_format_removes_trailing_whitespace() -> Result<()> {
    // Arrange
    let input = "name = \"test\"   \nversion = \"1.0.0\"  \n";

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    for line in formatted.lines() {
        assert_eq!(line, line.trim_end());
    }
    Ok(())
}

#[test]
fn test_format_ensures_trailing_newline() -> Result<()> {
    // Arrange
    let input = "name = \"test\"";

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.ends_with('\n'));
    Ok(())
}

// ============================================================================
// Idempotency Tests
// ============================================================================

#[test]
fn test_idempotency_simple_toml() -> Result<()> {
    // Arrange
    let input = r#"
version = "0.7.0"
name = "test"
"#;

    // Act & Assert
    assert!(verify_idempotency(input)?);
    Ok(())
}

#[test]
fn test_idempotency_with_sections() -> Result<()> {
    // Arrange
    let input = r#"
[meta]
name = "test"
version = "0.7.0"

[otel]
exporter = "stdout"
sample_ratio = 1.0
"#;

    // Act & Assert
    assert!(verify_idempotency(input)?);
    Ok(())
}

#[test]
fn test_idempotency_with_nested_tables() -> Result<()> {
    // Arrange
    let input = r#"
[parent]
key1 = "value1"
key2 = "value2"

[parent.child]
nested_key = "nested_value"
"#;

    // Act & Assert
    assert!(verify_idempotency(input)?);
    Ok(())
}

#[test]
fn test_format_is_idempotent() -> Result<()> {
    // Arrange
    let input = r#"
z = "3"
a = "1"
m = "2"
"#;

    // Act
    let first_pass = format_toml_content(input)?;
    let second_pass = format_toml_content(&first_pass)?;
    let third_pass = format_toml_content(&second_pass)?;

    // Assert
    assert_eq!(first_pass, second_pass);
    assert_eq!(second_pass, third_pass);
    Ok(())
}

// ============================================================================
// Comment Preservation Tests
// ============================================================================

#[test]
fn test_preserves_inline_comments() -> Result<()> {
    // Arrange
    let input = r#"
name = "test"  # This is a comment
version = "1.0.0"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("# This is a comment"));
    Ok(())
}

#[test]
fn test_preserves_block_comments() -> Result<()> {
    // Arrange
    let input = r#"
# This is a block comment
# spanning multiple lines
name = "test"
version = "1.0.0"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("# This is a block comment"));
    assert!(formatted.contains("# spanning multiple lines"));
    Ok(())
}

#[test]
fn test_preserves_section_header_comments() -> Result<()> {
    // Arrange
    let input = r#"
# Meta configuration
[meta]
name = "test"
version = "1.0.0"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("# Meta configuration"));
    Ok(())
}

// ============================================================================
// Nested Table Tests
// ============================================================================

#[test]
fn test_format_nested_tables() -> Result<()> {
    // Arrange
    let input = r#"
[parent]
z_key = "last"
a_key = "first"

[parent.child]
z_nested = "last"
a_nested = "first"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    let parent_section = formatted.split("[parent]").nth(1).unwrap();
    let a_pos = parent_section.find("a_key").unwrap();
    let z_pos = parent_section.find("z_key").unwrap();
    assert!(a_pos < z_pos);

    let child_section = formatted.split("[parent.child]").nth(1).unwrap();
    let a_nested_pos = child_section.find("a_nested").unwrap();
    let z_nested_pos = child_section.find("z_nested").unwrap();
    assert!(a_nested_pos < z_nested_pos);

    Ok(())
}

#[test]
fn test_format_deeply_nested_tables() -> Result<()> {
    // Arrange
    let input = r#"
[level1]
z = "1"
a = "2"

[level1.level2]
z = "1"
a = "2"

[level1.level2.level3]
z = "1"
a = "2"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("[level1]"));
    assert!(formatted.contains("[level1.level2]"));
    assert!(formatted.contains("[level1.level2.level3]"));

    // Each section should have sorted keys
    for section in ["level1", "level1.level2", "level1.level2.level3"] {
        let section_marker = format!("[{}]", section);
        if let Some(section_content) = formatted.split(&section_marker).nth(1) {
            if let Some(a_pos) = section_content.find("a = ") {
                if let Some(z_pos) = section_content.find("z = ") {
                    assert!(a_pos < z_pos);
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Inline Table Tests
// ============================================================================

#[test]
fn test_format_inline_tables() -> Result<()> {
    // Arrange
    let input = r#"
point = { z = 3, y = 2, x = 1 }
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("point = "));
    // Inline tables should be sorted: x, y, z
    let point_value = formatted.split("point = ").nth(1).unwrap();
    let x_pos = point_value.find("x").unwrap();
    let y_pos = point_value.find("y").unwrap();
    let z_pos = point_value.find("z").unwrap();
    assert!(x_pos < y_pos);
    assert!(y_pos < z_pos);

    Ok(())
}

#[test]
fn test_format_multiple_inline_tables() -> Result<()> {
    // Arrange
    let input = r#"
point1 = { z = 3, x = 1 }
point2 = { y = 2, a = 0 }
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("point1 = "));
    assert!(formatted.contains("point2 = "));

    // Each inline table should be sorted
    let point1_value = formatted.split("point1 = ").nth(1).unwrap().split('\n').next().unwrap();
    let x_pos = point1_value.find("x").unwrap();
    let z_pos = point1_value.find("z").unwrap();
    assert!(x_pos < z_pos);

    Ok(())
}

// ============================================================================
// Array Tests
// ============================================================================

#[test]
fn test_format_array_of_tables() -> Result<()> {
    // Arrange
    let input = r#"
[[steps]]
z_field = "last"
a_field = "first"
name = "step1"

[[steps]]
z_field = "last"
a_field = "first"
name = "step2"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("[[steps]]"));

    // Keys within array tables should be sorted
    let steps_sections: Vec<_> = formatted.split("[[steps]]").collect();
    for section in steps_sections.iter().skip(1) {
        if let Some(a_pos) = section.find("a_field") {
            if let Some(n_pos) = section.find("name") {
                if let Some(z_pos) = section.find("z_field") {
                    assert!(a_pos < n_pos);
                    assert!(n_pos < z_pos);
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_format_preserves_array_structure() -> Result<()> {
    // Arrange
    let input = r#"
[[scenario]]
name = "test1"
steps = [
    { command = ["echo", "hello"], name = "step1" }
]

[[scenario]]
name = "test2"
steps = []
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("[[scenario]]"));
    assert!(formatted.contains("steps = "));

    Ok(())
}

#[test]
fn test_format_inline_tables_in_arrays() -> Result<()> {
    // Arrange
    let input = r#"
items = [
    { z = 3, a = 1 },
    { z = 6, a = 4 }
]
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("items = "));

    // Inline tables in arrays should be sorted
    let items_section = formatted.split("items = ").nth(1).unwrap();
    // Each inline table should have 'a' before 'z'
    for table in items_section.split('}') {
        if table.contains('a') && table.contains('z') {
            let a_pos = table.find("a").unwrap();
            let z_pos = table.find("z").unwrap();
            assert!(a_pos < z_pos);
        }
    }

    Ok(())
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_format_handles_special_characters() -> Result<()> {
    // Arrange
    let input = r#"
message = "Hello\nWorld\t!"
path = "C:\\Users\\test"
unicode = "Hello 世界"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("message = "));
    assert!(formatted.contains("path = "));
    assert!(formatted.contains("unicode = "));

    Ok(())
}

#[test]
fn test_format_handles_multiline_strings() -> Result<()> {
    // Arrange
    let input = r#"
description = """
This is a multiline
string with multiple
lines of text.
"""
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("description = "));
    assert!(formatted.contains("\"\"\""));

    Ok(())
}

#[test]
fn test_format_handles_numbers_and_booleans() -> Result<()> {
    // Arrange
    let input = r#"
integer = 42
float = 3.14
boolean_true = true
boolean_false = false
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("boolean_false = false"));
    assert!(formatted.contains("boolean_true = true"));
    assert!(formatted.contains("float = 3.14"));
    assert!(formatted.contains("integer = 42"));

    Ok(())
}

#[test]
fn test_format_handles_dates() -> Result<()> {
    // Arrange
    let input = r#"
date = 2023-01-15
datetime = 2023-01-15T10:30:00Z
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("date = "));
    assert!(formatted.contains("datetime = "));

    Ok(())
}

#[test]
fn test_format_malformed_toml_returns_error() {
    // Arrange
    let input = r#"
[malformed
key without value
"#;

    // Act
    let result = format_toml_content(input);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_format_invalid_syntax_returns_error() {
    // Arrange
    let input = "key = [unclosed array";

    // Act
    let result = format_toml_content(input);

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// File Operations Tests
// ============================================================================

#[test]
fn test_format_toml_file_reads_and_formats() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let content = r#"
z_key = "last"
a_key = "first"
"#;

    fs::write(&file_path, content).unwrap();

    // Act
    let formatted = format_toml_file(&file_path)?;

    // Assert
    let a_pos = formatted.find("a_key").unwrap();
    let z_pos = formatted.find("z_key").unwrap();
    assert!(a_pos < z_pos);

    Ok(())
}

#[test]
fn test_format_toml_file_nonexistent_returns_error() {
    // Arrange
    let path = std::path::Path::new("/nonexistent/file.toml");

    // Act
    let result = format_toml_file(path);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_needs_formatting_detects_unformatted_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let unformatted = "z=\"last\"\na=\"first\"\n";
    fs::write(&file_path, unformatted).unwrap();

    // Act
    let needs_fmt = needs_formatting(&file_path)?;

    // Assert
    assert!(needs_fmt);

    Ok(())
}

#[test]
fn test_needs_formatting_formatted_file_returns_false() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    // Write formatted content
    let formatted = "a = \"first\"\nz = \"last\"\n";
    fs::write(&file_path, formatted).unwrap();

    // Format once to ensure it's properly formatted
    let content = format_toml_file(&file_path)?;
    fs::write(&file_path, content).unwrap();

    // Act
    let needs_fmt = needs_formatting(&file_path)?;

    // Assert
    assert!(!needs_fmt);

    Ok(())
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_format_large_file_with_many_keys() -> Result<()> {
    // Arrange
    let mut input = String::new();
    for i in (0..1000).rev() {
        input.push_str(&format!("key_{} = \"{}\"\n", i, i));
    }

    // Act
    let formatted = format_toml_content(&input)?;

    // Assert
    // Verify sorting: key_0 should come before key_999
    let key_0_pos = formatted.find("key_0 = ").unwrap();
    let key_999_pos = formatted.find("key_999 = ").unwrap();
    assert!(key_0_pos < key_999_pos);

    Ok(())
}

#[test]
fn test_format_large_file_with_many_sections() -> Result<()> {
    // Arrange
    let mut input = String::new();
    for i in (0..100).rev() {
        input.push_str(&format!("\n[section_{}]\n", i));
        input.push_str(&format!("z_key = \"{}\"\n", i));
        input.push_str(&format!("a_key = \"{}\"\n", i));
    }

    // Act
    let formatted = format_toml_content(&input)?;

    // Assert
    // Verify all sections exist
    for i in 0..100 {
        assert!(formatted.contains(&format!("[section_{}]", i)));
    }

    // Verify keys are sorted within each section
    for i in 0..100 {
        let section_marker = format!("[section_{}]", i);
        if let Some(section_start) = formatted.find(&section_marker) {
            let section_content = &formatted[section_start..];
            if let Some(next_section) = section_content.find("\n[") {
                let section_text = &section_content[..next_section];
                if let Some(a_pos) = section_text.find("a_key") {
                    if let Some(z_pos) = section_text.find("z_key") {
                        assert!(a_pos < z_pos);
                    }
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Real-World Configuration Tests
// ============================================================================

#[test]
fn test_format_clnrm_config() -> Result<()> {
    // Arrange
    let input = r#"
[meta]
version = "0.7.0"
name = "test_config"
description = "Test cleanroom configuration"

[otel]
sample_ratio = 1.0
exporter = "stdout"

[[scenario]]
timeout_ms = 5000
name = "test_scenario"

[[scenario.steps]]
service = "test_service"
name = "step1"
command = ["echo", "hello"]
expected_output_regex = "hello"

[services.test_service]
type = "generic_container"
image = "alpine:latest"
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    // Verify sections exist
    assert!(formatted.contains("[meta]"));
    assert!(formatted.contains("[otel]"));
    assert!(formatted.contains("[[scenario]]"));
    assert!(formatted.contains("[services.test_service]"));

    // Verify keys are sorted in meta section
    let meta_section = formatted.split("[meta]").nth(1).unwrap();
    let description_pos = meta_section.find("description").unwrap();
    let name_pos = meta_section.find("name").unwrap();
    let version_pos = meta_section.find("version").unwrap();
    assert!(description_pos < name_pos);
    assert!(name_pos < version_pos);

    // Verify idempotency
    assert!(verify_idempotency(&formatted)?);

    Ok(())
}

#[test]
fn test_format_v0_7_0_expectations_config() -> Result<()> {
    // Arrange
    let input = r#"
[expect.count]
spans_total = { eq = 5 }
errors_total = { eq = 0 }

[[expect.span]]
name = "clnrm.run"
attributes = { "test.name" = "my_test" }

[expect.order]
must_precede = [["clnrm.init", "clnrm.run"]]

[[expect.window]]
span_names = ["clnrm.*"]
duration_ms = { gte = 100, lte = 5000 }
"#;

    // Act
    let formatted = format_toml_content(input)?;

    // Assert
    assert!(formatted.contains("[expect.count]"));
    assert!(formatted.contains("[[expect.span]]"));
    assert!(formatted.contains("[expect.order]"));
    assert!(formatted.contains("[[expect.window]]"));

    // Verify idempotency
    assert!(verify_idempotency(&formatted)?);

    Ok(())
}
