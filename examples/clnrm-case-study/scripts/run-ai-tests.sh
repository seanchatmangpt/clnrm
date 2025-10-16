#!/bin/bash

# CLNRM AI Testing Script
# Runs comprehensive AI tests through CLNRM framework

set -e

echo "🚀 Starting CLNRM AI Testing Suite"
echo "=================================="

# Check if CLNRM is installed
if ! command -v clnrm &> /dev/null; then
    echo "❌ CLNRM not found. Please install CLNRM first:"
    echo "   brew install clnrm"
    exit 1
fi

# Check if OpenAI API key is set
if [ -z "$OPENAI_API_KEY" ]; then
    echo "❌ OPENAI_API_KEY environment variable not set"
    echo "   Please set your OpenAI API key:"
    echo "   export OPENAI_API_KEY=your_api_key_here"
    exit 1
fi

# Install required dependencies
echo "📦 Installing AI dependencies..."
npm install ollama-ai-provider

# Create tests directory if it doesn't exist
mkdir -p tests

echo ""
echo "🧪 Running AI Integration Tests"
echo "==============================="

# Test 1: Basic AI Integration
echo "1️⃣ Testing Vercel AI SDK Integration..."
if clnrm run tests/vercel-ai-integration.clnrm.toml; then
    echo "✅ Vercel AI SDK Integration: PASSED"
else
    echo "❌ Vercel AI SDK Integration: FAILED"
    exit 1
fi

echo ""
echo "2️⃣ Testing AI Character Interactions..."
if clnrm run tests/ai-character-interaction.clnrm.toml; then
    echo "✅ AI Character Interactions: PASSED"
else
    echo "❌ AI Character Interactions: FAILED"
    exit 1
fi

echo ""
echo "3️⃣ Running AI Performance Benchmarks..."
if clnrm run tests/ai-performance-benchmark.clnrm.toml; then
    echo "✅ AI Performance Benchmarks: PASSED"
else
    echo "❌ AI Performance Benchmarks: FAILED"
    exit 1
fi

echo ""
echo "4️⃣ Validating Production Readiness..."
if clnrm run tests/ai-production-readiness.clnrm.toml; then
    echo "✅ Production Readiness: PASSED"
else
    echo "❌ Production Readiness: FAILED"
    exit 1
fi

echo ""
echo "📊 Generating AI Test Report..."
clnrm report --format html --output ai-test-report.html

echo ""
echo "🎉 All AI Tests Completed Successfully!"
echo "======================================="
echo "📄 Test report generated: ai-test-report.html"
echo "🔍 View detailed results in the generated report"
echo ""
echo "✨ AI system is ready for Hasbro AI Studio integration!"
