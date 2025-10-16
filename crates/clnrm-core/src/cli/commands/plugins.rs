//! Plugins command implementation
//!
//! Handles listing and management of available service plugins.

use crate::error::{CleanroomError, Result};
use tracing::info;

/// List available plugins
pub fn list_plugins() -> Result<()> {
    // TODO: Implement actual plugin discovery and listing
    unimplemented!("Plugin listing: Need to implement actual plugin discovery from services/ module. Currently only Generic and SurrealDB plugins exist.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_plugins_returns_unimplemented() -> Result<()> {
        // Act
        let result = list_plugins();
        
        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Plugin listing: Need to implement actual plugin discovery"));
        
        Ok(())
    }

    #[test]
    fn test_list_plugins_error_type() -> Result<()> {
        // Act
        let result = list_plugins();
        
        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind, crate::error::ErrorKind::InternalError);
        
        Ok(())
    }

    #[test]
    fn test_list_plugins_error_message_contains_plugin_info() -> Result<()> {
        // Act
        let result = list_plugins();
        
        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("services/ module"));
        assert!(error.message.contains("Generic and SurrealDB plugins"));
        
        Ok(())
    }
}
