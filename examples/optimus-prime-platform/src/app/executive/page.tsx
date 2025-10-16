import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ExecutiveChat } from '@/components/executive-chat';
import { trackEvent } from '@/lib/telemetry';

export default function ExecutivePage() {
  // Track page view
  trackEvent('session_start', { mode: 'executive' });

  return (
    <div className="min-h-screen bg-gradient-to-br from-[hsl(var(--cyber-blue))]/10 to-[hsl(var(--steel))]/10">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <Card className="executive-panel mb-8">
          <CardHeader>
            <CardTitle className="text-3xl text-[hsl(var(--cyber-blue))] flex items-center gap-4">
              <div className="w-12 h-12 bg-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white font-bold text-xl">
                A
              </div>
              Executive Analytics Platform
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2 mb-4">
              <Badge className="bg-[hsl(var(--cyber-blue))] text-white">
                Executive Mode
              </Badge>
              <Badge className="bg-[hsl(var(--steel))] text-[hsl(var(--gunmetal))]">
                Real-time Analytics
              </Badge>
            </div>
            <p className="text-[hsl(var(--gunmetal))] text-lg">
              Get instant insights into KPIs, revenue metrics, and performance data.
              Ask questions and receive data-driven answers.
            </p>
          </CardContent>
        </Card>

        {/* Chat Interface */}
        <ExecutiveChat />
      </div>
    </div>
  );
}
