//! Network configuration for gVisor containers

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};

/// Network protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

/// Port mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Container port
    pub container: u16,
    /// Host port (optional, dynamically allocated if not specified)
    #[serde(default)]
    pub host: Option<u16>,
    /// Protocol (TCP or UDP)
    #[serde(default = "default_protocol")]
    pub protocol: Protocol,
}

fn default_protocol() -> Protocol {
    Protocol::Tcp
}

impl PortMapping {
    /// Validate port mapping
    pub fn validate(&self) -> Result<()> {
        if self.container == 0 {
            return Err(CleanroomError::validation_error(
                "Container port cannot be 0",
            ));
        }

        if let Some(host) = self.host {
            if host == 0 {
                return Err(CleanroomError::validation_error("Host port cannot be 0"));
            }
        }

        Ok(())
    }

    /// Get host port (allocated port or container port if not specified)
    pub fn host_port(&self) -> u16 {
        self.host.unwrap_or(self.container)
    }
}

/// Network mode for containers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    /// No network access (maximum isolation)
    None,
    /// Share host network stack
    Host,
    /// Isolated network namespace with bridge
    Bridge,
}

impl Default for NetworkMode {
    fn default() -> Self {
        Self::Bridge
    }
}

/// Network configuration for container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network mode
    #[serde(default)]
    pub mode: NetworkMode,
    /// Hostname for container
    pub hostname: Option<String>,
    /// DNS servers
    #[serde(default)]
    pub dns: Vec<String>,
    /// DNS search domains
    #[serde(default)]
    pub dns_search: Vec<String>,
    /// Extra hosts entries (hostname:ip)
    #[serde(default)]
    pub extra_hosts: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Bridge,
            hostname: None,
            dns: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            dns_search: Vec::new(),
            extra_hosts: Vec::new(),
        }
    }
}

impl NetworkConfig {
    /// Validate network configuration
    pub fn validate(&self) -> Result<()> {
        // Validate DNS servers are valid IP addresses
        for dns in &self.dns {
            if dns.parse::<std::net::IpAddr>().is_err() {
                return Err(CleanroomError::validation_error(format!(
                    "Invalid DNS server IP: {}",
                    dns
                )));
            }
        }

        // Validate extra hosts format
        for host in &self.extra_hosts {
            if !host.contains(':') {
                return Err(CleanroomError::validation_error(format!(
                    "Invalid extra host format '{}', expected 'hostname:ip'",
                    host
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_mapping_validation() {
        let valid = PortMapping {
            container: 8080,
            host: Some(8080),
            protocol: Protocol::Tcp,
        };
        assert!(valid.validate().is_ok());

        let invalid = PortMapping {
            container: 0,
            host: Some(8080),
            protocol: Protocol::Tcp,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_network_config_validation() {
        let valid = NetworkConfig {
            dns: vec!["8.8.8.8".to_string()],
            ..Default::default()
        };
        assert!(valid.validate().is_ok());

        let invalid = NetworkConfig {
            dns: vec!["not-an-ip".to_string()],
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }
}
