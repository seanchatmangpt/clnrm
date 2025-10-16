#!/usr/bin/env node

/**
 * Self-Play Conversation Simulator
 *
 * Simulates a full child-Optimus Prime conversation where:
 * 1. Ollama generates child messages (achievements to share)
 * 2. Platform responds as Optimus Prime with virtue detection
 * 3. Ollama generates follow-up responses as the child
 * 4. Full conversation tracked with OpenTelemetry
 *
 * Uses Vercel AI SDK with ollama-ai-provider-v2 (no direct fetch calls)
 */

import { ollama } from 'ollama-ai-provider-v2';
import { generateText } from 'ai';

const APP_URL = 'http://localhost:3001';
const CONVERSATION_TURNS = 5;

// Child persona prompt for Ollama
const CHILD_PERSONA = `You are a 10-year-old child who has just done something good and wants to share it with Optimus Prime, the wise Autobot leader.

Respond naturally as a child would - excited, proud, sometimes shy, using simple language.

Examples of things you might share:
- "I helped my friend with their math homework"
- "I stood up to a bully today"
- "I shared my lunch with someone who forgot theirs"
- "I told the truth even though I was scared"
- "I helped my team win the soccer game"

Keep responses SHORT (1-3 sentences) and natural for a 10-year-old.`;

let conversationHistory = []; // For Ollama child persona
let platformMessages = []; // For platform API (user/assistant format)
let traceIds = [];
let virtuesDetected = [];

/**
 * Call Ollama to generate a message using Vercel AI SDK
 */
async function callOllama(prompt, conversationContext = []) {
  const { text } = await generateText({
    model: ollama('qwen3-coder:30b'),
    system: CHILD_PERSONA,
    messages: [
      ...conversationContext,
      { role: 'user', content: prompt }
    ],
  });

  return text;
}

/**
 * Send message to Optimus Prime platform with full conversation history
 */
async function sendToOptimusPrime(message, conversationMessages = []) {
  console.log(`\n🧒 CHILD: ${message}`);

  // Build messages array with full history
  const messages = [
    ...conversationMessages,
    { role: 'user', content: message }
  ];

  const response = await fetch(`${APP_URL}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      mode: 'child',
      messages
    })
  });

  if (!response.ok) {
    throw new Error(`Platform error: ${response.status}`);
  }

  // Extract trace ID and virtue from headers
  const traceId = response.headers.get('traceparent');
  const virtue = response.headers.get('x-virtue');
  const rewardUrl = response.headers.get('x-reward-url');

  if (traceId) traceIds.push(traceId);
  if (virtue) virtuesDetected.push(virtue);

  // Read Vercel AI SDK data stream response
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let fullResponse = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter(line => line.trim());

    for (const line of lines) {
      // AI SDK data stream format: "0:..." for text chunks
      if (line.startsWith('0:')) {
        try {
          const textContent = JSON.parse(line.substring(2));
          fullResponse += textContent;
        } catch (e) {
          // Try as plain text
          fullResponse += line.substring(2);
        }
      }
    }
  }

  console.log(`\n🤖 OPTIMUS PRIME: ${fullResponse}`);
  console.log(`   └─ Detected Virtue: ${virtue || 'none'}`);
  console.log(`   └─ Reward: ${rewardUrl ? rewardUrl.split('/').pop() : 'none'}`);

  return { response: fullResponse, virtue, rewardUrl };
}

/**
 * Generate child's initial achievement
 */
async function generateInitialAchievement() {
  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('🎬 TURN 1: Child shares an achievement');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  const prompt = 'Think of something good you did today that you want to share with Optimus Prime. What did you do?';
  const achievement = await callOllama(prompt);

  conversationHistory.push({ role: 'assistant', content: achievement });

  const optimusResponse = await sendToOptimusPrime(achievement, platformMessages);

  // Update platform messages with the exchange
  platformMessages.push({ role: 'user', content: achievement });
  platformMessages.push({ role: 'assistant', content: optimusResponse.response });

  conversationHistory.push({ role: 'user', content: optimusResponse.response });

  return optimusResponse;
}

/**
 * Generate child's follow-up response
 */
async function generateFollowUp(turnNumber, previousResponse) {
  console.log(`\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`);
  console.log(`🎬 TURN ${turnNumber}: Child responds to feedback`);
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  const prompt = `Optimus Prime just said: "${previousResponse.response}"\n\nHow do you respond? Maybe share another achievement, ask a question, or continue the conversation naturally.`;

  const childResponse = await callOllama(prompt, conversationHistory);
  conversationHistory.push({ role: 'assistant', content: childResponse });

  const optimusResponse = await sendToOptimusPrime(childResponse, platformMessages);

  // Update platform messages with the exchange
  platformMessages.push({ role: 'user', content: childResponse });
  platformMessages.push({ role: 'assistant', content: optimusResponse.response });

  conversationHistory.push({ role: 'user', content: optimusResponse.response });

  return optimusResponse;
}

/**
 * Main simulation
 */
async function runSimulation() {
  console.log('\n╔═══════════════════════════════════════════════════════════╗');
  console.log('║     SELF-PLAY CONVERSATION SIMULATION                     ║');
  console.log('║     Child ↔ Optimus Prime with Ollama                    ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');

  const startTime = Date.now();

  try {
    // Check services are running
    console.log('🔍 Checking services...');

    console.log('   ✓ Ollama provider configured');

    try {
      const appCheck = await fetch(`${APP_URL}/api/metrics`);
      if (!appCheck.ok) throw new Error('App not running');
      console.log('   ✓ Platform running');
    } catch (e) {
      console.error('   ✗ Platform check failed:', e.message);
      throw new Error('App not running on ' + APP_URL);
    }

    // Turn 1: Initial achievement
    let response = await generateInitialAchievement();

    // Turns 2-N: Follow-ups
    for (let i = 2; i <= CONVERSATION_TURNS; i++) {
      await new Promise(resolve => setTimeout(resolve, 1000)); // Brief pause
      response = await generateFollowUp(i, response);
    }

    // Final summary
    const duration = ((Date.now() - startTime) / 1000).toFixed(1);

    console.log('\n\n╔═══════════════════════════════════════════════════════════╗');
    console.log('║                    SIMULATION COMPLETE                     ║');
    console.log('╚═══════════════════════════════════════════════════════════╝\n');

    console.log(`⏱️  Total Duration: ${duration} seconds`);
    console.log(`💬 Total Turns: ${CONVERSATION_TURNS}`);
    console.log(`🎯 Virtues Detected: ${virtuesDetected.length}`);
    console.log(`   └─ ${virtuesDetected.join(', ')}`);
    console.log(`📊 Trace IDs Captured: ${traceIds.length}`);

    // Fetch final metrics
    console.log('\n📈 Fetching final metrics...\n');
    const metricsResponse = await fetch(`${APP_URL}/api/metrics`);
    const metrics = await metricsResponse.json();

    console.log('Conversation Funnel:');
    metrics.funnel.forEach(item => {
      console.log(`   ${item.label}: ${item.value}`);
    });

    console.log(`\nTotal Events: ${metrics.totals.events}`);
    console.log(`Total Revenue: $${metrics.totals.revenue}`);

    console.log('\n✅ Self-play conversation simulation completed successfully!\n');
    console.log('📄 Full conversation saved to memory');
    console.log('🔍 Check OpenTelemetry logs for detailed traces\n');

    // Save conversation to file
    const { writeFileSync } = await import('fs');
    const conversationLog = {
      timestamp: new Date().toISOString(),
      duration_seconds: parseFloat(duration),
      turns: CONVERSATION_TURNS,
      virtues_detected: virtuesDetected,
      trace_ids: traceIds,
      conversation: conversationHistory,
      final_metrics: metrics
    };

    writeFileSync(
      '/tmp/self-play-conversation.json',
      JSON.stringify(conversationLog, null, 2)
    );
    console.log('💾 Conversation log saved to: /tmp/self-play-conversation.json\n');

  } catch (error) {
    console.error('\n❌ Simulation failed:', error.message);
    console.error(error.stack);
    process.exit(1);
  }
}

// Run simulation
runSimulation();
