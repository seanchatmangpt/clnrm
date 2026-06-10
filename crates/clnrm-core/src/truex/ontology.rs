// ontology module
pub struct Ontology {}

#[derive(Debug, Clone)]
pub struct Transition {
    pub select_condition: String,
}

#[derive(Debug, Clone)]
pub struct OntologyLaw {
    pub transitions: Vec<Transition>,
}
