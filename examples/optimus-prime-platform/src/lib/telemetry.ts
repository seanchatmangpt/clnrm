import { TelemetryEvent, EventType, MetricsData, StaticCorpData, STATIC_CORP_DATA } from './types';

// In-memory event store
let events: TelemetryEvent[] = [];
let abBuckets: Record<'A' | 'B', { views: number; clicks: number }> = {
  A: { views: 0, clicks: 0 },
  B: { views: 0, clicks: 0 }
};

export function trackEvent(event: EventType, payload: Record<string, any> = {}) {
  const telemetryEvent: TelemetryEvent = {
    id: crypto.randomUUID(),
    ts: Date.now(),
    event,
    payload
  };

  events.push(telemetryEvent);
  console.log('📊 Event tracked:', telemetryEvent);

  // Update A/B buckets for premium CTA tracking
  if (event === 'premium_view') {
    const variant = payload.variant as 'A' | 'B' || 'A';
    abBuckets[variant].views++;
  } else if (event === 'premium_click') {
    const variant = payload.variant as 'A' | 'B' || 'A';
    abBuckets[variant].clicks++;
  }
}

export function getEvents(): TelemetryEvent[] {
  return [...events];
}

export function clearEvents() {
  events = [];
  abBuckets = { A: { views: 0, clicks: 0 }, B: { views: 0, clicks: 0 } };
}

export function getMetrics(): MetricsData {
  // Generate mock revenue data for last 7 days
  const today = new Date();
  const revenue7 = {
    labels: [] as string[],
    data: [] as number[]
  };

  for (let i = 6; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    revenue7.labels.push(date.toISOString().split('T')[0]);

    // Generate some mock revenue data
    const baseRevenue = 1000 + Math.random() * 2000;
    const eventsForDay = events.filter(e =>
      new Date(e.ts).toDateString() === date.toDateString()
    );
    const eventMultiplier = Math.min(eventsForDay.length / 10, 2);
    revenue7.data.push(Math.round(baseRevenue * eventMultiplier));
  }

  // Calculate funnel data
  const sessions = events.filter(e => e.event === 'session_start').length;
  const messages = events.filter(e => e.event === 'message_sent').length;
  const virtues = events.filter(e => e.event === 'virtue_detected').length;
  const rewards = events.filter(e => e.event === 'reward_click').length;
  const premiumViews = events.filter(e => e.event === 'premium_view').length;
  const premiumClicks = events.filter(e => e.event === 'premium_click').length;

  const funnel: Array<{ label: string; value: number }> = [
    { label: 'Sessions', value: sessions },
    { label: 'Messages', value: messages },
    { label: 'Virtues', value: virtues },
    { label: 'Rewards', value: rewards },
    { label: 'Premium Views', value: premiumViews },
    { label: 'Premium Clicks', value: premiumClicks }
  ];

  // Calculate totals
  const totalRevenue = revenue7.data.reduce((sum, val) => sum + val, 0);

  return {
    ab: abBuckets,
    funnel,
    revenue7,
    totals: {
      revenue: totalRevenue,
      events: events.length
    }
  };
}

export function getStaticCorpData(): StaticCorpData {
  return STATIC_CORP_DATA;
}

// A/B testing utilities
export function getABVariant(): 'A' | 'B' {
  // Simple client-side assignment based on timestamp
  const variant = Date.now() % 2 === 0 ? 'A' : 'B';
  return variant as 'A' | 'B';
}

export function trackPremiumView(variant: 'A' | 'B') {
  trackEvent('premium_view', { variant });
}

export function trackPremiumClick(variant: 'A' | 'B') {
  trackEvent('premium_click', { variant });
}
