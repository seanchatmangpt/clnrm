/// Extended template fixtures for specific test scenarios

pub const TEMPLATE_WITH_COMMENTS: &str = r#"
# Main configuration section
[meta]
name = "commented_template"
description = "Template with extensive comments"

# OpenTelemetry configuration
[otel]
service_name = "test_service"
# Use local collector for testing
endpoint = "http://localhost:4318"

# Database service configuration
[service.postgres]
type = "postgres"
image = "postgres:15"
# Set test credentials
environment = { POSTGRES_PASSWORD = "test" }

# Test scenario
[[scenario]]
name = "test_connection"
# Verify database is reachable
description = "Check PostgreSQL connectivity"
"#;

pub const TEMPLATE_MULTILINE_VALUES: &str = r#"
[meta]
name = "multiline_test"
description = """
This is a multiline description
that spans multiple lines
for testing formatting
"""

[otel]
service_name = "test"

[service.app]
type = "generic"
"#;

pub const TEMPLATE_ARRAYS: &str = r#"
[meta]
name = "array_test"
tags = ["integration", "database", "cache"]

[otel]
service_name = "test"

[service.db]
type = "postgres"
ports = ["5432:5432"]
environment_list = [
    "POSTGRES_PASSWORD=test",
    "POSTGRES_DB=testdb",
    "POSTGRES_USER=testuser"
]
"#;

pub const TEMPLATE_NESTED_TABLES: &str = r#"
[meta]
name = "nested_test"

[otel]
service_name = "test"

[service.app]
type = "generic"

[service.app.config]
max_connections = 100
timeout = "30s"

[service.app.config.retry]
max_attempts = 3
backoff = "exponential"
"#;

pub const TEMPLATE_MULTIPLE_SERVICES: &str = r#"
[meta]
name = "multi_service_test"

[otel]
service_name = "test"

[service.postgres]
type = "postgres"
image = "postgres:15"

[service.redis]
type = "redis"
image = "redis:7"

[service.mongodb]
type = "mongodb"
image = "mongo:7"

[service.elasticsearch]
type = "elasticsearch"
image = "elasticsearch:8"

[service.nginx]
type = "nginx"
image = "nginx:latest"
"#;

pub const TEMPLATE_MULTIPLE_SCENARIOS: &str = r#"
[meta]
name = "multi_scenario_test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "scenario_1"
description = "First test scenario"
service = "db"

[[scenario]]
name = "scenario_2"
description = "Second test scenario"
service = "db"

[[scenario]]
name = "scenario_3"
description = "Third test scenario"
service = "db"
"#;

pub const TEMPLATE_SPECIAL_CHARACTERS: &str = r#"
[meta]
name = "special_chars_test"
description = "Template with special characters: !@#$%^&*()"

[otel]
service_name = "test-service_123"

[service.db]
type = "postgres"
password = "p@$$w0rd!"
"#;

pub const TEMPLATE_LARGE: &str = r#"
[meta]
name = "large_template_test"
description = "Template with many services for performance testing"
version = "1.0.0"
author = "Test Suite"
created = "2025-01-01"
tags = ["performance", "stress", "large"]

[otel]
service_name = "large_test_service"
endpoint = "http://localhost:4318"
sample_rate = 1.0
enable_logs = true
enable_metrics = true
enable_traces = true

[service.postgres_1]
type = "postgres"
image = "postgres:15"

[service.postgres_2]
type = "postgres"
image = "postgres:15"

[service.postgres_3]
type = "postgres"
image = "postgres:15"

[service.redis_1]
type = "redis"
image = "redis:7"

[service.redis_2]
type = "redis"
image = "redis:7"

[service.mongodb_1]
type = "mongodb"
image = "mongo:7"

[service.elasticsearch_1]
type = "elasticsearch"
image = "elasticsearch:8"

[service.nginx_1]
type = "nginx"
image = "nginx:latest"

[service.app_1]
type = "generic"
image = "alpine:latest"

[service.app_2]
type = "generic"
image = "alpine:latest"
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_non_empty() {
        let templates = vec![
            TEMPLATE_WITH_COMMENTS,
            TEMPLATE_MULTILINE_VALUES,
            TEMPLATE_ARRAYS,
            TEMPLATE_NESTED_TABLES,
            TEMPLATE_MULTIPLE_SERVICES,
            TEMPLATE_MULTIPLE_SCENARIOS,
            TEMPLATE_SPECIAL_CHARACTERS,
            TEMPLATE_LARGE,
        ];

        for template in templates {
            assert!(!template.is_empty(), "Template should not be empty");
        }
    }

    #[test]
    fn test_templates_have_required_sections() {
        let templates = vec![
            ("comments", TEMPLATE_WITH_COMMENTS),
            ("multiline", TEMPLATE_MULTILINE_VALUES),
            ("arrays", TEMPLATE_ARRAYS),
            ("nested", TEMPLATE_NESTED_TABLES),
            ("multi_service", TEMPLATE_MULTIPLE_SERVICES),
            ("multi_scenario", TEMPLATE_MULTIPLE_SCENARIOS),
        ];

        for (name, template) in templates {
            assert!(
                template.contains("[meta]"),
                "{} should have [meta] section",
                name
            );
            assert!(
                template.contains("[otel]"),
                "{} should have [otel] section",
                name
            );
        }
    }
}
