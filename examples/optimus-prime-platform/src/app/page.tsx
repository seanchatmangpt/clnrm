import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

export default function HomePage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-[hsl(var(--autobot-red))]/10 via-[hsl(var(--cyber-blue))]/10 to-[hsl(var(--energon))]/10">
      <div className="container mx-auto px-4 py-16">
        {/* Header */}
        <div className="text-center mb-16">
          <div className="flex justify-center mb-6">
            <div className="w-20 h-20 bg-gradient-to-br from-[hsl(var(--autobot-red))] to-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white font-bold text-3xl shadow-2xl">
              OP
            </div>
          </div>
          <h1 className="text-5xl font-bold text-[hsl(var(--gunmetal))] mb-4">
            Optimus Prime Character Platform
          </h1>
          <p className="text-xl text-[hsl(var(--gunmetal))]/80 mb-8 max-w-3xl mx-auto">
            A production-ready AI character engine that reinforces child virtues through Optimus Prime
            while providing executives with real-time analytics and revenue optimization.
          </p>

          <div className="flex flex-wrap justify-center gap-3 mb-8">
            <Badge className="bg-[hsl(var(--autobot-red))] text-white text-sm px-4 py-2">
              Child Leadership Development
            </Badge>
            <Badge className="bg-[hsl(var(--cyber-blue))] text-white text-sm px-4 py-2">
              Executive Analytics
            </Badge>
            <Badge className="bg-[hsl(var(--energon))] text-[hsl(var(--gunmetal))] text-sm px-4 py-2">
              Revenue Optimization
            </Badge>
          </div>
        </div>

        {/* Main Actions */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 max-w-6xl mx-auto">
          {/* Child Mode */}
          <Card className="child-panel hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-1">
            <CardHeader>
              <CardTitle className="text-2xl text-[hsl(var(--autobot-red))] flex items-center gap-3">
                <div className="w-10 h-10 bg-[hsl(var(--autobot-red))] rounded-full flex items-center justify-center text-white font-bold">
                  👦
                </div>
                Child Mode
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-[hsl(var(--gunmetal))] mb-6">
                Help children develop leadership qualities through Optimus Prime.
                Share achievements, receive recognition, and unlock rewards.
              </p>
              <Button asChild className="autobot-button w-full">
                <Link href="/child">
                  Start Leadership Journey
                </Link>
              </Button>
            </CardContent>
          </Card>

          {/* Executive Mode */}
          <Card className="executive-panel hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-1">
            <CardHeader>
              <CardTitle className="text-2xl text-[hsl(var(--cyber-blue))] flex items-center gap-3">
                <div className="w-10 h-10 bg-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white font-bold">
                  📊
                </div>
                Executive Mode
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-[hsl(var(--gunmetal))] mb-6">
                Access real-time analytics, KPI tracking, and performance metrics.
                Get data-driven insights for platform optimization.
              </p>
              <Button asChild className="cyber-button w-full">
                <Link href="/executive">
                  View Analytics
                </Link>
              </Button>
            </CardContent>
          </Card>

          {/* Admin Dashboard */}
          <Card className="bg-gradient-to-br from-[hsl(var(--gunmetal))]/10 to-[hsl(var(--steel))]/10 border-2 border-[hsl(var(--gunmetal))]/20 hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-1">
            <CardHeader>
              <CardTitle className="text-2xl text-[hsl(var(--gunmetal))] flex items-center gap-3">
                <div className="w-10 h-10 bg-[hsl(var(--gunmetal))] rounded-full flex items-center justify-center text-white font-bold">
                  ⚙️
                </div>
                Admin Dashboard
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-[hsl(var(--gunmetal))] mb-6">
                Comprehensive dashboard with real-time metrics, A/B testing results,
                and performance analytics for platform management.
              </p>
              <Button asChild className="w-full" variant="outline">
                <Link href="/admin/dashboard">
                  Access Dashboard
                </Link>
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Features Section */}
        <div className="mt-16 max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-[hsl(var(--gunmetal))] text-center mb-12">
            Platform Features
          </h2>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            <Card className="text-center child-panel">
              <CardContent className="p-6">
                <div className="w-12 h-12 bg-[hsl(var(--autobot-red))] rounded-full flex items-center justify-center text-white font-bold text-xl mx-auto mb-4">
                  🏆
                </div>
                <h3 className="font-semibold text-[hsl(var(--autobot-red))] mb-2">
                  Leadership Recognition
                </h3>
                <p className="text-sm text-[hsl(var(--gunmetal))]">
                  Optimus Prime recognizes and encourages leadership qualities in children.
                </p>
              </CardContent>
            </Card>

            <Card className="text-center executive-panel">
              <CardContent className="p-6">
                <div className="w-12 h-12 bg-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white font-bold text-xl mx-auto mb-4">
                  📈
                </div>
                <h3 className="font-semibold text-[hsl(var(--cyber-blue))] mb-2">
                  Real-time Analytics
                </h3>
                <p className="text-sm text-[hsl(var(--gunmetal))]">
                  Live KPI tracking and performance metrics for executives.
                </p>
              </CardContent>
            </Card>

            <Card className="text-center bg-gradient-to-br from-[hsl(var(--energon))]/20 to-[hsl(var(--autobot-red))]/20 border-[hsl(var(--energon))]">
              <CardContent className="p-6">
                <div className="w-12 h-12 bg-[hsl(var(--energon))] rounded-full flex items-center justify-center text-[hsl(var(--gunmetal))] font-bold text-xl mx-auto mb-4">
                  💎
                </div>
                <h3 className="font-semibold text-[hsl(var(--energon))] mb-2">
                  Premium Monetization
                </h3>
                <p className="text-sm text-[hsl(var(--gunmetal))]">
                  Transparent premium features with A/B testing for optimization.
                </p>
              </CardContent>
            </Card>

            <Card className="text-center bg-gradient-to-br from-[hsl(var(--steel))]/20 to-[hsl(var(--gunmetal))]/20 border-[hsl(var(--steel))]">
              <CardContent className="p-6">
                <div className="w-12 h-12 bg-[hsl(var(--steel))] rounded-full flex items-center justify-center text-[hsl(var(--gunmetal))] font-bold text-xl mx-auto mb-4">
                  🔒
                </div>
                <h3 className="font-semibold text-[hsl(var(--gunmetal))] mb-2">
                  Safety & Compliance
                </h3>
                <p className="text-sm text-[hsl(var(--gunmetal))]">
                  Leadership reframing and child-safe interactions.
                </p>
              </CardContent>
            </Card>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-16 text-center">
          <p className="text-[hsl(var(--gunmetal))]/60 mb-4">
            Built with Next.js, TypeScript, ShadCN UI, and Vercel AI SDK
          </p>
          <div className="flex justify-center gap-4 text-sm text-[hsl(var(--gunmetal))]/60">
            <span>• Production Ready</span>
            <span>• Real-time Analytics</span>
            <span>• A/B Testing</span>
            <span>• Child Safe</span>
          </div>
        </div>
      </div>
    </div>
  );
}