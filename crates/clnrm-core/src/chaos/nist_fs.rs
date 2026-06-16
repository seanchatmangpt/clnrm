use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

// ─── Inode Exhaustor ──────────────────────────────────────────────────────

/// Exhausts inodes in a directory by creating many small empty files.
pub struct InodeExhaustor;

impl InodeExhaustor {
    /// Create `count` empty files inside `dir` and return their paths.
    pub fn inject(dir: &Path, count: usize) -> Result<Vec<PathBuf>, crate::error::CleanroomError> {
        let mut paths = Vec::with_capacity(count);
        for i in 0..count {
            let p = dir.join(format!("clnrm_inode_{}.chaos", i));
            std::fs::File::create(&p).map_err(|e| {
                crate::error::CleanroomError::io_error(format!(
                    "InodeExhaustor: failed to create {:?}: {}",
                    p, e
                ))
            })?;
            paths.push(p);
        }
        Ok(paths)
    }

    /// Delete the files created by [`inject`].
    pub fn cleanup(files: &[PathBuf]) -> Result<(), crate::error::CleanroomError> {
        for path in files {
            std::fs::remove_file(path).map_err(|e| {
                crate::error::CleanroomError::io_error(format!(
                    "InodeExhaustor: cleanup failed for {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }
}

// ─── Permission Changer ───────────────────────────────────────────────────

/// Changes file permissions to simulate access-control chaos scenarios.
pub struct PermissionChanger;

impl PermissionChanger {
    /// Remove all permissions from `path` (chmod 000).
    pub fn make_unreadable(path: &Path) -> Result<(), crate::error::CleanroomError> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let perms = fs::Permissions::from_mode(0o000);
        fs::set_permissions(path, perms).map_err(|e| {
            crate::error::CleanroomError::io_error(format!(
                "PermissionChanger: chmod 000 failed for {:?}: {}",
                path, e
            ))
        })
    }

    /// Restore `path` to `original_mode` (e.g. 0o644).
    pub fn restore(path: &Path, original_mode: u32) -> Result<(), crate::error::CleanroomError> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let perms = fs::Permissions::from_mode(original_mode);
        fs::set_permissions(path, perms).map_err(|e| {
            crate::error::CleanroomError::io_error(format!(
                "PermissionChanger: chmod {:o} failed for {:?}: {}",
                original_mode, path, e
            ))
        })
    }
}

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
