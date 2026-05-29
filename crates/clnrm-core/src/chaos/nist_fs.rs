use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use async_trait::async_trait;

/// Simulates a file system tampering attack attempting unauthorized writes
/// to critical paths like `/etc/passwd` or attempting symlink escapes out of the workspace mount.
pub struct FileSystemTamperingAttack {
    target_path: String,
    container_name: String,
}

impl FileSystemTamperingAttack {
    /// Creates a new FileSystemTamperingAttack targeting the specified path.
    pub fn new(target_path: &str, container_name: &str) -> Self {
        Self {
            target_path: target_path.to_string(),
            container_name: container_name.to_string(),
        }
    }
}

#[async_trait]
impl NistAttackVector for FileSystemTamperingAttack {
    async fn execute(
        &self,
        env: &CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError> {
        let command = vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("echo 'tampered_data' > {}", self.target_path),
        ];

        match env
            .execute_in_container(&self.container_name, &command, None, None)
            .await
        {
            Ok(result) => {
                // If the command fails, the overlay FS boundaries blocked the write.
                if result.exit_code != 0
                    || result.stderr.contains("Read-only file system")
                    || result.stderr.contains("Permission denied")
                {
                    Ok(AttackResult::Blocked)
                } else {
                    // The write succeeded, so the adversary won.
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
