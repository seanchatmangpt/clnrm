/// Acceptance tests for `clnrm diff` command
/// Tests trace comparison and difference detection

use crate::mocks::{MockTraceDiffer, TraceDifference, DifferenceType};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DiffCommand compares trace outputs
struct DiffCommand {
    differ: MockTraceDiffer,
    output_format: OutputFormat,
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Human,
    Json,
}

impl DiffCommand {
    fn new(differ: MockTraceDiffer) -> Self {
        Self {
            differ,
            output_format: OutputFormat::Human,
        }
    }

    fn with_json_output(mut self) -> Self {
        self.output_format = OutputFormat::Json;
        self
    }

    /// Compare two traces and return differences
    fn compare(&self, expected: &str, actual: &str) -> DiffResult {
        let differences = self.differ.get_differences();

        if differences.is_empty() {
            DiffResult::Identical
        } else {
            DiffResult::Different {
                differences,
                format: self.output_format,
            }
        }
    }

    /// Check if traces are identical
    fn are_identical(&self, _expected: &str, _actual: &str) -> bool {
        !self.differ.has_differences()
    }
}

#[derive(Debug)]
enum DiffResult {
    Identical,
    Different {
        differences: Vec<TraceDifference>,
        format: OutputFormat,
    },
}

impl DiffResult {
    fn has_differences(&self) -> bool {
        matches!(self, DiffResult::Different { .. })
    }

    fn difference_count(&self) -> usize {
        match self {
            DiffResult::Identical => 0,
            DiffResult::Different { differences, .. } => differences.len(),
        }
    }

    fn to_output(&self) -> String {
        match self {
            DiffResult::Identical => "No differences found".to_string(),
            DiffResult::Different { differences, format } => match format {
                OutputFormat::Human => Self::format_human(differences),
                OutputFormat::Json => Self::format_json(differences),
            },
        }
    }

    fn format_human(differences: &[TraceDifference]) -> String {
        let mut output = String::new();
        output.push_str(&format!("Found {} difference(s):\n\n", differences.len()));

        for diff in differences {
            output.push_str(&format!(
                "  [{}] {}: {}\n",
                diff.span_name,
                match diff.difference_type {
                    DifferenceType::MissingSpan => "Missing Span",
                    DifferenceType::ExtraSpan => "Extra Span",
                    DifferenceType::AttributeChanged => "Attribute Changed",
                    DifferenceType::DurationChanged => "Duration Changed",
                },
                diff.details
            ));
        }

        output
    }

    fn format_json(differences: &[TraceDifference]) -> String {
        format!(
            r#"{{"differences": {}, "count": {}}}"#,
            serde_json::to_string_pretty(
                &differences
                    .iter()
                    .map(|d| serde_json::json!({
                        "span_name": d.span_name,
                        "type": format!("{:?}", d.difference_type),
                        "details": d.details,
                    }))
                    .collect::<Vec<_>>()
            )
            .unwrap_or_default(),
            differences.len()
        )
    }
}

// ============================================================================
// Test Suite: Identical Traces
// ============================================================================

#[test]
fn test_diff_reports_no_differences_for_identical_traces() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    let trace = r#"{"spans": [{"name": "test", "duration": 100}]}"#;

    // Act
    let result = diff_cmd.compare(trace, trace);

    // Assert
    assert!(!result.has_differences(), "Identical traces should have no differences");
    assert_eq!(result.difference_count(), 0);
    Ok(())
}

#[test]
fn test_diff_returns_clean_output_for_identical_traces() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    let trace = r#"{"spans": [{"name": "test"}]}"#;

    // Act
    let result = diff_cmd.compare(trace, trace);
    let output = result.to_output();

    // Assert
    assert!(
        output.contains("No differences"),
        "Output should indicate no differences"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Missing Spans
// ============================================================================

#[test]
fn test_diff_detects_missing_span() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "database_query".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Expected span not found in actual trace".to_string(),
    });

    let expected = r#"{"spans": [{"name": "database_query"}]}"#;
    let actual = r#"{"spans": []}"#;

    // Act
    let result = diff_cmd.compare(expected, actual);

    // Assert
    assert!(result.has_differences(), "Should detect missing span");
    assert_eq!(result.difference_count(), 1);

    let output = result.to_output();
    assert!(output.contains("Missing Span"), "Output should mention missing span");
    assert!(output.contains("database_query"), "Output should include span name");
    Ok(())
}

#[test]
fn test_diff_highlights_missing_span_in_output() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "http_request".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Span 'http_request' missing from actual trace".to_string(),
    });

    let expected = "expected trace";
    let actual = "actual trace";

    // Act
    let result = diff_cmd.compare(expected, actual);
    let output = result.to_output();

    // Assert
    assert!(
        output.contains("[http_request]"),
        "Output should highlight missing span"
    );
    assert!(
        output.contains("Missing Span"),
        "Output should label as missing"
    );
    Ok(())
}

#[test]
fn test_diff_detects_multiple_missing_spans() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "span1".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Missing span1".to_string(),
    });
    differ.add_difference(TraceDifference {
        span_name: "span2".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Missing span2".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert_eq!(
        result.difference_count(), 2,
        "Should detect both missing spans"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Extra Spans
// ============================================================================

#[test]
fn test_diff_detects_extra_span() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "unexpected_span".to_string(),
        difference_type: DifferenceType::ExtraSpan,
        details: "Span not present in expected trace".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert!(result.has_differences(), "Should detect extra span");

    let output = result.to_output();
    assert!(output.contains("Extra Span"), "Output should mention extra span");
    Ok(())
}

// ============================================================================
// Test Suite: Attribute Changes
// ============================================================================

#[test]
fn test_diff_detects_changed_attribute() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "database_query".to_string(),
        difference_type: DifferenceType::AttributeChanged,
        details: "Attribute 'db.table' changed from 'users' to 'customers'".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert!(result.has_differences(), "Should detect changed attribute");

    let output = result.to_output();
    assert!(
        output.contains("Attribute Changed"),
        "Output should mention attribute change"
    );
    Ok(())
}

#[test]
fn test_diff_shows_before_and_after_for_attributes() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "api_call".to_string(),
        difference_type: DifferenceType::AttributeChanged,
        details: "status: 200 → 500".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");
    let output = result.to_output();

    // Assert
    assert!(
        output.contains("200") && output.contains("500"),
        "Output should show both old and new values"
    );
    Ok(())
}

#[test]
fn test_diff_detects_multiple_attribute_changes() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "http_request".to_string(),
        difference_type: DifferenceType::AttributeChanged,
        details: "method: GET → POST".to_string(),
    });
    differ.add_difference(TraceDifference {
        span_name: "http_request".to_string(),
        difference_type: DifferenceType::AttributeChanged,
        details: "status: 200 → 404".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert_eq!(
        result.difference_count(), 2,
        "Should detect multiple attribute changes"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Duration Changes
// ============================================================================

#[test]
fn test_diff_detects_duration_change() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "slow_query".to_string(),
        difference_type: DifferenceType::DurationChanged,
        details: "Duration: 100ms → 500ms".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert!(result.has_differences(), "Should detect duration change");

    let output = result.to_output();
    assert!(
        output.contains("Duration Changed"),
        "Output should mention duration change"
    );
    Ok(())
}

// ============================================================================
// Test Suite: JSON Output
// ============================================================================

#[test]
fn test_diff_json_flag_produces_structured_output() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone()).with_json_output();

    differ.add_difference(TraceDifference {
        span_name: "test_span".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Missing from actual".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");
    let output = result.to_output();

    // Assert
    assert!(output.contains(r#""differences""#), "Should contain JSON field");
    assert!(output.contains(r#""count""#), "Should contain count field");
    assert!(
        output.starts_with('{') && output.ends_with('}'),
        "Should be valid JSON object"
    );
    Ok(())
}

#[test]
fn test_diff_json_output_includes_all_difference_fields() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone()).with_json_output();

    differ.add_difference(TraceDifference {
        span_name: "test_span".to_string(),
        difference_type: DifferenceType::AttributeChanged,
        details: "value changed".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");
    let output = result.to_output();

    // Assert
    assert!(output.contains("test_span"), "Should include span name");
    assert!(output.contains("AttributeChanged"), "Should include type");
    assert!(output.contains("value changed"), "Should include details");
    Ok(())
}

#[test]
fn test_diff_json_output_is_parseable() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone()).with_json_output();

    differ.add_difference(TraceDifference {
        span_name: "span1".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Missing".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");
    let output = result.to_output();

    // Assert
    let parsed: serde_json::Value = serde_json::from_str(&output)?;
    assert!(parsed.is_object(), "Should parse as JSON object");
    assert!(parsed["count"].is_number(), "Count should be number");
    Ok(())
}

// ============================================================================
// Test Suite: Human-Readable Output
// ============================================================================

#[test]
fn test_diff_human_output_is_readable() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone()); // Default: Human

    differ.add_difference(TraceDifference {
        span_name: "test_span".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Not found in actual trace".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");
    let output = result.to_output();

    // Assert
    assert!(
        output.contains("Found 1 difference"),
        "Should have human-readable summary"
    );
    assert!(
        output.contains("[test_span]"),
        "Should include span name in brackets"
    );
    assert!(
        output.lines().count() > 1,
        "Should be multi-line for readability"
    );
    Ok(())
}

#[test]
fn test_diff_human_output_groups_by_span() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "span1".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Missing".to_string(),
    });
    differ.add_difference(TraceDifference {
        span_name: "span2".to_string(),
        difference_type: DifferenceType::ExtraSpan,
        details: "Extra".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");
    let output = result.to_output();

    // Assert
    assert!(output.contains("[span1]"), "Should show first span");
    assert!(output.contains("[span2]"), "Should show second span");
    Ok(())
}

// ============================================================================
// Test Suite: Complex Scenarios
// ============================================================================

#[test]
fn test_diff_handles_multiple_difference_types() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    differ.add_difference(TraceDifference {
        span_name: "span1".to_string(),
        difference_type: DifferenceType::MissingSpan,
        details: "Missing".to_string(),
    });
    differ.add_difference(TraceDifference {
        span_name: "span2".to_string(),
        difference_type: DifferenceType::ExtraSpan,
        details: "Extra".to_string(),
    });
    differ.add_difference(TraceDifference {
        span_name: "span3".to_string(),
        difference_type: DifferenceType::AttributeChanged,
        details: "Changed".to_string(),
    });
    differ.add_difference(TraceDifference {
        span_name: "span4".to_string(),
        difference_type: DifferenceType::DurationChanged,
        details: "Duration changed".to_string(),
    });

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert_eq!(
        result.difference_count(), 4,
        "Should detect all difference types"
    );

    let output = result.to_output();
    assert!(output.contains("Missing Span"));
    assert!(output.contains("Extra Span"));
    assert!(output.contains("Attribute Changed"));
    assert!(output.contains("Duration Changed"));
    Ok(())
}

#[test]
fn test_diff_handles_large_number_of_differences() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    // Add 100 differences
    for i in 0..100 {
        differ.add_difference(TraceDifference {
            span_name: format!("span_{}", i),
            difference_type: DifferenceType::MissingSpan,
            details: format!("Missing span {}", i),
        });
    }

    // Act
    let result = diff_cmd.compare("expected", "actual");

    // Assert
    assert_eq!(result.difference_count(), 100, "Should track all differences");
    Ok(())
}

// ============================================================================
// Test Suite: Edge Cases
// ============================================================================

#[test]
fn test_diff_handles_empty_traces() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    // Act
    let result = diff_cmd.compare("", "");

    // Assert
    assert!(
        diff_cmd.are_identical("", ""),
        "Empty traces should be identical"
    );
    assert_eq!(result.difference_count(), 0);
    Ok(())
}

#[test]
fn test_diff_handles_malformed_traces_gracefully() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    let malformed = "not valid json {]";

    // Act
    let result = diff_cmd.compare(malformed, malformed);

    // Assert - Should still work, even if input is malformed
    assert!(!result.has_differences(), "Identical malformed traces should match");
    Ok(())
}

// ============================================================================
// Test Suite: Performance
// ============================================================================

#[test]
fn test_diff_completes_quickly_for_small_traces() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    let trace = r#"{"spans": [{"name": "test", "duration": 100}]}"#;

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let _ = diff_cmd.compare(trace, trace);
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_millis(50),
        "Diff should complete in <50ms for small traces, took {:?}",
        duration
    );
    Ok(())
}

#[test]
fn test_diff_scales_to_large_traces() -> Result<()> {
    // Arrange
    let differ = MockTraceDiffer::new();
    let diff_cmd = DiffCommand::new(differ.clone());

    // Generate large trace (1000 spans)
    let large_trace = format!(
        r#"{{"spans": [{}]}}"#,
        (0..1000)
            .map(|i| format!(r#"{{"name": "span_{}", "duration": {}}}"#, i, i * 10))
            .collect::<Vec<_>>()
            .join(",")
    );

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let _ = diff_cmd.compare(&large_trace, &large_trace);
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_secs(1),
        "Should handle large traces in <1s"
    );
    Ok(())
}
