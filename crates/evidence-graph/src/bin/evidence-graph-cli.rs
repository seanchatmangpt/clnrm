//! Evidence Graph Mining CLI
//!
//! Runs the complete pipeline to mine evidence from source code and documentation.

use evidence_graph::{Pipeline, PipelineConfig};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let root_path = if args.len() > 1 {
        args[1].clone()
    } else {
        ".".to_string()
    };

    let output_dir = if args.len() > 2 {
        args[2].clone()
    } else {
        ".".to_string()
    };

    println!("Evidence Graph Mining Pipeline");
    println!("================================");
    println!("Root path: {}", root_path);
    println!("Output dir: {}", output_dir);
    println!();

    let config = PipelineConfig {
        root_path,
        match_threshold: 0.6,
        excerpt_min_score: 0.5,
        output_dir,
    };

    let mut pipeline = Pipeline::new(config);
    pipeline.run().await?;

    println!();
    println!("Pipeline complete!");
    println!();

    // Print summary
    if let Some(graph) = pipeline.evidence_graph() {
        println!("Evidence Graph Summary:");
        println!("  Total nodes: {}", graph.nodes.len());
        println!("  Total edges: {}", graph.edges.len());
        println!("  Generated at: {}", graph.metadata.generated_at);
    }

    if let Some(coverage) = pipeline.coverage_report() {
        println!();
        println!("Coverage Report Summary:");
        println!("  Total concepts: {}", coverage.statistics.total_concepts);
        println!("  Total evidence nodes: {}", coverage.statistics.total_evidence_nodes);
        println!("  Overall avg strength: {:.2}", coverage.statistics.overall_avg_strength);
    }

    if let Some(gaps) = pipeline.gaps_report() {
        println!();
        println!("Gaps Report Summary:");
        println!("  Total gaps: {}", gaps.summary.total_gaps);
        println!("  Critical gaps (no evidence): {}", gaps.summary.critical_gaps);
        println!("  Weak gaps (strength < 0.5): {}", gaps.summary.weak_gaps);
    }

    // Write outputs
    println!();
    println!("Writing output files...");
    pipeline.write_outputs()?;

    Ok(())
}
