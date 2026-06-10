use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
}

impl Diagnostic {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            severity: Some(DiagnosticSeverity::Error),
            code: Some(code.to_string()),
            source: Some("POWL_Validator".to_string()),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PowlNode {
    Activity(String),
    Sequence(Vec<PowlNode>),
    Parallel(Vec<PowlNode>),
    Xor(Vec<PowlNode>),
    Loop {
        body: Box<PowlNode>,
        exit: Box<PowlNode>,
    },
}

#[derive(Debug, Clone)]
pub struct PowlWorkflow {
    pub name: String,
    pub root: PowlNode,
}

impl PowlWorkflow {
    pub fn new(name: String, root: PowlNode) -> Self {
        Self { name, root }
    }

    pub fn verify_trace(&self, trace: &[String]) -> Result<(), Vec<Diagnostic>> {
        let mut errors = Vec::new();
        match self.verify_node(&self.root, trace) {
            Ok(consumed) => {
                if consumed == trace.len() {
                    Ok(())
                } else {
                    errors.push(Diagnostic::new(
                        "POWL_UNEXPECTED_ACTIVITY",
                        &format!(
                            "Expected end of trace, but found extra activities starting with '{}'",
                            trace[consumed]
                        ),
                    ));
                    Err(errors)
                }
            }
            Err(mut errs) => {
                errors.append(&mut errs);
                Err(errors)
            }
        }
    }

    fn verify_node(&self, node: &PowlNode, trace: &[String]) -> Result<usize, Vec<Diagnostic>> {
        match node {
            PowlNode::Activity(act) => {
                if trace.is_empty() {
                    return Err(vec![Diagnostic::new(
                        "POWL_MISSING_ACTIVITY",
                        &format!("Expected activity '{}', but trace ended", act),
                    )]);
                }
                if trace[0] == *act {
                    Ok(1)
                } else {
                    Err(vec![Diagnostic::new(
                        "POWL_UNEXPECTED_ACTIVITY",
                        &format!("Expected activity '{}', but found '{}'", act, trace[0]),
                    )])
                }
            }
            PowlNode::Sequence(nodes) => {
                let mut consumed = 0;
                for sub_node in nodes {
                    match self.verify_node(sub_node, &trace[consumed..]) {
                        Ok(len) => consumed += len,
                        Err(errs) => {
                            // Adjust diagnostics if needed
                            return Err(errs);
                        }
                    }
                }
                Ok(consumed)
            }
            PowlNode::Xor(nodes) => {
                for sub_node in nodes {
                    if let Ok(len) = self.verify_node(sub_node, trace) {
                        return Ok(len);
                    }
                }
                Err(vec![Diagnostic::new(
                    "POWL_XOR_FAILED",
                    "None of the XOR branches matched the trace",
                )])
            }
            PowlNode::Loop { body, exit } => {
                // A Loop represents body executed one or more times, followed by exit.
                let mut consumed = 0;
                let mut body_count = 0;

                // Keep matching body as long as possible
                while consumed < trace.len() {
                    match self.verify_node(body, &trace[consumed..]) {
                        Ok(len) => {
                            consumed += len;
                            body_count += 1;
                        }
                        Err(_) => break,
                    }
                }

                if body_count == 0 {
                    return Err(vec![Diagnostic::new(
                        "POWL_LOOP_FAILED",
                        "Loop body must be executed at least once",
                    )]);
                }

                // Match exit
                match self.verify_node(exit, &trace[consumed..]) {
                    Ok(len) => Ok(consumed + len),
                    Err(errs) => Err(errs),
                }
            }
            PowlNode::Parallel(nodes) => {
                // A simplified parallel implementation that verifies all branches are present in the trace.
                // In a parallel block of activities, the trace must contain all of them in some interleaved order.
                // Let's count/collect the expected activities.
                let mut expected_activities = Vec::new();
                fn collect_activities(n: &PowlNode, list: &mut Vec<Vec<String>>) {
                    match n {
                        PowlNode::Activity(act) => list.push(vec![act.clone()]),
                        PowlNode::Xor(sub) => {
                            let mut options = Vec::new();
                            for s in sub {
                                if let PowlNode::Activity(act) = s {
                                    options.push(act.clone());
                                }
                            }
                            list.push(options);
                        }
                        PowlNode::Sequence(sub) => {
                            for s in sub {
                                collect_activities(s, list);
                            }
                        }
                        _ => {}
                    }
                }

                for node in nodes {
                    collect_activities(node, &mut expected_activities);
                }

                // check if the trace contains exactly one matching element from each choice in expected_activities
                let mut trace_items = trace.to_vec();
                let mut matched_choices = 0;

                for choice in &expected_activities {
                    let mut found = false;
                    for option in choice {
                        if let Some(pos) = trace_items.iter().position(|x| x == option) {
                            trace_items.remove(pos);
                            found = true;
                            break;
                        }
                    }
                    if found {
                        matched_choices += 1;
                    }
                }

                if matched_choices == expected_activities.len() && trace_items.is_empty() {
                    Ok(trace.len())
                } else {
                    Err(vec![Diagnostic::new(
                        "POWL_PARALLEL_FAILED",
                        "Parallel execution validation failed: trace does not match parallel branches"
                    )])
                }
            }
        }
    }
}
