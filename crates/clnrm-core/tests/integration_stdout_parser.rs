//! Integration tests for StdoutSpanParser
//!
//! Tests parsing OTEL spans from realistic container stdout scenarios.

use clnrm_core::otel::stdout_parser::StdoutSpanParser;
use clnrm_core::validation::span_validator::SpanKind;

#[test]
fn test_parse_realistic_container_output() {
    // Arrange - realistic container output with OTEL spans mixed in
    let stdout = r#"
[2024-01-15T10:30:45Z] INFO Starting test execution
[2024-01-15T10:30:45Z] DEBUG Initializing OTEL exporter
{"name":"clnrm.run","trace_id":"abc123def456","span_id":"root001","parent_span_id":null,"kind":"internal","attributes":{"test.name":"integration_test","test.file":"test_example.clnrm.toml"},"start_time_unix_nano":"1234567890000000","end_time_unix_nano":"1234567999000000"}
[2024-01-15T10:30:46Z] INFO Container created: alpine:latest
[2024-01-15T10:30:46Z] DEBUG Starting container...
{"name":"clnrm.step:setup","trace_id":"abc123def456","span_id":"step001","parent_span_id":"root001","kind":"internal","attributes":{"step.index":"0","step.name":"setup"},"events":["container.create","container.start"],"start_time_unix_nano":"1234567890100000","end_time_unix_nano":"1234567890500000"}
[2024-01-15T10:30:47Z] INFO Executing command: echo "Hello World"
Hello World
{"name":"clnrm.step:execute","trace_id":"abc123def456","span_id":"step002","parent_span_id":"root001","kind":"internal","attributes":{"step.index":"1","step.name":"execute","command":"echo"},"events":["command.start","command.complete"],"start_time_unix_nano":"1234567890600000","end_time_unix_nano":"1234567891000000"}
[2024-01-15T10:30:48Z] INFO Test completed successfully
[2024-01-15T10:30:48Z] DEBUG Cleaning up containers...
"#;

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert - should extract 3 spans, ignoring log lines
    assert_eq!(spans.len(), 3, "Should extract exactly 3 spans");

    // Validate root span
    let root_span = spans.iter().find(|s| s.name == "clnrm.run").unwrap();
    assert_eq!(root_span.trace_id, "abc123def456");
    assert_eq!(root_span.span_id, "root001");
    assert_eq!(root_span.parent_span_id, None);
    assert_eq!(root_span.kind, Some(SpanKind::Internal));
    assert_eq!(
        root_span
            .attributes
            .get("test.name")
            .and_then(|v| v.as_str()),
        Some("integration_test")
    );

    // Validate setup step span
    let setup_span = spans
        .iter()
        .find(|s| s.name == "clnrm.step:setup")
        .unwrap();
    assert_eq!(setup_span.parent_span_id, Some("root001".to_string()));
    assert_eq!(
        setup_span
            .attributes
            .get("step.name")
            .and_then(|v| v.as_str()),
        Some("setup")
    );
    let setup_events = setup_span.events.as_ref().unwrap();
    assert!(setup_events.contains(&"container.create".to_string()));
    assert!(setup_events.contains(&"container.start".to_string()));

    // Validate execute step span
    let execute_span = spans
        .iter()
        .find(|s| s.name == "clnrm.step:execute")
        .unwrap();
    assert_eq!(execute_span.parent_span_id, Some("root001".to_string()));
    assert_eq!(
        execute_span
            .attributes
            .get("command")
            .and_then(|v| v.as_str()),
        Some("echo")
    );

    // Verify all spans have same trace_id
    assert!(spans.iter().all(|s| s.trace_id == "abc123def456"));
}

#[test]
fn test_parse_otel_stdout_exporter_format() {
    // Arrange - format produced by OTEL stdout exporter
    let stdout = r#"
{"name":"http.request","trace_id":"trace123","span_id":"span123","parent_span_id":null,"kind":2,"attributes":{"http.method":"GET","http.url":"/api/test","http.status_code":"200"},"start_time_unix_nano":"1700000000000000","end_time_unix_nano":"1700000001000000"}
{"name":"db.query","trace_id":"trace123","span_id":"span124","parent_span_id":"span123","kind":3,"attributes":{"db.system":"postgresql","db.statement":"SELECT * FROM users"},"events":[{"name":"query.start"},{"name":"query.complete"}],"start_time_unix_nano":"1700000000100000","end_time_unix_nano":"1700000000900000"}
"#;

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert
    assert_eq!(spans.len(), 2);

    // Validate HTTP span
    let http_span = spans.iter().find(|s| s.name == "http.request").unwrap();
    assert_eq!(http_span.kind, Some(SpanKind::Server)); // kind=2 is Server
    assert_eq!(
        http_span
            .attributes
            .get("http.method")
            .and_then(|v| v.as_str()),
        Some("GET")
    );
    assert_eq!(http_span.start_time_unix_nano, Some(1700000000000000));
    assert_eq!(http_span.end_time_unix_nano, Some(1700000001000000));

    // Validate duration calculation
    let duration = http_span.duration_ms().unwrap();
    // Duration = (1700000001000000 - 1700000000000000) / 1_000_000 = 1000000000 / 1_000_000 = 1000ms
    // But the test data shows the end time is just 1 second later, so:
    // (1700000001000000 - 1700000000000000) = 1000000 nanoseconds = 1ms
    assert_eq!(duration, 1.0); // 1 millisecond

    // Validate DB span
    let db_span = spans.iter().find(|s| s.name == "db.query").unwrap();
    assert_eq!(db_span.kind, Some(SpanKind::Client)); // kind=3 is Client
    assert_eq!(db_span.parent_span_id, Some("span123".to_string()));
    let events = db_span.events.as_ref().unwrap();
    assert!(events.contains(&"query.start".to_string()));
}

#[test]
fn test_parse_with_debug_output_and_errors() {
    // Arrange - mixed output with debug info and error messages
    let stdout = r#"
DEBUG: Trace initialized
ERROR: Failed to connect to database (retrying...)
{"name":"retry.attempt","trace_id":"retry123","span_id":"attempt1","parent_span_id":null,"attributes":{"attempt":"1","error":"connection_timeout"}}
WARNING: High memory usage detected
DEBUG: Retry successful
{"name":"retry.attempt","trace_id":"retry123","span_id":"attempt2","parent_span_id":"attempt1","attributes":{"attempt":"2","status":"success"}}
INFO: Operation completed
"#;

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert - should extract 2 spans, ignore debug/error/warning lines
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].name, "retry.attempt");
    assert_eq!(spans[1].name, "retry.attempt");
    assert_eq!(
        spans[0]
            .attributes
            .get("attempt")
            .and_then(|v| v.as_str()),
        Some("1")
    );
    assert_eq!(
        spans[1]
            .attributes
            .get("status")
            .and_then(|v| v.as_str()),
        Some("success")
    );
}

#[test]
fn test_parse_with_json_logs_that_are_not_spans() {
    // Arrange - structured JSON logs that aren't spans
    let stdout = r#"
{"level":"info","timestamp":"2024-01-15T10:30:45Z","message":"Starting test"}
{"name":"actual.span","trace_id":"123","span_id":"s1","parent_span_id":null,"attributes":{}}
{"level":"debug","timestamp":"2024-01-15T10:30:46Z","message":"Container created"}
{"level":"error","timestamp":"2024-01-15T10:30:47Z","message":"Connection failed","error":"timeout"}
{"name":"another.span","trace_id":"123","span_id":"s2","parent_span_id":"s1","attributes":{}}
"#;

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert - should only extract actual spans, not log JSON
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].name, "actual.span");
    assert_eq!(spans[1].name, "another.span");
}

#[test]
fn test_parse_empty_and_whitespace_only_output() {
    // Arrange
    let stdout = "\n\n   \n\t\n\n";

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert
    assert_eq!(spans.len(), 0);
}

#[test]
fn test_parse_with_unicode_and_special_characters() {
    // Arrange - span with unicode characters in attributes
    let stdout = r#"{"name":"test.unicode","trace_id":"123","span_id":"s1","parent_span_id":null,"attributes":{"message":"Hello 世界! 🚀","emoji":"🎉"}}"#;

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert
    assert_eq!(spans.len(), 1);
    assert_eq!(
        spans[0]
            .attributes
            .get("message")
            .and_then(|v| v.as_str()),
        Some("Hello 世界! 🚀")
    );
}

#[test]
fn test_parse_large_batch_of_spans() {
    // Arrange - simulate large test run with many spans
    let mut stdout = String::new();
    for i in 0..100 {
        stdout.push_str(&format!(
            r#"{{"name":"test.span","trace_id":"trace123","span_id":"span{}","parent_span_id":null,"attributes":{{"index":"{}"}}}}
"#,
            i, i
        ));
    }

    // Act
    let spans = StdoutSpanParser::parse(&stdout).expect("Failed to parse spans");

    // Assert
    assert_eq!(spans.len(), 100);
    for (i, span) in spans.iter().enumerate() {
        let expected_index = i.to_string();
        assert_eq!(
            span.attributes.get("index").and_then(|v| v.as_str()),
            Some(expected_index.as_str())
        );
    }
}

#[test]
fn test_parse_with_malformed_json_mixed_in() {
    // Arrange - some valid spans, some malformed JSON
    let stdout = r#"
{"name":"valid1","trace_id":"123","span_id":"s1","parent_span_id":null,"attributes":{}}
{"malformed json without closing brace
{"name":"valid2","trace_id":"123","span_id":"s2","parent_span_id":"s1","attributes":{}}
{"also malformed": "missing fields"}
{"name":"valid3","trace_id":"123","span_id":"s3","parent_span_id":"s2","attributes":{}}
"#;

    // Act
    let spans = StdoutSpanParser::parse(stdout).expect("Failed to parse spans");

    // Assert - should extract only valid spans, log warnings for malformed
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].name, "valid1");
    assert_eq!(spans[1].name, "valid2");
    assert_eq!(spans[2].name, "valid3");
}
