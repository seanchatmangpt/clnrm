//! Test dyn ServicePlugin compatibility

use clnrm_core::cleanroom::{ServicePlugin, MockDatabasePlugin};

#[test]
fn test_dyn_service_plugin_compatibility() {
    // Test that ServicePlugin can be used as a trait object
    let plugin = MockDatabasePlugin::default();
    let plugin_dyn: &dyn ServicePlugin = &plugin;

    // Test that we can call methods through the trait object
    assert_eq!(plugin_dyn.name(), "mock_database");

    // Test that start/stop work without panicking
    let handle = plugin_dyn.start().unwrap();
    assert_eq!(handle.service_name, "mock_database");

    plugin_dyn.stop(handle).unwrap();

    println!("✅ dyn ServicePlugin compatibility verified!");
}
