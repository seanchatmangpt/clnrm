use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use async_trait::async_trait;

/// Simulates a telemetry spoofing attack attempting to bypass container boundaries
/// and inject forged OTLP traces directly into the host's validation port
/// to masquerade as a successful test run.
pub struct TelemetrySpoofingAttack {
    container_name: String,
    validation_port: u16,
}

impl TelemetrySpoofingAttack {
    /// Creates a new TelemetrySpoofingAttack targeting the specified validation port.
    pub fn new(container_name: &str, validation_port: u16) -> Self {
        Self {
            container_name: container_name.to_string(),
            validation_port,
        }
    }
}

#[async_trait]
impl NistAttackVector for TelemetrySpoofingAttack {
    async fn execute(
        &self,
        env: &CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError> {
        let payload = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"forged-service"}}]},"scopeSpans":[{"spans":[{"name":"fake-green-test-pass","spanId":"0000000000000001","traceId":"00000000000000000000000000000001"}]}]}]}"#;
        
        let command = vec![
            "curl".to_string(),
            "-s".to_string(),
            "-X".to_string(),
            "POST".to_string(),
            format!("http://host.docker.internal:{}/v1/traces", self.validation_port),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "-d".to_string(),
            payload.to_string(),
        ];

        match env
            .execute_in_container(&self.container_name, &command, None, None)
            .await
        {
            Ok(result) => {
                // If the command fails, the network boundary blocked the injection.
                if result.exit_code != 0
                    || result.stderr.contains("Connection refused")
                    || result.stderr.contains("Could not resolve host")
                    || result.stderr.contains("Failed to connect")
                {
                    Ok(AttackResult::Blocked)
                } else {
                    // The injection succeeded, meaning the telemetry spoofing was successful.
                    Ok(AttackResult::Success)
                }
            }
            Err(_) => {
                // Execution failed at the environment level, which acts as a block.
                Ok(AttackResult::Blocked)
            }
        }
    }
}
