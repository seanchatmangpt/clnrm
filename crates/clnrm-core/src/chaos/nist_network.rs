use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use async_trait::async_trait;

/// Simulates a network egress attack attempting to connect to external
/// unauthorized IPs (like 8.8.8.8) or perform DNS tunneling.
pub struct NetworkEgressAttack {
    target_ip: String,
    container_name: String,
}

impl NetworkEgressAttack {
    /// Creates a new NetworkEgressAttack targeting the specified IP.
    pub fn new(target_ip: &str, container_name: &str) -> Self {
        Self {
            target_ip: target_ip.to_string(),
            container_name: container_name.to_string(),
        }
    }
}

#[async_trait]
impl NistAttackVector for NetworkEgressAttack {
    async fn execute(
        &self,
        env: &CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError> {
        let command = vec![
            "ping".to_string(),
            "-c".to_string(),
            "1".to_string(),
            "-W".to_string(),
            "2".to_string(),
            self.target_ip.clone(),
        ];

        match env
            .execute_in_container(&self.container_name, &command, None, None)
            .await
        {
            Ok(result) => {
                // If the command fails, the network policies successfully blocked the egress.
                if result.exit_code != 0
                    || result.stderr.contains("Network is unreachable")
                    || result.stderr.contains("Operation not permitted")
                    || result.stdout.contains("100% packet loss")
                    || result.stderr.contains("100% packet loss")
                {
                    Ok(AttackResult::Blocked)
                } else {
                    // The egress succeeded, so the adversary won.
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
