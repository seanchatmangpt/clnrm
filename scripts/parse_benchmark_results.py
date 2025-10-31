#!/usr/bin/env python3
"""
Parse Criterion benchmark results and generate regression summary.
Usage: python3 parse_benchmark_results.py <results-file>
"""

import sys
import re
from typing import Dict, List, Tuple

# Baseline metrics from v1.2.2
BASELINES = {
    'otel_overhead_ms': 50.0,
    'container_startup_ms': 3000.0,
    'test_execution_ms': 30000.0,
    'memory_mb': 512.0,
    'binary_size_mb': 50.0,
}

THRESHOLD_PERCENT = 5.0  # 5% regression threshold


def parse_criterion_output(content: str) -> Dict[str, Dict[str, float]]:
    """Parse Criterion benchmark output and extract timing data."""
    benchmarks = {}

    # Pattern: bench_name   time:   [1.234 ms 1.456 ms 1.678 ms]
    pattern = r'(\S+)\s+time:\s+\[([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\]'

    for match in re.finditer(pattern, content):
        name = match.group(1)
        lower = float(match.group(2))
        lower_unit = match.group(3)
        estimate = float(match.group(4))
        estimate_unit = match.group(5)
        upper = float(match.group(6))
        upper_unit = match.group(7)

        # Convert to milliseconds
        estimate_ms = convert_to_ms(estimate, estimate_unit)

        benchmarks[name] = {
            'estimate_ms': estimate_ms,
            'lower_ms': convert_to_ms(lower, lower_unit),
            'upper_ms': convert_to_ms(upper, upper_unit),
        }

    return benchmarks


def convert_to_ms(value: float, unit: str) -> float:
    """Convert time value to milliseconds."""
    conversions = {
        'ns': 0.000001,
        'us': 0.001,
        'µs': 0.001,
        'ms': 1.0,
        's': 1000.0,
    }
    return value * conversions.get(unit, 1.0)


def check_regression(benchmark_name: str, actual_ms: float, baseline_ms: float) -> Tuple[bool, float]:
    """Check if benchmark shows regression."""
    if baseline_ms == 0:
        return False, 0.0

    delta = actual_ms - baseline_ms
    percent_change = (delta / baseline_ms) * 100.0

    is_regression = percent_change > THRESHOLD_PERCENT

    return is_regression, percent_change


def map_benchmark_to_baseline(benchmark_name: str) -> Tuple[str, float]:
    """Map benchmark name to baseline metric."""
    mappings = {
        'otel_overhead': ('otel_overhead_ms', BASELINES['otel_overhead_ms']),
        'container_startup': ('container_startup_ms', BASELINES['container_startup_ms']),
        'test_execution': ('test_execution_ms', BASELINES['test_execution_ms']),
        'memory': ('memory_mb', BASELINES['memory_mb']),
        'binary_size': ('binary_size_mb', BASELINES['binary_size_mb']),
    }

    for key, (metric_name, baseline) in mappings.items():
        if key in benchmark_name.lower():
            return metric_name, baseline

    return 'unknown', 0.0


def generate_summary(benchmarks: Dict[str, Dict[str, float]]) -> str:
    """Generate markdown summary of benchmark results."""
    summary = []
    summary.append("# Performance Regression Test Results")
    summary.append("")
    summary.append("## Baseline Comparison")
    summary.append("")
    summary.append("| Benchmark | Baseline | Actual | Delta | % Change | Status |")
    summary.append("|-----------|----------|--------|-------|----------|--------|")

    regressions = []
    improvements = []
    stable = []

    for bench_name, metrics in sorted(benchmarks.items()):
        metric_name, baseline = map_benchmark_to_baseline(bench_name)
        if baseline == 0:
            continue

        actual = metrics['estimate_ms']
        is_regression, percent_change = check_regression(bench_name, actual, baseline)

        # Format status
        if is_regression:
            status = "⚠️ REGRESSION"
            regressions.append(bench_name)
        elif percent_change < -THRESHOLD_PERCENT:
            status = "✅ IMPROVED"
            improvements.append(bench_name)
        else:
            status = "✅ STABLE"
            stable.append(bench_name)

        delta = actual - baseline

        summary.append(f"| `{bench_name}` | {baseline:.2f}ms | {actual:.2f}ms | "
                       f"{delta:+.2f}ms | {percent_change:+.2f}% | {status} |")

    summary.append("")
    summary.append("## Summary")
    summary.append("")
    summary.append(f"- **Total Benchmarks:** {len(benchmarks)}")
    summary.append(f"- **Regressions:** {len(regressions)} ⚠️")
    summary.append(f"- **Improvements:** {len(improvements)} ✅")
    summary.append(f"- **Stable:** {len(stable)} ✅")
    summary.append("")

    if regressions:
        summary.append("## ⚠️ Regressions Detected")
        summary.append("")
        summary.append("The following benchmarks show performance degradation >5%:")
        summary.append("")
        for bench in regressions:
            summary.append(f"- `{bench}`")
        summary.append("")
        summary.append("**Action Required:** Investigate and optimize before merging.")
    else:
        summary.append("## ✅ No Regressions")
        summary.append("")
        summary.append("All benchmarks are within acceptable performance thresholds.")

    if improvements:
        summary.append("")
        summary.append("## Performance Improvements")
        summary.append("")
        for bench in improvements:
            summary.append(f"- `{bench}` ✅")

    summary.append("")
    summary.append("## Baseline Metrics (v1.2.2)")
    summary.append("")
    summary.append("| Metric | Baseline |")
    summary.append("|--------|----------|")
    for metric, value in BASELINES.items():
        summary.append(f"| {metric} | {value} |")

    summary.append("")
    summary.append("## Thresholds")
    summary.append("")
    summary.append(f"- **Regression Threshold:** ±{THRESHOLD_PERCENT}%")
    summary.append("- **Memory Limit:** <512MB")
    summary.append("- **Binary Size Limit:** <60MB (with 20% buffer)")
    summary.append("")
    summary.append("---")
    summary.append("*Generated by parse_benchmark_results.py*")

    return "\n".join(summary)


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 parse_benchmark_results.py <results-file>")
        sys.exit(1)

    results_file = sys.argv[1]

    try:
        with open(results_file, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"Error: File '{results_file}' not found")
        sys.exit(1)

    # Parse benchmarks
    benchmarks = parse_criterion_output(content)

    if not benchmarks:
        print("Warning: No benchmark results found in file")
        print("")
        print("# Performance Regression Test Results")
        print("")
        print("⚠️ No benchmark data available for analysis.")
        sys.exit(0)

    # Generate summary
    summary = generate_summary(benchmarks)
    print(summary)


if __name__ == '__main__':
    main()
