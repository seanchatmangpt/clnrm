use thiserror::Error;

/// GGEN Error types
#[derive(Error, Debug)]
pub enum GgenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RDF parse error: {0}")]
    RdfParse(String),

    #[error("Ontology error: {0}")]
    OntologyError(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, GgenError>;

impl From<String> for GgenError {
    fn from(s: String) -> Self {
        GgenError::GenerationError(s)
    }
}

impl From<&str> for GgenError {
    fn from(s: &str) -> Self {
        GgenError::GenerationError(s.to_string())
    }
}
