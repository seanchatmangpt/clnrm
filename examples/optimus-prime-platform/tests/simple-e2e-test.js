#!/usr/bin/env node

/**
 * Simplified E2E Test: Optimus Prime Platform Flow with Transcript
 *
 * This test demonstrates:
 * 1. Child conversation with Optimus Prime
 * 2. Report card generation from conversation
 * 3. PDF generation
 * 4. Chain-of-thought evaluation
 * 5. Full transcript with OpenTelemetry traces
 */

import { ollama } from 'ollama-ai-provider-v2';
import { generateText } from 'ai';
import fs from 'fs/promises';
import path from 'path';

const API_BASE = 'http://localhost:3000';
const STUDENT_NAME = 'Alex Johnson';

// Test state with telemetry
const testState = {
  traces: [],
  transcript: [],
  startTime: Date.now(),
};

function logTrace(operation, duration, metadata = {}) {
  const trace = {
    timestamp: new Date().toISOString(),
    operation,
    duration: `${duration}ms`,
    metadata,
  };
  testState.traces.push(trace);
  console.log(`📊 [TRACE] ${operation} - ${duration}ms`);
  return trace;
}

function logSection(section, message, data = null) {
  const entry = {
    timestamp: new Date().toISOString(),
    section,
    message,
    data,
  };
  testState.transcript.push(entry);
  console.log(`\n✨ [${section}] ${message}`);
  if (data && Object.keys(data).length < 5) {
    console.log(JSON.stringify(data, null, 2));
  }
}

// Step 1: Simple conversation test
async function testConversation() {
  logSection('CONVERSATION', 'Testing child-Optimus Prime conversation');
  const startTime = Date.now();

  const messages = [
    {
      id: 'msg-1',
      role: 'user',
      content: 'Hi Optimus! I helped my friend with homework today!',
      timestamp: Date.now(),
    },
  ];

  const response = await fetch(`${API_BASE}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      mode: 'child',
      messages,
    }),
  });

  let fullReply = '';
  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter((line) => line.trim());

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        if (parsed.message?.content) {
          fullReply += parsed.message.content;
        }
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }

  const duration = Date.now() - startTime;
  logTrace('chat_message', duration, { messageLength: fullReply.length });

  logSection('OPTIMUS RESPONSE', 'First message received', {
    length: fullReply.length,
    preview: fullReply.substring(0, 100) + '...',
  });

  return {
    messages: [
      ...messages,
      {
        id: 'reply-1',
        role: 'assistant',
        content: fullReply,
        timestamp: Date.now(),
      },
    ],
    fullReply,
  };
}

// Step 2: Generate report card
async function testReportCard(conversationHistory) {
  logSection('REPORT_CARD', 'Generating report card from conversation');
  const startTime = Date.now();

  const response = await fetch(`${API_BASE}/api/report-card`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      studentName: STUDENT_NAME,
      conversationHistory,
      period: 'Q4 2025',
    }),
  });

  let reportCard = null;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  console.log('📥 Receiving report card stream...');

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter((line) => line.trim());

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        reportCard = parsed;
        console.log('📦 Partial report card received...');
      } catch (e) {
        console.error('Failed to parse:', line.substring(0, 100));
      }
    }
  }

  if (!reportCard) {
    throw new Error('No report card generated');
  }

  const duration = Date.now() - startTime;
  logTrace('report_card_generation', duration, {
    overallScore: reportCard.overallScore,
    virtues: Object.keys(reportCard.virtueAssessment || {}).length,
  });

  logSection('REPORT CARD', 'Report card generated successfully', {
    student: reportCard.studentName,
    score: reportCard.overallScore,
    achievements: reportCard.achievements?.length || 0,
  });

  return reportCard;
}

// Step 3: Generate PDF
async function testPDF(reportCard) {
  logSection('PDF', 'Generating PDF from report card');
  const startTime = Date.now();

  const response = await fetch(`${API_BASE}/api/report-card/pdf`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(reportCard),
  });

  const pdfBuffer = await response.arrayBuffer();
  const pdfPath = path.join(
    process.cwd(),
    'tests',
    `report-card-${STUDENT_NAME.replace(/\s+/g, '-')}.pdf`
  );
  await fs.writeFile(pdfPath, Buffer.from(pdfBuffer));

  const duration = Date.now() - startTime;
  logTrace('pdf_generation', duration, {
    size: `${(pdfBuffer.byteLength / 1024).toFixed(2)}KB`,
  });

  logSection('PDF', 'PDF generated and saved', {
    path: pdfPath,
    size: `${(pdfBuffer.byteLength / 1024).toFixed(2)}KB`,
  });

  return pdfPath;
}

// Step 4: Test evaluation with chain-of-thought (simulated)
async function testEvaluation(reportCard) {
  logSection('EVALUATION', 'Optimus Prime evaluating with chain-of-thought');
  const startTime = Date.now();

  // Simulate analysis data for evaluation
  const mockAnalysis = {
    documentType: 'report card',
    studentName: reportCard.studentName,
    overallPerformance: reportCard.overallScore >= 85 ? 'excellent' : 'good',
    grades: [
      { subject: 'Character Development', grade: 'A', score: reportCard.overallScore },
    ],
    strengths: reportCard.areasOfStrength || [],
    weaknesses: reportCard.areasForGrowth || [],
    achievements: reportCard.achievements?.map((a) => a.title) || [],
    virtuesDetected: Object.keys(reportCard.virtueAssessment || {}),
    teacherComments: reportCard.optimusPrimeMessage,
  };

  const response = await fetch(`${API_BASE}/api/vision/evaluate-with-reasoning`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ analysis: mockAnalysis }),
  });

  let evaluation = null;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  console.log('🧠 Receiving chain-of-thought evaluation...');

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter((line) => line.trim());

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        evaluation = parsed;
        if (parsed.reasoning) {
          console.log('💭 Chain-of-thought reasoning received...');
        }
        if (parsed.evaluation) {
          console.log('✅ Final evaluation received...');
        }
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }

  const duration = Date.now() - startTime;
  logTrace('evaluation_with_reasoning', duration, {
    grade: evaluation?.evaluation?.overallGrade,
    virtuesMastered: evaluation?.evaluation?.virtuesMastered?.length || 0,
  });

  logSection('EVALUATION', 'Chain-of-thought evaluation complete', {
    overallGrade: evaluation?.evaluation?.overallGrade,
    virtuesMastered: evaluation?.evaluation?.virtuesMastered || [],
    rewardType: evaluation?.evaluation?.reward?.type,
  });

  return evaluation;
}

// Step 5: Child responds to feedback
async function testChildResponse(evaluation) {
  logSection('RESPONSE', 'Generating child response to feedback');
  const startTime = Date.now();

  const prompt = `You are ${STUDENT_NAME}, a 10-year-old child who just received this feedback from Optimus Prime:

"${evaluation?.evaluation?.encouragement || 'Great work!'}"

Write a brief, enthusiastic response (2-3 sentences) showing your excitement about the feedback and reward.`;

  const { text } = await generateText({
    model: ollama('qwen3-coder:30b'),
    prompt,
  });

  const duration = Date.now() - startTime;
  logTrace('child_response', duration);

  logSection('CHILD RESPONSE', text);

  return text;
}

// Generate final transcript with all data
async function generateFinalTranscript(data) {
  logSection('TRANSCRIPT', 'Generating final session transcript');

  const totalDuration = Date.now() - testState.startTime;

  const transcript = {
    metadata: {
      studentName: STUDENT_NAME,
      testDate: new Date().toISOString(),
      totalDuration: `${(totalDuration / 1000).toFixed(2)}s`,
    },
    conversation: {
      turns: data.conversation.messages.length / 2,
      messages: data.conversation.messages,
    },
    reportCard: {
      overallScore: data.reportCard.overallScore,
      period: data.reportCard.period,
      virtueScores: Object.entries(data.reportCard.virtueAssessment || {}).map(
        ([virtue, assessment]) => ({
          virtue,
          score: assessment.score,
          feedback: assessment.feedback,
        })
      ),
      achievements: data.reportCard.achievements || [],
      message: data.reportCard.optimusPrimeMessage,
    },
    evaluation: {
      chainOfThought: {
        academicAnalysis: data.evaluation?.reasoning?.academicAnalysis,
        characterAssessment: data.evaluation?.reasoning?.characterAssessment,
        growthOpportunities: data.evaluation?.reasoning?.growthOpportunities,
        strengthsRecognition: data.evaluation?.reasoning?.strengthsRecognition,
      },
      final: {
        grade: data.evaluation?.evaluation?.overallGrade,
        virtuesMastered: data.evaluation?.evaluation?.virtuesMastered || [],
        areasToFocus: data.evaluation?.evaluation?.areasToFocus || [],
        encouragement: data.evaluation?.evaluation?.encouragement,
        advice: data.evaluation?.evaluation?.actionableAdvice || [],
        reward: data.evaluation?.evaluation?.reward,
      },
    },
    childResponse: data.childResponse,
    openTelemetry: {
      traces: testState.traces,
      totalOperations: testState.traces.length,
      averageLatency:
        (testState.traces.reduce((sum, t) => {
          const ms = parseInt(t.duration);
          return sum + ms;
        }, 0) /
          testState.traces.length).toFixed(2) + 'ms',
      fullTranscript: testState.transcript,
    },
  };

  // Save transcript
  const transcriptPath = path.join(
    process.cwd(),
    'tests',
    `transcript-${Date.now()}.json`
  );
  await fs.writeFile(transcriptPath, JSON.stringify(transcript, null, 2));

  logSection('TRANSCRIPT SAVED', transcriptPath, {
    size: `${(JSON.stringify(transcript).length / 1024).toFixed(2)}KB`,
  });

  // Print summary
  console.log('\n' + '='.repeat(80));
  console.log('🎉 E2E TEST SUMMARY');
  console.log('='.repeat(80));
  console.log(`👤 Student: ${STUDENT_NAME}`);
  console.log(`⏱️  Duration: ${(totalDuration / 1000).toFixed(2)}s`);
  console.log(`💬 Conversation Turns: ${data.conversation.messages.length / 2}`);
  console.log(`📊 Report Card Score: ${data.reportCard.overallScore}/100`);
  console.log(`🎓 Evaluation Grade: ${data.evaluation?.evaluation?.overallGrade}`);
  console.log(
    `⭐ Virtues Mastered: ${data.evaluation?.evaluation?.virtuesMastered?.join(', ') || 'none'}`
  );
  console.log(`🎁 Reward: ${data.evaluation?.evaluation?.reward?.type || 'none'}`);
  console.log(`📈 OpenTelemetry Traces: ${testState.traces.length} operations`);
  console.log(`📁 Transcript: ${transcriptPath}`);
  console.log('='.repeat(80));

  return transcript;
}

// Main test execution
async function runE2ETest() {
  console.log('🚀 Starting Simplified E2E Test\n');

  try {
    // Ensure tests directory exists
    await fs.mkdir(path.join(process.cwd(), 'tests'), { recursive: true });

    // Execute flow
    const conversation = await testConversation();
    const reportCard = await testReportCard(conversation.messages);
    const pdfPath = await testPDF(reportCard);
    const evaluation = await testEvaluation(reportCard);
    const childResponse = await testChildResponse(evaluation);

    // Generate final transcript
    await generateFinalTranscript({
      conversation,
      reportCard,
      pdfPath,
      evaluation,
      childResponse,
    });

    console.log('\n✅ E2E Test completed successfully!');
    process.exit(0);
  } catch (error) {
    console.error('\n❌ E2E Test failed:', error);
    console.error(error.stack);
    process.exit(1);
  }
}

// Run test
runE2ETest();
