//! Performance Analysis and Reporting Tool
//!
//! Analyzes benchmark results and generates optimization recommendations
//! for OTEL telemetry and Weaver validation overhead.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub min_ns: f64,
    pub max_ns: f64,
    pub iterations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub baseline: BenchmarkResult,
    pub with_feature: BenchmarkResult,
    pub overhead_percent: f64,
    pub overhead_ns: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: String,
    pub summary: PerformanceSummary,
    pub comparisons: Vec<PerformanceComparison>,
    pub optimizations: Vec<OptimizationRecommendation>,
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_benchmarks: usize,
    pub average_overhead_percent: f64,
    pub memory_overhead_mb: f64,
    pub throughput_items_per_sec: f64,
    pub overall_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub priority: Priority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub expected_improvement: String,
    pub implementation_complexity: Complexity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub operation: String,
    pub severity: Severity,
    pub impact: String,
    pub root_cause: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

pub struct PerformanceAnalyzer {
    results: HashMap<String, BenchmarkResult>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.insert(result.name.clone(), result);
    }

    pub fn calculate_overhead(&self, baseline_name: &str, feature_name: &str) -> Option<PerformanceComparison> {
        let baseline = self.results.get(baseline_name)?;
        let with_feature = self.results.get(feature_name)?;

        let overhead_ns = with_feature.mean_ns - baseline.mean_ns;
        let overhead_percent = (overhead_ns / baseline.mean_ns) * 100.0;

        let recommendation = Self::generate_recommendation(overhead_percent);

        Some(PerformanceComparison {
            baseline: baseline.clone(),
            with_feature: with_feature.clone(),
            overhead_percent,
            overhead_ns,
            recommendation,
        })
    }

    fn generate_recommendation(overhead_percent: f64) -> String {
        match overhead_percent {
            x if x < 5.0 => "Negligible overhead - acceptable for production".to_string(),
            x if x < 15.0 => "Low overhead - acceptable with minimal impact".to_string(),
            x if x < 30.0 => "Moderate overhead - consider optimization for hot paths".to_string(),
            x if x < 50.0 => "High overhead - optimization recommended".to_string(),
            _ => "Critical overhead - requires immediate optimization".to_string(),
        }
    }

    pub fn identify_bottlenecks(&self) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // Analyze container startup overhead
        if let Some(comparison) = self.calculate_overhead(
            "container_startup/without_otel",
            "container_startup/with_full_telemetry",
        ) {
            if comparison.overhead_percent > 20.0 {
                bottlenecks.push(PerformanceBottleneck {
                    operation: "Container Startup".to_string(),
                    severity: if comparison.overhead_percent > 50.0 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    impact: format!("{:.1}% overhead on container initialization", comparison.overhead_percent),
                    root_cause: "Span creation and context propagation during container startup".to_string(),
                    mitigation: "Consider lazy span initialization or batching early telemetry".to_string(),
                });
            }
        }

        // Analyze OTLP export latency
        for (name, result) in &self.results {
            if name.starts_with("otlp_export") && result.mean_ns > 1_000_000.0 {
                bottlenecks.push(PerformanceBottleneck {
                    operation: format!("OTLP Export: {}", name),
                    severity: Severity::Medium,
                    impact: format!("{:.2}ms export latency", result.mean_ns / 1_000_000.0),
                    root_cause: "Network serialization and transmission overhead".to_string(),
                    mitigation: "Implement batching and compression for exports".to_string(),
                });
            }
        }

        // Analyze Weaver validation overhead
        for (name, result) in &self.results {
            if name.starts_with("weaver_validation") && result.mean_ns > 5_000_000.0 {
                bottlenecks.push(PerformanceBottleneck {
                    operation: format!("Weaver Validation: {}", name),
                    severity: Severity::Medium,
                    impact: format!("{:.2}ms validation time", result.mean_ns / 1_000_000.0),
                    root_cause: "Schema lookup and validation processing".to_string(),
                    mitigation: "Cache schema lookups and parallelize validation".to_string(),
                });
            }
        }

        bottlenecks
    }

    pub fn generate_optimizations(&self) -> Vec<OptimizationRecommendation> {
        let mut optimizations = Vec::new();

        // Always recommend sampling for production
        optimizations.push(OptimizationRecommendation {
            priority: Priority::High,
            category: "Telemetry Configuration".to_string(),
            title: "Implement Adaptive Sampling".to_string(),
            description: "Use probabilistic sampling to reduce telemetry volume in production. Sample 100% during development/testing, 10-20% in production.".to_string(),
            expected_improvement: "60-80% reduction in telemetry overhead".to_string(),
            implementation_complexity: Complexity::Simple,
        });

        // Batch export optimization
        optimizations.push(OptimizationRecommendation {
            priority: Priority::High,
            category: "OTLP Export".to_string(),
            title: "Batch OTLP Exports".to_string(),
            description: "Configure OTLP exporter to batch spans/metrics before export. Use batch size of 512-1024 and max delay of 5 seconds.".to_string(),
            expected_improvement: "30-50% reduction in export overhead".to_string(),
            implementation_complexity: Complexity::Simple,
        });

        // Async export
        optimizations.push(OptimizationRecommendation {
            priority: Priority::Medium,
            category: "OTLP Export".to_string(),
            title: "Asynchronous Telemetry Export".to_string(),
            description: "Move OTLP export to background thread/task to avoid blocking test execution.".to_string(),
            expected_improvement: "Eliminate export latency from critical path".to_string(),
            implementation_complexity: Complexity::Moderate,
        });

        // Weaver caching
        optimizations.push(OptimizationRecommendation {
            priority: Priority::Medium,
            category: "Weaver Validation".to_string(),
            title: "Schema Lookup Caching".to_string(),
            description: "Cache Weaver schema lookups in memory to avoid repeated file I/O and parsing.".to_string(),
            expected_improvement: "40-60% reduction in validation overhead".to_string(),
            implementation_complexity: Complexity::Moderate,
        });

        // Selective instrumentation
        optimizations.push(OptimizationRecommendation {
            priority: Priority::Medium,
            category: "Telemetry Configuration".to_string(),
            title: "Selective Instrumentation".to_string(),
            description: "Only instrument critical paths and high-value operations. Disable metrics/logs for low-value operations.".to_string(),
            expected_improvement: "20-40% reduction in telemetry volume".to_string(),
            implementation_complexity: Complexity::Simple,
        });

        // Compression
        optimizations.push(OptimizationRecommendation {
            priority: Priority::Low,
            category: "OTLP Export".to_string(),
            title: "Enable OTLP Compression".to_string(),
            description: "Enable gzip compression for OTLP exports to reduce network bandwidth.".to_string(),
            expected_improvement: "50-70% reduction in network bandwidth".to_string(),
            implementation_complexity: Complexity::Simple,
        });

        // Parallel validation
        optimizations.push(OptimizationRecommendation {
            priority: Priority::Low,
            category: "Weaver Validation".to_string(),
            title: "Parallel Weaver Validation".to_string(),
            description: "Process Weaver validation in parallel for multiple telemetry items using thread pool.".to_string(),
            expected_improvement: "2-3x improvement in validation throughput".to_string(),
            implementation_complexity: Complexity::Complex,
        });

        optimizations
    }

    pub fn generate_report(&self) -> PerformanceReport {
        let comparisons = vec![
            self.calculate_overhead("container_startup/without_otel", "container_startup/with_full_telemetry"),
            self.calculate_overhead("test_execution/without_otel/100", "test_execution/with_otel/100"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let average_overhead = if !comparisons.is_empty() {
            comparisons.iter().map(|c| c.overhead_percent).sum::<f64>() / comparisons.len() as f64
        } else {
            0.0
        };

        let bottlenecks = self.identify_bottlenecks();
        let optimizations = self.generate_optimizations();

        let overall_assessment = match average_overhead {
            x if x < 10.0 => "Excellent - Minimal performance impact".to_string(),
            x if x < 25.0 => "Good - Acceptable overhead for observability benefits".to_string(),
            x if x < 40.0 => "Fair - Consider optimizations for production use".to_string(),
            _ => "Poor - Significant optimizations required".to_string(),
        };

        PerformanceReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary: PerformanceSummary {
                total_benchmarks: self.results.len(),
                average_overhead_percent: average_overhead,
                memory_overhead_mb: Self::estimate_memory_overhead(),
                throughput_items_per_sec: Self::estimate_throughput(&self.results),
                overall_assessment,
            },
            comparisons,
            optimizations,
            bottlenecks,
        }
    }

    fn estimate_memory_overhead() -> f64 {
        // Estimate based on typical usage: 1000 spans + 500 metrics + 500 logs
        let spans_bytes = 1000 * 512;
        let metrics_bytes = 500 * 128;
        let logs_bytes = 500 * 256;
        ((spans_bytes + metrics_bytes + logs_bytes) as f64) / (1024.0 * 1024.0)
    }

    fn estimate_throughput(results: &HashMap<String, BenchmarkResult>) -> f64 {
        // Calculate items/second based on OTLP export benchmarks
        results
            .iter()
            .filter(|(name, _)| name.contains("throughput"))
            .map(|(_, result)| 1_000_000_000.0 / result.mean_ns)
            .sum::<f64>()
    }

    pub fn print_report(&self, report: &PerformanceReport) {
        println!("# OTEL Telemetry & Weaver Performance Report");
        println!("Generated: {}\n", report.timestamp);

        println!("## Summary");
        println!("- Total Benchmarks: {}", report.summary.total_benchmarks);
        println!("- Average Overhead: {:.2}%", report.summary.average_overhead_percent);
        println!("- Memory Overhead: {:.2} MB", report.summary.memory_overhead_mb);
        println!("- Throughput: {:.0} items/sec", report.summary.throughput_items_per_sec);
        println!("- Assessment: {}\n", report.summary.overall_assessment);

        if !report.comparisons.is_empty() {
            println!("## Performance Comparisons");
            for comp in &report.comparisons {
                println!("\n### {} vs {}", comp.baseline.name, comp.with_feature.name);
                println!("- Baseline: {:.2}ms", comp.baseline.mean_ns / 1_000_000.0);
                println!("- With Feature: {:.2}ms", comp.with_feature.mean_ns / 1_000_000.0);
                println!("- Overhead: {:.2}% (+{:.2}ms)", comp.overhead_percent, comp.overhead_ns / 1_000_000.0);
                println!("- Recommendation: {}", comp.recommendation);
            }
            println!();
        }

        if !report.bottlenecks.is_empty() {
            println!("## Performance Bottlenecks");
            for bottleneck in &report.bottlenecks {
                println!("\n### {} [{:?}]", bottleneck.operation, bottleneck.severity);
                println!("- Impact: {}", bottleneck.impact);
                println!("- Root Cause: {}", bottleneck.root_cause);
                println!("- Mitigation: {}", bottleneck.mitigation);
            }
            println!();
        }

        if !report.optimizations.is_empty() {
            println!("## Optimization Recommendations");
            let mut sorted_opts = report.optimizations.clone();
            sorted_opts.sort_by_key(|o| o.priority.clone());

            for opt in sorted_opts {
                println!("\n### [{:?}] {}", opt.priority, opt.title);
                println!("Category: {}", opt.category);
                println!("Description: {}", opt.description);
                println!("Expected Improvement: {}", opt.expected_improvement);
                println!("Complexity: {:?}", opt.implementation_complexity);
            }
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new();
        assert_eq!(analyzer.results.len(), 0);
    }

    #[test]
    fn test_add_result() {
        let mut analyzer = PerformanceAnalyzer::new();
        let result = BenchmarkResult {
            name: "test_bench".to_string(),
            mean_ns: 1000.0,
            std_dev_ns: 50.0,
            min_ns: 900.0,
            max_ns: 1100.0,
            iterations: 100,
        };
        analyzer.add_result(result);
        assert_eq!(analyzer.results.len(), 1);
    }

    #[test]
    fn test_overhead_calculation() {
        let mut analyzer = PerformanceAnalyzer::new();

        analyzer.add_result(BenchmarkResult {
            name: "baseline".to_string(),
            mean_ns: 1000.0,
            std_dev_ns: 50.0,
            min_ns: 900.0,
            max_ns: 1100.0,
            iterations: 100,
        });

        analyzer.add_result(BenchmarkResult {
            name: "feature".to_string(),
            mean_ns: 1500.0,
            std_dev_ns: 75.0,
            min_ns: 1400.0,
            max_ns: 1600.0,
            iterations: 100,
        });

        let comparison = analyzer.calculate_overhead("baseline", "feature").unwrap();
        assert_eq!(comparison.overhead_ns, 500.0);
        assert_eq!(comparison.overhead_percent, 50.0);
    }

    #[test]
    fn test_recommendation_generation() {
        assert!(PerformanceAnalyzer::generate_recommendation(3.0).contains("Negligible"));
        assert!(PerformanceAnalyzer::generate_recommendation(10.0).contains("Low"));
        assert!(PerformanceAnalyzer::generate_recommendation(25.0).contains("Moderate"));
        assert!(PerformanceAnalyzer::generate_recommendation(45.0).contains("High"));
        assert!(PerformanceAnalyzer::generate_recommendation(60.0).contains("Critical"));
    }

    #[test]
    fn test_optimization_generation() {
        let analyzer = PerformanceAnalyzer::new();
        let optimizations = analyzer.generate_optimizations();

        assert!(!optimizations.is_empty());
        assert!(optimizations.iter().any(|o| o.title.contains("Sampling")));
        assert!(optimizations.iter().any(|o| o.title.contains("Batch")));
    }
}
