# MockDiffEngine Contract

## Purpose
Define interaction contract for comparing OTEL traces between baseline and current test runs to detect behavioral changes.

## Mock Trait Definition

```rust
use crate::core::{Result, TraceComparison, SpanTree};

/// Mock implementation of trace diff engine behavior
pub trait MockDiffEngine: Send + Sync {
    /// Compare two trace span trees
    ///
    /// Interactions to verify:
    /// - Called with baseline and current traces
    /// - Returns structured comparison report
    /// - Identifies added/removed/modified spans
    fn compare_traces(&self, baseline: &SpanTree, current: &SpanTree) -> Result<TraceComparison>;

    /// Compute structural diff between span trees
    ///
    /// Interactions to verify:
    /// - Analyzes span hierarchy
    /// - Detects topology changes
    fn compute_structural_diff(&self, baseline: &SpanTree, current: &SpanTree) -> Result<StructuralDiff>;

    /// Compare span attributes (tags, timing, status)
    ///
    /// Interactions to verify:
    /// - Deep comparison of span metadata
    /// - Identifies attribute changes
    fn compare_span_attributes(&self, baseline_span: &Span, current_span: &Span) -> Result<AttributeDiff>;

    /// Generate human-readable diff report
    ///
    /// Interactions to verify:
    /// - Formats comparison as readable text
    /// - Highlights significant differences
    fn format_diff(&self, comparison: &TraceComparison, format: DiffFormat) -> Result<String>;

    /// Calculate similarity score (0.0 = completely different, 1.0 = identical)
    fn calculate_similarity(&self, comparison: &TraceComparison) -> f64;

    /// Check if differences exceed threshold for concern
    fn has_significant_changes(&self, comparison: &TraceComparison, threshold: f64) -> bool;
}

/// Trace comparison result
#[derive(Debug, Clone, PartialEq)]
pub struct TraceComparison {
    pub baseline_span_count: usize,
    pub current_span_count: usize,
    pub added_spans: Vec<SpanInfo>,
    pub removed_spans: Vec<SpanInfo>,
    pub modified_spans: Vec<SpanModification>,
    pub unchanged_spans: Vec<SpanInfo>,
    pub structural_changes: Vec<StructuralChange>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpanInfo {
    pub span_id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpanModification {
    pub span_id: String,
    pub name: String,
    pub attribute_changes: Vec<AttributeChange>,
    pub timing_change: Option<TimingChange>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuralChange {
    pub kind: StructuralChangeKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructuralChangeKind {
    SpanReordered,
    ParentChanged,
    DepthChanged,
    BranchAdded,
    BranchRemoved,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffFormat {
    Text,
    Json,
    Html,
    Unified,
}
```

## Mock Implementation for Tests

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Test mock with interaction tracking and configurable comparisons
pub struct TestDiffEngine {
    /// Tracks compare_traces() calls
    comparison_calls: Arc<Mutex<Vec<ComparisonCall>>>,

    /// Tracks format_diff() calls
    format_calls: Arc<Mutex<Vec<FormatCall>>>,

    /// Configured comparison outcomes
    comparison_config: Arc<Mutex<HashMap<String, TraceComparison>>>,

    /// Default similarity score
    default_similarity: Arc<Mutex<f64>>,
}

#[derive(Debug, Clone)]
struct ComparisonCall {
    baseline_hash: String,
    current_hash: String,
    result: TraceComparison,
    timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
struct FormatCall {
    format: DiffFormat,
    output: String,
    timestamp: std::time::Instant,
}

impl TestDiffEngine {
    pub fn new() -> Self {
        Self {
            comparison_calls: Arc::new(Mutex::new(Vec::new())),
            format_calls: Arc::new(Mutex::new(Vec::new())),
            comparison_config: Arc::new(Mutex::new(HashMap::new())),
            default_similarity: Arc::new(Mutex::new(1.0)), // Default: identical
        }
    }

    /// Configure mock to return specific comparison result
    pub fn configure_comparison(&self, key: &str, comparison: TraceComparison) {
        self.comparison_config.lock().unwrap()
            .insert(key.to_string(), comparison);
    }

    /// Configure mock to detect added spans
    pub fn configure_added_spans(&self, key: &str, spans: Vec<SpanInfo>) {
        let comparison = TraceComparison {
            baseline_span_count: 5,
            current_span_count: 5 + spans.len(),
            added_spans: spans,
            removed_spans: vec![],
            modified_spans: vec![],
            unchanged_spans: vec![],
            structural_changes: vec![],
        };
        self.configure_comparison(key, comparison);
    }

    /// Configure mock to detect removed spans
    pub fn configure_removed_spans(&self, key: &str, spans: Vec<SpanInfo>) {
        let comparison = TraceComparison {
            baseline_span_count: 5,
            current_span_count: 5 - spans.len(),
            added_spans: vec![],
            removed_spans: spans,
            modified_spans: vec![],
            unchanged_spans: vec![],
            structural_changes: vec![],
        };
        self.configure_comparison(key, comparison);
    }

    /// Configure mock to detect no changes
    pub fn configure_no_changes(&self, key: &str) {
        let comparison = TraceComparison {
            baseline_span_count: 5,
            current_span_count: 5,
            added_spans: vec![],
            removed_spans: vec![],
            modified_spans: vec![],
            unchanged_spans: vec![],
            structural_changes: vec![],
        };
        self.configure_comparison(key, comparison);
    }

    /// Set default similarity score
    pub fn set_default_similarity(&self, score: f64) {
        *self.default_similarity.lock().unwrap() = score;
    }

    /// Verify compare_traces() was called
    pub fn verify_comparison_performed(&self) -> bool {
        !self.comparison_calls.lock().unwrap().is_empty()
    }

    /// Get comparison call count
    pub fn comparison_call_count(&self) -> usize {
        self.comparison_calls.lock().unwrap().len()
    }

    /// Get last comparison result
    pub fn last_comparison(&self) -> Option<TraceComparison> {
        self.comparison_calls.lock().unwrap()
            .last()
            .map(|call| call.result.clone())
    }

    /// Verify format_diff() was called with format
    pub fn verify_formatted_as(&self, format: DiffFormat) -> bool {
        self.format_calls.lock().unwrap()
            .iter()
            .any(|call| call.format == format)
    }

    /// Get last formatted output
    pub fn last_formatted_output(&self) -> Option<String> {
        self.format_calls.lock().unwrap()
            .last()
            .map(|call| call.output.clone())
    }
}

impl MockDiffEngine for TestDiffEngine {
    fn compare_traces(&self, baseline: &SpanTree, current: &SpanTree) -> Result<TraceComparison> {
        // Compute simple hash for lookup
        let key = format!("baseline_{}_current_{}",
            baseline.root_span_id(),
            current.root_span_id()
        );

        // Get configured comparison or create default
        let comparison = self.comparison_config.lock().unwrap()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| {
                // Default: no changes
                TraceComparison {
                    baseline_span_count: baseline.span_count(),
                    current_span_count: current.span_count(),
                    added_spans: vec![],
                    removed_spans: vec![],
                    modified_spans: vec![],
                    unchanged_spans: vec![],
                    structural_changes: vec![],
                }
            });

        // Track call
        self.comparison_calls.lock().unwrap().push(ComparisonCall {
            baseline_hash: baseline.root_span_id().to_string(),
            current_hash: current.root_span_id().to_string(),
            result: comparison.clone(),
            timestamp: std::time::Instant::now(),
        });

        Ok(comparison)
    }

    fn compute_structural_diff(&self, baseline: &SpanTree, current: &SpanTree) -> Result<StructuralDiff> {
        // Default: no structural changes
        Ok(StructuralDiff {
            changes: vec![],
            severity: StructuralChangeSeverity::None,
        })
    }

    fn compare_span_attributes(&self, baseline_span: &Span, current_span: &Span) -> Result<AttributeDiff> {
        Ok(AttributeDiff {
            added_attributes: vec![],
            removed_attributes: vec![],
            modified_attributes: vec![],
        })
    }

    fn format_diff(&self, comparison: &TraceComparison, format: DiffFormat) -> Result<String> {
        let output = match format {
            DiffFormat::Text => self.format_as_text(comparison),
            DiffFormat::Json => self.format_as_json(comparison),
            DiffFormat::Html => self.format_as_html(comparison),
            DiffFormat::Unified => self.format_as_unified(comparison),
        };

        // Track call
        self.format_calls.lock().unwrap().push(FormatCall {
            format,
            output: output.clone(),
            timestamp: std::time::Instant::now(),
        });

        Ok(output)
    }

    fn calculate_similarity(&self, comparison: &TraceComparison) -> f64 {
        // Use configured similarity or compute from comparison
        if comparison.added_spans.is_empty()
            && comparison.removed_spans.is_empty()
            && comparison.modified_spans.is_empty()
        {
            1.0 // Identical
        } else {
            *self.default_similarity.lock().unwrap()
        }
    }

    fn has_significant_changes(&self, comparison: &TraceComparison, threshold: f64) -> bool {
        let similarity = self.calculate_similarity(comparison);
        similarity < threshold
    }
}

impl TestDiffEngine {
    fn format_as_text(&self, comparison: &TraceComparison) -> String {
        format!(
            "Trace Comparison:\n  Added: {}\n  Removed: {}\n  Modified: {}",
            comparison.added_spans.len(),
            comparison.removed_spans.len(),
            comparison.modified_spans.len()
        )
    }

    fn format_as_json(&self, comparison: &TraceComparison) -> String {
        serde_json::to_string_pretty(comparison).unwrap_or_default()
    }

    fn format_as_html(&self, comparison: &TraceComparison) -> String {
        format!("<div>Added: {}</div>", comparison.added_spans.len())
    }

    fn format_as_unified(&self, comparison: &TraceComparison) -> String {
        "--- baseline\n+++ current\n".to_string()
    }
}
```

## Test Examples

### Example 1: Verify Diff Computed After Test Runs

```rust
#[tokio::test]
async fn test_auto_baselining_compares_traces_after_execution() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());
    let baseline_trace = create_test_span_tree("baseline");
    let current_trace = create_test_span_tree("current");

    let auto_baseline = AutoBaselineOrchestrator::new(
        mock_diff.clone(),
        baseline_store.clone(),
    );

    // Act
    auto_baseline.compare_with_baseline(&baseline_trace, &current_trace).await?;

    // Assert - Verify comparison performed
    assert!(
        mock_diff.verify_comparison_performed(),
        "Diff engine should compare traces"
    );
    assert_eq!(mock_diff.comparison_call_count(), 1);

    Ok(())
}
```

### Example 2: Verify Added Spans Detected

```rust
#[tokio::test]
async fn test_diff_engine_detects_added_spans() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());

    let added_span = SpanInfo {
        span_id: "new_span_123".to_string(),
        name: "new_database_query".to_string(),
        parent_id: Some("root".to_string()),
    };

    mock_diff.configure_added_spans("test", vec![added_span.clone()]);

    let baseline = create_simple_span_tree();
    let current = create_span_tree_with_extra_span();

    // Act
    let comparison = mock_diff.compare_traces(&baseline, &current)?;

    // Assert - Verify added span detected
    assert_eq!(comparison.added_spans.len(), 1);
    assert_eq!(comparison.added_spans[0].name, "new_database_query");
    assert!(comparison.removed_spans.is_empty());
    assert!(comparison.modified_spans.is_empty());

    Ok(())
}
```

### Example 3: Verify Removed Spans Detected

```rust
#[tokio::test]
async fn test_diff_engine_detects_removed_spans() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());

    let removed_span = SpanInfo {
        span_id: "old_span_456".to_string(),
        name: "deprecated_cache_check".to_string(),
        parent_id: Some("root".to_string()),
    };

    mock_diff.configure_removed_spans("test", vec![removed_span.clone()]);

    let baseline = create_span_tree_with_cache_span();
    let current = create_simple_span_tree();

    // Act
    let comparison = mock_diff.compare_traces(&baseline, &current)?;

    // Assert - Verify removed span detected
    assert!(comparison.added_spans.is_empty());
    assert_eq!(comparison.removed_spans.len(), 1);
    assert_eq!(comparison.removed_spans[0].name, "deprecated_cache_check");

    Ok(())
}
```

### Example 4: Verify No Changes Detected for Identical Traces

```rust
#[tokio::test]
async fn test_diff_engine_reports_no_changes_for_identical_traces() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());
    mock_diff.configure_no_changes("test");

    let baseline = create_simple_span_tree();
    let current = create_simple_span_tree(); // Identical

    // Act
    let comparison = mock_diff.compare_traces(&baseline, &current)?;

    // Assert - Verify no differences detected
    assert!(comparison.added_spans.is_empty());
    assert!(comparison.removed_spans.is_empty());
    assert!(comparison.modified_spans.is_empty());
    assert_eq!(
        mock_diff.calculate_similarity(&comparison),
        1.0,
        "Similarity should be 1.0 for identical traces"
    );

    Ok(())
}
```

### Example 5: Verify Diff Formatting

```rust
#[tokio::test]
async fn test_diff_engine_formats_comparison_as_text() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());

    let comparison = TraceComparison {
        baseline_span_count: 5,
        current_span_count: 6,
        added_spans: vec![SpanInfo {
            span_id: "123".to_string(),
            name: "new_span".to_string(),
            parent_id: None,
        }],
        removed_spans: vec![],
        modified_spans: vec![],
        unchanged_spans: vec![],
        structural_changes: vec![],
    };

    // Act
    let text_output = mock_diff.format_diff(&comparison, DiffFormat::Text)?;

    // Assert - Verify formatting
    assert!(mock_diff.verify_formatted_as(DiffFormat::Text));
    assert!(text_output.contains("Added: 1"));
    assert!(text_output.contains("Removed: 0"));

    Ok(())
}
```

### Example 6: Verify Significance Threshold

```rust
#[tokio::test]
async fn test_auto_baseline_flags_significant_changes() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());
    mock_diff.set_default_similarity(0.75); // 75% similarity

    let comparison = TraceComparison {
        baseline_span_count: 10,
        current_span_count: 12,
        added_spans: vec![/* 2 added */],
        removed_spans: vec![],
        modified_spans: vec![],
        unchanged_spans: vec![],
        structural_changes: vec![],
    };

    let threshold = 0.90; // Require 90% similarity

    // Act
    let is_significant = mock_diff.has_significant_changes(&comparison, threshold);

    // Assert - Verify threshold detection
    assert!(
        is_significant,
        "75% similarity should be flagged as significant when threshold is 90%"
    );

    Ok(())
}
```

### Example 7: Verify Multiple Format Support

```rust
#[tokio::test]
async fn test_diff_engine_supports_multiple_output_formats() -> Result<()> {
    // Arrange
    let mock_diff = Arc::new(TestDiffEngine::new());

    let comparison = create_test_comparison();

    // Act - Generate all formats
    let text = mock_diff.format_diff(&comparison, DiffFormat::Text)?;
    let json = mock_diff.format_diff(&comparison, DiffFormat::Json)?;
    let html = mock_diff.format_diff(&comparison, DiffFormat::Html)?;
    let unified = mock_diff.format_diff(&comparison, DiffFormat::Unified)?;

    // Assert - Verify all formats generated
    assert!(mock_diff.verify_formatted_as(DiffFormat::Text));
    assert!(mock_diff.verify_formatted_as(DiffFormat::Json));
    assert!(mock_diff.verify_formatted_as(DiffFormat::Html));
    assert!(mock_diff.verify_formatted_as(DiffFormat::Unified));

    assert!(!text.is_empty());
    assert!(!json.is_empty());
    assert!(!html.is_empty());
    assert!(!unified.is_empty());

    Ok(())
}
```

## Interaction Patterns to Verify

### Pattern 1: Baseline Comparison Workflow
```
1. Load baseline trace from storage
2. Execute current test, capture trace
3. compare_traces(baseline, current)
4. format_diff(comparison, format)
5. Display or store diff report
```

### Pattern 2: Significant Change Detection
```
1. compare_traces(baseline, current)
2. calculate_similarity(comparison)
3. has_significant_changes(comparison, threshold)
4. If significant:
   a. Alert user
   b. Prompt for baseline update
```

### Pattern 3: Structural Analysis
```
1. compare_traces(baseline, current)
2. compute_structural_diff(baseline, current)
3. Identify topology changes
4. Report structural issues
```

## Contract Guarantees

### Pre-conditions
- Baseline and current traces must be valid SpanTrees
- Traces must have identifiable root spans

### Post-conditions
- TraceComparison accurately reflects differences
- Similarity score in range [0.0, 1.0]
- Format output is well-formed for format type

### Invariants
- Identical traces produce similarity = 1.0
- Added + removed + unchanged = total spans
- Formatting preserves comparison data

## Mock Configuration Helpers

```rust
impl TestDiffEngine {
    /// Configure complete diff scenario
    pub fn with_diff_scenario(self, scenario: DiffScenario) -> Self {
        match scenario {
            DiffScenario::NoChanges => {
                self.set_default_similarity(1.0);
            }
            DiffScenario::MinorChanges => {
                self.set_default_similarity(0.95);
            }
            DiffScenario::MajorChanges => {
                self.set_default_similarity(0.60);
            }
            DiffScenario::CompletelyDifferent => {
                self.set_default_similarity(0.10);
            }
        }
        self
    }

    /// Reset all tracking state
    pub fn reset(&self) {
        self.comparison_calls.lock().unwrap().clear();
        self.format_calls.lock().unwrap().clear();
    }
}

pub enum DiffScenario {
    NoChanges,
    MinorChanges,
    MajorChanges,
    CompletelyDifferent,
}
```

## Design Notes

1. **Trace Comparison**: Deep structural analysis of span trees
2. **Multiple Formats**: Support various output formats for different consumers
3. **Similarity Scoring**: Quantitative measure of behavioral equivalence
4. **Interaction Tracking**: Complete audit trail of comparisons
5. **Threshold Detection**: Configurable significance for auto-baselining
