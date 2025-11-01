//! Debug test: Verify variable extraction from [variables] section

use clnrm_core::config::loader::load_config_from_file;
use std::fs;

#[test]
fn test_variables_section_extraction() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test-variables-extraction.clnrm.toml");

    let content = r#"
[meta]
name = "variables_test"
version = "1.0.0"

[variables]
test_message = "Hello from template"
container_image = "alpine:latest"

[service.alpine]
type = "generic_container"
image = "{{ container_image }}"

[[scenario]]
name = "test"
service = "alpine"
run = "echo {{ test_message }}"
"#;

    fs::write(&config_path, content).expect("Failed to write test config");

    // Load and verify
    let result = load_config_from_file(&config_path);

    // Print debug info if it fails
    if result.is_err() {
        eprintln!("ERROR: {:?}", result.err());
        panic!("Config loading failed");
    }

    let config = result.unwrap();

    // Check that template was substituted
    let services = config.service.expect("Should have services");
    let alpine_service = services.get("alpine").expect("Should have alpine service");
    let image = alpine_service.image.as_ref().expect("Should have image");

    println!("Image value: {}", image);

    assert_eq!(
        image, "alpine:latest",
        "Image should be substituted from [variables] section"
    );

    // Check scenario run command
    let scenario = &config.scenario[0];
    let run_cmd = scenario.run.as_ref().expect("Should have run command");

    println!("Run command: {}", run_cmd);

    assert_eq!(
        run_cmd, "echo Hello from template",
        "Run command should have substituted template variable"
    );

    // Clean up
    let _ = fs::remove_file(&config_path);
}
