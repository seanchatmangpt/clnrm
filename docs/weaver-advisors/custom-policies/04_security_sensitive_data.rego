# Policy: Detect potentially sensitive data in attributes
# Purpose: Prevent PII/secrets from being included in telemetry
# Severity: VIOLATION (security issue)

package live_check_advice

import rego.v1

# Patterns that indicate sensitive data
sensitive_patterns := {
    "password": "Passwords must never be in telemetry",
    "secret": "Secrets must never be in telemetry",
    "token": "Tokens must never be in telemetry",
    "api_key": "API keys must never be in telemetry",
    "apikey": "API keys must never be in telemetry",
    "credential": "Credentials must never be in telemetry",
    "ssn": "Social Security Numbers must never be in telemetry",
    "credit_card": "Credit card numbers must never be in telemetry",
    "private_key": "Private keys must never be in telemetry"
}

# Check attribute names for sensitive patterns
deny contains advice if {
    input.sample.attribute
    attr_name := lower(input.sample.attribute.name)

    # Check if name contains sensitive pattern
    pattern := sensitive_patterns[key]
    contains(attr_name, key)

    advice := {
        "type": "advice",
        "advice_type": "sensitive_attribute_name",
        "advice_level": "violation",
        "message": sprintf(
            "Attribute name '%s' suggests sensitive data. %s",
            [input.sample.attribute.name, pattern]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "detected_pattern": key,
            "security_impact": "high",
            "action": "Remove this attribute or use a secure vault reference"
        }
    }
}

# Check string values for patterns that look like secrets
warn contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "string"
    attr_value := input.sample.attribute.value

    # Look for base64-encoded values (potential secrets)
    # Simple heuristic: long strings with only alphanumeric and +/=
    value_length := count(attr_value)
    value_length > 40

    # Basic base64 pattern check
    regex.match(`^[A-Za-z0-9+/]+=*$`, attr_value)

    advice := {
        "type": "advice",
        "advice_type": "potential_encoded_secret",
        "advice_level": "improvement",
        "message": sprintf(
            "Attribute '%s' contains base64-like value. Verify this is not a secret or token.",
            [input.sample.attribute.name]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "value_length": value_length,
            "warning": "If this is a secret, remove immediately"
        }
    }
}

# Detect UUID-like patterns in non-ID fields (might be sensitive)
warn contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "string"
    attr_name := input.sample.attribute.name
    attr_value := input.sample.attribute.value

    # Not a designated ID field
    not contains(attr_name, "id")
    not contains(attr_name, "uuid")

    # Looks like a UUID
    regex.match(`^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$`, attr_value)

    advice := {
        "type": "advice",
        "advice_type": "uuid_in_non_id_field",
        "advice_level": "improvement",
        "message": sprintf(
            "Attribute '%s' contains UUID-like value. Ensure this is not a sensitive identifier.",
            [attr_name]
        ),
        "context": {
            "attribute_name": attr_name,
            "suggestion": "Use .id or .uuid suffix for identifier fields"
        }
    }
}

# Check for email addresses (PII)
deny contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "string"
    attr_value := input.sample.attribute.value

    # Simple email pattern
    regex.match(`[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`, attr_value)

    # Unless it's an allowed monitoring/service account pattern
    not regex.match(`^(monitoring|service|no-?reply)@`, attr_value)

    advice := {
        "type": "advice",
        "advice_type": "pii_email_address",
        "advice_level": "violation",
        "message": sprintf(
            "Attribute '%s' contains email address (PII). Use hashed or anonymized identifiers instead.",
            [input.sample.attribute.name]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "privacy_impact": "high",
            "action": "Hash or remove email addresses from telemetry"
        }
    }
}
