use serde::{Deserialize, Serialize};

/// Configuration for NIST adversarial attack vector testing.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NistAdversarialConfig {
    /// Enable sandbox escape tests. Defaults to false.
    #[serde(default)]
    pub enable_escape_tests: bool,

    /// Enable Denial of Service (DoS) tests. Defaults to false.
    #[serde(default)]
    pub enable_dos_tests: bool,

    /// Enable resource exhaustion tests. Defaults to false.
    #[serde(default)]
    pub enable_resource_exhaustion_tests: bool,

    /// Enable privilege escalation tests. Defaults to false.
    #[serde(default)]
    pub enable_privilege_escalation_tests: bool,

    /// Enable network exfiltration tests. Defaults to false.
    #[serde(default)]
    pub enable_network_exfiltration_tests: bool,

    /// Maximum time in seconds to allow an adversarial test to run.
    #[serde(default = "default_test_timeout")]
    pub test_timeout_seconds: u32,

    /// Maximum allowed memory usage in MB for adversarial tests.
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u32,
}

impl Default for NistAdversarialConfig {
    fn default() -> Self {
        Self {
            enable_escape_tests: false,
            enable_dos_tests: false,
            enable_resource_exhaustion_tests: false,
            enable_privilege_escalation_tests: false,
            enable_network_exfiltration_tests: false,
            test_timeout_seconds: default_test_timeout(),
            max_memory_mb: default_max_memory_mb(),
        }
    }
}

fn default_test_timeout() -> u32 {
    30 // 30 seconds default timeout
}

fn default_max_memory_mb() -> u32 {
    512 // 512 MB default limit
}
