use clnrm_core::environment::sigma::{ContentHash, SemVer, SigmaBase, TelemetryDef};
use clnrm_core::service::registry::ServiceMetadata;
use std::collections::HashMap;

#[tokio::test]
async fn test_service_metadata_export_env_snapshot() {
    let mut metadata = ServiceMetadata::new(
        "svc-1".to_string(),
        "test-service".to_string(),
        "container-1".to_string(),
    );
    metadata.add_port(8080, 10000);
    metadata.add_endpoint("url".to_string(), "http://localhost:10000".to_string());

    let env = metadata.export_env();

    // Sort to ensure deterministic output
    let mut sorted_env: Vec<_> = env.into_iter().collect();
    sorted_env.sort_by(|a, b| a.0.cmp(&b.0));

    let json_output = serde_json::to_string_pretty(&sorted_env).expect("Failed to serialize");

    let expected_snapshot = r#"[
  [
    "TEST_SERVICE_HOST",
    "127.0.0.1"
  ],
  [
    "TEST_SERVICE_PORT",
    "10000"
  ],
  [
    "TEST_SERVICE_PORT_8080",
    "10000"
  ],
  [
    "TEST_SERVICE_URL",
    "http://localhost:10000"
  ]
]"#;

    assert_eq!(
        json_output, expected_snapshot,
        "Snapshot mismatch for ServiceRegistry::export_env()"
    );
}
