// OpenTelemetry instrumentation for Next.js
// This file is automatically loaded by Next.js when present at the root

import { NodeSDK } from '@opentelemetry/sdk-node';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-http';
import { PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { ATTR_SERVICE_NAME, ATTR_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';
import { ConsoleSpanExporter } from '@opentelemetry/sdk-trace-node';
import { ConsoleMetricExporter } from '@opentelemetry/sdk-metrics';

// Configure OpenTelemetry SDK
const sdk = new NodeSDK({
  serviceName: 'optimus-prime-platform',

  // Trace exporter - use console for development, OTLP for production
  traceExporter: process.env.OTEL_EXPORTER_OTLP_ENDPOINT
    ? new OTLPTraceExporter({
        url: `${process.env.OTEL_EXPORTER_OTLP_ENDPOINT}/v1/traces`,
      })
    : new ConsoleSpanExporter(),

  // Metric exporter with periodic export (every 60 seconds)
  metricReader: new PeriodicExportingMetricReader({
    exporter: process.env.OTEL_EXPORTER_OTLP_ENDPOINT
      ? new OTLPMetricExporter({
          url: `${process.env.OTEL_EXPORTER_OTLP_ENDPOINT}/v1/metrics`,
        })
      : new ConsoleMetricExporter(),
    exportIntervalMillis: 60000,
  }),

  // Auto-instrumentations for Node.js
  instrumentations: [
    getNodeAutoInstrumentations({
      '@opentelemetry/instrumentation-fs': {
        enabled: false, // Disable file system instrumentation to reduce noise
      },
    }),
  ],
});

// Start the SDK
sdk.start();

// Graceful shutdown on process termination
// Note: Only run in Node.js runtime (not Edge runtime)
if (typeof process !== 'undefined' && process.on) {
  process.on('SIGTERM', () => {
    sdk
      .shutdown()
      .then(() => console.log('OpenTelemetry terminated'))
      .catch((error) => console.error('Error terminating OpenTelemetry', error));
  });
}

// Export for Next.js instrumentation hook
export async function register() {
  // The SDK is already started above, this is just for Next.js compatibility
  console.log('OpenTelemetry instrumentation registered');
}
