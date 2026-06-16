//! Graph command - Visualize OpenTelemetry trace graphs
//!
//! Generates ASCII, DOT, JSON, or Mermaid visualizations of trace spans and their relationships.

use crate::cli::types::GraphFormat;
use crate::error::{CleanroomError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

    // Apply filter if provided (using regex matching)
    let spans = if let Some(filter_pattern) = filter {
        let span_nodes: Vec<SpanNode> = trace_data
            .spans
            .iter()
            .map(|s| SpanNode {
                span_id: s.span_id.clone(),
                name: s.name.clone(),
                parent_id: s.parent_span_id.clone(),
            })
            .collect();
        let filtered_nodes = filter_spans(&span_nodes, filter_pattern)?;
        let filtered_ids: HashSet<&str> =
            filtered_nodes.iter().map(|n| n.span_id.as_str()).collect();
        trace_data
            .spans
            .into_iter()
            .filter(|span| filtered_ids.contains(span.span_id.as_str()))
            .collect()
    } else {
        trace_data.spans
    };

    if spans.is_empty() {
        tracing::info!("No spans found in trace");
        return Ok(());
    }

    info!("Found {} span(s) to visualize", spans.len());

    // Generate visualization based on format
    match format {
        GraphFormat::Ascii => {
            let output = generate_ascii_tree(&spans, highlight_missing)?;
            tracing::info!("{}", output);
        }
        GraphFormat::Dot => {
            let output = generate_dot_graph(&spans)?;
            tracing::info!("{}", output);
        }
        GraphFormat::Json => {
            let output = generate_json_graph(&spans)?;
            tracing::info!("{}", output);
        }
        GraphFormat::Mermaid => {
            let output = generate_mermaid_diagram(&spans)?;
            tracing::info!("{}", output);
        }
    }

    Ok(())
}

/// Load trace data from file
fn load_trace_data(path: &Path) -> Result<TraceData> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to read trace file {}: {}",
            path.display(),
            e
        ))
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
        render_span_tree(
            root,
            &children_map,
            &mut output,
            "",
            true,
            highlight_missing,
        );
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
    output.push_str(&format!(
        "{}{} {} ({})\n",
        prefix, connector, span.name, span.kind
    ));

    // Render children
    if let Some(children) = children_map.get(&span.span_id) {
        let child_prefix = format!("{}{}   ", prefix, if is_last { " " } else { "│" });

        for (idx, child) in children.iter().enumerate() {
            let is_last_child = idx == children.len() - 1;
            render_span_tree(
                child,
                children_map,
                output,
                &child_prefix,
                is_last_child,
                highlight_missing,
            );
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

/// Node in a span dependency graph
#[derive(Debug, Clone)]
pub struct SpanNode {
    pub span_id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

/// Detect cycles in span graph using DFS
///
/// Uses DFS with coloring (white=0/not visited, gray=1/in stack, black=2/done).
/// Returns each cycle as a Vec<String> of span_ids.
pub fn detect_cycles(spans: &[SpanNode]) -> Vec<Vec<String>> {
    // Build child->parent adjacency list (parent_id -> vec of child span_ids)
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for span in spans {
        if let Some(parent_id) = &span.parent_id {
            adjacency
                .entry(parent_id.clone())
                .or_default()
                .push(span.span_id.clone());
        }
        // Ensure every node appears in the map even if childless
        adjacency.entry(span.span_id.clone()).or_default();
    }

    // Color: 0 = white (unvisited), 1 = gray (in stack), 2 = black (done)
    let mut color: HashMap<String, u8> = HashMap::new();
    let mut cycles: Vec<Vec<String>> = Vec::new();

    let all_ids: Vec<String> = spans.iter().map(|s| s.span_id.clone()).collect();

    for start in &all_ids {
        if color.get(start).copied().unwrap_or(0) == 0 {
            let mut stack: Vec<(String, usize)> = vec![(start.clone(), 0)];
            let mut path: Vec<String> = Vec::new();

            while let Some((node, child_idx)) = stack.last_mut() {
                let node = node.clone();
                if color.get(&node).copied().unwrap_or(0) == 0 {
                    // First visit: mark gray, push onto path
                    color.insert(node.clone(), 1);
                    path.push(node.clone());
                }

                let children = adjacency.get(&node).cloned().unwrap_or_default();
                if *child_idx < children.len() {
                    let child = children[*child_idx].clone();
                    *stack.last_mut().unwrap() = (node.clone(), *child_idx + 1); // OK: Safe unwrap - stack non-empty (inside while-let loop)

                    match color.get(&child).copied().unwrap_or(0) {
                        0 => {
                            // Unvisited: recurse
                            stack.push((child, 0));
                        }
                        1 => {
                            // Gray: back edge found - cycle detected
                            if let Some(cycle_start) = path.iter().position(|n| *n == child) {
                                let cycle: Vec<String> = path[cycle_start..].to_vec();
                                cycles.push(cycle);
                            }
                        }
                        _ => {
                            // Black: already processed, skip
                        }
                    }
                } else {
                    // All children processed: mark black, pop from path and stack
                    color.insert(node.clone(), 2);
                    path.pop();
                    stack.pop();
                }
            }
        }
    }

    cycles
}

/// Filter spans by regex pattern on name
pub fn filter_spans<'a>(spans: &'a [SpanNode], pattern: &str) -> Result<Vec<&'a SpanNode>> {
    let re = Regex::new(pattern).map_err(|e| {
        CleanroomError::validation_error(format!("Invalid regex pattern '{}': {}", pattern, e))
    })?;

    Ok(spans
        .iter()
        .filter(|span| re.is_match(&span.name))
        .collect())
}
