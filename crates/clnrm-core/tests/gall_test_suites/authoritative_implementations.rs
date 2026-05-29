//! GALL-AUTH-1: Authoritative Implementation Verification
//!
//! These tests verify that recently implemented "Oracle Gaps" are
//! mathematically sound and not just returning mocked/hardcoded success.

use clnrm_core::capabilities::scenario::{CapabilityId, CapabilityScenario, ScenarioId};
use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::environment::sigma::{SemVer, SigmaBase, TelemetryDef};
use std::collections::HashMap;

#[test]
fn gall_auth_content_hashing_uniqueness() {
    // ARRANGE
    let sigma1 = SigmaBase {
        version: SemVer::new(1, 0, 0),
        hash: clnrm_core::environment::sigma::ContentHash::from_string(""),
        description: "Test 1".to_string(),
        services: HashMap::new(),
        networks: HashMap::new(),
        volumes: HashMap::new(),
        volume_mounts: HashMap::new(),
        telemetry: TelemetryDef {
            otel_collector: None,
            weaver: None,
            service_instrumentation: HashMap::new(),
        },
        metadata: HashMap::new(),
        created_at: "".to_string(),
    };

    let mut sigma2 = sigma1.clone();
    sigma2.description = "Test 2".to_string();

    // IGNITE
    let hash1 = sigma1.compute_hash();
    let hash2 = sigma2.compute_hash();

    // MEASURE
    assert_ne!(
        hash1.0, hash2.0,
        "Different content must produce different hashes (no hardcoding)"
    );
    assert_ne!(hash1.0, "", "Hash must not be empty");
}

#[test]
fn gall_auth_scenario_mathematical_effect_validation() {
    // ARRANGE: Scenario requesting 'network_egress' but its capability only allows 'file_read'
    use clnrm_core::backend::capabilities::BackendCapabilityRegistry;
    use clnrm_core::capabilities::effects::{Effect, EffectSet};

    let mut scenario = CapabilityScenario::new(ScenarioId("test".to_string()), "Test Scenario");
    scenario = scenario.with_capability(CapabilityId("restricted_cap".to_string()));

    let mut requested_effects = EffectSet::new();
    requested_effects.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });
    scenario = scenario.with_effects(requested_effects);

    // IGNITE
    let registry = BackendCapabilityRegistry::default();
    let result = scenario.validate_effects(&registry);

    // MEASURE
    assert!(
        result.is_err(),
        "Scenario must fail if it requests effects not authorized by its capabilities"
    );
}

#[test]
fn gall_auth_otel_array_attribute_conversion() {
    // ARRANGE: Create an OTel span with an array attribute
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::trace::SpanData as OtelSpanData;
    use std::time::SystemTime;

    let span = OtelSpanData {
        span_context: SpanContext::new(
            TraceId::from_bytes([1; 16]),
            SpanId::from_bytes([1; 8]),
            TraceFlags::default(),
            false,
            TraceState::default(),
        ),
        parent_span_id: SpanId::INVALID,
        parent_span_is_remote: false,
        span_kind: SpanKind::Internal,
        name: "test-span".into(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now(),
        attributes: vec![opentelemetry::KeyValue::new(
            "tags",
            opentelemetry::Value::Array(opentelemetry::Array::String(
                vec!["v1".into(), "v2".into()].into(),
            )),
        )],
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: opentelemetry::trace::Status::Ok,
        dropped_attributes_count: 0,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test").build(),
    };

    // IGNITE
    let validator = clnrm_core::validation::SpanValidator::from_span_data(&[span])
        .expect("Failed to create validator");
    let converted = &validator.all_spans()[0];

    // MEASURE
    let tags = converted
        .attributes
        .get("tags")
        .expect("Tags attribute missing");
    assert!(
        tags.is_array(),
        "Array attributes must be correctly converted to JSON arrays, not strings"
    );
    assert_eq!(tags[0], "v1");
    assert_eq!(tags[1], "v2");
}

#[test]
fn gall_auth_otel_validator_attribute_check() {
    // ARRANGE
    use clnrm_core::validation::SpanAssertion;
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::trace::SpanData as OtelSpanData;
    use std::time::SystemTime;

    let span = OtelSpanData {
        span_context: SpanContext::new(
            TraceId::from_bytes([1; 16]),
            SpanId::from_bytes([1; 8]),
            TraceFlags::default(),
            false,
            TraceState::default(),
        ),
        parent_span_id: SpanId::INVALID,
        parent_span_is_remote: false,
        span_kind: SpanKind::Internal,
        name: "my-span".into(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now(),
        attributes: vec![opentelemetry::KeyValue::new("status", "success")],
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: opentelemetry::trace::Status::Ok,
        dropped_attributes_count: 0,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test").build(),
    };

    let validator = clnrm_core::validation::SpanValidator::from_span_data(&[span])
        .expect("Failed to create validator");

    // ACT: Assert for wrong attribute value
    let assertion = SpanAssertion::SpanAttribute {
        name: "my-span".to_string(),
        attribute_key: "status".to_string(),
        attribute_value: "failed".to_string(),
    };

    // Use the lower-level validate_assertion which returns Result<()>
    // and correctly implements the matching logic.
    let result = validator.validate_assertion(&assertion);

    // MEASURE
    assert!(
        result.is_err(),
        "Validation must fail if attributes don't match (no hardcoded success)"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("no span 'my-span' has attribute 'status' = 'failed'"),
        "Error message must be accurate: {}",
        err_msg
    );
}

#[tokio::test]
async fn gall_auth_service_lifecycle_realism() {
    // ARRANGE
    let env = CleanroomEnvironment::new()
        .await
        .expect("Failed to create env");
    let service_id = "test-db";

    // IGNITE
    let result = env.start_service(service_id).await;

    // MEASURE
    match result {
        Ok(_) => {
            let services = env.services().await;
            assert!(
                services.active_services.contains_key(service_id),
                "Service must be present in registry after start"
            );
        }
        Err(e) => {
            let err_str = format!("{:?}", e);
            assert!(
                !err_str.contains("Refusal"),
                "Service start should not return a Refusal: {}",
                err_str
            );
            assert!(
                !err_str.contains("unimplemented"),
                "Service start should not be unimplemented: {}",
                err_str
            );
        }
    }
}
