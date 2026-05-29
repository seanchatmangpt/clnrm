//! MACRO-GALL-1: The Macro Lifecycle Gate
//!
//! This test ensures that the `cleanroom_test` macro infrastructure
//! (specifically the `with_database`, `with_cache`, etc. helpers and plugins)
//! does NOT silently pretend to execute lifecycles that are unimplemented.
//! If the capability is not fully wired to gVisor, it must explicitly fail.

use clnrm_core::cleanroom::{ServiceHandle, ServicePlugin};
use clnrm_core::macros::{with_database, DatabaseServicePlugin};

#[tokio::test]
async fn macro_gall_1_refuses_fake_database_lifecycle() {
    // Arrange
    let plugin = DatabaseServicePlugin::new("postgres:latest");

    // Act & Assert for Plugin
    let result = plugin.start();
    assert!(
        result.is_err(),
        "MACRO-GALL-1 Failed: DatabaseServicePlugin silently pretended to start a database without actual gVisor bindings. It must explicitly refuse."
    );

    // Act & Assert for Declarative Helper
    let helper_result = with_database("postgres:latest").await;
    assert!(
        helper_result.is_err(),
        "MACRO-GALL-1 Failed: with_database() helper silently pretended to set up a database without actual gVisor bindings. It must explicitly refuse."
    );
}
