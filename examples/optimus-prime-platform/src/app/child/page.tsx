import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ChildChat } from "@/components/child-chat";
import { trackEvent } from "@/lib/telemetry";

export default function ChildPage() {
  // Track page view
  trackEvent("session_start", { mode: "child" });

  return (
    <div className="min-h-screen bg-gradient-to-br from-[hsl(var(--autobot-red))]/10 to-[hsl(var(--energon))]/10">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <Card className="child-panel mb-8">
          <CardHeader>
            <CardTitle className="text-3xl text-[hsl(var(--autobot-red))] flex items-center gap-4">
              <div className="w-12 h-12 bg-[hsl(var(--autobot-red))] rounded-full flex items-center justify-center text-white font-bold text-xl">
                O
              </div>
              Optimus Prime Character Platform
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2 mb-4">
              <Badge className="bg-[hsl(var(--energon))] text-[hsl(var(--gunmetal))]">
                Child Mode
              </Badge>
              <Badge className="bg-[hsl(var(--autobot-red))] text-white">
                Leadership Development
              </Badge>
            </div>
            <p className="text-[hsl(var(--gunmetal))] text-lg">
              Share your achievements and let Optimus Prime recognize your
              leadership qualities. Unlock rewards and premium adventures!
            </p>
          </CardContent>
        </Card>

        {/* Chat Interface */}
        <ChildChat />
      </div>
    </div>
  );
}
