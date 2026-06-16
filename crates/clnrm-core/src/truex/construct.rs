// Full circuit/constraint builder for TrueX zero-knowledge proofs
// Supports R1CS constraint system construction for lattice-based ZK proofs

use std::collections::HashMap;

/// A variable in the constraint system (wire)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire(pub usize);

/// A linear combination: sum of (coefficient * wire)
#[derive(Debug, Clone)]
pub struct LinearComb {
    pub terms: Vec<(i64, Wire)>,
    pub constant: i64,
}

impl LinearComb {
    /// Create a linear combination containing a single wire with coefficient 1.
    pub fn new(wire: Wire) -> Self {
        LinearComb {
            terms: vec![(1, wire)],
            constant: 0,
        }
    }

    /// Create a linear combination that is a constant value (no wire terms).
    pub fn constant(v: i64) -> Self {
        LinearComb {
            terms: vec![],
            constant: v,
        }
    }

    /// Add a term `coeff * wire` to this linear combination.
    pub fn add(mut self, coeff: i64, wire: Wire) -> Self {
        self.terms.push((coeff, wire));
        self
    }

    /// Evaluate this linear combination against the provided witness mapping.
    /// Any wire not present in `witness` contributes 0.
    pub fn evaluate(&self, witness: &HashMap<Wire, i64>) -> i64 {
        let mut sum = self.constant;
        for (coeff, wire) in &self.terms {
            let val = witness.get(wire).copied().unwrap_or(0);
            sum += coeff * val;
        }
        sum
    }
}

/// An R1CS constraint: A * B = C
#[derive(Debug, Clone)]
pub struct Constraint {
    pub a: LinearComb,
    pub b: LinearComb,
    pub c: LinearComb,
}

/// The full R1CS constraint system
pub struct Construct {
    pub num_variables: usize,
    pub constraints: Vec<Constraint>,
    pub public_inputs: Vec<Wire>,
    pub witness: HashMap<Wire, i64>,
}

impl Construct {
    /// Create an empty R1CS system.
    /// Wire 0 is reserved as the constant-1 wire and is pre-set in the witness.
    pub fn new() -> Self {
        let mut witness = HashMap::new();
        // Wire 0 = constant 1
        witness.insert(Wire(0), 1);
        Construct {
            num_variables: 1,
            constraints: Vec::new(),
            public_inputs: Vec::new(),
            witness,
        }
    }

    /// Allocate a new wire and return it.
    pub fn add_wire(&mut self) -> Wire {
        let id = self.num_variables;
        self.num_variables += 1;
        Wire(id)
    }

    /// Allocate a wire, mark it as a public input, and set its witness value.
    pub fn add_public_input(&mut self, value: i64) -> Wire {
        let wire = self.add_wire();
        self.public_inputs.push(wire.clone());
        self.witness.insert(wire.clone(), value);
        wire
    }

    /// Push an R1CS constraint A * B = C.
    pub fn add_constraint(&mut self, a: LinearComb, b: LinearComb, c: LinearComb) {
        self.constraints.push(Constraint { a, b, c });
    }

    /// Set the witness value for a wire.
    pub fn set_witness(&mut self, wire: Wire, value: i64) {
        self.witness.insert(wire, value);
    }

    /// Verify all constraints against the current witness.
    /// Returns `true` if every constraint satisfies A*B == C.
    pub fn verify(&self) -> bool {
        for constraint in &self.constraints {
            let a_val = constraint.a.evaluate(&self.witness);
            let b_val = constraint.b.evaluate(&self.witness);
            let c_val = constraint.c.evaluate(&self.witness);
            if a_val * b_val != c_val {
                return false;
            }
        }
        true
    }

    /// Create a multiplication gate: out = a * b.
    /// Allocates output wire, sets its witness, and adds the constraint.
    pub fn mul_gate(&mut self, a: Wire, b: Wire) -> Wire {
        let a_val = self.witness.get(&a).copied().unwrap_or(0);
        let b_val = self.witness.get(&b).copied().unwrap_or(0);
        let out = self.add_wire();
        self.witness.insert(out.clone(), a_val * b_val);
        let lc_a = LinearComb::new(a);
        let lc_b = LinearComb::new(b);
        let lc_c = LinearComb::new(out.clone());
        self.add_constraint(lc_a, lc_b, lc_c);
        out
    }

    /// Create an addition gate: out = a + b.
    /// Uses the constraint (a + b) * 1 = out.
    pub fn add_gate(&mut self, a: Wire, b: Wire) -> Wire {
        let a_val = self.witness.get(&a).copied().unwrap_or(0);
        let b_val = self.witness.get(&b).copied().unwrap_or(0);
        let out = self.add_wire();
        self.witness.insert(out.clone(), a_val + b_val);
        // A = a + b (two terms, no constant)
        let lc_a = LinearComb {
            terms: vec![(1, a), (1, b)],
            constant: 0,
        };
        // B = 1 (constant)
        let lc_b = LinearComb::constant(1);
        let lc_c = LinearComb::new(out.clone());
        self.add_constraint(lc_a, lc_b, lc_c);
        out
    }

    /// Assert that two wires hold equal values: (a - b) * 1 = 0.
    pub fn assert_equal(&mut self, a: Wire, b: Wire) {
        // A = a - b
        let lc_a = LinearComb {
            terms: vec![(1, a), (-1, b)],
            constant: 0,
        };
        // B = 1
        let lc_b = LinearComb::constant(1);
        // C = 0
        let lc_c = LinearComb::constant(0);
        self.add_constraint(lc_a, lc_b, lc_c);
    }

    /// Return the number of public input wires.
    pub fn input_count(&self) -> usize {
        self.public_inputs.len()
    }
}

impl Default for Construct {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_construct_verifies() {
        let c = Construct::new();
        assert!(c.verify());
        assert_eq!(c.input_count(), 0);
    }

    #[test]
    fn test_mul_gate() {
        let mut c = Construct::new();
        let a = c.add_public_input(3);
        let b = c.add_public_input(4);
        let out = c.mul_gate(a, b);
        assert_eq!(c.witness[&out], 12);
        assert!(c.verify());
    }

    #[test]
    fn test_add_gate() {
        let mut c = Construct::new();
        let a = c.add_public_input(5);
        let b = c.add_public_input(7);
        let out = c.add_gate(a, b);
        assert_eq!(c.witness[&out], 12);
        assert!(c.verify());
    }

    #[test]
    fn test_assert_equal() {
        let mut c = Construct::new();
        let a = c.add_public_input(42);
        let b = c.add_public_input(42);
        c.assert_equal(a, b);
        assert!(c.verify());
    }

    #[test]
    fn test_assert_equal_fails() {
        let mut c = Construct::new();
        let a = c.add_public_input(1);
        let b = c.add_public_input(2);
        c.assert_equal(a, b);
        assert!(!c.verify());
    }

    #[test]
    fn test_linear_comb_evaluate() {
        let mut witness = HashMap::new();
        witness.insert(Wire(1), 3_i64);
        witness.insert(Wire(2), 5_i64);
        let lc = LinearComb::new(Wire(1)).add(2, Wire(2)).add(1, Wire(3));
        // 1*3 + 2*5 + 1*0 + 0 = 13
        assert_eq!(lc.evaluate(&witness), 13);
    }
}
