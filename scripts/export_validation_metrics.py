#!/usr/bin/env python3
"""
Prometheus Metrics Exporter for Weaver Validation Results

Exports telemetry validation metrics in Prometheus format for monitoring.

Usage:
    python3 scripts/export_validation_metrics.py --port 9090
"""

from prometheus_client import start_http_server, Gauge, Counter, Info
import json
import time
import argparse
import logging
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Define Prometheus metrics
telemetry_violations = Gauge(
    'telemetry_violations',
    'Number of telemetry schema violations',
    ['environment']
)

telemetry_warnings = Gauge(
    'telemetry_warnings',
    'Number of telemetry schema warnings',
    ['environment']
)

telemetry_coverage = Gauge(
    'telemetry_coverage_percent',
    'Telemetry registry coverage percentage',
    ['environment']
)

validation_runs = Counter(
    'telemetry_validation_runs_total',
    'Total number of validation runs',
    ['environment', 'status']
)

validation_info = Info(
    'telemetry_validation',
    'Telemetry validation metadata'
)

def read_latest_validation(output_dir, environment='production'):
    """Read latest validation result from output directory."""
    try:
        output_path = Path(output_dir)

        if not output_path.exists():
            logger.warning(f"Output directory not found: {output_dir}")
            return None

        # Find most recent validation output
        validation_files = sorted(output_path.glob('*/live_check.json'))

        if not validation_files:
            validation_files = sorted(output_path.glob('live_check.json'))

        if not validation_files:
            logger.warning(f"No validation files found in {output_dir}")
            return None

        latest_file = validation_files[-1]
        logger.debug(f"Reading validation from {latest_file}")

        with open(latest_file) as f:
            data = json.load(f)

        # Extract statistics
        stats = data.get('statistics', {})
        advice_counts = stats.get('advice_level_counts', {})

        violations = advice_counts.get('violation', 0)
        warnings = advice_counts.get('warning', 0)
        coverage = stats.get('registry_coverage', 0.0)

        return {
            'violations': violations,
            'warnings': warnings,
            'coverage': coverage * 100,  # Convert to percentage
            'status': 'pass' if violations == 0 else 'fail',
            'timestamp': data.get('timestamp', 'unknown')
        }

    except (json.JSONDecodeError, KeyError) as e:
        logger.error(f"Error reading validation file: {e}")
        return None

def update_metrics(output_dir, environment='production'):
    """Update Prometheus metrics from validation results."""
    result = read_latest_validation(output_dir, environment)

    if result:
        logger.info(f"Updating metrics for {environment}: "
                   f"violations={result['violations']}, "
                   f"warnings={result['warnings']}, "
                   f"coverage={result['coverage']:.1f}%")

        # Update gauges
        telemetry_violations.labels(environment=environment).set(result['violations'])
        telemetry_warnings.labels(environment=environment).set(result['warnings'])
        telemetry_coverage.labels(environment=environment).set(result['coverage'])

        # Increment counter
        validation_runs.labels(
            environment=environment,
            status=result['status']
        ).inc()

        # Update info
        validation_info.info({
            'environment': environment,
            'last_validation': result['timestamp']
        })

    else:
        logger.warning(f"No validation data available for {environment}")

def main():
    parser = argparse.ArgumentParser(
        description='Export telemetry validation metrics to Prometheus',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Start metrics server on default port
  python3 scripts/export_validation_metrics.py

  # Custom port and directory
  python3 scripts/export_validation_metrics.py --port 9090 --output-dir .weaver/production

  # Monitor development validation
  python3 scripts/export_validation_metrics.py --environment development --output-dir .weaver/dev
        """
    )
    parser.add_argument(
        '--port', type=int, default=9090,
        help='Metrics server port (default: 9090)'
    )
    parser.add_argument(
        '--output-dir', default='.weaver/production',
        help='Validation output directory (default: .weaver/production)'
    )
    parser.add_argument(
        '--environment', default='production',
        help='Environment name (default: production)'
    )
    parser.add_argument(
        '--interval', type=int, default=60,
        help='Update interval in seconds (default: 60)'
    )
    parser.add_argument(
        '--verbose', action='store_true',
        help='Enable verbose logging'
    )

    args = parser.parse_args()

    if args.verbose:
        logger.setLevel(logging.DEBUG)

    # Start Prometheus HTTP server
    start_http_server(args.port)
    logger.info(f"📊 Metrics server started on port {args.port}")
    logger.info(f"   Metrics endpoint: http://localhost:{args.port}/metrics")
    logger.info(f"   Output directory: {args.output_dir}")
    logger.info(f"   Environment: {args.environment}")
    logger.info(f"   Update interval: {args.interval}s")
    print()

    # Update metrics periodically
    try:
        while True:
            try:
                update_metrics(args.output_dir, args.environment)
                logger.info(f"✅ Metrics updated at {time.strftime('%Y-%m-%d %H:%M:%S')}")
            except Exception as e:
                logger.error(f"❌ Error updating metrics: {e}")

            time.sleep(args.interval)

    except KeyboardInterrupt:
        logger.info("\n👋 Shutting down metrics server")
        return 0

if __name__ == '__main__':
    exit(main())
