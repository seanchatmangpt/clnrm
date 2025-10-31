#!/usr/bin/env python3
"""
Coverage Dashboard Script
Displays historical telemetry coverage trends and statistics

Usage:
    python3 scripts/coverage_dashboard.py [--days N] [--baseline N]
"""

import json
import glob
import argparse
from datetime import datetime, timedelta
from pathlib import Path

def load_history(history_dir, days=30):
    """Load coverage history from JSON files."""
    cutoff = datetime.now() - timedelta(days=days)

    history_files = sorted(glob.glob(f"{history_dir}/*.json"))
    records = []

    for filepath in history_files:
        try:
            with open(filepath) as f:
                record = json.load(f)
                record_time = datetime.fromisoformat(record['timestamp'])

                if record_time >= cutoff:
                    records.append(record)
        except (json.JSONDecodeError, KeyError, ValueError) as e:
            print(f"Warning: Skipping invalid file {filepath}: {e}")
            continue

    return records

def print_dashboard(records, baseline=80):
    """Print ASCII dashboard of coverage trends."""
    print("=" * 80)
    print("📊 TELEMETRY COVERAGE DASHBOARD")
    print("=" * 80)
    print()

    if not records:
        print("No coverage data found.")
        print("Run: ./scripts/track_coverage.sh")
        return

    # Current status
    latest = records[-1]
    coverage = latest['coverage_percent']
    violations = latest['violations']
    warnings = latest['warnings']

    status = "✅ PASS" if coverage >= baseline and violations == 0 else "❌ FAIL"

    print(f"Current Status: {status}")
    print(f"Coverage: {coverage}% (baseline: {baseline}%)")
    print(f"Violations: {violations}")
    print(f"Warnings: {warnings}")
    print(f"Last Updated: {latest['timestamp']}")
    print(f"Git Commit: {latest.get('git_commit', 'unknown')}")
    print(f"Git Branch: {latest.get('git_branch', 'unknown')}")
    print()

    # Trend chart
    print("📈 Coverage Trend (last 30 days):")
    print()

    max_records = min(30, len(records))
    for record in records[-max_records:]:
        date = datetime.fromisoformat(record['timestamp']).strftime('%Y-%m-%d')
        coverage = record['coverage_percent']
        violations = record['violations']

        bar_length = coverage
        bar = '█' * (bar_length // 2)

        violation_marker = f" ⚠️ {violations} violations" if violations > 0 else ""

        print(f"{date}  {bar:<50} {coverage:3d}%{violation_marker}")

    print()

    # Statistics
    coverages = [r['coverage_percent'] for r in records]
    avg_coverage = sum(coverages) / len(coverages)
    min_coverage = min(coverages)
    max_coverage = max(coverages)

    total_violations = sum(r['violations'] for r in records)

    print("📊 Statistics:")
    print(f"   Average Coverage: {avg_coverage:.1f}%")
    print(f"   Min Coverage: {min_coverage}%")
    print(f"   Max Coverage: {max_coverage}%")
    print(f"   Total Violations: {total_violations}")
    print(f"   Data Points: {len(records)}")
    print()

    # Health check
    if avg_coverage >= baseline and total_violations == 0:
        print("✅ Telemetry health: EXCELLENT")
    elif avg_coverage >= baseline * 0.9:
        print("⚠️  Telemetry health: GOOD (some violations)")
    else:
        print("❌ Telemetry health: NEEDS IMPROVEMENT")

    print("=" * 80)

def main():
    parser = argparse.ArgumentParser(
        description='Display telemetry coverage dashboard',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Show last 30 days with 80% baseline
  python3 scripts/coverage_dashboard.py

  # Show last 7 days
  python3 scripts/coverage_dashboard.py --days 7

  # Custom baseline
  python3 scripts/coverage_dashboard.py --baseline 90
        """
    )
    parser.add_argument('--days', type=int, default=30, help='Days of history to show (default: 30)')
    parser.add_argument('--history-dir', default='.weaver/coverage/history', help='History directory')
    parser.add_argument('--baseline', type=int, default=80, help='Coverage baseline percentage (default: 80)')

    args = parser.parse_args()

    # Check if history directory exists
    if not Path(args.history_dir).exists():
        print(f"❌ History directory not found: {args.history_dir}")
        print("   Run: ./scripts/track_coverage.sh")
        return 1

    records = load_history(args.history_dir, args.days)
    print_dashboard(records, args.baseline)

    return 0

if __name__ == '__main__':
    exit(main())
