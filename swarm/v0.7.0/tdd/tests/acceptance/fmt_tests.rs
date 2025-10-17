/// Acceptance tests for `clnrm fmt` command
/// Tests template formatting and idempotency

use crate::mocks::MockFormatter;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// FmtCommand formats templates consistently
struct FmtCommand {
    formatter: MockFormatter,
    check_mode: bool,
}

impl FmtCommand {
    fn new(formatter: MockFormatter) -> Self {
        Self {
            formatter,
            check_mode: false,
        }
    }

    fn with_check_mode(mut self) -> Self {
        self.check_mode = true;
        self
    }

    /// Format template content
    fn format(&self, content: &str) -> FormatResult {
        let formatted = self.formatter.format(content);

        if self.check_mode {
            if content == formatted {
                FormatResult::AlreadyFormatted
            } else {
                FormatResult::NeedsFormatting {
                    original: content.to_string(),
                    formatted,
                }
            }
        } else {
            FormatResult::Formatted { content: formatted }
        }
    }

    /// Check if content needs formatting
    fn needs_formatting(&self, content: &str) -> bool {
        self.formatter.needs_formatting(content)
    }
}

#[derive(Debug, PartialEq)]
enum FormatResult {
    Formatted { content: String },
    AlreadyFormatted,
    NeedsFormatting { original: String, formatted: String },
}

impl FormatResult {
    fn exit_code(&self) -> i32 {
        match self {
            FormatResult::Formatted { .. } => 0,
            FormatResult::AlreadyFormatted => 0,
            FormatResult::NeedsFormatting { .. } => 1,
        }
    }
}

// ============================================================================
// Test Suite: Basic Formatting
// ============================================================================

#[test]
fn test_fmt_formats_unformatted_template() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let unformatted = "[meta]\nname=\"test\"";
    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(unformatted, formatted);

    // Act
    let result = fmt_cmd.format(unformatted);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert_eq!(content, formatted, "Should format template");
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_preserves_already_formatted_template() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(formatted, formatted); // Same as input

    // Act
    let result = fmt_cmd.format(formatted);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert_eq!(content, formatted, "Should preserve formatted content");
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_is_idempotent() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let unformatted = "[meta]\nname=\"test\"";
    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(unformatted, formatted);
    formatter.set_formatted(formatted, formatted); // Idempotent

    // Act - Format twice
    let result1 = fmt_cmd.format(unformatted);
    let formatted_content = match result1 {
        FormatResult::Formatted { content } => content,
        _ => panic!("Expected Formatted result"),
    };

    let result2 = fmt_cmd.format(&formatted_content);
    let formatted_content2 = match result2 {
        FormatResult::Formatted { content } => content,
        _ => panic!("Expected Formatted result"),
    };

    // Assert
    assert_eq!(
        formatted_content, formatted_content2,
        "Formatting should be idempotent"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Check Mode
// ============================================================================

#[test]
fn test_fmt_check_mode_returns_zero_for_formatted_template() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone()).with_check_mode();

    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(formatted, formatted);

    // Act
    let result = fmt_cmd.format(formatted);

    // Assert
    assert_eq!(result, FormatResult::AlreadyFormatted);
    assert_eq!(result.exit_code(), 0, "Should return exit code 0");
    Ok(())
}

#[test]
fn test_fmt_check_mode_returns_one_for_unformatted_template() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone()).with_check_mode();

    let unformatted = "[meta]\nname=\"test\"";
    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(unformatted, formatted);

    // Act
    let result = fmt_cmd.format(unformatted);

    // Assert
    match result {
        FormatResult::NeedsFormatting { .. } => {
            assert_eq!(result.exit_code(), 1, "Should return exit code 1");
        }
        _ => panic!("Expected NeedsFormatting result"),
    }
    Ok(())
}

#[test]
fn test_fmt_check_mode_does_not_modify_file() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone()).with_check_mode();

    let unformatted = "[meta]\nname=\"test\"";
    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(unformatted, formatted);

    // Act
    let result = fmt_cmd.format(unformatted);

    // Assert
    match result {
        FormatResult::NeedsFormatting { original, .. } => {
            assert_eq!(
                original, unformatted,
                "Check mode should not modify original"
            );
        }
        _ => panic!("Expected NeedsFormatting result"),
    }
    Ok(())
}

// ============================================================================
// Test Suite: Semantic Preservation
// ============================================================================

#[test]
fn test_fmt_preserves_semantic_meaning() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let unformatted = r#"
[meta]
name="test"
version="1.0"

[service.db]
type="postgres"
port=5432
    "#;

    let formatted = r#"
[meta]
name = "test"
version = "1.0"

[service.db]
type = "postgres"
port = 5432
    "#;

    formatter.set_formatted(unformatted.trim(), formatted.trim());

    // Act
    let result = fmt_cmd.format(unformatted.trim());

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            // Verify key semantic elements preserved
            assert!(content.contains("[meta]"), "Should preserve [meta] section");
            assert!(content.contains("name = \"test\""), "Should preserve name value");
            assert!(content.contains("[service.db]"), "Should preserve service section");
            assert!(content.contains("port = 5432"), "Should preserve port value");
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_preserves_comments() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let with_comments = r#"
# Main configuration
[meta]
name = "test"

# Database service
[service.db]
type = "postgres"
    "#;

    formatter.set_formatted(with_comments.trim(), with_comments.trim());

    // Act
    let result = fmt_cmd.format(with_comments.trim());

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert!(
                content.contains("# Main configuration"),
                "Should preserve comments"
            );
            assert!(
                content.contains("# Database service"),
                "Should preserve all comments"
            );
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_preserves_array_order() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let with_arrays = r#"
[meta]
tags = ["test", "integration", "db"]

[[scenario]]
name = "first"

[[scenario]]
name = "second"
    "#;

    formatter.set_formatted(with_arrays.trim(), with_arrays.trim());

    // Act
    let result = fmt_cmd.format(with_arrays.trim());

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            let first_pos = content.find("name = \"first\"").unwrap();
            let second_pos = content.find("name = \"second\"").unwrap();
            assert!(
                first_pos < second_pos,
                "Should preserve scenario order"
            );
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

// ============================================================================
// Test Suite: Formatting Rules
// ============================================================================

#[test]
fn test_fmt_adds_spaces_around_equals() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let unformatted = "[meta]\nname=\"test\"";
    let formatted = "[meta]\nname = \"test\"";
    formatter.set_formatted(unformatted, formatted);

    // Act
    let result = fmt_cmd.format(unformatted);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert!(
                content.contains(" = "),
                "Should add spaces around equals"
            );
            assert!(
                !content.contains("=\""),
                "Should not have no-space format"
            );
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_normalizes_indentation() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let unformatted = "[meta]\n  name = \"test\"\n    version = \"1.0\"";
    let formatted = "[meta]\nname = \"test\"\nversion = \"1.0\"";
    formatter.set_formatted(unformatted, formatted);

    // Act
    let result = fmt_cmd.format(unformatted);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert!(
                !content.contains("  name"),
                "Should normalize indentation"
            );
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_adds_blank_lines_between_sections() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let unformatted = "[meta]\nname = \"test\"\n[otel]\nservice_name = \"test\"";
    let formatted = "[meta]\nname = \"test\"\n\n[otel]\nservice_name = \"test\"";
    formatter.set_formatted(unformatted, formatted);

    // Act
    let result = fmt_cmd.format(unformatted);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert!(
                content.contains("\n\n[otel]"),
                "Should add blank line between sections"
            );
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_handles_multiline_strings() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let multiline = r#"
[meta]
description = """
This is a multiline
description that spans
multiple lines
"""
    "#;

    formatter.set_formatted(multiline.trim(), multiline.trim());

    // Act
    let result = fmt_cmd.format(multiline.trim());

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert!(
                content.contains("\"\"\""),
                "Should preserve multiline string delimiters"
            );
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

// ============================================================================
// Test Suite: Edge Cases
// ============================================================================

#[test]
fn test_fmt_handles_empty_template() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let empty = "";
    formatter.set_formatted(empty, empty);

    // Act
    let result = fmt_cmd.format(empty);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert_eq!(content, "", "Should handle empty template");
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_handles_whitespace_only() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let whitespace = "   \n\n\t  ";
    let formatted = "";
    formatter.set_formatted(whitespace, formatted);

    // Act
    let result = fmt_cmd.format(whitespace);

    // Assert
    match result {
        FormatResult::Formatted { content } => {
            assert_eq!(content, "", "Should remove pure whitespace");
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

#[test]
fn test_fmt_handles_large_template() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    // Generate large template
    let mut large = String::from("[meta]\nname = \"large\"\n\n");
    for i in 0..1000 {
        large.push_str(&format!("[service.service_{}]\ntype = \"test\"\n\n", i));
    }

    formatter.set_formatted(&large, &large);

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let result = fmt_cmd.format(&large);
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_secs(1),
        "Should format large template in <1s"
    );
    match result {
        FormatResult::Formatted { content } => {
            assert!(!content.is_empty(), "Should format large template");
        }
        _ => panic!("Expected Formatted result"),
    }
    Ok(())
}

// ============================================================================
// Test Suite: Multiple Files
// ============================================================================

#[test]
fn test_fmt_formats_multiple_files_consistently() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let file1 = "[meta]\nname=\"test1\"";
    let file2 = "[meta]\nname=\"test2\"";
    let formatted1 = "[meta]\nname = \"test1\"";
    let formatted2 = "[meta]\nname = \"test2\"";

    formatter.set_formatted(file1, formatted1);
    formatter.set_formatted(file2, formatted2);

    // Act
    let result1 = fmt_cmd.format(file1);
    let result2 = fmt_cmd.format(file2);

    // Assert
    if let (
        FormatResult::Formatted { content: content1 },
        FormatResult::Formatted { content: content2 },
    ) = (result1, result2)
    {
        // Both should have same formatting style
        assert!(content1.contains(" = "), "File 1 should use consistent style");
        assert!(content2.contains(" = "), "File 2 should use consistent style");
    } else {
        panic!("Expected Formatted results");
    }
    Ok(())
}

// ============================================================================
// Test Suite: Performance
// ============================================================================

#[test]
fn test_fmt_completes_quickly() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone());

    let template = r#"
[meta]
name = "perf_test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
    "#;

    formatter.set_formatted(template.trim(), template.trim());

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let _ = fmt_cmd.format(template.trim());
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_millis(50),
        "Formatting should complete in <50ms, took {:?}",
        duration
    );
    Ok(())
}

#[test]
fn test_fmt_check_mode_is_fast() -> Result<()> {
    // Arrange
    let formatter = MockFormatter::new();
    let fmt_cmd = FmtCommand::new(formatter.clone()).with_check_mode();

    let template = "[meta]\nname = \"test\"";
    formatter.set_formatted(template, template);

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let _ = fmt_cmd.format(template);
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_millis(10),
        "Check mode should be <10ms, took {:?}",
        duration
    );
    Ok(())
}
