# Diff Engine Architecture - v0.7.0

## Overview

The diff engine compares OTEL trace structures between test runs, highlighting missing/extra spans, attribute changes, and structural differences. Provides ASCII tree visualization for human-readable diffs.

## Architecture Components

### 1. DiffEngine

```rust
// crates/clnrm-core/src/diff/engine.rs
pub struct DiffEngine {
    baseline_loader: BaselineLoader,
    tree_builder: SpanTreeBuilder,
    comparator: TraceComparator,
    visualizer: DiffVisualizer,
}

#[derive(Debug, Clone)]
pub struct DiffConfig {
    /// Path to baseline trace JSON
    pub baseline_path: PathBuf,

    /// Ignore timestamp differences
    pub ignore_timestamps: bool,

    /// Ignore attribute value changes (only check presence)
    pub ignore_attribute_values: bool,

    /// Attribute keys to always ignore in diff
    pub ignore_attributes: Vec<String>,

    /// Show only differences (hide matching spans)
    pub show_only_diffs: bool,

    /// Maximum diff tree depth to display
    pub max_display_depth: usize,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            baseline_path: PathBuf::from(".clnrm/trace.json"),
            ignore_timestamps: true,
            ignore_attribute_values: false,
            ignore_attributes: vec![
                "thread.id".to_string(),
                "process.pid".to_string(),
            ],
            show_only_diffs: false,
            max_display_depth: 10,
        }
    }
}
```

### 2. Baseline Loader

```rust
// crates/clnrm-core/src/diff/baseline.rs
pub struct BaselineLoader {
    cache: Arc<Mutex<HashMap<PathBuf, TraceBaseline>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceBaseline {
    /// Test name
    pub test_name: String,

    /// Baseline version
    pub version: String,

    /// Recorded at timestamp
    pub recorded_at: SystemTime,

    /// Span tree structure
    pub spans: Vec<BaselineSpan>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSpan {
    /// Span name (operation)
    pub name: String,

    /// Span ID
    pub span_id: String,

    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,

    /// Span attributes
    pub attributes: HashMap<String, String>,

    /// Expected children (for structure validation)
    pub children: Vec<String>, // Child span IDs
}

impl BaselineLoader {
    /// Load baseline from file
    pub fn load(&self, path: &Path) -> Result<TraceBaseline> {
        // Check cache first
        {
            let cache = self.cache.lock()
                .map_err(|e| CleanroomError::internal_error(
                    format!("Failed to lock cache: {}", e)
                ))?;

            if let Some(baseline) = cache.get(path) {
                return Ok(baseline.clone());
            }
        }

        // Load from file
        let content = std::fs::read_to_string(path)
            .map_err(|e| CleanroomError::io_error(
                format!("Failed to read baseline: {}", e)
            ))?;

        let baseline: TraceBaseline = serde_json::from_str(&content)
            .map_err(|e| CleanroomError::serialization_error(
                format!("Failed to parse baseline: {}", e)
            ))?;

        // Update cache
        {
            let mut cache = self.cache.lock()
                .map_err(|e| CleanroomError::internal_error(
                    format!("Failed to lock cache: {}", e)
                ))?;
            cache.insert(path.to_path_buf(), baseline.clone());
        }

        Ok(baseline)
    }

    /// Save baseline to file
    pub fn save(&self, path: &Path, baseline: &TraceBaseline) -> Result<()> {
        let json = serde_json::to_string_pretty(baseline)
            .map_err(|e| CleanroomError::serialization_error(
                format!("Failed to serialize baseline: {}", e)
            ))?;

        std::fs::write(path, json)
            .map_err(|e| CleanroomError::io_error(
                format!("Failed to write baseline: {}", e)
            ))?;

        // Update cache
        {
            let mut cache = self.cache.lock()
                .map_err(|e| CleanroomError::internal_error(
                    format!("Failed to lock cache: {}", e)
                ))?;
            cache.insert(path.to_path_buf(), baseline.clone());
        }

        Ok(())
    }
}
```

### 3. Span Tree Builder

```rust
// crates/clnrm-core/src/diff/tree.rs
pub struct SpanTreeBuilder;

#[derive(Debug, Clone)]
pub struct SpanTree {
    pub root: SpanNode,
}

#[derive(Debug, Clone)]
pub struct SpanNode {
    pub span: BaselineSpan,
    pub children: Vec<SpanNode>,
    pub depth: usize,
}

impl SpanTreeBuilder {
    /// Build tree from flat span list
    pub fn build(spans: Vec<BaselineSpan>) -> Result<SpanTree> {
        // Find root spans (no parent)
        let root_spans: Vec<_> = spans.iter()
            .filter(|s| s.parent_span_id.is_none())
            .collect();

        if root_spans.is_empty() {
            return Err(CleanroomError::validation_error(
                "No root spans found in trace"
            ));
        }

        if root_spans.len() > 1 {
            return Err(CleanroomError::validation_error(
                format!("Multiple root spans found: {}", root_spans.len())
            ));
        }

        let root_span = root_spans[0].clone();

        // Build tree recursively
        let root_node = Self::build_node(root_span, &spans, 0)?;

        Ok(SpanTree { root: root_node })
    }

    fn build_node(
        span: BaselineSpan,
        all_spans: &[BaselineSpan],
        depth: usize,
    ) -> Result<SpanNode> {
        // Find children
        let children: Vec<SpanNode> = all_spans.iter()
            .filter(|s| s.parent_span_id.as_ref() == Some(&span.span_id))
            .map(|child_span| Self::build_node(child_span.clone(), all_spans, depth + 1))
            .collect::<Result<Vec<_>>>()?;

        Ok(SpanNode {
            span,
            children,
            depth,
        })
    }
}
```

### 4. Trace Comparator

```rust
// crates/clnrm-core/src/diff/comparator.rs
pub struct TraceComparator {
    config: DiffConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Spans in baseline but not in current
    pub missing_spans: Vec<SpanDiff>,

    /// Spans in current but not in baseline
    pub extra_spans: Vec<SpanDiff>,

    /// Spans with attribute changes
    pub modified_spans: Vec<SpanDiff>,

    /// Overall diff summary
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanDiff {
    /// Span name
    pub name: String,

    /// Baseline span (if exists)
    pub baseline: Option<BaselineSpan>,

    /// Current span (if exists)
    pub current: Option<BaselineSpan>,

    /// Attribute differences
    pub attribute_diffs: Vec<AttributeDiff>,

    /// Structural differences
    pub structural_diffs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDiff {
    pub key: String,
    pub baseline_value: Option<String>,
    pub current_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_baseline_spans: usize,
    pub total_current_spans: usize,
    pub missing_count: usize,
    pub extra_count: usize,
    pub modified_count: usize,
    pub match_rate: f64, // Percentage of matching spans
}

impl TraceComparator {
    /// Compare baseline with current trace
    pub fn compare(
        &self,
        baseline: &TraceBaseline,
        current: &TraceBaseline,
    ) -> Result<DiffResult> {
        // Build span maps for efficient lookup
        let baseline_map = Self::build_span_map(&baseline.spans);
        let current_map = Self::build_span_map(&current.spans);

        let mut missing_spans = Vec::new();
        let mut extra_spans = Vec::new();
        let mut modified_spans = Vec::new();

        // Find missing spans (in baseline, not in current)
        for (name, baseline_span) in &baseline_map {
            if !current_map.contains_key(name) {
                missing_spans.push(SpanDiff {
                    name: name.clone(),
                    baseline: Some(baseline_span.clone()),
                    current: None,
                    attribute_diffs: vec![],
                    structural_diffs: vec!["Span missing in current trace".to_string()],
                });
            }
        }

        // Find extra spans and modifications
        for (name, current_span) in &current_map {
            match baseline_map.get(name) {
                None => {
                    // Extra span (not in baseline)
                    extra_spans.push(SpanDiff {
                        name: name.clone(),
                        baseline: None,
                        current: Some(current_span.clone()),
                        attribute_diffs: vec![],
                        structural_diffs: vec!["Span not in baseline".to_string()],
                    });
                }
                Some(baseline_span) => {
                    // Compare attributes
                    let attr_diffs = self.compare_attributes(
                        &baseline_span.attributes,
                        &current_span.attributes,
                    );

                    // Compare structure
                    let struct_diffs = self.compare_structure(baseline_span, current_span);

                    if !attr_diffs.is_empty() || !struct_diffs.is_empty() {
                        modified_spans.push(SpanDiff {
                            name: name.clone(),
                            baseline: Some(baseline_span.clone()),
                            current: Some(current_span.clone()),
                            attribute_diffs: attr_diffs,
                            structural_diffs: struct_diffs,
                        });
                    }
                }
            }
        }

        // Calculate summary
        let total_baseline = baseline_map.len();
        let total_current = current_map.len();
        let match_rate = if total_baseline > 0 {
            ((total_baseline - missing_spans.len() - modified_spans.len()) as f64
                / total_baseline as f64)
                * 100.0
        } else {
            0.0
        };

        Ok(DiffResult {
            missing_spans,
            extra_spans,
            modified_spans,
            summary: DiffSummary {
                total_baseline_spans: total_baseline,
                total_current_spans: total_current,
                missing_count: missing_spans.len(),
                extra_count: extra_spans.len(),
                modified_count: modified_spans.len(),
                match_rate,
            },
        })
    }

    fn build_span_map(spans: &[BaselineSpan]) -> HashMap<String, BaselineSpan> {
        spans.iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect()
    }

    fn compare_attributes(
        &self,
        baseline: &HashMap<String, String>,
        current: &HashMap<String, String>,
    ) -> Vec<AttributeDiff> {
        let mut diffs = Vec::new();

        // Combine all keys
        let mut all_keys: HashSet<String> = baseline.keys().cloned().collect();
        all_keys.extend(current.keys().cloned());

        for key in all_keys {
            // Skip ignored attributes
            if self.config.ignore_attributes.contains(&key) {
                continue;
            }

            let baseline_val = baseline.get(&key).cloned();
            let current_val = current.get(&key).cloned();

            // Check if values differ
            let differs = if self.config.ignore_attribute_values {
                // Only check presence
                baseline_val.is_some() != current_val.is_some()
            } else {
                // Check actual values
                baseline_val != current_val
            };

            if differs {
                diffs.push(AttributeDiff {
                    key,
                    baseline_value: baseline_val,
                    current_value: current_val,
                });
            }
        }

        diffs
    }

    fn compare_structure(
        &self,
        baseline: &BaselineSpan,
        current: &BaselineSpan,
    ) -> Vec<String> {
        let mut diffs = Vec::new();

        // Compare parent
        if baseline.parent_span_id != current.parent_span_id {
            diffs.push(format!(
                "Parent changed: {:?} → {:?}",
                baseline.parent_span_id,
                current.parent_span_id
            ));
        }

        // Compare children count
        if baseline.children.len() != current.children.len() {
            diffs.push(format!(
                "Child count changed: {} → {}",
                baseline.children.len(),
                current.children.len()
            ));
        }

        diffs
    }
}
```

### 5. Diff Visualizer

ASCII tree visualization with color-coded diffs.

```rust
// crates/clnrm-core/src/diff/visualizer.rs
pub struct DiffVisualizer {
    config: DiffConfig,
}

impl DiffVisualizer {
    /// Generate ASCII tree diff visualization
    pub fn visualize(&self, diff: &DiffResult, baseline_tree: &SpanTree) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "Trace Diff Summary:\n\
             Baseline: {} spans\n\
             Current:  {} spans\n\
             Match Rate: {:.1}%\n\n",
            diff.summary.total_baseline_spans,
            diff.summary.total_current_spans,
            diff.summary.match_rate
        ));

        // Missing spans
        if !diff.missing_spans.is_empty() {
            output.push_str(&format!("\n❌ Missing Spans ({}):\n", diff.missing_spans.len()));
            for span_diff in &diff.missing_spans {
                output.push_str(&format!("  - {}\n", span_diff.name));
            }
        }

        // Extra spans
        if !diff.extra_spans.is_empty() {
            output.push_str(&format!("\n➕ Extra Spans ({}):\n", diff.extra_spans.len()));
            for span_diff in &diff.extra_spans {
                output.push_str(&format!("  + {}\n", span_diff.name));
            }
        }

        // Modified spans
        if !diff.modified_spans.is_empty() {
            output.push_str(&format!("\n✏️  Modified Spans ({}):\n", diff.modified_spans.len()));
            for span_diff in &diff.modified_spans {
                output.push_str(&format!("  ~ {}\n", span_diff.name));

                // Show attribute changes
                for attr_diff in &span_diff.attribute_diffs {
                    output.push_str(&format!(
                        "      {}: {:?} → {:?}\n",
                        attr_diff.key,
                        attr_diff.baseline_value,
                        attr_diff.current_value
                    ));
                }
            }
        }

        // Tree visualization
        output.push_str("\n\nSpan Tree:\n");
        output.push_str(&self.render_tree_node(&baseline_tree.root, diff, "", true));

        output
    }

    fn render_tree_node(
        &self,
        node: &SpanNode,
        diff: &DiffResult,
        prefix: &str,
        is_last: bool,
    ) -> String {
        if node.depth > self.config.max_display_depth {
            return String::new();
        }

        let mut output = String::new();

        // Determine node status
        let status = self.get_node_status(&node.span.name, diff);

        // Draw tree branch
        let branch = if is_last { "└── " } else { "├── " };
        let status_icon = match status {
            NodeStatus::Match => "✓",
            NodeStatus::Missing => "❌",
            NodeStatus::Extra => "➕",
            NodeStatus::Modified => "✏️ ",
        };

        output.push_str(&format!("{}{}{} {}\n", prefix, branch, status_icon, node.span.name));

        // Draw children
        let child_prefix = format!("{}{}",
            prefix,
            if is_last { "    " } else { "│   " }
        );

        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            output.push_str(&self.render_tree_node(child, diff, &child_prefix, is_last_child));
        }

        output
    }

    fn get_node_status(&self, span_name: &str, diff: &DiffResult) -> NodeStatus {
        if diff.missing_spans.iter().any(|d| d.name == span_name) {
            NodeStatus::Missing
        } else if diff.extra_spans.iter().any(|d| d.name == span_name) {
            NodeStatus::Extra
        } else if diff.modified_spans.iter().any(|d| d.name == span_name) {
            NodeStatus::Modified
        } else {
            NodeStatus::Match
        }
    }
}

enum NodeStatus {
    Match,
    Missing,
    Extra,
    Modified,
}
```

## Example Output

```
Trace Diff Summary:
Baseline: 12 spans
Current:  13 spans
Match Rate: 83.3%

❌ Missing Spans (1):
  - database.query.users

➕ Extra Spans (2):
  + cache.get.session
  + cache.set.session

✏️  Modified Spans (1):
  ~ http.request
      http.status_code: "200" → "201"
      http.method: "GET" → "POST"


Span Tree:
└── ✓ clnrm.run
    ├── ✓ clnrm.test
    │   ├── ✓ clnrm.service.start
    │   ├── ✏️  http.request
    │   │   ├── ➕ cache.get.session
    │   │   ├── ❌ database.query.users
    │   │   └── ➕ cache.set.session
    │   └── ✓ clnrm.assertion.validate
    └── ✓ clnrm.cleanup
```

## CLI Integration

```bash
# Compare with baseline
clnrm diff tests/test_auth.clnrm.toml.tera

# Update baseline (save current as new baseline)
clnrm diff --update-baseline tests/

# Show only differences
clnrm diff --only-diffs tests/

# Export diff as JSON
clnrm diff --format json --output diff.json tests/

# Ignore specific attributes
clnrm diff --ignore thread.id,process.pid tests/
```

## Configuration

```toml
# .clnrm/config.toml
[diff]
baseline_path = ".clnrm/trace.json"
ignore_timestamps = true
ignore_attribute_values = false
show_only_diffs = false
max_display_depth = 10

[diff.ignore_attributes]
keys = ["thread.id", "process.pid", "timestamp"]
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_detects_missing_span() -> Result<()> {
        // Arrange
        let baseline = create_baseline_with_spans(vec!["span1", "span2"]);
        let current = create_baseline_with_spans(vec!["span1"]);
        let comparator = TraceComparator::new(DiffConfig::default());

        // Act
        let diff = comparator.compare(&baseline, &current)?;

        // Assert
        assert_eq!(diff.missing_spans.len(), 1);
        assert_eq!(diff.missing_spans[0].name, "span2");

        Ok(())
    }

    #[test]
    fn test_diff_detects_attribute_change() -> Result<()> {
        // Arrange
        let mut baseline_span = create_span("span1");
        baseline_span.attributes.insert("key".to_string(), "value1".to_string());

        let mut current_span = create_span("span1");
        current_span.attributes.insert("key".to_string(), "value2".to_string());

        let baseline = TraceBaseline {
            test_name: "test".to_string(),
            version: "1.0".to_string(),
            recorded_at: SystemTime::now(),
            spans: vec![baseline_span],
            metadata: HashMap::new(),
        };

        let current = TraceBaseline {
            spans: vec![current_span],
            ..baseline.clone()
        };

        let comparator = TraceComparator::new(DiffConfig::default());

        // Act
        let diff = comparator.compare(&baseline, &current)?;

        // Assert
        assert_eq!(diff.modified_spans.len(), 1);
        assert_eq!(diff.modified_spans[0].attribute_diffs.len(), 1);

        Ok(())
    }
}
```

## Future Enhancements

1. **Visual Web UI**: Interactive diff viewer with collapsible trees
2. **Regression Detection**: Alert on unexpected changes
3. **Smart Diffing**: Ignore cosmetic changes, focus on semantic diffs
4. **Baseline Versioning**: Track baseline evolution over time
