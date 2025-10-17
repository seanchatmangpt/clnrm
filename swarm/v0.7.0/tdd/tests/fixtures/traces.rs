/// Extended trace fixtures for diff testing

pub const TRACE_IDENTICAL_1: &str = r#"{
  "spans": [
    {"name": "request", "duration": 100}
  ]
}"#;

pub const TRACE_IDENTICAL_2: &str = r#"{
  "spans": [
    {"name": "request", "duration": 100}
  ]
}"#;

pub const TRACE_MISSING_SPAN_EXPECTED: &str = r#"{
  "spans": [
    {"name": "request", "duration": 100},
    {"name": "database", "duration": 50}
  ]
}"#;

pub const TRACE_MISSING_SPAN_ACTUAL: &str = r#"{
  "spans": [
    {"name": "request", "duration": 100}
  ]
}"#;

pub const TRACE_EXTRA_SPAN_EXPECTED: &str = r#"{
  "spans": [
    {"name": "request", "duration": 100}
  ]
}"#;

pub const TRACE_EXTRA_SPAN_ACTUAL: &str = r#"{
  "spans": [
    {"name": "request", "duration": 100},
    {"name": "unexpected", "duration": 25}
  ]
}"#;

pub const TRACE_ATTRIBUTE_CHANGED_EXPECTED: &str = r#"{
  "spans": [
    {
      "name": "request",
      "duration": 100,
      "attributes": {
        "http.status_code": 200
      }
    }
  ]
}"#;

pub const TRACE_ATTRIBUTE_CHANGED_ACTUAL: &str = r#"{
  "spans": [
    {
      "name": "request",
      "duration": 100,
      "attributes": {
        "http.status_code": 500
      }
    }
  ]
}"#;

pub const TRACE_DURATION_CHANGED_EXPECTED: &str = r#"{
  "spans": [
    {"name": "slow_query", "duration": 100}
  ]
}"#;

pub const TRACE_DURATION_CHANGED_ACTUAL: &str = r#"{
  "spans": [
    {"name": "slow_query", "duration": 500}
  ]
}"#;

pub const TRACE_MULTIPLE_DIFFERENCES: &str = r#"{
  "spans": [
    {"name": "span1", "duration": 100},
    {"name": "span2", "duration": 200},
    {"name": "extra_span", "duration": 50}
  ]
}"#;

pub const TRACE_NESTED_SPANS: &str = r#"{
  "spans": [
    {
      "name": "parent",
      "duration": 300,
      "children": [
        {"name": "child1", "duration": 100},
        {"name": "child2", "duration": 150}
      ]
    }
  ]
}"#;

pub const TRACE_LARGE_1000_SPANS: &str = r#"{
  "spans": []
}"#; // Would be generated programmatically

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_trace_pairs_defined() {
        assert!(!TRACE_IDENTICAL_1.is_empty());
        assert!(!TRACE_IDENTICAL_2.is_empty());
        assert!(!TRACE_MISSING_SPAN_EXPECTED.is_empty());
        assert!(!TRACE_MISSING_SPAN_ACTUAL.is_empty());
    }

    #[test]
    fn test_traces_are_valid_json() {
        use serde_json;

        let traces = vec![
            TRACE_IDENTICAL_1,
            TRACE_MISSING_SPAN_EXPECTED,
            TRACE_ATTRIBUTE_CHANGED_EXPECTED,
            TRACE_DURATION_CHANGED_EXPECTED,
        ];

        for trace in traces {
            assert!(
                serde_json::from_str::<serde_json::Value>(trace).is_ok(),
                "Trace should be valid JSON"
            );
        }
    }
}
