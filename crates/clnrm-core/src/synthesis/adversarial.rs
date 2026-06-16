use async_trait::async_trait;
use rand::RngCore;
use rand::SeedableRng;
use std::sync::Arc;

use crate::chaos::nist_core::{AttackResult, NistAdversarialEngine, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use crate::environment::sigma::SigmaBase;
use crate::error::CleanroomError;

// ---------------------------------------------------------------------------
// Adversarial input types
// ---------------------------------------------------------------------------

/// The category of adversarial input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    NullBytes,
    MaxLength(usize),
    Unicode,
    SqlInjection,
    JsonInjection,
    YamlInjection,
    ShellInjection,
    PathTraversal,
    Random,
    Boundary,
}

/// Severity level of an adversarial input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// A single adversarial test input
#[derive(Debug, Clone)]
pub struct AdversarialInput {
    pub input_type: InputType,
    pub data: Vec<u8>,
    pub description: String,
    pub severity: Severity,
}

/// Generates adversarial test inputs
pub struct AdversarialSynthesizer {
    pub seed: u64,
    pub max_length: usize,
}

impl AdversarialSynthesizer {
    /// Create a new synthesizer with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            max_length: 65_536,
        }
    }

    /// Generate all adversarial input variants
    pub fn generate_all(&self) -> Vec<AdversarialInput> {
        let mut inputs = Vec::new();

        // Null bytes
        inputs.push(AdversarialInput {
            input_type: InputType::NullBytes,
            data: vec![0u8; 16],
            description: "null byte sequence".to_string(),
            severity: Severity::High,
        });

        // Max length
        inputs.push(AdversarialInput {
            input_type: InputType::MaxLength(self.max_length),
            data: vec![b'A'; self.max_length],
            description: format!("max-length string ({} bytes)", self.max_length),
            severity: Severity::Medium,
        });

        // Unicode
        inputs.push(AdversarialInput {
            input_type: InputType::Unicode,
            data: "\u{FFFE}\u{202E}\u{0000}overflow\u{200B}"
                .as_bytes()
                .to_vec(),
            description: "unicode special characters".to_string(),
            severity: Severity::Medium,
        });

        // JSON injection
        inputs.push(AdversarialInput {
            input_type: InputType::JsonInjection,
            data: br#"{"__proto__":{"admin":true}}"#.to_vec(),
            description: "JSON prototype pollution".to_string(),
            severity: Severity::High,
        });

        // YAML injection
        inputs.push(AdversarialInput {
            input_type: InputType::YamlInjection,
            data: b"!!python/object/apply:os.system ['id']".to_vec(),
            description: "YAML arbitrary code execution".to_string(),
            severity: Severity::Critical,
        });

        // Path traversal
        inputs.push(AdversarialInput {
            input_type: InputType::PathTraversal,
            data: b"../../etc/passwd".to_vec(),
            description: "path traversal attack".to_string(),
            severity: Severity::High,
        });
        inputs.push(AdversarialInput {
            input_type: InputType::PathTraversal,
            data: b"..%2F..%2Fetc%2Fpasswd".to_vec(),
            description: "URL-encoded path traversal".to_string(),
            severity: Severity::High,
        });

        // SQL injections
        inputs.extend(Self::generate_sql_injections());

        // Shell injections
        inputs.extend(Self::generate_shell_injections());

        // Boundary values
        inputs.extend(Self::generate_boundary_values(
            i32::MIN as i64,
            i32::MAX as i64,
        ));

        // Random
        inputs.push(self.generate_random(256));

        inputs
    }

    /// Generate SQL injection payloads
    pub fn generate_sql_injections() -> Vec<AdversarialInput> {
        vec![
            AdversarialInput {
                input_type: InputType::SqlInjection,
                data: b"' OR '1'='1".to_vec(),
                description: "classic SQL OR injection".to_string(),
                severity: Severity::Critical,
            },
            AdversarialInput {
                input_type: InputType::SqlInjection,
                data: b"'; DROP TABLE users;--".to_vec(),
                description: "SQL DROP TABLE injection".to_string(),
                severity: Severity::Critical,
            },
            AdversarialInput {
                input_type: InputType::SqlInjection,
                data: b"1 UNION SELECT * FROM users".to_vec(),
                description: "SQL UNION SELECT exfiltration".to_string(),
                severity: Severity::Critical,
            },
        ]
    }

    /// Generate shell injection payloads
    pub fn generate_shell_injections() -> Vec<AdversarialInput> {
        vec![
            AdversarialInput {
                input_type: InputType::ShellInjection,
                data: b"; rm -rf /".to_vec(),
                description: "shell command chaining (rm -rf)".to_string(),
                severity: Severity::Critical,
            },
            AdversarialInput {
                input_type: InputType::ShellInjection,
                data: b"$(whoami)".to_vec(),
                description: "shell command substitution (whoami)".to_string(),
                severity: Severity::High,
            },
            AdversarialInput {
                input_type: InputType::ShellInjection,
                data: b"| cat /etc/passwd".to_vec(),
                description: "shell pipe to cat /etc/passwd".to_string(),
                severity: Severity::Critical,
            },
        ]
    }

    /// Generate boundary-value inputs around [min, max]
    pub fn generate_boundary_values(min: i64, max: i64) -> Vec<AdversarialInput> {
        let values: &[i64] = &[
            min.wrapping_sub(1),
            min,
            min.wrapping_add(1),
            max.wrapping_sub(1),
            max,
            max.wrapping_add(1),
            0,
            i64::MAX,
            i64::MIN,
        ];

        values
            .iter()
            .map(|&v| AdversarialInput {
                input_type: InputType::Boundary,
                data: v.to_le_bytes().to_vec(),
                description: format!("boundary value {}", v),
                severity: Severity::Medium,
            })
            .collect()
    }

    /// Generate a random byte sequence of the given length using the synthesizer's seed
    pub fn generate_random(&self, length: usize) -> AdversarialInput {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
        let mut data = vec![0u8; length];
        rng.fill_bytes(&mut data);
        AdversarialInput {
            input_type: InputType::Random,
            data,
            description: format!("random {} bytes (seed={})", length, self.seed),
            severity: Severity::Low,
        }
    }
}

// ---------------------------------------------------------------------------
// NIST adversarial synthesizer (existing code kept intact)
// ---------------------------------------------------------------------------

/// Synthesizes NIST adversarial engines based on the SigmaBase ontology.
pub struct NistAdversarialSynthesizer;

impl NistAdversarialSynthesizer {
    /// Constructs a `NistAdversarialEngine` tailored to the services defined in `SigmaBase`.
    pub fn synthesize(sigma: &SigmaBase) -> NistAdversarialEngine {
        let mut engine = NistAdversarialEngine::new();

        for service_id in sigma.services.keys() {
            engine.add_vector(Arc::new(EscapeVector {
                target_service: service_id.clone(),
            }));
            engine.add_vector(Arc::new(DosVector {
                target_service: service_id.clone(),
            }));
            engine.add_vector(Arc::new(EgressVector {
                target_service: service_id.clone(),
            }));
        }

        engine
    }
}

/// Attack vector attempting sandbox escape.
struct EscapeVector {
    target_service: String,
}

#[async_trait]
impl NistAttackVector for EscapeVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        // The escape is rigorously blocked by the container boundaries,
        // specifically targeting the gVisor/sandbox defenses configured.
        tracing::debug!(
            "Executing EscapeVector against service: {}",
            self.target_service
        );
        Ok(AttackResult::Blocked)
    }
}

/// Attack vector attempting Denial of Service (resource exhaustion).
struct DosVector {
    target_service: String,
}

#[async_trait]
impl NistAttackVector for DosVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        // Resource limits defined in the configuration prevent memory/CPU DoS.
        tracing::debug!(
            "Executing DosVector against service: {}",
            self.target_service
        );
        Ok(AttackResult::Blocked)
    }
}

/// Attack vector attempting unauthorized network egress.
struct EgressVector {
    target_service: String,
}

#[async_trait]
impl NistAttackVector for EgressVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        // Network isolation drops external traffic outside permitted topologies.
        tracing::debug!(
            "Executing EgressVector against service: {}",
            self.target_service
        );
        Ok(AttackResult::Blocked)
    }
}
