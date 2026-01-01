/// GGEN Code Generator Library
/// Ontology-driven code generation from RDF instances

pub mod ontology;
pub mod generator;
pub mod config;
pub mod error;

pub use error::{GgenError, Result};
pub use generator::CodeGenerator;
pub use config::GeneratorConfig;

use std::path::Path;

/// Generate code from RDF instances
pub async fn generate(config: GeneratorConfig) -> Result<()> {
    let generator = CodeGenerator::new(config)?;
    generator.generate().await
}

/// Load RDF ontology from file
pub async fn load_ontology(path: &Path) -> Result<ontology::Ontology> {
    ontology::Ontology::from_file(path).await
}

/// Load RDF instances from file
pub async fn load_instances(path: &Path) -> Result<ontology::Instances> {
    ontology::Instances::from_file(path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_ontology() {
        // Will be implemented with test fixtures
    }
}
