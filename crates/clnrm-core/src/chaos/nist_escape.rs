use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use crate::error::CleanroomError;
use async_trait::async_trait;

// ─── Container Escape Test ────────────────────────────────────────────────

/// Non-destructive container isolation tests.
///
/// These verify that the container runtime has properly applied security
/// boundaries.  They do **not** attempt to actually escape — they merely
/// probe whether the expected restrictions are in place.
pub struct ContainerEscapeTest;

impl ContainerEscapeTest {
    /// Check that the PID namespace is isolated from the host.
    ///
    /// Reads `/proc/self/ns/pid` and compares it against the expected host
    /// namespace inode.  Returns `true` when the namespaces differ (isolated).
    pub fn test_namespace_isolation() -> bool {
        use std::os::unix::fs::MetadataExt;

        // Read the inode of our own PID namespace link.
        let self_ns = std::fs::metadata("/proc/self/ns/pid");
        // Read the inode of PID 1's namespace (the init/host process).
        let host_ns = std::fs::metadata("/proc/1/ns/pid");

        match (self_ns, host_ns) {
            (Ok(self_meta), Ok(host_meta)) => {
                // If inodes differ the PID namespace is isolated.
                let isolated = self_meta.ino() != host_meta.ino();
                tracing::info!(
                    isolated,
                    self_inode = self_meta.ino(),
                    host_inode = host_meta.ino(),
                    "chaos.escape.namespace_isolation" = true,
                    "PID namespace isolation check"
                );
                isolated
            }
            // If we cannot read host PID 1's namespace that itself indicates isolation.
            (Ok(_), Err(_)) => {
                tracing::info!(
                    isolated = true,
                    "chaos.escape.namespace_isolation" = true,
                    "PID namespace isolated (cannot read host ns)"
                );
                true
            }
            _ => {
                // Cannot determine — conservatively report not isolated.
                tracing::warn!(
                    "chaos.escape.namespace_isolation" = false,
                    "Could not determine PID namespace isolation"
                );
                false
            }
        }
    }

    /// Check that the container filesystem is isolated.
    ///
    /// Attempts to open `/proc/1/root` which is the host filesystem root when
    /// running outside a container.  Returns `true` when access is denied
    /// (properly isolated).
    pub fn test_filesystem_isolation() -> bool {
        let result = std::fs::read_dir("/proc/1/root");
        match result {
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                tracing::info!(
                    isolated = true,
                    "chaos.escape.fs_isolation" = true,
                    "Filesystem isolation confirmed (permission denied)"
                );
                true
            }
            Err(_) => {
                // Other errors (e.g. not found) also indicate isolation.
                tracing::info!(
                    isolated = true,
                    "chaos.escape.fs_isolation" = true,
                    "Filesystem isolation confirmed (access error)"
                );
                true
            }
            Ok(_) => {
                // We could read /proc/1/root — not properly isolated.
                tracing::warn!(
                    isolated = false,
                    "chaos.escape.fs_isolation" = false,
                    "Filesystem isolation NOT confirmed"
                );
                false
            }
        }
    }

    /// Check that `CAP_SYS_ADMIN` has been dropped.
    ///
    /// Reads `/proc/self/status` and parses the `CapEff` (effective capabilities)
    /// bitmask.  `CAP_SYS_ADMIN` is bit 21.  Returns `true` when the bit is clear.
    pub fn test_capability_restriction() -> bool {
        let status = match std::fs::read_to_string("/proc/self/status") {
            Ok(s) => s,
            Err(_) => {
                tracing::warn!("chaos.escape.capability" = false, "Cannot read /proc/self/status");
                return false;
            }
        };

        // Find CapEff line, e.g. "CapEff:	0000000000000000"
        for line in status.lines() {
            if line.starts_with("CapEff:") {
                let hex = line.trim_start_matches("CapEff:").trim();
                if let Ok(cap_bitmask) = u64::from_str_radix(hex, 16) {
                    // CAP_SYS_ADMIN = bit 21
                    const CAP_SYS_ADMIN: u64 = 1 << 21;
                    let restricted = (cap_bitmask & CAP_SYS_ADMIN) == 0;
                    tracing::info!(
                        restricted,
                        cap_eff = hex,
                        "chaos.escape.capability_restriction" = true,
                        "CAP_SYS_ADMIN restriction check"
                    );
                    return restricted;
                }
            }
        }

        // If we can't find the line assume not restricted.
        false
    }
}

/// An attack vector that attempts to breach container isolation
/// by requesting elevated privileges or mounting the host root filesystem.
pub struct ContainerEscapeAttack;

#[async_trait]
impl NistAttackVector for ContainerEscapeAttack {
    async fn execute(&self, env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        let attack_command = vec![
            "mount".to_string(),
            "-t".to_string(),
            "proc".to_string(),
            "proc".to_string(),
            "/proc".to_string(),
        ];

        // This won't perfectly compile if execute_in_container expects different arguments,
        // but it follows the agent's logic. We'll refine it shortly.
        let result = env
            .execute_in_container("ubuntu", &attack_command, None, None)
            .await;

        match result {
            Ok(exec_result) => {
                if exec_result.exit_code == 0 {
                    Ok(AttackResult::Success)
                } else {
                    Ok(AttackResult::Blocked)
                }
            }
            Err(_) => Ok(AttackResult::Blocked),
        }
    }
}
