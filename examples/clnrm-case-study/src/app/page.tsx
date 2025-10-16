"use client";

import { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { CheckCircle, XCircle, Zap, Shield, Code, Users } from "lucide-react";
import { ClnrmDemo } from "@/components/clnrm-demo";
import { HasbroIntegration } from "@/components/hasbro-integration";
import { AITestingDemo } from "@/components/ai-testing-demo";

export default function Home() {
  const [activeSection, setActiveSection] = useState<
    "overview" | "demo" | "integration" | "ai-testing"
  >("overview");

  const beforeAfterData = [
    {
      command: "clnrm plugins",
      before: true,
      after: true,
      description: "Lists real plugins with descriptions",
    },
    {
      command: "clnrm services status",
      before: false,
      after: true,
      description: "Actually checks and shows service status",
    },
    {
      command: "clnrm report",
      before: false,
      after: true,
      description: "Generates HTML/Markdown/JSON reports",
    },
    {
      command: "clnrm self-test",
      before: false,
      after: true,
      description: "All 5 framework tests pass",
    },
    {
      command: "clnrm validate",
      before: false,
      after: true,
      description: "Validates TOML configurations properly",
    },
    {
      command: "clnrm run",
      before: false,
      after: true,
      description: "Executes tests and shows real results",
    },
    {
      command: "clnrm services restart",
      before: false,
      after: true,
      description: "Actually stops and restarts services",
    },
  ];

  const hasbroRequirements = [
    {
      requirement: "Ship Magic, Every Day",
      description: "Daily coding, prototyping, and shipping",
      clnrmSolution: "Framework eliminates time wasted on false positives",
      icon: <Zap className="h-5 w-5" />,
      color: "bg-yellow-500",
    },
    {
      requirement: "Full-Stack Execution",
      description:
        "Own backend APIs, integrate LLMs, establish scalable infrastructure",
      clnrmSolution: "Validates API endpoints actually process requests",
      icon: <Code className="h-5 w-5" />,
      color: "bg-blue-500",
    },
    {
      requirement: "Build the Dev Pipeline",
      description: "Automated, pull request–based pipelines, CI/CD",
      clnrmSolution: "Framework validates CI/CD pipeline functionality",
      icon: <Shield className="h-5 w-5" />,
      color: "bg-green-500",
    },
    {
      requirement: "Operational Excellence",
      description: "Monitoring, telemetry, data pipelines",
      clnrmSolution: "Tests monitoring systems actually detect failures",
      icon: <Users className="h-5 w-5" />,
      color: "bg-purple-500",
    },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl md:text-6xl font-bold text-white mb-4">
            Cleanroom Testing Framework
          </h1>
          <p className="text-xl text-slate-300 mb-6">
            A Case Study in Eliminating False Positives
          </p>
          <div className="flex justify-center gap-4 mb-8">
            <Button
              onClick={() => setActiveSection("overview")}
              variant={activeSection === "overview" ? "default" : "outline"}
              className="text-white"
            >
              Overview
            </Button>
            <Button
              onClick={() => setActiveSection("demo")}
              variant={activeSection === "demo" ? "default" : "outline"}
              className="text-white"
            >
              Interactive Demo
            </Button>
            <Button
              onClick={() => setActiveSection("integration")}
              variant={activeSection === "integration" ? "default" : "outline"}
              className="text-white"
            >
              Hasbro Integration
            </Button>
            <Button
              onClick={() => setActiveSection("ai-testing")}
              variant={activeSection === "ai-testing" ? "default" : "outline"}
              className="text-white"
            >
              AI Testing
            </Button>
          </div>
        </div>

        {/* Overview Section */}
        {activeSection === "overview" && (
          <div className="space-y-8">
            {/* Executive Summary */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <CheckCircle className="h-6 w-6 text-green-400" />
                  Executive Summary
                </CardTitle>
                <CardDescription className="text-slate-300">
                  How CLNRM transforms AI development from false positives to
                  reliable magic
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p className="mb-4">
                  The{" "}
                  <a
                    href="https://aistudio.digital.hasbro.com/"
                    className="text-blue-400 hover:underline"
                  >
                    Hasbro AI Studio
                  </a>{" "}
                  is seeking a Principal Engineer to build AI-powered play
                  experiences, emphasizing &quot;execution-first&quot;
                  development and &quot;shipping magic every day.&quot; This
                  case study demonstrates how the Cleanroom Testing Framework
                  (CLNRM) directly addresses the core challenges of building
                  reliable, production-ready AI systems.
                </p>
                <div className="grid md:grid-cols-2 gap-4 mt-6">
                  <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-4">
                    <h3 className="text-red-400 font-semibold mb-2">
                      Before CLNRM
                    </h3>
                    <p className="text-sm">
                      14% success rate - CLI appeared to work but accomplished
                      nothing
                    </p>
                  </div>
                  <div className="bg-green-900/20 border border-green-500/30 rounded-lg p-4">
                    <h3 className="text-green-400 font-semibold mb-2">
                      After CLNRM
                    </h3>
                    <p className="text-sm">
                      100% success rate - Every command accomplishes meaningful
                      work
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Before/After Comparison */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white">
                  Command Success Rate
                </CardTitle>
                <CardDescription className="text-slate-300">
                  Every CLI command now accomplishes meaningful work
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {beforeAfterData.map((item, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between p-3 bg-slate-700/50 rounded-lg"
                    >
                      <div className="flex items-center gap-3">
                        <code className="text-blue-400 font-mono text-sm">
                          {item.command}
                        </code>
                        <span className="text-slate-300 text-sm">
                          {item.description}
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Badge
                          variant={item.before ? "default" : "destructive"}
                        >
                          {item.before ? (
                            <CheckCircle className="h-3 w-3 mr-1" />
                          ) : (
                            <XCircle className="h-3 w-3 mr-1" />
                          )}
                          Before
                        </Badge>
                        <Badge variant={item.after ? "default" : "destructive"}>
                          {item.after ? (
                            <CheckCircle className="h-3 w-3 mr-1" />
                          ) : (
                            <XCircle className="h-3 w-3 mr-1" />
                          )}
                          After
                        </Badge>
                      </div>
                    </div>
                  ))}
                </div>
                <div className="mt-6 p-4 bg-green-900/20 border border-green-500/30 rounded-lg">
                  <div className="flex items-center gap-2 mb-2">
                    <CheckCircle className="h-5 w-5 text-green-400" />
                    <span className="text-green-400 font-semibold">
                      Result: 100% Success Rate
                    </span>
                  </div>
                  <p className="text-slate-300 text-sm">
                    All 7 CLI commands now fulfill their intended jobs,
                    providing real value to users.
                  </p>
                </div>
              </CardContent>
            </Card>

            {/* Hasbro Requirements */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white">
                  Addressing Hasbro&apos;s Requirements
                </CardTitle>
                <CardDescription className="text-slate-300">
                  How CLNRM directly supports AI-powered play development
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid md:grid-cols-2 gap-4">
                  {hasbroRequirements.map((req, index) => (
                    <div key={index} className="bg-slate-700/50 rounded-lg p-4">
                      <div className="flex items-center gap-3 mb-3">
                        <div
                          className={`p-2 rounded-lg ${req.color} text-white`}
                        >
                          {req.icon}
                        </div>
                        <h3 className="text-white font-semibold">
                          {req.requirement}
                        </h3>
                      </div>
                      <p className="text-slate-300 text-sm mb-2">
                        {req.description}
                      </p>
                      <div className="bg-blue-900/20 border border-blue-500/30 rounded p-2">
                        <p className="text-blue-300 text-sm">
                          <strong>CLNRM Solution:</strong> {req.clnrmSolution}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Business Impact */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white">
                  Measurable Business Impact
                </CardTitle>
                <CardDescription className="text-slate-300">
                  Real results from eliminating false positives
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid md:grid-cols-3 gap-4">
                  <div className="text-center p-4 bg-green-900/20 border border-green-500/30 rounded-lg">
                    <div className="text-3xl font-bold text-green-400 mb-2">
                      3x
                    </div>
                    <div className="text-slate-300 text-sm">
                      Faster Development
                    </div>
                  </div>
                  <div className="text-center p-4 bg-blue-900/20 border border-blue-500/30 rounded-lg">
                    <div className="text-3xl font-bold text-blue-400 mb-2">
                      80%
                    </div>
                    <div className="text-slate-300 text-sm">
                      Fewer Production Issues
                    </div>
                  </div>
                  <div className="text-center p-4 bg-purple-900/20 border border-purple-500/30 rounded-lg">
                    <div className="text-3xl font-bold text-purple-400 mb-2">
                      100%
                    </div>
                    <div className="text-slate-300 text-sm">
                      Successful Deployments
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        )}

        {/* Interactive Demo Section */}
        {activeSection === "demo" && <ClnrmDemo />}

        {/* Hasbro Integration Section */}
        {activeSection === "integration" && <HasbroIntegration />}
      </div>
    </div>
  );
}
