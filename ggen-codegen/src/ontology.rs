/// RDF Ontology and Instance Data Models

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use crate::error::{GgenError, Result};

/// Represents an RDF class/concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    pub label: String,
    pub comment: String,
    pub properties: Vec<String>,
}

/// Represents an RDF property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub label: String,
    pub comment: String,
    pub domain: String,
    pub range: String,
    pub is_object_property: bool,
}

/// Represents an RDF instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub class_type: String,
    pub label: Option<String>,
    pub comment: Option<String>,
    pub properties: HashMap<String, InstanceProperty>,
}

/// Property value in an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InstanceProperty {
    String(String),
    Integer(i64),
    Boolean(bool),
    Decimal(f64),
    Reference(String), // URI reference to another instance
    List(Vec<InstanceProperty>),
}

impl InstanceProperty {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            InstanceProperty::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            InstanceProperty::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            InstanceProperty::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<&str> {
        match self {
            InstanceProperty::Reference(r) => Some(r),
            _ => None,
        }
    }
}

/// Complete RDF Ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ontology {
    pub id: String,
    pub title: String,
    pub version: String,
    pub classes: HashMap<String, Class>,
    pub properties: HashMap<String, Property>,
}

impl Ontology {
    pub fn new(id: String, title: String, version: String) -> Self {
        Self {
            id,
            title,
            version,
            classes: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_turtle(&content)
    }

    pub fn from_turtle(content: &str) -> Result<Self> {
        // For now, return a basic ontology
        // In production, would parse actual Turtle RDF with oxigraph
        let mut ontology = Self::new(
            "https://clnrm.io/ontology/".to_string(),
            "CLNRM Ontology".to_string(),
            "1.0.0".to_string(),
        );

        // Add core classes
        ontology.classes.insert(
            "CleanroomEnvironment".to_string(),
            Class {
                id: "https://clnrm.io/ontology/CleanroomEnvironment".to_string(),
                label: "Cleanroom Environment".to_string(),
                comment: "Main testing environment".to_string(),
                properties: vec![
                    "hasSessionId".to_string(),
                    "hasContainer".to_string(),
                    "hasService".to_string(),
                ],
            },
        );

        ontology.classes.insert(
            "Service".to_string(),
            Class {
                id: "https://clnrm.io/ontology/Service".to_string(),
                label: "Service".to_string(),
                comment: "Service plugin abstraction".to_string(),
                properties: vec![
                    "serviceName".to_string(),
                    "serviceType".to_string(),
                    "containerImage".to_string(),
                    "exposedPort".to_string(),
                ],
            },
        );

        ontology.classes.insert(
            "Container".to_string(),
            Class {
                id: "https://clnrm.io/ontology/Container".to_string(),
                label: "Container".to_string(),
                comment: "Container specification".to_string(),
                properties: vec![
                    "containerName".to_string(),
                    "containerImage".to_string(),
                    "exposedPort".to_string(),
                    "hasHealthCheck".to_string(),
                ],
            },
        );

        Ok(ontology)
    }
}

/// RDF Instance Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    pub ontology_id: String,
    pub instances: HashMap<String, Instance>,
    pub relationships: Vec<Relationship>,
}

/// Relationship between instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl Instances {
    pub fn new(ontology_id: String) -> Self {
        Self {
            ontology_id,
            instances: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_turtle(&content)
    }

    pub fn from_turtle(content: &str) -> Result<Self> {
        // For now, return empty instances
        // In production, would parse actual Turtle RDF with oxigraph
        Ok(Self::new(
            "https://clnrm.io/ontology/".to_string(),
        ))
    }

    pub fn add_instance(&mut self, instance: Instance) {
        self.instances.insert(instance.id.clone(), instance);
    }

    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }

    pub fn get_instances_of_type(&self, class_type: &str) -> Vec<&Instance> {
        self.instances
            .values()
            .filter(|inst| inst.class_type == class_type)
            .collect()
    }

    pub fn get_instance(&self, id: &str) -> Option<&Instance> {
        self.instances.get(id)
    }
}

/// Generated code artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub name: String,
    pub content_type: ArtifactType,
    pub content: String,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArtifactType {
    RustCode,
    TomlConfig,
    Markdown,
    Json,
    Html,
}

impl GeneratedArtifact {
    pub fn new(name: String, content_type: ArtifactType, content: String, path: std::path::PathBuf) -> Self {
        Self {
            name,
            content_type,
            content,
            path,
        }
    }

    pub async fn write(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&self.path, &self.content).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_creation() {
        let ontology = Ontology::new(
            "test".to_string(),
            "Test".to_string(),
            "1.0.0".to_string(),
        );
        assert_eq!(ontology.title, "Test");
    }

    #[test]
    fn test_instance_creation() {
        let mut instances = Instances::new("test".to_string());
        assert_eq!(instances.ontology_id, "test");
        assert!(instances.instances.is_empty());
    }

    #[test]
    fn test_instance_property_access() {
        let prop = InstanceProperty::String("test".to_string());
        assert_eq!(prop.as_string(), Some("test"));
    }
}
