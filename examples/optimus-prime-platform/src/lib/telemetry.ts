import { trace, metrics, context, SpanStatusCode } from '@opentelemetry/api';
import {
  TelemetryEvent,
  EventType,
  MetricsData,
  StaticCorpData,
  STATIC_CORP_DATA,
  VirtueHistory,
} from "./types";

// OpenTelemetry tracer and meter
const tracer = trace.getTracer('optimus-prime-platform', '0.1.0');
const meter = metrics.getMeter('optimus-prime-platform', '0.1.0');

// OpenTelemetry Metrics
const eventCounter = meter.createCounter('events.total', {
  description: 'Total number of events tracked',
});

const sessionCounter = meter.createCounter('sessions.total', {
  description: 'Total number of sessions started',
});

const virtueCounter = meter.createCounter('virtues.detected', {
  description: 'Total number of virtues detected',
});

const premiumViewCounter = meter.createCounter('premium.views', {
  description: 'Total premium CTA views',
});

const premiumClickCounter = meter.createCounter('premium.clicks', {
  description: 'Total premium CTA clicks',
});

const rewardViewCounter = meter.createCounter('rewards.views', {
  description: 'Total reward views',
});

const rewardClickCounter = meter.createCounter('rewards.clicks', {
  description: 'Total reward clicks',
});

const abTestViewsCounter = meter.createCounter('ab_test.views', {
  description: 'A/B test variant views',
});

const abTestClicksCounter = meter.createCounter('ab_test.clicks', {
  description: 'A/B test variant clicks',
});

// In-memory stores (for backwards compatibility with existing queries)
// These will be populated by OTel events for dashboard queries
let events: TelemetryEvent[] = [];
let abBuckets: Record<
  "A" | "B",
  { variant: "A" | "B"; views: number; clicks: number }
> = {
  A: { variant: "A", views: 0, clicks: 0 },
  B: { variant: "B", views: 0, clicks: 0 },
};
let virtueHistory: VirtueHistory[] = [];

/**
 * Track an event using OpenTelemetry
 */
export function trackEvent(
  event: EventType,
  payload: Record<string, unknown> = {}
) {
  const span = tracer.startSpan(`event.${event}`);

  try {
    const telemetryEvent: TelemetryEvent = {
      id: crypto.randomUUID(),
      ts: Date.now(),
      event,
      payload,
    };

    // Add span attributes
    span.setAttributes({
      'event.type': event,
      'event.id': telemetryEvent.id,
      'event.timestamp': telemetryEvent.ts,
      ...Object.entries(payload).reduce((acc, [key, value]) => {
        acc[`event.payload.${key}`] = String(value);
        return acc;
      }, {} as Record<string, string>),
    });

    // Store in memory for backwards compatibility
    events.push(telemetryEvent);

    // Increment OpenTelemetry counters
    eventCounter.add(1, { 'event.type': event });

    // Track specific event types
    switch (event) {
      case 'session_start':
        sessionCounter.add(1, { mode: String(payload.mode || 'unknown') });
        break;

      case 'virtue_detected':
        virtueCounter.add(1, { virtue: String(payload.virtue || 'unknown') });
        break;

      case 'premium_view':
        const viewVariant = (payload.variant as "A" | "B") || "A";
        premiumViewCounter.add(1, { variant: viewVariant });
        abTestViewsCounter.add(1, { variant: viewVariant });
        abBuckets[viewVariant].views++;
        break;

      case 'premium_click':
        const clickVariant = (payload.variant as "A" | "B") || "A";
        premiumClickCounter.add(1, { variant: clickVariant });
        abTestClicksCounter.add(1, { variant: clickVariant });
        abBuckets[clickVariant].clicks++;
        break;

      case 'reward_view':
        rewardViewCounter.add(1, { virtue: String(payload.virtue || 'unknown') });
        break;

      case 'reward_click':
        rewardClickCounter.add(1, { virtue: String(payload.virtue || 'unknown') });
        break;
    }

    span.setStatus({ code: SpanStatusCode.OK });
  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    throw error;
  } finally {
    span.end();
  }
}

/**
 * Track virtue detection with OpenTelemetry
 */
export function trackVirtue(virtue: string, achievement: string) {
  const span = tracer.startSpan('virtue.track', {
    attributes: {
      'virtue.type': virtue,
      'virtue.achievement': achievement,
    },
  });

  try {
    const historyEntry: VirtueHistory = {
      id: crypto.randomUUID(),
      virtue,
      timestamp: Date.now(),
      achievement,
    };

    virtueHistory.push(historyEntry);
    trackEvent("virtue_detected", { virtue, achievement });

    span.setStatus({ code: SpanStatusCode.OK });
  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    throw error;
  } finally {
    span.end();
  }
}

/**
 * Track premium CTA view
 */
export function trackPremiumView(variant: "A" | "B") {
  trackEvent("premium_view", { variant });
}

/**
 * Track premium CTA click
 */
export function trackPremiumClick(variant: "A" | "B") {
  trackEvent("premium_click", { variant });
}

/**
 * Track reward view
 */
export function trackRewardView(virtue: string, variant: "A" | "B") {
  trackEvent("reward_view", { virtue, variant });
}

// Backwards compatibility getters
export function getEvents(): TelemetryEvent[] {
  return [...events];
}

export function getVirtueHistory(): VirtueHistory[] {
  return [...virtueHistory];
}

export function getVirtueCount(): Record<string, number> {
  return virtueHistory.reduce((acc, item) => {
    acc[item.virtue] = (acc[item.virtue] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);
}

export function getRewardMetrics() {
  const views = events.filter((e) => e.event === "reward_view").length;
  const clicks = events.filter((e) => e.event === "reward_click").length;
  const conversions = events.filter(
    (e) =>
      e.event === "purchase" &&
      (e.payload as any).type === "reward_conversion"
  ).length;

  return {
    views,
    clicks,
    conversions,
    ctr: views > 0 ? (clicks / views) * 100 : 0,
    conversionRate: clicks > 0 ? (conversions / clicks) * 100 : 0,
  };
}

export function getMetrics(): MetricsData {
  // Generate revenue data for last 7 days
  const today = new Date();
  const revenue7 = {
    labels: [] as string[],
    data: [] as number[],
  };

  for (let i = 6; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    revenue7.labels.push(date.toISOString().split("T")[0]);

    const baseRevenue = 1000 + Math.random() * 2000;
    const eventsForDay = events.filter(
      (e) => new Date(e.ts).toDateString() === date.toDateString()
    );
    const eventMultiplier = Math.min(eventsForDay.length / 10, 2);
    revenue7.data.push(Math.round(baseRevenue * eventMultiplier));
  }

  // Calculate funnel data
  const sessions = events.filter((e) => e.event === "session_start").length;
  const messages = events.filter((e) => e.event === "message_sent").length;
  const virtues = events.filter((e) => e.event === "virtue_detected").length;
  const rewards = events.filter((e) => e.event === "reward_click").length;
  const premiumViews = events.filter((e) => e.event === "premium_view").length;
  const premiumClicks = events.filter(
    (e) => e.event === "premium_click"
  ).length;

  const funnel: Array<{ label: string; value: number }> = [
    { label: "Sessions", value: sessions },
    { label: "Messages", value: messages },
    { label: "Virtues", value: virtues },
    { label: "Rewards", value: rewards },
    { label: "Premium Views", value: premiumViews },
    { label: "Premium Clicks", value: premiumClicks },
  ];

  const totalRevenue = revenue7.data.reduce((sum, val) => sum + val, 0);

  return {
    ab: abBuckets,
    funnel,
    revenue7,
    totals: {
      revenue: totalRevenue,
      events: events.length,
    },
  };
}

export function clearEvents() {
  events = [];
  abBuckets = {
    A: { variant: "A", views: 0, clicks: 0 },
    B: { variant: "B", views: 0, clicks: 0 },
  };
  virtueHistory = [];
}

export function getStaticCorpData(): StaticCorpData {
  return STATIC_CORP_DATA;
}

export function getABVariant(): "A" | "B" {
  const variant = Date.now() % 2 === 0 ? "A" : "B";
  return variant as "A" | "B";
}
