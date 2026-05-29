import re

with open("crates/clnrm-core/src/validation/otel/validator.rs", "r") as f:
    content = f.read()

# 1. Fix validate_span_real
span_refusal_re = re.compile(
    r'(\s*// OTEL-GALL-1 Refusal\s*unimplemented!\("OTEL-GALL-1 Refusal: Span validation logic must evaluate ALL matching spans, not just the first\. Cannot silently ignore other spans\."\);\s*)'
    r'let span = spans\.first\(\)\.ok_or_else\(\|\|\s*\{\s*CleanroomError::validation_error\(format!\(\s*"No span data available for span \'\.\{\}\'",\s*assertion\.name\s*\)\s*\)\s*\}\)\?;\s*'
    r'let mut errors = Vec::new\(\);\s*'
    r'let mut actual_attributes = HashMap::new\(\);\s*'
    r'// Validate span attributes against real span data\s*'
    r'for \(expected_key, expected_value\) in &assertion\.attributes \{\s*'
    r'if expected_key\.is_empty\(\) \{\s*'
    r'errors\.push\("Attribute key cannot be empty"\.to_string\(\)\);\s*'
    r'continue;\s*'
    r'\}\s*'
    r'// Look for the attribute in the real span data\s*'
    r'let found_attribute = span\s*\.attributes\s*\.iter\(\)\s*\.find\(\|kv\| kv\.key\.as_str\(\) == expected_key\);\s*'
    r'match found_attribute \{\s*'
    r'Some\(kv\) => \{\s*'
    r'let actual_value = kv\.value\.as_str\(\);\s*'
    r'actual_attributes\.insert\(expected_key\.clone\(\), actual_value\.to_string\(\)\);\s*'
    r'if actual_value != \*expected_value \{\s*'
    r'errors\.push\(format!\(\s*"Attribute \'\.\{\}\' expected \'\.\{\}\' but found \'\.\{\}\'",\s*expected_key, expected_value, actual_value\s*\)\);\s*'
    r'\}\s*'
    r'\}\s*'
    r'None => \{\s*'
    r'errors\.push\(format!\(\s*"Required attribute \'\.\{\}\' not found in span \'\.\{\}\'",\s*expected_key, assertion\.name\s*\)\);\s*'
    r'\}\s*'
    r'\}\s*'
    r'\}\s*'
    r'// Validate duration constraints against real span data\s*'
    r'let actual_duration_ms =\s*'
    r'if assertion\.min_duration_ms\.is_some\(\) \|\| assertion\.max_duration_ms\.is_some\(\) \{\s*'
    r'// For OtelSpanData, start_time and end_time are SystemTime, not Option<SystemTime>\s*'
    r'match span\.end_time\.duration_since\(span\.start_time\) \{\s*'
    r'Ok\(duration\) => \{\s*'
    r'let duration_ns = duration\.as_nanos\(\);\s*'
    r'let duration_ms = duration_ns as f64 / 1_000_000\.0; // Convert nanoseconds to milliseconds\s*'
    r'Some\(duration_ms\)\s*'
    r'\}\s*'
    r'Err\(e\) => \{\s*'
    r'errors\.push\(format!\("Failed to calculate span duration: \.\{\}", e\)\);\s*'
    r'None\s*'
    r'\}\s*'
    r'\}\s*'
    r'\} else \{\s*'
    r'None\s*'
    r'\};\s*'
    r'if let Some\(duration\) = actual_duration_ms \{\s*'
    r'if let Some\(min_duration\) = assertion\.min_duration_ms \{\s*'
    r'if duration < min_duration \{\s*'
    r'errors\.push\(format!\(\s*"Span duration \{\:\.2\}ms is below minimum \{\:\.2\}ms",\s*duration, min_duration\s*\)\);\s*'
    r'\}\s*'
    r'\}\s*'
    r'if let Some\(max_duration\) = assertion\.max_duration_ms \{\s*'
    r'if duration > max_duration \{\s*'
    r'errors\.push\(format!\(\s*"Span duration \{\:\.2\}ms exceeds maximum \{\:\.2\}ms",\s*duration, max_duration\s*\)\);\s*'
    r'\}\s*'
    r'\}\s*'
    r'\}\s*'
    r'Ok\(SpanValidationResult \{\s*'
    r'passed: errors\.is_empty\(\),\s*'
    r'span_name: assertion\.name\.clone\(\),\s*'
    r'errors,\s*'
    r'actual_attributes,\s*'
    r'actual_duration_ms,\s*'
    r'\}\)',
    re.MULTILINE
)

span_replacement = """        if spans.is_empty() {
            return Err(CleanroomError::validation_error(format!(
                "No span data available for span '{}'",
                assertion.name
            )));
        }

        let mut all_errors = Vec::new();
        let mut best_attributes = HashMap::new();
        let mut best_duration = None;
        let mut any_passed = false;

        for (idx, span) in spans.iter().enumerate() {
            let mut errors = Vec::new();
            let mut actual_attributes = HashMap::new();

            for (expected_key, expected_value) in &assertion.attributes {
                if expected_key.is_empty() {
                    errors.push(format!("Span [{}]: Attribute key cannot be empty", idx));
                    continue;
                }

                let found_attribute = span
                    .attributes
                    .iter()
                    .find(|kv| kv.key.as_str() == expected_key);

                match found_attribute {
                    Some(kv) => {
                        let actual_value = kv.value.as_str();
                        actual_attributes.insert(expected_key.clone(), actual_value.to_string());

                        if actual_value != *expected_value {
                            errors.push(format!(
                                "Span [{}]: Attribute '{}' expected '{}' but found '{}'",
                                idx, expected_key, expected_value, actual_value
                            ));
                        }
                    }
                    None => {
                        errors.push(format!(
                            "Span [{}]: Required attribute '{}' not found",
                            idx, expected_key
                        ));
                    }
                }
            }

            let actual_duration_ms =
                if assertion.min_duration_ms.is_some() || assertion.max_duration_ms.is_some() {
                    match span.end_time.duration_since(span.start_time) {
                        Ok(duration) => {
                            let duration_ns = duration.as_nanos();
                            let duration_ms = duration_ns as f64 / 1_000_000.0;
                            Some(duration_ms)
                        }
                        Err(e) => {
                            errors.push(format!("Span [{}]: Failed to calculate span duration: {}", idx, e));
                            None
                        }
                    }
                } else {
                    None
                };

            if let Some(duration) = actual_duration_ms {
                if let Some(min_duration) = assertion.min_duration_ms {
                    if duration < min_duration {
                        errors.push(format!(
                            "Span [{}]: Span duration {:.2}ms is below minimum {:.2}ms",
                            idx, duration, min_duration
                        ));
                    }
                }

                if let Some(max_duration) = assertion.max_duration_ms {
                    if duration > max_duration {
                        errors.push(format!(
                            "Span [{}]: Span duration {:.2}ms exceeds maximum {:.2}ms",
                            idx, duration, max_duration
                        ));
                    }
                }
            }

            if errors.is_empty() {
                any_passed = true;
                best_attributes = actual_attributes;
                best_duration = actual_duration_ms;
                all_errors.clear();
                break;
            } else {
                if idx == 0 {
                    best_attributes = actual_attributes;
                    best_duration = actual_duration_ms;
                }
                all_errors.extend(errors);
            }
        }

        Ok(SpanValidationResult {
            passed: any_passed,
            span_name: assertion.name.clone(),
            errors: all_errors,
            actual_attributes: best_attributes,
            actual_duration_ms: best_duration,
        })"""

content_new, n = span_refusal_re.subn(span_replacement, content)
if n == 0:
    print("FAILED to replace validate_span_real refusal")
else:
    print(f"Replaced {n} occurrences in validate_span_real")

# 2. Fix validate_export
export_refusal_re = re.compile(r'(\s*// OTEL-GALL-1 Refusal\s*unimplemented!\("OTEL-GALL-1 Refusal: validate_export must establish actual collector connectivity and verify OTLP export\. It cannot return a fake success\."\);\s*)')

export_replacement = """        // Establish actual collector connectivity
        let parsed_url = match url::Url::parse(endpoint) {
            Ok(u) => u,
            Err(e) => return Err(CleanroomError::validation_error(format!("Invalid URL: {}", e))),
        };

        let host = match parsed_url.host_str() {
            Some(h) => h,
            None => return Err(CleanroomError::validation_error("URL must have a host")),
        };

        let port = parsed_url.port_or_known_default().unwrap_or(80);

        let addrs = match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:{}", host, port)) {
            Ok(a) => a,
            Err(e) => return Err(CleanroomError::validation_error(format!("Failed to resolve host: {}", e))),
        };

        let mut connected = false;
        for addr in addrs {
            if std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(5)).is_ok() {
                connected = true;
                break;
            }
        }

        if !connected {
            return Err(CleanroomError::validation_error(format!("Failed to connect to {}", endpoint)));
        }

        Ok(true)
"""

content_new, n = export_refusal_re.subn(export_replacement, content_new)
if n == 0:
    print("FAILED to replace validate_export refusal")
else:
    print(f"Replaced {n} occurrences in validate_export")


# 3. Fix validate_export_real
export_real_refusal_re = re.compile(r'(\s*// OTEL-GALL-1 Refusal\s*unimplemented!\("OTEL-GALL-1 Refusal: validate_export_functionality must actually generate test spans and verify they reach the collector\. It cannot return a EXAMPLE-ONLY: placeholder success\."\);\s*)')

export_real_replacement = """        // Generate test spans and verify they reach the collector
        use opentelemetry_sdk::trace::SdkTracerProvider;
        use opentelemetry::trace::{Tracer, TracerProvider as _};
        use opentelemetry_otlp::WithExportConfig;

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| CleanroomError::validation_error(format!("Failed to build exporter: {}", e)))?;

        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
            
        let tracer = provider.tracer("test-export-tracer");
        
        tracer.in_span("test-export-span", |_cx| {
            // Test span to verify connectivity
        });

        // Force flush to ensure it reaches the collector
        if let Err(e) = provider.force_flush() {
            return Err(CleanroomError::validation_error(format!("Failed to flush spans to collector: {:?}", e)));
        }

        Ok(true)
"""

content_new, n = export_real_refusal_re.subn(export_real_replacement, content_new)
if n == 0:
    print("FAILED to replace validate_export_real refusal")
else:
    print(f"Replaced {n} occurrences in validate_export_real")


with open("crates/clnrm-core/src/validation/otel/validator.rs", "w") as f:
    f.write(content_new)

