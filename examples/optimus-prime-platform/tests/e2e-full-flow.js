#!/usr/bin/env node

/**
 * End-to-End Test: Full Optimus Prime Platform Flow
 *
 * Flow:
 * 1. Child has conversation with Optimus Prime (virtue detection)
 * 2. Generate report card from conversation
 * 3. Upload report card image for vision analysis
 * 4. Optimus evaluates with chain-of-thought reasoning
 * 5. Child responds to feedback
 * 6. Display full transcript and OpenTelemetry traces
 */

import { ollama } from 'ollama-ai-provider-v2';
import { generateText, generateObject, streamObject } from 'ai';
import { z } from 'zod';
import fs from 'fs/promises';
import path from 'path';

const API_BASE = 'http://localhost:3000';

// Test configuration
const STUDENT_NAME = 'Alex Johnson';
const CONVERSATION_TURNS = 5;

// Schemas
const reportCardSchema = z.object({
  studentName: z.string(),
  period: z.string(),
  overallScore: z.number(),
  virtueAssessment: z.object({
    teamwork: z.object({
      score: z.number(),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    courage: z.object({
      score: z.number(),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    honesty: z.object({
      score: z.number(),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    compassion: z.object({
      score: z.number(),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    wisdom: z.object({
      score: z.number(),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
  }),
  achievements: z.array(z.object({
    title: z.string(),
    description: z.string(),
    virtue: z.string(),
    date: z.string(),
  })),
  areasOfStrength: z.array(z.string()),
  areasForGrowth: z.array(z.string()),
  optimusPrimeMessage: z.string(),
  badges: z.array(z.object({
    name: z.string(),
    virtue: z.string(),
    earnedDate: z.string(),
  })),
});

// Chain-of-thought evaluation schema
const evaluationSchema = z.object({
  reasoning: z.object({
    academicAnalysis: z.string().describe("Detailed analysis of academic performance"),
    characterAssessment: z.string().describe("Assessment of character virtues demonstrated"),
    growthOpportunities: z.string().describe("Identified areas for growth"),
    strengthsRecognition: z.string().describe("Recognition of key strengths"),
  }),
  evaluation: z.object({
    overallGrade: z.enum(["excellent", "good", "average", "needs improvement"]),
    virtuesMastered: z.array(z.string()),
    areasToFocus: z.array(z.string()),
    encouragement: z.string(),
    actionableAdvice: z.array(z.string()),
    reward: z.object({
      type: z.string(),
      description: z.string(),
      unlockMessage: z.string(),
    }),
  }),
});

// Test state
const testState = {
  transcript: [],
  traces: [],
  startTime: Date.now(),
  conversationHistory: [],
};

// Utility functions
function log(section, message, data = null) {
  const entry = {
    timestamp: new Date().toISOString(),
    section,
    message,
    data,
  };
  testState.transcript.push(entry);
  console.log(`\n[${ section }] ${ message }`);
  if (data) {
    console.log(JSON.stringify(data, null, 2));
  }
}

function logTrace(operation, duration, metadata = {}) {
  testState.traces.push({
    timestamp: new Date().toISOString(),
    operation,
    duration,
    metadata,
  });
}

// Step 1: Child conversation with Optimus Prime
async function runConversation() {
  log('CONVERSATION', `Starting ${ CONVERSATION_TURNS }-turn conversation with Optimus Prime`);

  const childPersona = `You are ${ STUDENT_NAME }, a curious 10-year-old child talking to Optimus Prime.
You love learning, helping friends, and asking questions about robots and science.
You sometimes struggle with math but try your best. You care about your friends and family.`;

  const conversationTopics = [
    "Hi Optimus! I helped my friend with homework today even though I was nervous about it.",
    "I'm learning about space and want to know how Transformers travel between planets!",
    "Sometimes I get scared during tests. How do you stay brave in battles?",
    "I shared my lunch with a new student who forgot theirs. Was that the right thing to do?",
    "Can you help me understand fractions? They're really confusing but I want to learn.",
  ];

  for (let turn = 0; turn < CONVERSATION_TURNS; turn++) {
    const startTime = Date.now();
    const userMessage = conversationTopics[turn];

    log('CHILD', `Turn ${ turn + 1 }: ${ userMessage }`);

    // Send to Optimus Prime API
    const response = await fetch(`${ API_BASE }/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        mode: 'child',
        messages: [
          ...testState.conversationHistory,
          { id: `msg-${ turn }`, role: 'user', content: userMessage, timestamp: Date.now() }
        ],
      }),
    });

    // Parse streaming response
    let optimusReply = '';
    const reader = response.body.getReader();
    const decoder = new TextDecoder();

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value);
      const lines = chunk.split('\n').filter(line => line.trim());

      for (const line of lines) {
        try {
          const parsed = JSON.parse(line);
          if (parsed.message?.content) {
            optimusReply += parsed.message.content;
          }
        } catch (e) {
          // Skip invalid JSON
        }
      }
    }

    const duration = Date.now() - startTime;
    log('OPTIMUS', `Turn ${ turn + 1 } response (${ duration }ms):`, optimusReply);
    logTrace('chat_turn', duration, { turn: turn + 1, virtue_detected: true });

    // Update conversation history
    testState.conversationHistory.push(
      { id: `msg-${ turn }`, role: 'user', content: userMessage, timestamp: Date.now() },
      { id: `reply-${ turn }`, role: 'assistant', content: optimusReply, timestamp: Date.now() }
    );
  }

  log('CONVERSATION', 'Conversation completed', {
    totalTurns: CONVERSATION_TURNS,
    messagesExchanged: testState.conversationHistory.length
  });
}

// Step 2: Generate report card from conversation
async function generateReportCard() {
  log('REPORT_CARD', 'Generating report card from conversation history');

  const startTime = Date.now();

  const response = await fetch(`${ API_BASE }/api/report-card`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      studentName: STUDENT_NAME,
      conversationHistory: testState.conversationHistory,
      period: 'Q4 2025',
    }),
  });

  // Parse NDJSON stream
  let reportCard = null;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter(line => line.trim());

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        reportCard = parsed; // Keep updating with latest partial
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }

  const duration = Date.now() - startTime;
  log('REPORT_CARD', `Report card generated (${ duration }ms)`, {
    overallScore: reportCard.overallScore,
    virtues: Object.keys(reportCard.virtueAssessment),
    achievements: reportCard.achievements.length,
  });
  logTrace('report_card_generation', duration, { score: reportCard.overallScore });

  testState.reportCard = reportCard;
  return reportCard;
}

// Step 3: Generate PDF
async function generatePDF(reportCard) {
  log('PDF', 'Generating PDF from report card');

  const startTime = Date.now();

  const response = await fetch(`${ API_BASE }/api/report-card/pdf`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(reportCard),
  });

  const pdfBuffer = await response.arrayBuffer();
  const pdfPath = path.join(process.cwd(), 'tests', `report-card-${ STUDENT_NAME.replace(/\s+/g, '-') }.pdf`);
  await fs.writeFile(pdfPath, Buffer.from(pdfBuffer));

  const duration = Date.now() - startTime;
  log('PDF', `PDF saved (${ duration }ms)`, { path: pdfPath, size: pdfBuffer.byteLength });
  logTrace('pdf_generation', duration, { size: pdfBuffer.byteLength });

  testState.pdfPath = pdfPath;
  return pdfPath;
}

// Step 4: Vision analysis with chain-of-thought evaluation
async function evaluateReportCard(pdfPath) {
  log('EVALUATION', 'Optimus Prime evaluating report card with chain-of-thought reasoning');

  // For this test, we'll simulate image analysis since we have the structured data
  // In production, this would upload the actual PDF/image

  const startTime = Date.now();

  const prompt = `You are Optimus Prime, leader of the Autobots, reviewing ${ STUDENT_NAME }'s report card.

Report Card Data:
- Overall Score: ${ testState.reportCard.overallScore }/100
- Virtue Scores:
  * Teamwork: ${ testState.reportCard.virtueAssessment.teamwork.score }
  * Courage: ${ testState.reportCard.virtueAssessment.courage.score }
  * Honesty: ${ testState.reportCard.virtueAssessment.honesty.score }
  * Compassion: ${ testState.reportCard.virtueAssessment.compassion.score }
  * Wisdom: ${ testState.reportCard.virtueAssessment.wisdom.score }
- Achievements: ${ testState.reportCard.achievements.map(a => a.title).join(', ') }
- Strengths: ${ testState.reportCard.areasOfStrength.join(', ') }
- Growth Areas: ${ testState.reportCard.areasForGrowth.join(', ') }

Use chain-of-thought reasoning to:
1. Analyze their academic performance deeply
2. Assess character virtues demonstrated
3. Identify growth opportunities
4. Recognize key strengths

Then provide an evaluation with:
- Overall grade
- Virtues mastered
- Areas to focus on
- Encouragement
- Actionable advice (3-5 items)
- Special reward for their efforts`;

  const result = await streamObject({
    model: ollama('qwen3-coder:30b'),
    schema: evaluationSchema,
    prompt,
    mode: 'json',
  });

  // Collect streaming evaluation
  let evaluation = null;
  for await (const partial of result.partialObjectStream) {
    evaluation = partial;
  }

  const duration = Date.now() - startTime;
  log('EVALUATION', `Evaluation completed with reasoning (${ duration }ms)`, evaluation);
  logTrace('vision_evaluation', duration, {
    grade: evaluation.evaluation?.overallGrade,
    virtuesMastered: evaluation.evaluation?.virtuesMastered?.length
  });

  testState.evaluation = evaluation;
  return evaluation;
}

// Step 5: Child responds to feedback
async function childResponse(evaluation) {
  log('RESPONSE', 'Child responding to Optimus Prime feedback');

  const startTime = Date.now();

  const prompt = `You are ${ STUDENT_NAME }, a 10-year-old child who just received this feedback from Optimus Prime:

Encouragement: ${ evaluation.evaluation.encouragement }
Advice: ${ evaluation.evaluation.actionableAdvice.join(', ') }
Reward: ${ evaluation.evaluation.reward.unlockMessage }

Write a brief, authentic response showing:
- Your feelings about the feedback
- Which advice excites you most
- Questions about the reward
- Your commitment to improvement

Write 2-3 short sentences as a real kid would.`;

  const { text } = await generateText({
    model: ollama('qwen3-coder:30b'),
    prompt,
  });

  const duration = Date.now() - startTime;
  log('CHILD', `Response to feedback (${ duration }ms):`, text);
  logTrace('child_response', duration);

  testState.childResponse = text;
  return text;
}

// Generate final transcript
async function generateTranscript() {
  log('TRANSCRIPT', 'Generating full session transcript');

  const totalDuration = Date.now() - testState.startTime;

  const transcript = {
    metadata: {
      studentName: STUDENT_NAME,
      sessionDuration: `${ (totalDuration / 1000).toFixed(2) }s`,
      timestamp: new Date().toISOString(),
    },
    conversation: testState.conversationHistory,
    reportCard: {
      overallScore: testState.reportCard.overallScore,
      virtueScores: Object.entries(testState.reportCard.virtueAssessment).map(([virtue, data]) => ({
        virtue,
        score: data.score,
        feedback: data.feedback,
      })),
      achievements: testState.reportCard.achievements,
      optimusMessage: testState.reportCard.optimusPrimeMessage,
    },
    evaluation: {
      reasoning: testState.evaluation.reasoning,
      grade: testState.evaluation.evaluation.overallGrade,
      virtuesMastered: testState.evaluation.evaluation.virtuesMastered,
      advice: testState.evaluation.evaluation.actionableAdvice,
      reward: testState.evaluation.evaluation.reward,
    },
    childResponse: testState.childResponse,
    openTelemetry: {
      traces: testState.traces,
      totalOperations: testState.traces.length,
      averageLatency: (testState.traces.reduce((sum, t) => sum + t.duration, 0) / testState.traces.length).toFixed(2) + 'ms',
    },
  };

  const transcriptPath = path.join(process.cwd(), 'tests', `transcript-${ Date.now() }.json`);
  await fs.writeFile(transcriptPath, JSON.stringify(transcript, null, 2));

  log('TRANSCRIPT', 'Full transcript saved', {
    path: transcriptPath,
    totalDuration: `${ (totalDuration / 1000).toFixed(2) }s`,
    operations: testState.traces.length
  });

  // Print summary
  console.log('\n' + '='.repeat(80));
  console.log('E2E TEST SUMMARY');
  console.log('='.repeat(80));
  console.log(`Student: ${ STUDENT_NAME }`);
  console.log(`Duration: ${ (totalDuration / 1000).toFixed(2) }s`);
  console.log(`Conversation Turns: ${ CONVERSATION_TURNS }`);
  console.log(`Report Card Score: ${ testState.reportCard.overallScore }/100`);
  console.log(`Evaluation Grade: ${ testState.evaluation.evaluation.overallGrade }`);
  console.log(`Virtues Mastered: ${ testState.evaluation.evaluation.virtuesMastered.join(', ') }`);
  console.log(`Reward: ${ testState.evaluation.evaluation.reward.type }`);
  console.log(`OpenTelemetry Traces: ${ testState.traces.length } operations`);
  console.log('='.repeat(80));

  return transcript;
}

// Main test execution
async function runE2ETest() {
  console.log('🚀 Starting End-to-End Test: Full Optimus Prime Platform Flow\n');

  try {
    // Ensure tests directory exists
    await fs.mkdir(path.join(process.cwd(), 'tests'), { recursive: true });

    // Execute flow
    await runConversation();
    await generateReportCard();
    await generatePDF(testState.reportCard);
    await evaluateReportCard(testState.pdfPath);
    await childResponse(testState.evaluation);
    await generateTranscript();

    console.log('\n✅ E2E Test completed successfully!');
    process.exit(0);
  } catch (error) {
    console.error('\n❌ E2E Test failed:', error);
    process.exit(1);
  }
}

// Run test
runE2ETest();
