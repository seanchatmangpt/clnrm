//! Generative Grammar Engine for TrueX
//!
//! Implements BNF-like grammar parsing, random string generation, validation,
//! and grammar composition for generative constitution enforcement.

use rand::Rng;
use std::collections::HashMap;

use crate::error::{CleanroomError, Result};

/// A BNF-like production grammar.
///
/// Grammar rules take the form: `<symbol> ::= production1 | production2`
/// Each production is a sequence of terminals (lowercase) and non-terminals (<symbol>).
/// The grammar generates strings by recursively expanding from the start symbol.
#[derive(Debug, Clone)]
pub struct Grammar {
    /// Map from non-terminal name to list of alternative productions (each production is a list of symbols).
    pub rules: HashMap<String, Vec<Vec<String>>>,
    /// The start symbol for derivation.
    pub start_symbol: String,
    /// Maximum recursion depth during generation / validation.
    max_depth: usize,
}

impl Grammar {
    /// Create a new Grammar with default max depth.
    pub fn new(start_symbol: String) -> Self {
        Self {
            rules: HashMap::new(),
            start_symbol,
            max_depth: 20,
        }
    }

    /// Add a rule: `lhs` expands to one of the `productions`.
    /// Each production is a `Vec<String>` of symbols (terminals or `<NonTerminal>`).
    pub fn add_rule(&mut self, lhs: String, productions: Vec<Vec<String>>) {
        self.rules.entry(lhs).or_default().extend(productions);
    }

    /// Parse a BNF-like grammar definition string.
    ///
    /// Syntax:
    /// ```text
    /// <start> ::= <a> " " <b> | literal
    /// <a> ::= hello | world
    /// <b> ::= foo | bar
    /// ```
    ///
    /// Lines starting with `//` or `#` are comments and are skipped.
    /// Each line must contain `::=`.
    /// Alternatives are separated by ` | `.
    /// Each alternative is space-split into symbols.
    /// Quoted strings are treated as single terminal tokens (quotes stripped).
    pub fn from_str(grammar_def: &str) -> Result<Self> {
        let mut grammar = Grammar {
            rules: HashMap::new(),
            start_symbol: String::new(),
            max_depth: 20,
        };

        let mut first_rule = true;
        for line in grammar_def.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, "::=").collect();
            if parts.len() != 2 {
                return Err(CleanroomError::validation_error(format!(
                    "Invalid grammar rule (missing '::='): {}",
                    line
                )));
            }

            let lhs = parts[0].trim().to_string();
            if lhs.is_empty() {
                return Err(CleanroomError::validation_error(
                    "Grammar rule left-hand side cannot be empty",
                ));
            }

            if first_rule {
                grammar.start_symbol = lhs.clone();
                first_rule = false;
            }

            let rhs = parts[1].trim();
            let alternatives: Vec<Vec<String>> = rhs
                .split(" | ")
                .map(|alt| parse_production(alt.trim()))
                .collect();

            grammar.rules.entry(lhs).or_default().extend(alternatives);
        }

        if grammar.start_symbol.is_empty() {
            return Err(CleanroomError::validation_error("Grammar has no rules"));
        }

        Ok(grammar)
    }

    /// Generate a random string by derivation from the start symbol.
    ///
    /// Uses the provided RNG to pick alternatives at each step.
    /// Stops expanding at `max_depth` to avoid infinite recursion.
    pub fn generate(&self, rng: &mut impl Rng) -> String {
        self.expand_symbol(&self.start_symbol, rng, 0)
    }

    /// Generate a random string from a specific symbol.
    pub fn generate_from(&self, symbol: &str, rng: &mut impl Rng) -> String {
        self.expand_symbol(symbol, rng, 0)
    }

    fn expand_symbol(&self, symbol: &str, rng: &mut impl Rng, depth: usize) -> String {
        // If the symbol is a non-terminal wrapped in < >
        let symbol = if symbol.starts_with('<') && symbol.ends_with('>') {
            &symbol[1..symbol.len() - 1]
        } else {
            symbol
        };

        if depth >= self.max_depth {
            // Return the symbol as a terminal at max depth
            return symbol.to_string();
        }

        if let Some(alternatives) = self.rules.get(symbol) {
            if alternatives.is_empty() {
                return symbol.to_string();
            }
            // Pick a random alternative
            let idx = rng.random_range(0..alternatives.len());
            let production = &alternatives[idx];
            production
                .iter()
                .map(|sym| {
                    // Recursively expand non-terminals
                    if (sym.starts_with('<') && sym.ends_with('>'))
                        || self.rules.contains_key(sym.as_str())
                    {
                        self.expand_symbol(sym, rng, depth + 1)
                    } else {
                        // Terminal — return as-is
                        sym.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join("")
        } else {
            // Not a known non-terminal — treat as terminal
            symbol.to_string()
        }
    }

    /// Check whether the given input string can be generated by this grammar.
    ///
    /// Uses CYK-inspired recursive descent for simple grammars.
    /// For grammars with left-recursion this may not terminate — max depth is applied.
    pub fn validate_string(&self, input: &str) -> bool {
        let tokens = tokenize(input);
        self.matches_symbol(&self.start_symbol, &tokens, 0, tokens.len(), 0)
            .is_some()
    }

    fn matches_symbol(
        &self,
        symbol: &str,
        tokens: &[String],
        start: usize,
        end: usize,
        depth: usize,
    ) -> Option<usize> {
        if depth >= self.max_depth {
            return None;
        }

        let symbol = if symbol.starts_with('<') && symbol.ends_with('>') {
            &symbol[1..symbol.len() - 1]
        } else {
            symbol
        };

        if let Some(alternatives) = self.rules.get(symbol) {
            for production in alternatives {
                if let Some(consumed) =
                    self.matches_production(production, tokens, start, end, depth + 1)
                {
                    return Some(consumed);
                }
            }
            None
        } else {
            // Terminal symbol — check if tokens[start] matches
            if start < end && start < tokens.len() && tokens[start] == symbol {
                Some(start + 1)
            } else {
                None
            }
        }
    }

    fn matches_production(
        &self,
        production: &[String],
        tokens: &[String],
        start: usize,
        end: usize,
        depth: usize,
    ) -> Option<usize> {
        let mut pos = start;
        for sym in production {
            if let Some(next) = self.matches_symbol(sym, tokens, pos, end, depth + 1) {
                pos = next;
            } else {
                return None;
            }
        }
        Some(pos)
    }

    /// Sequential composition: first derive from `a`, then from `b`.
    ///
    /// Creates a new grammar where the start symbol generates strings that are
    /// concatenations of strings from `a` and then `b`.
    pub fn compose(a: &Grammar, b: &Grammar) -> Grammar {
        let mut rules = HashMap::new();

        // Prefix 'a' rules with "a__" and 'b' rules with "b__" to avoid collisions
        for (sym, alts) in &a.rules {
            let new_sym = format!("a__{}", sym);
            let new_alts: Vec<Vec<String>> = alts
                .iter()
                .map(|prod| {
                    prod.iter()
                        .map(|s| prefix_symbol(s, "a__", &a.rules))
                        .collect()
                })
                .collect();
            rules.insert(new_sym, new_alts);
        }

        for (sym, alts) in &b.rules {
            let new_sym = format!("b__{}", sym);
            let new_alts: Vec<Vec<String>> = alts
                .iter()
                .map(|prod| {
                    prod.iter()
                        .map(|s| prefix_symbol(s, "b__", &b.rules))
                        .collect()
                })
                .collect();
            rules.insert(new_sym, new_alts);
        }

        // The composed start symbol generates a then b
        let composed_start = "composed__start".to_string();
        let a_start = format!("a__{}", a.start_symbol);
        let b_start = format!("b__{}", b.start_symbol);
        rules.insert(composed_start.clone(), vec![vec![a_start, b_start]]);

        Grammar {
            rules,
            start_symbol: composed_start,
            max_depth: a.max_depth.max(b.max_depth),
        }
    }

    /// Set the maximum recursion depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Returns the set of defined non-terminal symbols.
    pub fn non_terminals(&self) -> Vec<&String> {
        self.rules.keys().collect()
    }

    /// Returns true if the grammar has a rule for the given symbol.
    pub fn has_rule(&self, symbol: &str) -> bool {
        self.rules.contains_key(symbol)
    }
}

/// Prefix a symbol reference with a namespace, if it refers to a known non-terminal.
fn prefix_symbol(sym: &str, prefix: &str, known: &HashMap<String, Vec<Vec<String>>>) -> String {
    let inner = if sym.starts_with('<') && sym.ends_with('>') {
        &sym[1..sym.len() - 1]
    } else {
        sym
    };

    if known.contains_key(inner) {
        format!("{}{}", prefix, inner)
    } else {
        sym.to_string()
    }
}

/// Parse a single production (alternative) into a list of symbols.
/// Handles quoted strings as single terminals, and `<NonTerminal>` references.
fn parse_production(alt: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut chars = alt.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '"' | '\'' => {
                // Quoted terminal string
                let quote = c;
                chars.next();
                let mut token = String::new();
                for ch in chars.by_ref() {
                    if ch == quote {
                        break;
                    }
                    token.push(ch);
                }
                if !token.is_empty() {
                    symbols.push(token);
                }
            }
            '<' => {
                // Non-terminal reference like <symbol>
                let mut token = String::new();
                for ch in chars.by_ref() {
                    token.push(ch);
                    if ch == '>' {
                        break;
                    }
                }
                // Store inner name (without < >) for lookup
                let inner = token
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string();
                symbols.push(inner);
            }
            ' ' | '\t' => {
                chars.next();
                // skip whitespace between symbols
            }
            _ => {
                // Bare terminal or identifier
                let mut token = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == '\t' {
                        break;
                    }
                    token.push(ch);
                    chars.next();
                }
                if !token.is_empty() {
                    symbols.push(token);
                }
            }
        }
    }

    symbols
}

/// Tokenize input string: split on whitespace into tokens.
/// Punctuation adjacent to words is kept as part of the token (matching terminal behavior).
fn tokenize(input: &str) -> Vec<String> {
    input.split_whitespace().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_grammar_from_str_and_generate() {
        let def = "
            greeting ::= <hello> <name>
            hello ::= hello | hi
            name ::= world | rust
        ";
        let g = Grammar::from_str(def).expect("parse failed");
        assert_eq!(g.start_symbol, "greeting");

        let mut rng = SmallRng::seed_from_u64(42);
        let output = g.generate(&mut rng);
        // Should be "helloworld", "hiworld", "hellorust", or "hirust" (tokens joined without separator)
        assert!(!output.is_empty(), "generated string should not be empty");
    }

    #[test]
    fn test_grammar_validate() {
        let def = "greeting ::= hello world | hi there";
        let g = Grammar::from_str(def).expect("parse failed");

        // split into tokens matches
        assert!(g.validate_string("hello world"));
        assert!(g.validate_string("hi there"));
        assert!(!g.validate_string("bye world"));
    }

    #[test]
    fn test_grammar_compose() {
        let def_a = "A ::= hello";
        let def_b = "B ::= world";
        let ga = Grammar::from_str(def_a).expect("parse a");
        let gb = Grammar::from_str(def_b).expect("parse b");
        let gc = Grammar::compose(&ga, &gb);

        let mut rng = SmallRng::seed_from_u64(0);
        let result = gc.generate(&mut rng);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_grammar_add_rule() {
        let mut g = Grammar::new("start".to_string());
        g.add_rule(
            "start".to_string(),
            vec![vec!["foo".to_string()], vec!["bar".to_string()]],
        );
        assert!(g.has_rule("start"));
        let mut rng = SmallRng::seed_from_u64(7);
        let s = g.generate(&mut rng);
        assert!(s == "foo" || s == "bar");
    }
}
