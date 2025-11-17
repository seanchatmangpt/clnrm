//! Evidence Graph Advanced Analysis Tool
//!
//! Generates comprehensive analysis reports including:
//! - Meta-claim synthesis
//! - Concept dependency graphs
//! - Maturity assessments
//! - Multi-repository validation (breadth)
//! - Quality metrics and recommendations (depth)

use evidence_graph::{AdvancedReport, ExtendedAnalysis, MultiRepoConfig};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let coverage_file = if args.len() > 1 {
        args[1].clone()
    } else {
        "concept_coverage.json".to_string()
    };

    let output_dir = if args.len() > 2 {
        args[2].clone()
    } else {
        ".".to_string()
    };

    println!("Evidence Graph Advanced Analysis");
    println!("=================================");
    println!("Coverage file: {}", coverage_file);
    println!("Output dir: {}", output_dir);
    println!();

    // Load coverage report
    let coverage_content = fs::read_to_string(&coverage_file)?;
    let coverage: evidence_graph::ConceptCoverageReport =
        serde_json::from_str(&coverage_content)?;

    println!("[1] Synthesizing meta-claims...");
    let meta_claims = ExtendedAnalysis::synthesize_meta_claims(&[]);
    println!("    Found {} meta-claims", meta_claims.len());

    println!("[2] Inferring concept relationships...");
    let relationships = ExtendedAnalysis::infer_concept_relationships();
    println!("    Mapped {} relationships", relationships.len());

    println!("[3] Generating markdown analysis report...");
    let markdown_report = AdvancedReport::generate_markdown(&coverage, &meta_claims, &relationships);
    let md_path = format!("{}/ANALYSIS_REPORT.md", output_dir);
    fs::write(&md_path, markdown_report)?;
    println!("    Wrote {}", md_path);

    println!("[4] Generating dependency graph (Graphviz)...");
    let dot_graph = AdvancedReport::generate_dependency_graph(&relationships);
    let dot_path = format!("{}/concept_dependencies.dot", output_dir);
    fs::write(&dot_path, dot_graph)?;
    println!("    Wrote {}", dot_path);

    println!("[5] Generating maturity assessment...");
    let maturity_report = AdvancedReport::generate_maturity_report(&coverage);
    let maturity_path = format!("{}/MATURITY_ASSESSMENT.md", output_dir);
    fs::write(&maturity_path, maturity_report)?;
    println!("    Wrote {}", maturity_path);

    println!("[6] Analyzing multi-repository configuration...");
    let multi_repo_config = MultiRepoConfig::graph_universe_organs();
    println!("    Configured {} additional repos", multi_repo_config.additional_repos.len());
    println!("    Graph-universe organ systems:");
    for repo in &multi_repo_config.additional_repos {
        println!("      - {} (priority: {:.0}%)", repo.repo_id, repo.priority * 100.0);
    }

    println!("[7] Inferring system dependencies...");
    let repo_deps = multi_repo_config.infer_repo_dependencies();
    println!("    Found {} system dependencies", repo_deps.len());

    // Generate multi-repo summary
    let mut multi_repo_summary = String::from("# Multi-Repository Analysis\n\n");
    multi_repo_summary.push_str("## Graph-Universe Organ Systems\n\n");
    multi_repo_summary.push_str("The following systems implement the graph-universe thesis:\n\n");

    for repo in &multi_repo_config.additional_repos {
        multi_repo_summary.push_str(&format!(
            "### {} (Priority: {:.0}%)\n",
            repo.repo_id,
            repo.priority * 100.0
        ));
        multi_repo_summary.push_str(&format!("**Domain**: {}\n\n", repo.domain));
    }

    multi_repo_summary.push_str("## System Dependencies\n\n");
    multi_repo_summary.push_str("| From | To | Type |\n");
    multi_repo_summary.push_str("|---|----|------|\n");

    for (from, to) in &repo_deps {
        multi_repo_summary.push_str(&format!("| {} | {} | integration |\n", from, to));
    }

    let summary_path = format!("{}/MULTI_REPO_ANALYSIS.md", output_dir);
    fs::write(&summary_path, multi_repo_summary)?;
    println!("    Wrote {}", summary_path);

    println!();
    println!("Analysis Complete!");
    println!();
    println!("Generated Reports:");
    println!("  - {} (Comprehensive analysis with recommendations)", md_path);
    println!(
        "  - {} (Dependency graph for visualization)",
        dot_path
    );
    println!(
        "  - {} (Concept maturity assessment)",
        maturity_path
    );
    println!(
        "  - {} (Multi-repository configuration)",
        summary_path
    );

    println!();
    println!("Next Steps:");
    println!("  1. Render dependency graph: dot -Tsvg concept_dependencies.dot -o dependencies.svg");
    println!("  2. Review ANALYSIS_REPORT.md for detailed findings");
    println!("  3. Use maturity assessment to prioritize gap closing");

    Ok(())
}
