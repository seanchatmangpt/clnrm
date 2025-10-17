# Diff Contract (London TDD)

## Interface Design (Outside-In)

The Diff tool is a collaborator that compares OpenTelemetry traces. From the user's perspective:

```
User runs: clnrm diff baseline.json current.json
Expected: Compare span trees, show first failing span, highlight attribute differences
```

## Mock Contract

```rust
pub trait TraceDiffer: Send + Sync {
    /// Compare two trace files and return differences
    fn diff(&self, baseline: &Path, current: &Path, options: DiffOptions) -> Result<DiffResult>;

    /// Find first failing span (early exit for fast feedback)
    fn find_first_failure(&self, baseline: &Path, current: &Path) -> Result<Option<SpanDiff>>;
}

pub struct DiffOptions {
    pub format: DiffFormat,
    pub only_changes: bool,
    pub ignore_timing: bool,
    pub ignore_span_ids: bool,
}

pub enum DiffFormat {
    Tree,           // ASCII tree visualization
    Json,           // Structured JSON diff
    SideBySide,     // Side-by-side comparison
}

pub struct DiffResult {
    pub baseline_path: PathBuf,
    pub current_path: PathBuf,
    pub identical: bool,
    pub span_diffs: Vec<SpanDiff>,
    pub first_failure: Option<SpanDiff>,
    pub summary: DiffSummary,
}

pub struct SpanDiff {
    pub span_name: String,
    pub diff_type: DiffType,
    pub baseline_attrs: Option<HashMap<String, String>>,
    pub current_attrs: Option<HashMap<String, String>>,
    pub attr_diffs: Vec<AttributeDiff>,
}

pub enum DiffType {
    Added,          // Span in current but not baseline
    Removed,        // Span in baseline but not current
    Modified,       // Span exists in both but attributes differ
    Identical,      // No differences
}

pub struct AttributeDiff {
    pub key: String,
    pub baseline_value: Option<String>,
    pub current_value: Option<String>,
}

pub struct DiffSummary {
    pub total_spans_baseline: usize,
    pub total_spans_current: usize,
    pub spans_added: usize,
    pub spans_removed: usize,
    pub spans_modified: usize,
    pub spans_identical: usize,
}
```

## Interaction Expectations (Behavior Verification)

### Scenario: User runs `clnrm diff baseline.json current.json`

```rust
#[test]
fn test_diff_detects_added_span() {
    // Arrange: Set up mock collaborators
    let mock_reader = MockTraceReader::new();
    let mock_parser = MockOtelParser::new();
    let mock_tree_builder = MockSpanTreeBuilder::new();
    let mock_comparator = MockSpanComparator::new();

    // Configure mock expectations
    mock_reader.expect_read()
        .with(eq(Path::new("baseline.json")))
        .times(1)
        .returning(|_| Ok(r#"{"spans": [{"name": "span1"}]}"#.to_string()));

    mock_reader.expect_read()
        .with(eq(Path::new("current.json")))
        .times(1)
        .returning(|_| Ok(r#"{"spans": [{"name": "span1"}, {"name": "span2"}]}"#.to_string()));

    mock_parser.expect_parse()
        .times(2)
        .returning(|content| parse_otel_json(content));

    mock_tree_builder.expect_build()
        .times(2)
        .returning(|spans| build_tree(spans));

    mock_comparator.expect_compare()
        .times(1)
        .returning(|baseline_tree, current_tree| {
            DiffResult {
                baseline_path: PathBuf::from("baseline.json"),
                current_path: PathBuf::from("current.json"),
                identical: false,
                span_diffs: vec![
                    SpanDiff {
                        span_name: "span2".to_string(),
                        diff_type: DiffType::Added,
                        baseline_attrs: None,
                        current_attrs: Some(HashMap::new()),
                        attr_diffs: vec![],
                    }
                ],
                first_failure: Some(SpanDiff {
                    span_name: "span2".to_string(),
                    diff_type: DiffType::Added,
                    baseline_attrs: None,
                    current_attrs: Some(HashMap::new()),
                    attr_diffs: vec![],
                }),
                summary: DiffSummary {
                    total_spans_baseline: 1,
                    total_spans_current: 2,
                    spans_added: 1,
                    spans_removed: 0,
                    spans_modified: 0,
                    spans_identical: 1,
                },
            }
        });

    // Act: Run diff command
    let diff_command = DiffCommand::new(
        mock_reader,
        mock_parser,
        mock_tree_builder,
        mock_comparator,
    );

    let result = diff_command.run(
        Path::new("baseline.json"),
        Path::new("current.json"),
        DiffOptions {
            format: DiffFormat::Tree,
            only_changes: false,
            ignore_timing: true,
            ignore_span_ids: true,
        },
    );

    // Assert: Verify interaction sequence and result
    assert!(result.is_ok());
    let diff_result = result.unwrap();
    assert!(!diff_result.identical);
    assert_eq!(diff_result.summary.spans_added, 1);
    assert!(diff_result.first_failure.is_some());
}
```

### Scenario: Detect attribute differences

```rust
#[test]
fn test_diff_detects_attribute_changes() {
    // Arrange
    let mock_reader = MockTraceReader::new();
    let mock_comparator = MockSpanComparator::new();

    let baseline_span = Span {
        name: "http.request".to_string(),
        attributes: hashmap! {
            "http.method" => "GET",
            "http.status_code" => "200",
        },
    };

    let current_span = Span {
        name: "http.request".to_string(),
        attributes: hashmap! {
            "http.method" => "POST",  // Changed!
            "http.status_code" => "200",
        },
    };

    mock_comparator.expect_compare_spans()
        .with(eq(&baseline_span), eq(&current_span))
        .times(1)
        .returning(|baseline, current| {
            SpanDiff {
                span_name: "http.request".to_string(),
                diff_type: DiffType::Modified,
                baseline_attrs: Some(baseline.attributes.clone()),
                current_attrs: Some(current.attributes.clone()),
                attr_diffs: vec![
                    AttributeDiff {
                        key: "http.method".to_string(),
                        baseline_value: Some("GET".to_string()),
                        current_value: Some("POST".to_string()),
                    }
                ],
            }
        });

    // Act
    let diff_command = DiffCommand::new(mock_reader, mock_comparator);
    let result = diff_command.run(
        Path::new("baseline.json"),
        Path::new("current.json"),
        DiffOptions::default(),
    );

    // Assert
    assert!(result.is_ok());
    let diff_result = result.unwrap();
    assert_eq!(diff_result.summary.spans_modified, 1);
    let first_failure = diff_result.first_failure.unwrap();
    assert!(matches!(first_failure.diff_type, DiffType::Modified));
    assert_eq!(first_failure.attr_diffs.len(), 1);
}
```

### Scenario: Find first failure (early exit)

```rust
#[test]
fn test_diff_finds_first_failure_quickly() {
    // Arrange
    let mock_reader = MockTraceReader::new();
    let mock_comparator = MockSpanComparator::new();

    // Simulate 1000 spans, with first failure at span 5
    mock_comparator.expect_find_first_difference()
        .times(1)
        .returning(|baseline_tree, current_tree| {
            // Should stop early, not compare all 1000 spans
            Some(SpanDiff {
                span_name: "span_005".to_string(),
                diff_type: DiffType::Removed,
                baseline_attrs: Some(HashMap::new()),
                current_attrs: None,
                attr_diffs: vec![],
            })
        });

    // Act
    let diff_command = DiffCommand::new(mock_reader, mock_comparator);
    let start = std::time::Instant::now();
    let result = diff_command.find_first_failure(
        Path::new("baseline.json"),
        Path::new("current.json"),
    );
    let elapsed = start.elapsed();

    // Assert: Should be fast (early exit)
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
    assert!(elapsed < Duration::from_millis(100)); // Should not iterate all spans
}
```

## Critical Interaction Sequence

1. User → DiffCommand: run(baseline, current, options)
2. DiffCommand → TraceReader: read(baseline)
3. DiffCommand → TraceReader: read(current)
4. DiffCommand → OtelParser: parse(baseline_content)
5. DiffCommand → OtelParser: parse(current_content)
6. DiffCommand → SpanTreeBuilder: build(baseline_spans)
7. DiffCommand → SpanTreeBuilder: build(current_spans)
8. DiffCommand → SpanComparator: compare(baseline_tree, current_tree)
9. **Early exit**: SpanComparator: find_first_difference() (for fast feedback)
10. DiffCommand → User: Display DiffResult

## Diff Output Formats

### Tree (ASCII visualization)
```
Trace Diff: baseline.json vs current.json

Root
├─ span1 [IDENTICAL]
│  ├─ http.method: "GET"
│  └─ http.status_code: "200"
├─ span2 [MODIFIED]
│  ├─ http.method: "GET" → "POST"  ← DIFF
│  └─ http.status_code: "200"
└─ span3 [ADDED]
   └─ (new span in current)

Summary: 1 added, 0 removed, 1 modified, 1 identical

First failure: span2 (http.method changed)
```

### JSON (structured)
```json
{
  "identical": false,
  "span_diffs": [
    {
      "span_name": "span2",
      "diff_type": "Modified",
      "attr_diffs": [
        {
          "key": "http.method",
          "baseline_value": "GET",
          "current_value": "POST"
        }
      ]
    }
  ],
  "first_failure": {
    "span_name": "span2",
    "diff_type": "Modified"
  },
  "summary": {
    "spans_added": 0,
    "spans_removed": 0,
    "spans_modified": 1
  }
}
```

### Side-by-side
```
BASELINE                    | CURRENT
--------------------------- | ---------------------------
span1                       | span1
  http.method: "GET"        |   http.method: "GET"
  http.status_code: "200"   |   http.status_code: "200"
                            |
span2                       | span2
  http.method: "GET"        |   http.method: "POST"  ← DIFF
  http.status_code: "200"   |   http.status_code: "200"
                            |
                            | span3  ← ADDED
```

## Performance Contract

- Find first failure: &lt;100ms for typical trace (early exit)
- Full diff: &lt;500ms for typical trace
- No Docker operations
- Streaming comparison (don't load entire trace in memory)

## Error Scenarios

### Invalid JSON
```rust
mock_reader.expect_read()
    .returning(|_| Ok("invalid json".to_string()));
mock_parser.expect_parse()
    .returning(|_| Err(CleanroomError::validation_error("Invalid JSON")));
```

### File not found
```rust
mock_reader.expect_read()
    .returning(|_| Err(CleanroomError::io_error("File not found")));
```

### Incompatible trace formats
```rust
mock_comparator.expect_compare()
    .returning(|_, _| Err(CleanroomError::validation_error("Incompatible trace versions")));
```

## Implementation Notes

- Use `serde_json` for OTEL JSON parsing
- Build span tree using parent_span_id relationships
- Comparator should support early exit (find_first_difference)
- Ignore timing attributes by default (trace_id, span_id, timestamps)
- Tree visualization using `ptree` or custom ASCII art
- All trait methods MUST be sync
