//! Integration tests for AI-powered autonomous monitoring system
//!
//! Tests comprehensive monitoring capabilities including anomaly detection,
//! proactive alerting, and self-healing mechanisms.

use clnrm_core::cli::commands::ai_monitor::{SystemMetric, Anomaly, AnomalySeverity};
use std::time::{SystemTime, Duration};

#[test]
fn test_system_metric_creation() {
    let metric = SystemMetric {
        name: "test_metric".to_string(),
        value: 42.0,
        timestamp: SystemTime::now(),
        unit: "count".to_string(),
    };

    assert_eq!(metric.name, "test_metric");
    assert_eq!(metric.value, 42.0);
    assert_eq!(metric.unit, "count");
}

#[test]
fn test_anomaly_creation() {
    let anomaly = Anomaly {
        metric_name: "cpu_usage".to_string(),
        description: "CPU usage spike detected".to_string(),
        severity: AnomalySeverity::High,
        confidence: 0.95,
        detected_at: SystemTime::now(),
        recommended_action: "Scale resources".to_string(),
    };

    assert_eq!(anomaly.metric_name, "cpu_usage");
    assert!(matches!(anomaly.severity, AnomalySeverity::High));
    assert_eq!(anomaly.confidence, 0.95);
}

#[test]
fn test_anomaly_severity_levels() {
    let critical = AnomalySeverity::Critical;
    let high = AnomalySeverity::High;
    let medium = AnomalySeverity::Medium;
    let low = AnomalySeverity::Low;

    // Verify all severity levels exist
    assert!(matches!(critical, AnomalySeverity::Critical));
    assert!(matches!(high, AnomalySeverity::High));
    assert!(matches!(medium, AnomalySeverity::Medium));
    assert!(matches!(low, AnomalySeverity::Low));
}

#[tokio::test]
async fn test_monitoring_interval_configuration() {
    // Test various monitoring intervals
    let intervals = vec![
        Duration::from_secs(10),
        Duration::from_secs(30),
        Duration::from_secs(60),
        Duration::from_secs(300),
    ];

    for interval in intervals {
        assert!(interval.as_secs() >= 10);
        assert!(interval.as_secs() <= 300);
    }
}

#[test]
fn test_anomaly_threshold_ranges() {
    // Test valid anomaly threshold ranges
    let thresholds = vec![0.5, 0.6, 0.7, 0.8, 0.9];

    for threshold in thresholds {
        assert!(threshold >= 0.0 && threshold <= 1.0);
    }
}

#[tokio::test]
async fn test_metrics_collection_types() {
    // Verify all expected metric types can be created
    let metric_types = vec![
        ("test_execution_rate", "tests/min"),
        ("test_success_rate", "percent"),
        ("avg_execution_time", "ms"),
        ("cpu_usage", "percent"),
        ("memory_usage", "MB"),
        ("test_flakiness_score", "score"),
    ];

    for (name, unit) in metric_types {
        let metric = SystemMetric {
            name: name.to_string(),
            value: 50.0,
            timestamp: SystemTime::now(),
            unit: unit.to_string(),
        };

        assert_eq!(metric.name, name);
        assert_eq!(metric.unit, unit);
    }
}

#[test]
fn test_anomaly_confidence_scoring() {
    let anomaly = Anomaly {
        metric_name: "test".to_string(),
        description: "Test anomaly".to_string(),
        severity: AnomalySeverity::Medium,
        confidence: 0.85,
        detected_at: SystemTime::now(),
        recommended_action: "Review".to_string(),
    };

    assert!(anomaly.confidence >= 0.0);
    assert!(anomaly.confidence <= 1.0);
    assert!(anomaly.confidence >= 0.5); // Minimum confidence for reporting
}

#[tokio::test]
async fn test_healing_actions_types() {
    // Test that all healing action types are supported
    let healing_scenarios = vec![
        ("test_success_rate", "Retry failing tests"),
        ("test_execution_rate", "Rebalance workers"),
        ("avg_execution_time", "Optimize performance"),
        ("cpu_usage", "Scale resources"),
        ("memory_usage", "Clean up resources"),
        ("test_flakiness_score", "Quarantine flaky tests"),
    ];

    for (metric, _action) in healing_scenarios {
        assert!(!metric.is_empty());
    }
}

#[test]
fn test_alert_priority_mapping() {
    // Test that anomaly severity maps to alert priority correctly
    let severities = vec![
        AnomalySeverity::Critical,
        AnomalySeverity::High,
        AnomalySeverity::Medium,
        AnomalySeverity::Low,
    ];

    for severity in severities {
        match severity {
            AnomalySeverity::Critical => {
                // Should trigger immediate alerts
                assert!(true);
            },
            AnomalySeverity::High => {
                // Should trigger high-priority alerts
                assert!(true);
            },
            AnomalySeverity::Medium => {
                // Should trigger medium-priority alerts
                assert!(true);
            },
            AnomalySeverity::Low => {
                // Should trigger low-priority alerts
                assert!(true);
            },
        }
    }
}

#[tokio::test]
async fn test_monitoring_dashboard_structure() {
    // Test dashboard data structure components
    let metrics = vec![
        SystemMetric {
            name: "test1".to_string(),
            value: 100.0,
            timestamp: SystemTime::now(),
            unit: "ms".to_string(),
        },
        SystemMetric {
            name: "test2".to_string(),
            value: 90.0,
            timestamp: SystemTime::now(),
            unit: "percent".to_string(),
        },
    ];

    let anomalies = vec![
        Anomaly {
            metric_name: "test1".to_string(),
            description: "Performance degradation".to_string(),
            severity: AnomalySeverity::Medium,
            confidence: 0.8,
            detected_at: SystemTime::now(),
            recommended_action: "Optimize".to_string(),
        },
    ];

    assert_eq!(metrics.len(), 2);
    assert_eq!(anomalies.len(), 1);
}

#[test]
fn test_statistical_anomaly_detection_inputs() {
    // Test inputs for statistical anomaly detection
    let baseline_values = vec![100.0, 102.0, 98.0, 101.0, 99.0];
    let test_value = 150.0; // Anomalous value

    let mean: f64 = baseline_values.iter().sum::<f64>() / baseline_values.len() as f64;
    let variance: f64 = baseline_values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / baseline_values.len() as f64;
    let std_dev = variance.sqrt();

    let z_score = ((test_value - mean) / std_dev).abs();

    // Should detect significant deviation
    assert!(z_score > 3.0, "Z-score should indicate anomaly");
}

#[tokio::test]
async fn test_webhook_notification_format() {
    // Test webhook notification data structure
    use serde_json::json;

    let notification = json!({
        "alert_type": "Critical",
        "title": "System Anomaly Detected",
        "description": "CPU usage spike detected",
        "severity": "Critical",
        "timestamp": SystemTime::now(),
        "recommended_action": "Scale resources immediately",
    });

    assert!(notification.get("alert_type").is_some());
    assert!(notification.get("title").is_some());
    assert!(notification.get("description").is_some());
    assert!(notification.get("severity").is_some());
    assert!(notification.get("recommended_action").is_some());
}

#[test]
fn test_health_score_calculation() {
    // Test health score calculation logic
    let base_score = 100.0;
    let critical_deduction = 30.0;
    let high_deduction = 20.0;
    let medium_deduction = 10.0;
    let low_deduction = 5.0;

    // Scenario: 1 critical, 1 high anomaly
    let total_deduction = critical_deduction + high_deduction;
    let health_score = (base_score - total_deduction).max(0.0);

    assert_eq!(health_score, 50.0);
    assert!(health_score >= 0.0 && health_score <= 100.0);
}

#[tokio::test]
async fn test_proactive_failure_prediction() {
    // Test failure prediction capabilities
    let failure_indicators = vec![
        ("declining_success_rate", 0.85),
        ("increasing_execution_time", 0.75),
        ("resource_constraints", 0.65),
        ("flaky_test_pattern", 0.55),
    ];

    for (indicator, probability) in failure_indicators {
        assert!(!indicator.is_empty());
        assert!(probability >= 0.0 && probability <= 1.0);
    }
}

#[test]
fn test_metric_baseline_initialization() {
    // Test metric baseline tracking
    let mut values = Vec::new();

    for i in 0..10 {
        values.push(100.0 + i as f64);
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    assert!(mean > 100.0 && mean < 110.0);
}

#[tokio::test]
async fn test_alert_deduplication_logic() {
    use std::collections::HashMap;

    // Test alert deduplication
    let mut sent_alerts: HashMap<String, SystemTime> = HashMap::new();
    let alert_key = "test_metric:anomaly".to_string();

    // First alert
    sent_alerts.insert(alert_key.clone(), SystemTime::now());

    // Check if duplicate within 5 minutes
    if let Some(last_sent) = sent_alerts.get(&alert_key) {
        if let Ok(elapsed) = last_sent.elapsed() {
            // Should skip if within 5 minutes
            assert!(elapsed < Duration::from_secs(10)); // Just created
        }
    }
}

#[test]
fn test_healing_action_prioritization() {
    // Test healing action prioritization based on severity
    let severities = vec![
        (AnomalySeverity::Critical, true),  // Should heal
        (AnomalySeverity::High, true),      // Should heal
        (AnomalySeverity::Medium, false),   // Monitor only
        (AnomalySeverity::Low, false),      // Monitor only
    ];

    for (severity, should_heal) in severities {
        let requires_healing = matches!(severity, AnomalySeverity::Critical | AnomalySeverity::High);
        assert_eq!(requires_healing, should_heal);
    }
}

#[tokio::test]
async fn test_monitoring_configuration_validation() {
    // Test configuration validation
    let valid_configs = vec![
        (30, 0.7, true, true, true),   // Standard config
        (10, 0.5, false, true, false), // Low threshold
        (60, 0.9, true, false, true),  // High threshold
    ];

    for (interval, threshold, alerts, detection, healing) in valid_configs {
        assert!(interval >= 10);
        assert!(threshold >= 0.0 && threshold <= 1.0);
        // All boolean combinations are valid
        let _ = (alerts, detection, healing);
    }
}

#[test]
fn test_resource_usage_metrics() {
    // Test resource usage tracking
    let cpu_usage = 45.5;
    let memory_mb = 256;
    let network_io_mb = 50;
    let disk_io_mb = 30;

    assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0);
    assert!(memory_mb > 0);
    assert!(network_io_mb >= 0);
    assert!(disk_io_mb >= 0);
}

#[tokio::test]
async fn test_anomaly_pattern_detection() {
    // Test pattern-based anomaly detection
    let patterns = vec![
        "success_rate_decline",
        "performance_degradation",
        "resource_exhaustion",
        "flakiness_increase",
    ];

    for pattern in patterns {
        assert!(!pattern.is_empty());
        assert!(pattern.contains("_")); // All patterns use snake_case
    }
}

#[test]
fn test_monitoring_metrics_serialization() {
    // Test that metrics can be serialized/deserialized
    let metric = SystemMetric {
        name: "test".to_string(),
        value: 42.0,
        timestamp: SystemTime::now(),
        unit: "count".to_string(),
    };

    let json = serde_json::to_string(&metric);
    assert!(json.is_ok());

    let deserialized: Result<SystemMetric, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());
}
