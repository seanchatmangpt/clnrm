with open("crates/clnrm-core/src/validation/otel/validator.rs", "r") as f:
    lines = f.readlines()

start_idx = -1
end_idx = -1

for i, line in enumerate(lines):
    if "unimplemented!(\"OTEL-GALL-1 Refusal: Span validation logic must evaluate ALL matching spans, not just the first. Cannot silently ignore other spans.\");" in line:
        start_idx = i - 1 # Include the previous comment line
        break

if start_idx != -1:
    for i in range(start_idx, len(lines)):
        if "Ok(SpanValidationResult {" in lines[i]:
            # Find the closing brace for the struct
            for j in range(i, len(lines)):
                if "})" in lines[j]:
                    end_idx = j
                    break
            break

if start_idx != -1 and end_idx != -1:
    replacement = """        if spans.is_empty() {
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
        })
"""
    new_lines = lines[:start_idx] + [replacement] + lines[end_idx+1:]
    with open("crates/clnrm-core/src/validation/otel/validator.rs", "w") as f:
        f.writelines(new_lines)
    print("Replaced successfully")
else:
    print(f"FAILED: start_idx={start_idx}, end_idx={end_idx}")

