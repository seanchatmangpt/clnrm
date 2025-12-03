//! Test script to generate CLI from RDF template
//! This is a temporary test to verify the template works

use std::path::PathBuf;
use ggen_core::{Template, Graph};
use tera::Context;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template_path = PathBuf::from("templates/cli-v2/cli-stack.tmpl");
    let template_content = std::fs::read_to_string(&template_path)?;
    
    println!("📖 Parsing template...");
    let mut template = Template::parse(&template_content)?;
    
    println!("📊 Creating RDF graph...");
    let mut graph = Graph::new()?;
    
    // Load RDF files from frontmatter
    if let Some(rdf_files) = &template.front.rdf {
        for rdf_file in rdf_files {
            let rdf_path = PathBuf::from(rdf_file);
            println!("   Loading RDF: {}", rdf_path.display());
            graph.load_path(&rdf_path)?;
        }
    }
    
    println!("🔍 Executing SPARQL queries...");
    let mut tera = ggen_core::tera_env::build_tera_minimal()?;
    let context = Context::new();
    
    // Process graph (executes SPARQL queries)
    template.process_graph(&mut graph, &mut tera, &context, &template_path)?;
    
    println!("✨ Rendering template...");
    let rendered = template.render(&mut tera, &context)?;
    
    println!("📝 Rendered content length: {} bytes", rendered.len());
    println!("📄 First 500 chars:");
    println!("{}", &rendered[..rendered.len().min(500)]);
    
    Ok(())
}

