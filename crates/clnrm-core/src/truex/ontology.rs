// Full ontology/state-machine for TrueX trust relationships
// Laws define permitted state transitions; violations are recorded as Refusals

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateId(pub String);

/// A guard condition on a transition
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: StateId,
    pub to: StateId,
    pub select_condition: String, // boolean expression as string
    pub effects: Vec<String>,     // side effects to emit
}

/// A law defines which transitions are permitted
#[derive(Debug, Clone)]
pub struct OntologyLaw {
    pub name: String,
    pub transitions: Vec<Transition>,
}

pub struct Ontology {
    pub laws: Vec<OntologyLaw>,
    pub current_state: StateId,
    pub history: Vec<(StateId, StateId, String)>, // (from, to, condition)
}

impl Ontology {
    /// Create a new ontology starting in `initial_state` with no laws.
    pub fn new(initial_state: &str) -> Self {
        Ontology {
            laws: Vec::new(),
            current_state: StateId(initial_state.to_string()),
            history: Vec::new(),
        }
    }

    /// Register an additional law.
    pub fn add_law(&mut self, law: OntologyLaw) {
        self.laws.push(law);
    }

    /// Return all transitions from the current state across all laws.
    pub fn permitted_transitions(&self) -> Vec<&Transition> {
        self.laws
            .iter()
            .flat_map(|law| law.transitions.iter())
            .filter(|t| t.from == self.current_state)
            .collect()
    }

    /// Attempt to apply a transition whose `select_condition` matches `condition`.
    ///
    /// Finds the first matching transition, moves to its target state, records
    /// the transition in history, and returns the list of effects.
    /// Returns `Err` if no matching transition is found from the current state.
    pub fn apply(&mut self, condition: &str) -> Result<Vec<String>, String> {
        // Collect the matching transition data (clone to avoid borrow issues)
        let matched = self
            .laws
            .iter()
            .flat_map(|law| law.transitions.iter())
            .find(|t| t.from == self.current_state && t.select_condition == condition)
            .map(|t| (t.to.clone(), t.select_condition.clone(), t.effects.clone()));

        match matched {
            Some((to, cond, effects)) => {
                let from = self.current_state.clone();
                self.history.push((from, to.clone(), cond));
                self.current_state = to;
                Ok(effects)
            }
            None => Err(format!(
                "No transition from state '{}' with condition '{}'",
                self.current_state.0, condition
            )),
        }
    }

    /// BFS from the current state to determine whether `target` is reachable.
    pub fn can_reach(&self, target: &str) -> bool {
        let target_id = StateId(target.to_string());
        if self.current_state == target_id {
            return true;
        }

        // Build adjacency: state -> set of reachable states
        let mut adj: HashMap<&StateId, Vec<&StateId>> = HashMap::new();
        for law in &self.laws {
            for t in &law.transitions {
                adj.entry(&t.from).or_default().push(&t.to);
            }
        }

        let mut visited: HashSet<&StateId> = HashSet::new();
        let mut queue: VecDeque<&StateId> = VecDeque::new();
        queue.push_back(&self.current_state);
        visited.insert(&self.current_state);

        while let Some(state) = queue.pop_front() {
            if let Some(neighbors) = adj.get(state) {
                for next in neighbors {
                    if *next == &target_id {
                        return true;
                    }
                    if visited.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
        }
        false
    }

    /// Return the full transition history as a slice.
    pub fn state_history(&self) -> &[(StateId, StateId, String)] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn traffic_light_ontology() -> Ontology {
        let mut o = Ontology::new("red");
        let law = OntologyLaw {
            name: "traffic_light".to_string(),
            transitions: vec![
                Transition {
                    from: StateId("red".to_string()),
                    to: StateId("green".to_string()),
                    select_condition: "go".to_string(),
                    effects: vec!["emit:green_light".to_string()],
                },
                Transition {
                    from: StateId("green".to_string()),
                    to: StateId("yellow".to_string()),
                    select_condition: "slow".to_string(),
                    effects: vec!["emit:yellow_light".to_string()],
                },
                Transition {
                    from: StateId("yellow".to_string()),
                    to: StateId("red".to_string()),
                    select_condition: "stop".to_string(),
                    effects: vec!["emit:red_light".to_string()],
                },
            ],
        };
        o.add_law(law);
        o
    }

    #[test]
    fn test_new_ontology() {
        let o = Ontology::new("idle");
        assert_eq!(o.current_state, StateId("idle".to_string()));
        assert!(o.history.is_empty());
        assert!(o.laws.is_empty());
    }

    #[test]
    fn test_permitted_transitions() {
        let o = traffic_light_ontology();
        let pts = o.permitted_transitions();
        assert_eq!(pts.len(), 1);
        assert_eq!(pts[0].select_condition, "go");
    }

    #[test]
    fn test_apply_success() {
        let mut o = traffic_light_ontology();
        let effects = o.apply("go").unwrap();
        assert_eq!(o.current_state, StateId("green".to_string()));
        assert_eq!(effects, vec!["emit:green_light".to_string()]);
        assert_eq!(o.history.len(), 1);
        let (from, to, cond) = &o.history[0];
        assert_eq!(from, &StateId("red".to_string()));
        assert_eq!(to, &StateId("green".to_string()));
        assert_eq!(cond, "go");
    }

    #[test]
    fn test_apply_no_matching_transition() {
        let mut o = traffic_light_ontology();
        let err = o.apply("stop").unwrap_err();
        assert!(err.contains("red"));
        assert!(err.contains("stop"));
    }

    #[test]
    fn test_can_reach_direct() {
        let o = traffic_light_ontology();
        assert!(o.can_reach("green"));
    }

    #[test]
    fn test_can_reach_indirect() {
        let o = traffic_light_ontology();
        assert!(o.can_reach("yellow"));
        assert!(o.can_reach("red")); // current state
    }

    #[test]
    fn test_can_reach_unreachable() {
        let o = traffic_light_ontology();
        assert!(!o.can_reach("purple"));
    }

    #[test]
    fn test_state_history() {
        let mut o = traffic_light_ontology();
        o.apply("go").unwrap();
        o.apply("slow").unwrap();
        let hist = o.state_history();
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[1].1, StateId("yellow".to_string()));
    }

    #[test]
    fn test_full_cycle() {
        let mut o = traffic_light_ontology();
        o.apply("go").unwrap();
        o.apply("slow").unwrap();
        o.apply("stop").unwrap();
        assert_eq!(o.current_state, StateId("red".to_string()));
        assert_eq!(o.history.len(), 3);
    }
}
