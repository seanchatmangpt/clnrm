//! Graph command - Visualize OpenTelemetry trace graphs
//!
//! Generates ASCII, DOT, JSON, or Mermaid visualizations of trace spans and their relationships.

use crate::cli::types::GraphFormat;
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

#[derive(Debug, Deserialize, Serialize)]
struct Span {
    #[serde(default)]
    name: String,
    #[serde(default)]
    span_id: String,
    #[serde(default)]
    parent_span_id: Option<String>,
    #[serde(default)]
    trace_id: String,
    #[serde(default)]
    kind: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TraceData {
    #[serde(default)]
    spans: Vec<Span>,
}

/// Visualize OpenTelemetry trace graph
pub fn visualize_graph(
    trace_path: &Path,
    format: &GraphFormat,
    highlight_missing: bool,
    filter: Option<&str>,
) -> Result<()> {
    info!("Loading trace from {}", trace_path.display());

    // Load trace data
    let trace_data = load_trace_data(trace_path)?;

    // Apply filter if provided
    let spans = if let Some(filter_pattern) = filter {
        trace_data
            .spans
            .into_iter()
            .filter(|span| span.name.contains(filter_pattern))
            .collect()
    } else {
        trace_data.spans
    };

    if spans.is_empty() {
        println!("No spans found in trace");
        return Ok(());
    }

    info!("Found {} span(s) to visualize", spans.len());

    // Generate visualization based on format
    match format {
        GraphFormat::Ascii => {
            let output = generate_ascii_tree(&spans, highlight_missing)?;
            println!("{}", output);
        }
        GraphFormat::Dot => {
            let output = generate_dot_graph(&spans)?;
            println!("{}", output);
        }
        GraphFormat::Json => {
            let output = generate_json_graph(&spans)?;
            println!("{}", output);
        }
        GraphFormat::Mermaid => {
            let output = generate_mermaid_diagram(&spans)?;
            println!("{}", output);
        }
    }

    Ok(())
}

/// Load trace data from file
fn load_trace_data(path: &Path) -> Result<TraceData> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!("Failed to read trace file {}: {}", path.display(), e))
    })?;

    serde_json::from_str(&content).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to parse trace JSON: {}", e))
    })
}

/// Generate ASCII tree visualization
fn generate_ascii_tree(spans: &[Span], highlight_missing: bool) -> Result<String> {
    debug!("Generating ASCII tree visualization");

    let mut output = String::new();
    output.push_str("OpenTelemetry Trace Graph\n");
    output.push_str("=========================\n\n");

    // Build parent-child relationships
    let mut children_map: HashMap<String, Vec<&Span>> = HashMap::new();
    let mut root_spans = Vec::new();

    for span in spans {
        if let Some(parent_id) = &span.parent_span_id {
            children_map
                .entry(parent_id.clone())
                .or_default()
                .push(span);
        } else {
            root_spans.push(span);
        }
    }

    // Render tree starting from root spans
    for root in root_spans {
        render_span_tree(root, &children_map, &mut output, "", true, highlight_missing);
    }

    if output.trim().is_empty() {
        output.push_str("(no spans to display)\n");
    }

    Ok(output)
}

/// Recursively render span tree
fn render_span_tree(
    span: &Span,
    children_map: &HashMap<String, Vec<&Span>>,
    output: &mut String,
    prefix: &str,
    is_last: bool,
    highlight_missing: bool,
) {
    // Render current span
    let connector = if is_last { "└──" } else { "├──" };
    output.push_str(&format!("{}{} {} ({})\n", prefix, connector, span.name, span.kind));

    // Render children
    if let Some(children) = children_map.get(&span.span_id) {
        let child_prefix = format!("{}{}   ", prefix, if is_last { " " } else { "│" });

        for (idx, child) in children.iter().enumerate() {
            let is_last_child = idx == children.len() - 1;
            render_span_tree(child, children_map, output, &child_prefix, is_last_child, highlight_missing);
        }
    } else if highlight_missing && !children_map.is_empty() {
        let child_prefix = format!("{}{}   ", prefix, if is_last { " " } else { "│" });
        output.push_str(&format!("{}└── (no children)\n", child_prefix));
    }
}

/// Generate DOT graph for Graphviz
fn generate_dot_graph(spans: &[Span]) -> Result<String> {
    debug!("Generating DOT graph");

    let mut output = String::new();
    output.push_str("digraph trace {\n");
    output.push_str("  rankdir=TB;\n");
    output.push_str("  node [shape=box, style=rounded];\n\n");

    // Add nodes
    for span in spans {
        let label = format!("{}\\n{}", span.name, span.kind);
        output.push_str(&format!("  \"{}\" [label=\"{}\"];\n", span.span_id, label));
    }

    output.push('\n');

    // Add edges
    for span in spans {
        if let Some(parent_id) = &span.parent_span_id {
            output.push_str(&format!("  \"{}\" -> \"{}\";\n", parent_id, span.span_id));
        }
    }

    output.push_str("}\n");

    Ok(output)
}

/// Generate JSON graph structure
fn generate_json_graph(spans: &[Span]) -> Result<String> {
    debug!("Generating JSON graph");

    #[derive(Serialize)]
    struct JsonGraph {
        nodes: Vec<JsonNode>,
        edges: Vec<JsonEdge>,
    }

    #[derive(Serialize)]
    struct JsonNode {
        id: String,
        name: String,
        kind: String,
    }

    #[derive(Serialize)]
    struct JsonEdge {
        source: String,
        target: String,
    }

    let nodes: Vec<JsonNode> = spans
        .iter()
        .map(|span| JsonNode {
            id: span.span_id.clone(),
            name: span.name.clone(),
            kind: span.kind.clone(),
        })
        .collect();

    let edges: Vec<JsonEdge> = spans
        .iter()
        .filter_map(|span| {
            span.parent_span_id.as_ref().map(|parent_id| JsonEdge {
                source: parent_id.clone(),
                target: span.span_id.clone(),
            })
        })
        .collect();

    let graph = JsonGraph { nodes, edges };

    serde_json::to_string_pretty(&graph).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to serialize JSON graph: {}", e))
    })
}

/// Generate Mermaid diagram
fn generate_mermaid_diagram(spans: &[Span]) -> Result<String> {
    debug!("Generating Mermaid diagram");

    let mut output = String::new();
    output.push_str("```mermaid\n");
    output.push_str("graph TD\n");

    // Add nodes and edges
    for span in spans {
        let node_id = sanitize_mermaid_id(&span.span_id);
        let label = format!("{}[{}]", node_id, span.name);
        output.push_str(&format!("  {}\n", label));

        if let Some(parent_id) = &span.parent_span_id {
            let parent_node_id = sanitize_mermaid_id(parent_id);
            output.push_str(&format!("  {} --> {}\n", parent_node_id, node_id));
        }
    }

    output.push_str("```\n");

    Ok(output)
}

/// Sanitize span ID for Mermaid
fn sanitize_mermaid_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_sample_trace_data() -> TraceData {
        TraceData {
            spans: vec![
                Span {
                    name: "Root Operation".to_string(),
                    span_id: "span-001".to_string(),
                    parent_span_id: None,
                    trace_id: "trace-123".to_string(),
                    kind: "SERVER".to_string(),
                },
                Span {
                    name: "HTTP Request".to_string(),
                    span_id: "span-002".to_string(),
                    parent_span_id: Some("span-001".to_string()),
                    trace_id: "trace-123".to_string(),
                    kind: "CLIENT".to_string(),
                },
                Span {
                    name: "Database Query".to_string(),
                    span_id: "span-003".to_string(),
                    parent_span_id: Some("span-002".to_string()),
                    trace_id: "trace-123".to_string(),
                    kind: "CLIENT".to_string(),
                },
                Span {
                    name: "Cache Lookup".to_string(),
                    span_id: "span-004".to_string(),
                    parent_span_id: Some("span-002".to_string()),
                    trace_id: "trace-123".to_string(),
                    kind: "CLIENT".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("abc-123"), "abc_123");
        assert_eq!(sanitize_mermaid_id("span.id.123"), "span_id_123");
        assert_eq!(sanitize_mermaid_id("test@span#id"), "test_span_id");
    }

    #[test]
    fn test_generate_ascii_tree_empty() -> Result<()> {
        // Arrange
        let spans = vec![];

        // Act
        let result = generate_ascii_tree(&spans, false)?;

        // Assert
        assert!(result.contains("(no spans to display)"));
        Ok(())
    }

    #[test]
    fn test_generate_ascii_tree_with_spans() -> Result<()> {
        // Arrange
        let trace_data = create_sample_trace_data();

        // Act
        let result = generate_ascii_tree(&trace_data.spans, false)?;

        // Assert
        assert!(result.contains("Root Operation"));
        assert!(result.contains("HTTP Request"));
        assert!(result.contains("Database Query"));
        assert!(result.contains("Cache Lookup"));
        assert!(result.contains("SERVER"));
        assert!(result.contains("CLIENT"));

        Ok(())
    }

    #[test]
    fn test_generate_dot_graph() -> Result<()> {
        // Arrange
        let trace_data = create_sample_trace_data();

        // Act
        let result = generate_dot_graph(&trace_data.spans)?;

        // Assert
        assert!(result.contains("digraph trace"));
        assert!(result.contains("rankdir=TB"));
        assert!(result.contains("span-001"));
        assert!(result.contains("span-002"));
        assert!(result.contains("Root Operation"));
        assert!(result.contains("HTTP Request"));
        assert!(result.contains("->"));

        Ok(())
    }

    #[test]
    fn test_generate_json_graph() -> Result<()> {
        // Arrange
        let trace_data = create_sample_trace_data();

        // Act
        let result = generate_json_graph(&trace_data.spans)?;

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&result)
            .map_err(|e| CleanroomError::serialization_error(e.to_string()))?;

        assert!(parsed["nodes"].is_array());
        assert!(parsed["edges"].is_array());
        assert_eq!(parsed["nodes"].as_array().unwrap().len(), 4);
        assert_eq!(parsed["edges"].as_array().unwrap().len(), 3);

        Ok(())
    }

    #[test]
    fn test_generate_mermaid_diagram() -> Result<()> {
        // Arrange
        let trace_data = create_sample_trace_data();

        // Act
        let result = generate_mermaid_diagram(&trace_data.spans)?;

        // Assert
        assert!(result.contains("```mermaid"));
        assert!(result.contains("graph TD"));
        assert!(result.contains("span_001"));
        assert!(result.contains("Root Operation"));
        assert!(result.contains("-->"));
        assert!(result.contains("```"));

        Ok(())
    }

    #[test]
    fn test_load_trace_data_with_invalid_json() -> Result<()> {
        // Arrange
        let temp_file = NamedTempFile::new()
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;
        fs::write(temp_file.path(), "invalid json")
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        let result = load_trace_data(temp_file.path());

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Failed to parse trace JSON"));

        Ok(())
    }

    #[test]
    fn test_load_trace_data_with_valid_json() -> Result<()> {
        // Arrange
        let temp_file = NamedTempFile::new()
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let trace_data = create_sample_trace_data();
        let json = serde_json::to_string(&trace_data)
            .map_err(|e| CleanroomError::serialization_error(e.to_string()))?;
        fs::write(temp_file.path(), json)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        let result = load_trace_data(temp_file.path())?;

        // Assert
        assert_eq!(result.spans.len(), 4);
        assert_eq!(result.spans[0].name, "Root Operation");

        Ok(())
    }

    #[test]
    fn test_visualize_graph_ascii_format() -> Result<()> {
        // Arrange
        let temp_file = NamedTempFile::new()
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let trace_data = create_sample_trace_data();
        let json = serde_json::to_string(&trace_data)
            .map_err(|e| CleanroomError::serialization_error(e.to_string()))?;
        fs::write(temp_file.path(), json)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        let result = visualize_graph(temp_file.path(), &GraphFormat::Ascii, false, None);

        // Assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_visualize_graph_with_filter() -> Result<()> {
        // Arrange
        let temp_file = NamedTempFile::new()
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let trace_data = create_sample_trace_data();
        let json = serde_json::to_string(&trace_data)
            .map_err(|e| CleanroomError::serialization_error(e.to_string()))?;
        fs::write(temp_file.path(), json)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        let result = visualize_graph(temp_file.path(), &GraphFormat::Json, false, Some("Database"));

        // Assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_visualize_graph_with_nonexistent_file() -> Result<()> {
        // Arrange
        let nonexistent_path = std::path::PathBuf::from("/nonexistent/trace.json");

        // Act
        let result = visualize_graph(&nonexistent_path, &GraphFormat::Ascii, false, None);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Failed to read trace file"));

        Ok(())
    }

    #[test]
    fn test_all_output_formats() -> Result<()> {
        // Arrange
        let temp_file = NamedTempFile::new()
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let trace_data = create_sample_trace_data();
        let json = serde_json::to_string(&trace_data)
            .map_err(|e| CleanroomError::serialization_error(e.to_string()))?;
        fs::write(temp_file.path(), json)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act & Assert - Test all formats
        assert!(visualize_graph(temp_file.path(), &GraphFormat::Ascii, false, None).is_ok());
        assert!(visualize_graph(temp_file.path(), &GraphFormat::Dot, false, None).is_ok());
        assert!(visualize_graph(temp_file.path(), &GraphFormat::Json, false, None).is_ok());
        assert!(visualize_graph(temp_file.path(), &GraphFormat::Mermaid, false, None).is_ok());

        Ok(())
    }
}
