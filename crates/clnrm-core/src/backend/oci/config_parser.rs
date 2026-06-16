//! OCI config.json parser and runtime config generator

use super::{
    CpuResources, LinuxConfig, MemoryResources, MountConfig, NamespaceConfig, OciImageConfig,
    ProcessConfig, ResourcesConfig, RootConfig, SeccompConfig, SeccompSyscall,
};
use crate::backend::Cmd;
use crate::error::Result;
use crate::policy::{Policy, SecurityLevel};
use serde::{Deserialize, Serialize};

/// OCI runtime configuration (config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(rename = "ociVersion")]
    pub oci_version: String,
    pub process: ProcessConfig,
    pub root: RootConfig,
    pub hostname: String,
    pub mounts: Vec<MountConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<LinuxConfig>,
}

/// OCI image config parser
#[derive(Debug)]
pub struct ConfigParser;

impl ConfigParser {
    /// Parse OCI image config
    pub fn parse(config_data: &[u8]) -> Result<OciImageConfig> {
        let config: OciImageConfig = serde_json::from_slice(config_data)?;
        Ok(config)
    }

    /// Convert OCI image config to runtime config.json for runsc
    pub fn to_runtime_config(
        &self,
        image_config: &OciImageConfig,
        cmd_override: Option<&Cmd>,
        policy: Option<&Policy>,
    ) -> Result<RuntimeConfig> {
        let config_data = &image_config.config;

        // Build process config
        let mut process = ProcessConfig {
            terminal: false,
            user: config_data
                .user
                .clone()
                .unwrap_or_else(|| "0:0".to_string()),
            args: Vec::new(),
            env: config_data.env.clone().unwrap_or_default(),
            cwd: config_data
                .working_dir
                .clone()
                .unwrap_or_else(|| "/".to_string()),
            capabilities: None,
            rlimits: vec![],
            no_new_privileges: true,
        };

        // Handle command override or use image defaults
        if let Some(cmd) = cmd_override {
            // Use provided command
            process.args = vec![cmd.bin.clone()];
            process.args.extend_from_slice(&cmd.args);

            // Merge environment variables
            for (key, value) in &cmd.env {
                // Remove existing env var with same key
                process.env.retain(|e| !e.starts_with(&format!("{}=", key)));
                // Add new env var
                process.env.push(format!("{}={}", key, value));
            }

            // Override working directory if specified
            if let Some(workdir) = &cmd.workdir {
                process.cwd = workdir.to_string_lossy().to_string();
            }
        } else {
            // Use image's CMD and ENTRYPOINT
            if let Some(entrypoint) = &config_data.entrypoint {
                process.args.extend_from_slice(entrypoint);
            }
            if let Some(cmd) = &config_data.cmd {
                process.args.extend_from_slice(cmd);
            }

            // Ensure we have at least a shell if no command
            if process.args.is_empty() {
                process.args = vec!["/bin/sh".to_string()];
            }
        }

        // Apply policy-based environment variables if policy is provided
        if let Some(p) = policy {
            for (key, value) in p.to_env() {
                process.env.push(format!("{}={}", key, value));
            }
        }

        // Build runtime config
        let mut runtime_config = RuntimeConfig {
            oci_version: "1.0.2".to_string(),
            process,
            root: RootConfig {
                path: "rootfs".to_string(),
                readonly: policy
                    .map(|p| p.security.enable_filesystem_isolation)
                    .unwrap_or(false),
            },
            hostname: "clnrm-container".to_string(),
            mounts: self.default_mounts(),
            linux: Some(LinuxConfig {
                namespaces: vec![
                    NamespaceConfig {
                        typ: "pid".to_string(),
                    },
                    NamespaceConfig {
                        typ: "network".to_string(),
                    },
                    NamespaceConfig {
                        typ: "ipc".to_string(),
                    },
                    NamespaceConfig {
                        typ: "uts".to_string(),
                    },
                    NamespaceConfig {
                        typ: "mount".to_string(),
                    },
                ],
                resources: policy.map(|p| ResourcesConfig {
                    memory: Some(MemoryResources {
                        limit: Some(p.resources.max_memory_usage_bytes as i64),
                        reservation: None,
                    }),
                    cpu: Some(CpuResources {
                        shares: Some(1024),
                        quota: Some((p.resources.max_cpu_usage_percent * 1000.0) as i64),
                        period: Some(100000),
                    }),
                }),
                masked_paths: vec![
                    "/proc/kcore".to_string(),
                    "/proc/latency_stats".to_string(),
                    "/proc/timer_list".to_string(),
                    "/proc/sched_debug".to_string(),
                ],
                readonly_paths: vec![
                    "/proc/asound".to_string(),
                    "/proc/bus".to_string(),
                    "/proc/fs".to_string(),
                    "/proc/irq".to_string(),
                    "/proc/sys".to_string(),
                    "/proc/sysrq-trigger".to_string(),
                ],
                seccomp: None,
            }),
        };

        // Apply seccomp profile if required by security level
        if let Some(p) = policy {
            if p.security.security_level != SecurityLevel::Low {
                runtime_config.linux.as_mut().unwrap().seccomp = // OK: linux is Some when security level is non-Low
                    Some(self.generate_seccomp_profile(p));
            }
        }

        Ok(runtime_config)
    }

    /// Generate seccomp profile based on policy
    fn generate_seccomp_profile(&self, policy: &Policy) -> SeccompConfig {
        let mut syscalls = vec![SeccompSyscall {
            names: vec![
                "clone".to_string(),
                "mount".to_string(),
                "umount2".to_string(),
                "ptrace".to_string(),
            ],
            action: "SCMP_ACT_ERRNO".to_string(),
        }];

        if policy.security.security_level == SecurityLevel::Locked {
            syscalls.push(SeccompSyscall {
                names: vec![
                    "socket".to_string(),
                    "connect".to_string(),
                    "bind".to_string(),
                ],
                action: "SCMP_ACT_ERRNO".to_string(),
            });
        }

        SeccompConfig {
            default_action: "SCMP_ACT_ALLOW".to_string(),
            syscalls,
        }
    }

    /// Default mounts for container
    fn default_mounts(&self) -> Vec<MountConfig> {
        vec![
            MountConfig {
                destination: "/proc".to_string(),
                typ: "proc".to_string(),
                source: "proc".to_string(),
                options: vec![],
            },
            MountConfig {
                destination: "/dev".to_string(),
                typ: "tmpfs".to_string(),
                source: "tmpfs".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "strictatime".to_string(),
                    "mode=755".to_string(),
                ],
            },
            MountConfig {
                destination: "/dev/pts".to_string(),
                typ: "devpts".to_string(),
                source: "devpts".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "newinstance".to_string(),
                ],
            },
            MountConfig {
                destination: "/dev/shm".to_string(),
                typ: "tmpfs".to_string(),
                source: "shm".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "mode=1777".to_string(),
                ],
            },
            MountConfig {
                destination: "/sys".to_string(),
                typ: "sysfs".to_string(),
                source: "sysfs".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "ro".to_string(),
                ],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_generation() {
        let parser = ConfigParser;

        // Create minimal image config
        let image_config = OciImageConfig {
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            config: super::super::OciContainerConfig {
                user: Some("1000:1000".to_string()),
                exposed_ports: None,
                env: Some(vec!["PATH=/usr/bin:/bin".to_string()]),
                cmd: Some(vec!["sh".to_string()]),
                volumes: None,
                working_dir: Some("/app".to_string()),
                entrypoint: None,
                labels: None,
            },
            rootfs: super::super::OciRootfs {
                typ: "layers".to_string(),
                diff_ids: vec![],
            },
            history: None,
        };

        let runtime_config = parser.to_runtime_config(&image_config, None, None).unwrap();

        assert_eq!(runtime_config.oci_version, "1.0.2");
        assert_eq!(runtime_config.process.user, "1000:1000");
        assert_eq!(runtime_config.process.cwd, "/app");
        assert!(runtime_config.process.args.contains(&"sh".to_string()));
        assert!(!runtime_config.mounts.is_empty());
    }
}
