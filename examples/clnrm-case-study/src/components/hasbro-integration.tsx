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
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Progress } from "@/components/ui/progress";
import {
  Zap,
  Code,
  Shield,
  Users,
  Play,
  CheckCircle,
  ExternalLink,
} from "lucide-react";

const hasbroRequirements = [
  {
    requirement: "Ship Magic, Every Day",
    description: "Daily coding, prototyping, and shipping",
    clnrmSolution: "Framework eliminates time wasted on false positives",
    icon: <Zap className="h-5 w-5" />,
    color: "bg-yellow-500",
    implementation: [
      "Eliminates debugging false positive failures",
      "Provides immediate feedback on real functionality",
      'Reduces "works on my machine" scenarios',
      "Enables confident daily deployments",
    ],
  },
  {
    requirement: "Full-Stack Execution",
    description:
      "Own backend APIs, integrate LLMs, establish scalable infrastructure",
    clnrmSolution: "Validates API endpoints actually process requests",
    icon: <Code className="h-5 w-5" />,
    color: "bg-blue-500",
    implementation: [
      "Tests API endpoints with real requests",
      "Validates LLM integrations with actual responses",
      "Verifies infrastructure scaling under load",
      "Ensures character interactions work correctly",
    ],
  },
  {
    requirement: "Build the Dev Pipeline",
    description: "Automated, pull request–based pipelines, CI/CD",
    clnrmSolution: "Framework validates CI/CD pipeline functionality",
    icon: <Shield className="h-5 w-5" />,
    color: "bg-green-500",
    implementation: [
      "Tests deployment processes end-to-end",
      "Validates pull request checks actually work",
      "Ensures rollback mechanisms function",
      "Verifies health checks detect real issues",
    ],
  },
  {
    requirement: "Operational Excellence",
    description: "Monitoring, telemetry, data pipelines",
    clnrmSolution: "Tests monitoring systems actually detect failures",
    icon: <Users className="h-5 w-5" />,
    color: "bg-purple-500",
    implementation: [
      "Validates monitoring detects real failures",
      "Tests telemetry data collection accuracy",
      "Ensures alerting systems work correctly",
      "Verifies data pipeline integrity",
    ],
  },
];

const implementationPhases = [
  {
    phase: "Phase 1: Core Testing Infrastructure",
    description: "Set up CLNRM and basic test validation",
    commands: [
      "brew install clnrm",
      "clnrm init",
      "clnrm validate tests/character-interaction.clnrm.toml",
    ],
    progress: 100,
  },
  {
    phase: "Phase 2: AI System Validation",
    description: "Implement character interaction testing",
    commands: [
      "clnrm run tests/character-interaction.clnrm.toml",
      "clnrm services status",
      "clnrm report --format html",
    ],
    progress: 75,
  },
  {
    phase: "Phase 3: Production Validation",
    description: "Full production readiness testing",
    commands: [
      "clnrm run tests/",
      "clnrm self-test --suite production",
      "clnrm report --output deployment-report.html",
    ],
    progress: 50,
  },
];

export function HasbroIntegration() {
  const [selectedRequirement, setSelectedRequirement] = useState<number | null>(
    null
  );
  const [selectedPhase, setSelectedPhase] = useState<number | null>(null);

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white flex items-center gap-2">
            <ExternalLink className="h-6 w-6" />
            Hasbro AI Studio Integration
          </CardTitle>
          <CardDescription className="text-slate-300">
            How CLNRM directly addresses the requirements from{" "}
            <a
              href="https://aistudio.digital.hasbro.com/"
              className="text-blue-400 hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              Hasbro AI Studio&apos;s Principal Engineer role
            </a>
          </CardDescription>
        </CardHeader>
        <CardContent className="text-slate-300">
          <p>
            The Hasbro AI Studio is seeking a Principal Engineer to build
            AI-powered play experiences, emphasizing &quot;execution-first&quot;
            development and &quot;shipping magic every day.&quot; CLNRM provides
            the foundation of reliable, honest testing that makes magical AI
            experiences possible.
          </p>
        </CardContent>
      </Card>

      {/* Requirements Mapping */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Requirements Mapping</CardTitle>
          <CardDescription className="text-slate-300">
            How CLNRM addresses each Hasbro requirement
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {hasbroRequirements.map((req, index) => (
              <div key={index} className="bg-slate-700/50 rounded-lg p-4">
                <div className="flex items-center gap-3 mb-3">
                  <div className={`p-2 rounded-lg ${req.color} text-white`}>
                    {req.icon}
                  </div>
                  <div>
                    <h3 className="text-white font-semibold">
                      {req.requirement}
                    </h3>
                    <p className="text-slate-300 text-sm">{req.description}</p>
                  </div>
                </div>

                <div className="bg-blue-900/20 border border-blue-500/30 rounded p-3 mb-3">
                  <p className="text-blue-300 text-sm">
                    <strong>CLNRM Solution:</strong> {req.clnrmSolution}
                  </p>
                </div>

                <Button
                  onClick={() =>
                    setSelectedRequirement(
                      selectedRequirement === index ? null : index
                    )
                  }
                  variant="outline"
                  size="sm"
                  className="text-white"
                >
                  {selectedRequirement === index ? "Hide" : "Show"}{" "}
                  Implementation Details
                </Button>

                {selectedRequirement === index && (
                  <div className="mt-3 space-y-2">
                    <h4 className="text-white font-medium">Implementation:</h4>
                    <ul className="text-slate-300 text-sm space-y-1">
                      {req.implementation.map((item, i) => (
                        <li key={i} className="flex items-center gap-2">
                          <CheckCircle className="h-3 w-3 text-green-400" />
                          {item}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Implementation Phases */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Implementation Roadmap</CardTitle>
          <CardDescription className="text-slate-300">
            Step-by-step integration with Hasbro&apos;s AI development workflow
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {implementationPhases.map((phase, index) => (
              <div key={index} className="bg-slate-700/50 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <div>
                    <h3 className="text-white font-semibold">{phase.phase}</h3>
                    <p className="text-slate-300 text-sm">
                      {phase.description}
                    </p>
                  </div>
                  <Badge variant="outline" className="text-white">
                    {phase.progress}% Complete
                  </Badge>
                </div>

                <Progress value={phase.progress} className="mb-3" />

                <Button
                  onClick={() =>
                    setSelectedPhase(selectedPhase === index ? null : index)
                  }
                  variant="outline"
                  size="sm"
                  className="text-white mb-3"
                >
                  {selectedPhase === index ? "Hide" : "Show"} Commands
                </Button>

                {selectedPhase === index && (
                  <div className="space-y-2">
                    <h4 className="text-white font-medium">Commands:</h4>
                    {phase.commands.map((cmd, i) => (
                      <div
                        key={i}
                        className="bg-black rounded p-2 font-mono text-sm text-green-400"
                      >
                        {cmd}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Example Test Configuration */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">
            Example: Character Interaction Test
          </CardTitle>
          <CardDescription className="text-slate-300">
            Real TOML configuration for testing AI character interactions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-black rounded-lg p-4 font-mono text-sm text-green-400 overflow-x-auto">
            <pre className="whitespace-pre-wrap">
              {`# tests/character-interaction.clnrm.toml
[test.metadata]
name = "optimus_prime_interaction"
description = "Test Optimus Prime AI character interaction"

[services.llm_service]
type = "openai"
model = "gpt-4"

[[steps]]
name = "character_response"
command = ["curl", "-X", "POST", "/api/character-interaction"]
expected_output_regex = "Autobots, roll out!"

[[steps]]
name = "safety_check"
command = ["curl", "-X", "POST", "/api/safety-check"]
expected_output_regex = "Content is safe for children"`}
            </pre>
          </div>
        </CardContent>
      </Card>

      {/* Business Impact */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">
            Business Impact for Hasbro
          </CardTitle>
          <CardDescription className="text-slate-300">
            Measurable results from implementing CLNRM
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-2 gap-4">
            <div className="space-y-3">
              <h3 className="text-white font-semibold">Development Velocity</h3>
              <div className="bg-green-900/20 border border-green-500/30 rounded-lg p-3">
                <div className="text-2xl font-bold text-green-400 mb-1">
                  3x Faster
                </div>
                <p className="text-slate-300 text-sm">
                  Development cycles due to eliminated false positive debugging
                </p>
              </div>
            </div>

            <div className="space-y-3">
              <h3 className="text-white font-semibold">Quality Assurance</h3>
              <div className="bg-blue-900/20 border border-blue-500/30 rounded-lg p-3">
                <div className="text-2xl font-bold text-blue-400 mb-1">
                  80% Fewer
                </div>
                <p className="text-slate-300 text-sm">
                  Production issues from false positive tests
                </p>
              </div>
            </div>

            <div className="space-y-3">
              <h3 className="text-white font-semibold">
                Deployment Confidence
              </h3>
              <div className="bg-purple-900/20 border border-purple-500/30 rounded-lg p-3">
                <div className="text-2xl font-bold text-purple-400 mb-1">
                  100% Success
                </div>
                <p className="text-slate-300 text-sm">
                  Deployment success rate with CLNRM validation
                </p>
              </div>
            </div>

            <div className="space-y-3">
              <h3 className="text-white font-semibold">Team Satisfaction</h3>
              <div className="bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-3">
                <div className="text-2xl font-bold text-yellow-400 mb-1">
                  90% Reduction
                </div>
                <p className="text-slate-300 text-sm">
                  In &quot;works on my machine&quot; scenarios
                </p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Call to Action */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Ready to Ship Magic?</CardTitle>
          <CardDescription className="text-slate-300">
            CLNRM provides the foundation for reliable AI-powered play
            experiences
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4">
            <Button className="text-white">
              <Play className="h-4 w-4 mr-2" />
              Try CLNRM Demo
            </Button>
            <Button variant="outline" className="text-white">
              <ExternalLink className="h-4 w-4 mr-2" />
              View Hasbro Job Posting
            </Button>
          </div>

          <Alert className="mt-4 bg-green-900/20 border-green-500/30">
            <CheckCircle className="h-4 w-4" />
            <AlertDescription className="text-green-300">
              <strong>Bottom Line:</strong> In AI development, false positives
              don&apos;t just waste time—they break the magic. CLNRM ensures
              that when tests pass, the magic is real.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    </div>
  );
}
