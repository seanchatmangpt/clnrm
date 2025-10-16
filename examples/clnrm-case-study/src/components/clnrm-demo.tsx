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
import { Play, Terminal, CheckCircle, XCircle, Loader2 } from "lucide-react";

interface CommandResult {
  command: string;
  output: string;
  success: boolean;
  duration: number;
}

const demoCommands = [
  {
    command: "clnrm plugins",
    description: "List available service plugins",
    beforeOutput:
      "📦 Available Service Plugins:\n✅ generic_container (alpine, ubuntu, debian)\n✅ surreal_db (database integration)\n✅ network_tools (curl, wget, netcat)",
    afterOutput:
      "📦 Available Service Plugins:\n✅ generic_container (alpine, ubuntu, debian)\n✅ surreal_db (database integration)\n✅ network_tools (curl, wget, netcat)\n\n🔧 Plugin Capabilities:\n  • Container lifecycle management\n  • Service health monitoring\n  • Network connectivity testing\n  • Database integration testing\n  • Custom service plugins",
    success: true,
  },
  {
    command: "clnrm services status",
    description: "Check service status",
    beforeOutput:
      "📊 Service Status:\n✅ No services currently running\n💡 Run 'clnrm run <test_file>' to start services",
    afterOutput:
      "📊 Service Status:\n✅ No services currently running\n💡 Run 'clnrm run <test_file>' to start services\n\n🔍 Service Registry Status:\n  • Environment: Active\n  • Health Checks: Enabled\n  • Logging: Configured",
    success: true,
  },
  {
    command: "clnrm report --format html",
    description: "Generate test report",
    beforeOutput: "Report generation not implemented",
    afterOutput:
      "<!DOCTYPE html>\n<html>\n<head>\n<title>Cleanroom Test Report</title>\n</head>\n<body>\n<h1>Cleanroom Test Report</h1>\n<p><strong>Total Tests:</strong> 5</p>\n<p><strong>Passed:</strong> 5</p>\n<p><strong>Failed:</strong> 0</p>\n<p><strong>Duration:</strong> 2ms</p>\n</body>\n</html>",
    success: true,
  },
  {
    command: "clnrm self-test",
    description: "Run framework self-tests",
    beforeOutput:
      "Framework Self-Test Results:\nTotal Tests: 5\nPassed: 4\nFailed: 1\nDuration: 1ms\n\n❌ test_container_lifecycle (0ms)\n   Error: Container backend not available",
    afterOutput:
      "Framework Self-Test Results:\nTotal Tests: 5\nPassed: 5\nFailed: 0\nDuration: 1ms\n\n✅ validate_framework (0ms)\n✅ test_container_lifecycle (0ms)\n✅ test_plugin_system (0ms)\n✅ test_cli_functionality (1ms)\n✅ test_otel_integration (0ms)",
    success: true,
  },
  {
    command: "clnrm validate tests/basic.clnrm.toml",
    description: "Validate test configuration",
    beforeOutput: "Error: TOML parse error: missing field `test`",
    afterOutput:
      "✅ Configuration valid: basic_test (2 steps, 1 services)\n✅ All validation checks passed\n✅ Ready for test execution",
    success: true,
  },
  {
    command: "clnrm run tests/basic.clnrm.toml",
    description: "Execute test suite",
    beforeOutput:
      "Error: Test execution failed\nCannot parse test configuration",
    afterOutput:
      "🚀 Executing test: basic_test\n📝 Description: Basic integration test\n📋 Step 1: hello_world\n🔧 Executing: echo Hello from cleanroom!\n📤 Output: Hello from cleanroom!\n✅ Step 'hello_world' completed successfully\n🎉 Test 'basic_test' completed successfully!\nTest Results: 1 passed, 0 failed",
    success: true,
  },
];

export function ClnrmDemo() {
  const [selectedCommand, setSelectedCommand] = useState<number | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [showBefore, setShowBefore] = useState(true);
  const [results, setResults] = useState<CommandResult[]>([]);

  const runCommand = async (commandIndex: number) => {
    setIsRunning(true);
    setSelectedCommand(commandIndex);

    const command = demoCommands[commandIndex];
    const output = showBefore ? command.beforeOutput : command.afterOutput;

    // Simulate command execution
    await new Promise((resolve) => setTimeout(resolve, 1500));

    const result: CommandResult = {
      command: command.command,
      output,
      success: command.success,
      duration: Math.floor(Math.random() * 100) + 50,
    };

    setResults((prev) => [
      ...prev.filter((r) => r.command !== command.command),
      result,
    ]);
    setIsRunning(false);
  };

  const runAllCommands = async () => {
    setIsRunning(true);
    setResults([]);

    for (let i = 0; i < demoCommands.length; i++) {
      await runCommand(i);
      await new Promise((resolve) => setTimeout(resolve, 500));
    }

    setIsRunning(false);
  };

  return (
    <div className="space-y-6">
      {/* Demo Controls */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white flex items-center gap-2">
            <Terminal className="h-6 w-6" />
            Interactive CLI Demo
          </CardTitle>
          <CardDescription className="text-slate-300">
            See the difference between false positives and real functionality
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 mb-6">
            <Button
              onClick={() => setShowBefore(!showBefore)}
              variant={showBefore ? "destructive" : "default"}
              className="text-white"
            >
              {showBefore ? "Show False Positives" : "Show Real Implementation"}
            </Button>
            <Button
              onClick={runAllCommands}
              disabled={isRunning}
              className="text-white"
            >
              {isRunning ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Running All Commands...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4 mr-2" />
                  Run All Commands
                </>
              )}
            </Button>
          </div>

          <Alert className="bg-blue-900/20 border-blue-500/30">
            <AlertDescription className="text-blue-300">
              {showBefore
                ? "Showing false positives - commands appear to work but accomplish nothing"
                : "Showing real implementation - commands actually do meaningful work"}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Command List */}
      <div className="grid md:grid-cols-2 gap-4">
        {demoCommands.map((cmd, index) => (
          <Card key={index} className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white text-sm font-mono">
                {cmd.command}
              </CardTitle>
              <CardDescription className="text-slate-300 text-xs">
                {cmd.description}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button
                onClick={() => runCommand(index)}
                disabled={isRunning}
                size="sm"
                className="w-full mb-3"
              >
                {isRunning && selectedCommand === index ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Running...
                  </>
                ) : (
                  <>
                    <Play className="h-4 w-4 mr-2" />
                    Run Command
                  </>
                )}
              </Button>

              {results.find((r) => r.command === cmd.command) && (
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Badge variant={cmd.success ? "default" : "destructive"}>
                      {cmd.success ? (
                        <CheckCircle className="h-3 w-3 mr-1" />
                      ) : (
                        <XCircle className="h-3 w-3 mr-1" />
                      )}
                      {cmd.success ? "Success" : "Failed"}
                    </Badge>
                    <span className="text-slate-400 text-xs">
                      {results.find((r) => r.command === cmd.command)?.duration}
                      ms
                    </span>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Output Display */}
      {selectedCommand !== null &&
        results.find(
          (r) => r.command === demoCommands[selectedCommand].command
        ) && (
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Terminal className="h-5 w-5" />
                Command Output
              </CardTitle>
              <CardDescription className="text-slate-300">
                {demoCommands[selectedCommand].command}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-black rounded-lg p-4 font-mono text-sm text-green-400 overflow-x-auto">
                <pre className="whitespace-pre-wrap">
                  {
                    results.find(
                      (r) => r.command === demoCommands[selectedCommand].command
                    )?.output
                  }
                </pre>
              </div>
            </CardContent>
          </Card>
        )}

      {/* Summary */}
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Demo Summary</CardTitle>
          <CardDescription className="text-slate-300">
            Key differences between false positives and real implementation
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-2 gap-4">
            <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-4">
              <h3 className="text-red-400 font-semibold mb-2 flex items-center gap-2">
                <XCircle className="h-4 w-4" />
                False Positives
              </h3>
              <ul className="text-slate-300 text-sm space-y-1">
                <li>• Commands return success but do nothing</li>
                <li>• &quot;Not implemented&quot; messages</li>
                <li>• Fake status updates</li>
                <li>• Broken test execution</li>
                <li>• No real validation</li>
              </ul>
            </div>
            <div className="bg-green-900/20 border border-green-500/30 rounded-lg p-4">
              <h3 className="text-green-400 font-semibold mb-2 flex items-center gap-2">
                <CheckCircle className="h-4 w-4" />
                Real Implementation
              </h3>
              <ul className="text-slate-300 text-sm space-y-1">
                <li>• Commands actually execute and validate</li>
                <li>• Real report generation</li>
                <li>• Actual service management</li>
                <li>• Working test execution</li>
                <li>• Comprehensive validation</li>
              </ul>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
