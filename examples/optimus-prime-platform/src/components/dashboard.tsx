'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { MetricsData } from '@/lib/types';
import { trackEvent } from '@/lib/telemetry';

export function Dashboard() {
  const [metrics, setMetrics] = useState<MetricsData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Track dashboard view
    trackEvent('session_start', { mode: 'executive' });

    // Load initial metrics
    loadMetrics();

    // Auto-refresh every 3 seconds
    const interval = setInterval(loadMetrics, 3000);

    return () => clearInterval(interval);
  }, []);

  const loadMetrics = async () => {
    try {
      const response = await fetch('/api/metrics');
      const data = await response.json();
      setMetrics(data);
    } catch (error) {
      console.error('Failed to load metrics:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading || !metrics) {
    return (
      <Card className="executive-panel">
        <CardContent className="p-8 text-center">
          <div className="animate-spin w-8 h-8 border-4 border-[hsl(var(--cyber-blue))] border-t-transparent rounded-full mx-auto mb-4"></div>
          <p className="text-[hsl(var(--gunmetal))]">Loading analytics...</p>
        </CardContent>
      </Card>
    );
  }

  const ctrA = metrics.ab.A.views > 0 ? (metrics.ab.A.clicks / metrics.ab.A.views) * 100 : 0;
  const ctrB = metrics.ab.B.views > 0 ? (metrics.ab.B.clicks / metrics.ab.B.views) * 100 : 0;

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="executive-panel">
        <CardHeader>
          <CardTitle className="text-2xl text-[hsl(var(--cyber-blue))] flex items-center gap-3">
            <div className="w-8 h-8 bg-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white font-bold">
              📊
            </div>
            Executive Dashboard
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[hsl(var(--gunmetal))]">
            Real-time analytics and performance metrics for the Optimus Prime platform.
          </p>
        </CardContent>
      </Card>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="executive-panel">
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-[hsl(var(--cyber-blue))] mb-2">
              ${metrics.totals.revenue.toLocaleString()}
            </div>
            <p className="text-[hsl(var(--gunmetal))] text-sm">
              Total Revenue (7d)
            </p>
          </CardContent>
        </Card>

        <Card className="executive-panel">
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-[hsl(var(--cyber-blue))] mb-2">
              {metrics.totals.events}
            </div>
            <p className="text-[hsl(var(--gunmetal))] text-sm">
              Total Events
            </p>
          </CardContent>
        </Card>

        <Card className="executive-panel">
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-[hsl(var(--cyber-blue))] mb-2">
              {((ctrA + ctrB) / 2).toFixed(1)}%
            </div>
            <p className="text-[hsl(var(--gunmetal))] text-sm">
              Avg Premium CTR
            </p>
          </CardContent>
        </Card>
      </div>

      {/* A/B Testing Results */}
      <Card className="executive-panel">
        <CardHeader>
          <CardTitle className="text-[hsl(var(--cyber-blue))]">
            A/B Test Results - Premium CTA
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-6">
            <div className="text-center">
              <h3 className="text-lg font-semibold text-[hsl(var(--autobot-red))] mb-2">
                Variant A
              </h3>
              <div className="text-3xl font-bold text-[hsl(var(--cyber-blue))] mb-1">
                {ctrA.toFixed(1)}%
              </div>
              <p className="text-sm text-[hsl(var(--gunmetal))]">
                {metrics.ab.A.views} views, {metrics.ab.A.clicks} clicks
              </p>
            </div>

            <div className="text-center">
              <h3 className="text-lg font-semibold text-[hsl(var(--energon))] mb-2">
                Variant B
              </h3>
              <div className="text-3xl font-bold text-[hsl(var(--cyber-blue))] mb-1">
                {ctrB.toFixed(1)}%
              </div>
              <p className="text-sm text-[hsl(var(--gunmetal))]">
                {metrics.ab.B.views} views, {metrics.ab.B.clicks} clicks
              </p>
            </div>
          </div>

          {ctrA > ctrB && (
            <div className="mt-4 p-3 bg-green-900/20 border border-green-500/30 rounded-lg">
              <p className="text-green-300 text-sm">
                🎯 Variant A is performing better! Consider making it the default.
              </p>
            </div>
          )}

          {ctrB > ctrA && (
            <div className="mt-4 p-3 bg-green-900/20 border border-green-500/30 rounded-lg">
              <p className="text-green-300 text-sm">
                🎯 Variant B is performing better! Consider making it the default.
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Revenue Chart */}
      <Card className="executive-panel">
        <CardHeader>
          <CardTitle className="text-[hsl(var(--cyber-blue))]">
            Revenue Trend (7 Days)
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {metrics.revenue7.labels.map((label, index) => (
              <div key={label} className="flex items-center justify-between">
                <span className="text-[hsl(var(--gunmetal))] font-medium">
                  {new Date(label).toLocaleDateString()}
                </span>
                <div className="flex items-center gap-3">
                  <div className="w-32 h-2 bg-[hsl(var(--steel))]/30 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-[hsl(var(--energon))] transition-all duration-500"
                      style={{
                        width: `${(metrics.revenue7.data[index] / Math.max(...metrics.revenue7.data)) * 100}%`
                      }}
                    />
                  </div>
                  <span className="text-[hsl(var(--cyber-blue))] font-semibold w-20 text-right">
                    ${metrics.revenue7.data[index].toLocaleString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Funnel Analysis */}
      <Card className="executive-panel">
        <CardHeader>
          <CardTitle className="text-[hsl(var(--cyber-blue))]">
            User Journey Funnel
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {metrics.funnel.map((step, index) => (
              <div key={step.label} className="flex items-center justify-between">
                <span className="text-[hsl(var(--gunmetal))] font-medium">
                  {step.label}
                </span>
                <div className="flex items-center gap-3">
                  <div className="w-24 h-2 bg-[hsl(var(--steel))]/30 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-[hsl(var(--autobot-red))] transition-all duration-500"
                      style={{
                        width: `${(step.value / Math.max(...metrics.funnel.map(s => s.value))) * 100}%`
                      }}
                    />
                  </div>
                  <span className="text-[hsl(var(--cyber-blue))] font-semibold w-16 text-right">
                    {step.value}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Key Insights */}
      <Card className="executive-panel">
        <CardHeader>
          <CardTitle className="text-[hsl(var(--cyber-blue))]">
            Key Insights
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="p-3 bg-[hsl(var(--cyber-blue))]/10 border border-[hsl(var(--cyber-blue))]/20 rounded-lg">
              <p className="text-[hsl(var(--cyber-blue))] font-medium">
                💡 Conversion Rate: {((metrics.funnel[5]?.value || 0) / (metrics.funnel[0]?.value || 1) * 100).toFixed(1)}% from session to premium click
              </p>
            </div>

            {metrics.revenue7.data.length > 0 && (
              <div className="p-3 bg-[hsl(var(--energon))]/10 border border-[hsl(var(--energon))]/20 rounded-lg">
                <p className="text-[hsl(var(--gunmetal))] font-medium">
                  📈 Best Day: {metrics.revenue7.labels[metrics.revenue7.data.indexOf(Math.max(...metrics.revenue7.data))]} (${Math.max(...metrics.revenue7.data).toLocaleString()})
                </p>
              </div>
            )}

            <div className="p-3 bg-[hsl(var(--steel))]/10 border border-[hsl(var(--steel))]/20 rounded-lg">
              <p className="text-[hsl(var(--gunmetal))] font-medium">
                🎯 Total Engagement: {metrics.totals.events} events tracked across all sessions
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
