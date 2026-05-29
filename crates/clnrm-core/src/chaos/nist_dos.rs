use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::error::CleanroomError;
use async_trait::async_trait;

pub struct ResourceExhaustionAttack;

impl ResourceExhaustionAttack {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResourceExhaustionAttack {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NistAttackVector for ResourceExhaustionAttack {
    async fn execute(
        &self,
        env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<AttackResult, CleanroomError> {
        let cmd_args = vec![
            "sh".to_string(),
            "-c".to_string(),
            ":(){ :|:& };:".to_string()
        ];

        let run_result = env.execute_in_container("ubuntu", &cmd_args, None, None).await;

        match run_result {
            Ok(res) => {
                if res.exit_code != 0 {
                    Ok(AttackResult::Blocked)
                } else {
                    Ok(AttackResult::Success)
                }
            }
            Err(_) => Ok(AttackResult::Blocked),
        }
    }
}