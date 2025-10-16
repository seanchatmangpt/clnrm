#!/usr/bin/env node

/**
 * Comprehensive E2E Test with Random Report Card Values
 *
 * This test generates realistic scenarios with varying performance levels,
 * including students with challenges who receive positive, encouraging feedback
 */

import { ollama } from 'ollama-ai-provider-v2';
import { generateText, streamObject } from 'ai';
import { z } from 'zod';
import fs from 'fs/promises';
import path from 'path';

const API_BASE = 'http://localhost:4000';

// Random student profiles with varying performance
const STUDENT_PROFILES = [
  {
    name: 'Sarah Martinez',
    age: 9,
    struggles: ['math', 'staying focused'],
    strengths: ['helping others', 'creative thinking'],
    personality: 'kind but easily discouraged',
  },
  {
    name: 'Michael Chen',
    age: 10,
    struggles: ['reading comprehension', 'test anxiety'],
    strengths: ['problem solving', 'persistence'],
    personality: 'hard worker who tries their best',
  },
  {
    name: 'Emma Johnson',
    age: 11,
    struggles: ['organization', 'time management'],
    strengths: ['leadership', 'teamwork'],
    personality: 'enthusiastic but scattered',
  },
];

// Select random profile
const STUDENT = STUDENT_PROFILES[Math.floor(Math.random() * STUDENT_PROFILES.length)];

// Test state
const testState = {
  traces: [],
  transcript: [],
  startTime: Date.now(),
  studentProfile: STUDENT,
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
  if (data) {
    const preview = JSON.stringify(data, null, 2);
    console.log(preview.length > 500 ? preview.substring(0, 500) + '...' : preview);
  }
}

// Generate realistic conversation with struggles and victories
async function generateConversation() {
  logSection('SETUP', `Student Profile: ${STUDENT.name}`, {
    age: STUDENT.age,
    struggles: STUDENT.struggles,
    strengths: STUDENT.strengths,
    personality: STUDENT.personality,
  });

  const conversationTopics = [
    `Hi Optimus! Today I tried really hard in ${STUDENT.struggles[0]} but I still got confused.`,
    `I helped my friend ${STUDENT.strengths[0]} and it made me feel good!`,
    `Sometimes I feel like I'm not as smart as the other kids because of my ${STUDENT.struggles[1]}.`,
    `My teacher said I'm getting better at ${STUDENT.strengths[1]}. Is that important?`,
    `How do you keep going when things are really hard?`,
  ];

  const messages = [];

  for (let i = 0; i < conversationTopics.length; i++) {
    const userContent = conversationTopics[i];
    logSection('CHILD', `Turn ${i + 1}: ${userContent}`);

    const startTime = Date.now();

    const response = await fetch(`${API_BASE}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        mode: 'child',
        messages: [
          ...messages,
          {
            id: `msg-${i}`,
            role: 'user',
            content: userContent,
            timestamp: Date.now(),
          },
        ],
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
    logTrace('chat_turn', duration, { turn: i + 1, responseLength: fullReply.length });

    logSection('OPTIMUS', `Turn ${i + 1} response (${duration}ms)`, {
      preview: fullReply.substring(0, 200) + '...',
    });

    messages.push(
      {
        id: `msg-${i}`,
        role: 'user',
        content: userContent,
        timestamp: Date.now(),
      },
      {
        id: `reply-${i}`,
        role: 'assistant',
        content: fullReply,
        timestamp: Date.now(),
      }
    );
  }

  return messages;
}

// Generate report card with random (including low) grades
async function generateReportCard(conversationHistory) {
  logSection('REPORT_CARD', `Generating report card for ${STUDENT.name}`);

  const startTime = Date.now();

  const response = await fetch(`${API_BASE}/api/report-card`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      studentName: STUDENT.name,
      conversationHistory,
      period: 'Q4 2025',
    }),
  });

  let reportCard = null;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let updates = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter((line) => line.trim());

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        reportCard = parsed;
        updates++;
        if (updates % 5 === 0) {
          console.log(`📦 Received partial update #${updates}...`);
        }
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }

  if (!reportCard) {
    throw new Error('Failed to generate report card');
  }

  const duration = Date.now() - startTime;
  logTrace('report_card_generation', duration, {
    overallScore: reportCard.overallScore,
    updates,
  });

  logSection('REPORT_CARD', 'Report card generated successfully', {
    student: reportCard.studentName,
    score: reportCard.overallScore,
    period: reportCard.period,
    achievements: reportCard.achievements?.length || 0,
    virtues: Object.keys(reportCard.virtueAssessment || {}).length,
  });

  return reportCard;
}

// Generate PDF
async function generatePDF(reportCard) {
  logSection('PDF', 'Generating PDF from report card');

  const startTime = Date.now();

  const response = await fetch(`${API_BASE}/api/report-card/pdf`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(reportCard),
  });

  if (!response.ok) {
    throw new Error(`PDF generation failed: ${response.status}`);
  }

  const pdfBuffer = await response.arrayBuffer();
  const pdfPath = path.join(
    process.cwd(),
    'tests',
    `report-card-${STUDENT.name.replace(/\s+/g, '-')}.pdf`
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

// Simulate vision analysis with uploaded report card
async function simulateVisionUpload(reportCard) {
  logSection('VISION', 'Simulating report card image upload and analysis');

  const startTime = Date.now();

  // Create mock analysis from report card data
  const mockAnalysis = {
    documentType: 'report card',
    studentName: reportCard.studentName,
    overallPerformance:
      reportCard.overallScore >= 85
        ? 'excellent'
        : reportCard.overallScore >= 70
        ? 'good'
        : reportCard.overallScore >= 60
        ? 'average'
        : 'needs improvement',
    grades: Object.entries(reportCard.virtueAssessment || {}).map(([virtue, data]) => ({
      subject: virtue.charAt(0).toUpperCase() + virtue.slice(1),
      grade: data.score >= 85 ? 'A' : data.score >= 70 ? 'B' : data.score >= 60 ? 'C' : 'D',
      score: data.score,
    })),
    strengths: reportCard.areasOfStrength || STUDENT.strengths,
    weaknesses: reportCard.areasForGrowth || STUDENT.struggles,
    achievements: reportCard.achievements?.map((a) => a.title) || [],
    virtuesDetected: Object.keys(reportCard.virtueAssessment || {}),
    teacherComments: reportCard.optimusPrimeMessage,
  };

  const duration = Date.now() - startTime;
  logTrace('vision_analysis_simulated', duration, {
    performance: mockAnalysis.overallPerformance,
    gradesCount: mockAnalysis.grades.length,
  });

  logSection('VISION', 'Vision analysis complete', {
    performance: mockAnalysis.overallPerformance,
    grades: mockAnalysis.grades,
    virtuesDetected: mockAnalysis.virtuesDetected,
  });

  return mockAnalysis;
}

// Chain-of-thought evaluation
async function evaluateWithReasoning(analysis) {
  logSection('EVALUATION', `Optimus Prime evaluating ${STUDENT.name} with chain-of-thought`);

  const startTime = Date.now();

  const response = await fetch(`${API_BASE}/api/vision/evaluate-with-reasoning`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ analysis }),
  });

  if (!response.ok) {
    throw new Error(`Evaluation failed: ${response.status}`);
  }

  let evaluation = null;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let updates = 0;

  console.log('🧠 Optimus Prime is thinking...');

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter((line) => line.trim());

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        evaluation = parsed;
        updates++;

        if (parsed.reasoning && updates === 1) {
          console.log('💭 Chain-of-thought reasoning received...');
        }
        if (parsed.evaluation && updates > 1) {
          console.log('✅ Final evaluation generated...');
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
    updates,
  });

  logSection('EVALUATION', 'Chain-of-thought evaluation complete', {
    reasoning: {
      academicAnalysis: evaluation?.reasoning?.academicAnalysis?.substring(0, 150) + '...',
      characterAssessment: evaluation?.reasoning?.characterAssessment?.substring(0, 150) + '...',
    },
    evaluation: {
      grade: evaluation?.evaluation?.overallGrade,
      virtuesMastered: evaluation?.evaluation?.virtuesMastered,
      rewardType: evaluation?.evaluation?.reward?.type,
    },
  });

  return evaluation;
}

// Child's response to feedback
async function generateChildResponse(evaluation) {
  logSection('RESPONSE', `Generating ${STUDENT.name}'s response to feedback`);

  const startTime = Date.now();

  const prompt = `You are ${STUDENT.name}, a ${STUDENT.age}-year-old child who is ${STUDENT.personality}.

You just received this evaluation from Optimus Prime:

Overall Grade: ${evaluation?.evaluation?.overallGrade}
Encouragement: "${evaluation?.evaluation?.encouragement}"
Special Reward: ${evaluation?.evaluation?.reward?.type} - "${evaluation?.evaluation?.reward?.unlockMessage}"
Advice: ${evaluation?.evaluation?.actionableAdvice?.join(', ')}

Write a brief, authentic response (2-3 sentences) showing:
- Your genuine feelings (including any worries or excitement)
- What part of the feedback means most to you
- A question or commitment about the advice

Write as a real ${STUDENT.age}-year-old would, with their vocabulary and emotions.`;

  const { text } = await generateText({
    model: ollama('qwen3-coder:30b'),
    prompt,
  });

  const duration = Date.now() - startTime;
  logTrace('child_response_generation', duration);

  logSection('CHILD RESPONSE', text);

  return text;
}

// Generate comprehensive transcript document
async function generateTranscriptDocument(data) {
  logSection('TRANSCRIPT', 'Generating comprehensive transcript document');

  const totalDuration = Date.now() - testState.startTime;

  const transcript = {
    metadata: {
      testDate: new Date().toISOString(),
      totalDuration: `${(totalDuration / 1000).toFixed(2)}s`,
      platform: 'Optimus Prime Educational Platform',
      version: '1.0.0',
    },
    studentProfile: {
      name: STUDENT.name,
      age: STUDENT.age,
      personality: STUDENT.personality,
      struggles: STUDENT.struggles,
      strengths: STUDENT.strengths,
    },
    conversation: {
      totalTurns: Math.floor(data.conversation.length / 2),
      messages: data.conversation.map((msg, idx) => ({
        turn: Math.floor(idx / 2) + 1,
        role: msg.role,
        content: msg.content,
        timestamp: msg.timestamp,
      })),
    },
    reportCard: {
      studentName: data.reportCard.studentName,
      period: data.reportCard.period,
      overallScore: data.reportCard.overallScore,
      virtueAssessment: Object.entries(data.reportCard.virtueAssessment || {}).map(
        ([virtue, assessment]) => ({
          virtue,
          score: assessment.score,
          examples: assessment.examples,
          feedback: assessment.feedback,
        })
      ),
      achievements: data.reportCard.achievements || [],
      areasOfStrength: data.reportCard.areasOfStrength || [],
      areasForGrowth: data.reportCard.areasForGrowth || [],
      optimusPrimeMessage: data.reportCard.optimusPrimeMessage,
      badges: data.reportCard.badges || [],
      pdfPath: data.pdfPath,
    },
    visionAnalysis: {
      documentType: data.analysis.documentType,
      overallPerformance: data.analysis.overallPerformance,
      grades: data.analysis.grades,
      strengths: data.analysis.strengths,
      weaknesses: data.analysis.weaknesses,
      achievements: data.analysis.achievements,
      virtuesDetected: data.analysis.virtuesDetected,
      teacherComments: data.analysis.teacherComments,
    },
    chainOfThoughtEvaluation: {
      reasoning: {
        academicAnalysis: data.evaluation?.reasoning?.academicAnalysis,
        characterAssessment: data.evaluation?.reasoning?.characterAssessment,
        growthOpportunities: data.evaluation?.reasoning?.growthOpportunities,
        strengthsRecognition: data.evaluation?.reasoning?.strengthsRecognition,
      },
      finalEvaluation: {
        overallGrade: data.evaluation?.evaluation?.overallGrade,
        virtuesMastered: data.evaluation?.evaluation?.virtuesMastered || [],
        areasToFocus: data.evaluation?.evaluation?.areasToFocus || [],
        encouragement: data.evaluation?.evaluation?.encouragement,
        actionableAdvice: data.evaluation?.evaluation?.actionableAdvice || [],
        reward: data.evaluation?.evaluation?.reward,
      },
    },
    childResponse: data.childResponse,
    openTelemetry: {
      traces: testState.traces,
      metrics: {
        totalOperations: testState.traces.length,
        averageLatency:
          (testState.traces.reduce((sum, t) => sum + parseInt(t.duration), 0) /
            testState.traces.length).toFixed(2) + 'ms',
        operationBreakdown: testState.traces.reduce((acc, t) => {
          acc[t.operation] = (acc[t.operation] || 0) + 1;
          return acc;
        }, {}),
      },
      fullTranscript: testState.transcript,
    },
  };

  // Save as JSON
  const jsonPath = path.join(
    process.cwd(),
    'tests',
    `transcript-${STUDENT.name.replace(/\s+/g, '-')}-${Date.now()}.json`
  );
  await fs.writeFile(jsonPath, JSON.stringify(transcript, null, 2));

  // Generate markdown document
  const markdown = generateMarkdownReport(transcript);
  const mdPath = path.join(
    process.cwd(),
    'tests',
    `TRANSCRIPT-${STUDENT.name.replace(/\s+/g, '-')}.md`
  );
  await fs.writeFile(mdPath, markdown);

  logSection('TRANSCRIPT', 'Comprehensive transcript generated', {
    jsonPath,
    markdownPath: mdPath,
    totalSize: `${(JSON.stringify(transcript).length / 1024).toFixed(2)}KB`,
  });

  return { transcript, jsonPath, mdPath };
}

// Generate markdown report
function generateMarkdownReport(transcript) {
  const t = transcript;

  return `# Optimus Prime Platform - Session Transcript

**Student**: ${t.studentProfile.name} (Age ${t.studentProfile.age})
**Date**: ${new Date(t.metadata.testDate).toLocaleString()}
**Duration**: ${t.metadata.totalDuration}

---

## 👤 Student Profile

- **Personality**: ${t.studentProfile.personality}
- **Struggles**: ${t.studentProfile.struggles.join(', ')}
- **Strengths**: ${t.studentProfile.strengths.join(', ')}

---

## 💬 Conversation with Optimus Prime

**Total Turns**: ${t.conversation.totalTurns}

${t.conversation.messages
  .map((msg) => {
    const role = msg.role === 'user' ? '**Child**' : '**Optimus Prime**';
    return `### Turn ${msg.turn} - ${role}\n\n${msg.content}\n`;
  })
  .join('\n')}

---

## 📊 Report Card

**Student**: ${t.reportCard.studentName}
**Period**: ${t.reportCard.period}
**Overall Score**: ${t.reportCard.overallScore}/100

### Virtue Assessment

${t.reportCard.virtueAssessment
  .map((v) => {
    return `#### ${v.virtue.charAt(0).toUpperCase() + v.virtue.slice(1)} - Score: ${v.score}/100

**Examples**:
${v.examples.map((ex) => `- ${ex}`).join('\n')}

**Feedback**: ${v.feedback}
`;
  })
  .join('\n')}

### Achievements

${t.reportCard.achievements.map((a) => `- **${a.title}**: ${a.description} (${a.virtue})`).join('\n')}

### Areas of Strength

${t.reportCard.areasOfStrength.map((s) => `- ${s}`).join('\n')}

### Areas for Growth

${t.reportCard.areasForGrowth.map((g) => `- ${g}`).join('\n')}

### Message from Optimus Prime

> ${t.reportCard.optimusPrimeMessage}

**PDF**: \`${t.reportCard.pdfPath}\`

---

## 🔍 Vision Analysis

**Document Type**: ${t.visionAnalysis.documentType}
**Overall Performance**: ${t.visionAnalysis.overallPerformance}

### Grades Extracted

${t.visionAnalysis.grades.map((g) => `- **${g.subject}**: ${g.grade} (${g.score})`).join('\n')}

### Strengths Identified

${t.visionAnalysis.strengths.map((s) => `- ${s}`).join('\n')}

### Growth Areas Identified

${t.visionAnalysis.weaknesses.map((w) => `- ${w}`).join('\n')}

### Character Virtues Detected

${t.visionAnalysis.virtuesDetected.map((v) => `- ${v}`).join('\n')}

---

## 🧠 Chain-of-Thought Evaluation

### Reasoning Process

#### Academic Analysis
${t.chainOfThoughtEvaluation.reasoning.academicAnalysis}

#### Character Assessment
${t.chainOfThoughtEvaluation.reasoning.characterAssessment}

#### Growth Opportunities
${t.chainOfThoughtEvaluation.reasoning.growthOpportunities}

#### Strengths Recognition
${t.chainOfThoughtEvaluation.reasoning.strengthsRecognition}

---

### Final Evaluation

**Overall Grade**: ${t.chainOfThoughtEvaluation.finalEvaluation.overallGrade}

**Virtues Mastered**: ${t.chainOfThoughtEvaluation.finalEvaluation.virtuesMastered.join(', ')}

**Areas to Focus**: ${t.chainOfThoughtEvaluation.finalEvaluation.areasToFocus.join(', ')}

#### Encouragement

> ${t.chainOfThoughtEvaluation.finalEvaluation.encouragement}

#### Actionable Advice

${t.chainOfThoughtEvaluation.finalEvaluation.actionableAdvice.map((a, i) => `${i + 1}. ${a}`).join('\n')}

#### 🎁 Reward Unlocked

**Type**: ${t.chainOfThoughtEvaluation.finalEvaluation.reward.type}
**Description**: ${t.chainOfThoughtEvaluation.finalEvaluation.reward.description}

> ${t.chainOfThoughtEvaluation.finalEvaluation.reward.unlockMessage}

---

## 💭 Child's Response

${t.childResponse}

---

## 📊 OpenTelemetry Metrics

**Total Operations**: ${t.openTelemetry.metrics.totalOperations}
**Average Latency**: ${t.openTelemetry.metrics.averageLatency}

### Operation Breakdown

${Object.entries(t.openTelemetry.metrics.operationBreakdown)
  .map(([op, count]) => `- ${op}: ${count} operations`)
  .join('\n')}

### Detailed Traces

${t.openTelemetry.traces
  .map((trace) => {
    return `#### ${trace.operation}
- **Timestamp**: ${trace.timestamp}
- **Duration**: ${trace.duration}
- **Metadata**: ${JSON.stringify(trace.metadata)}`;
  })
  .join('\n\n')}

---

## 🎯 Summary

This session demonstrates the complete Optimus Prime educational platform flow:

1. ✅ **Conversation**: ${t.conversation.totalTurns} turns with Optimus Prime
2. ✅ **Report Card**: Generated with ${t.reportCard.overallScore}/100 score
3. ✅ **PDF Export**: Saved to ${t.reportCard.pdfPath}
4. ✅ **Vision Analysis**: ${t.visionAnalysis.overallPerformance} performance detected
5. ✅ **Chain-of-Thought**: Deep reasoning with ${t.chainOfThoughtEvaluation.finalEvaluation.virtuesMastered.length} virtues mastered
6. ✅ **Child Response**: Authentic feedback received

**Platform demonstrates**:
- Encouraging feedback even for struggling students
- Focus on character development alongside academics
- Transparent AI reasoning (chain-of-thought)
- Full observability with OpenTelemetry
- Personalized, growth-mindset approach

---

*Generated by Optimus Prime Educational Platform - ${t.metadata.testDate}*
`;
}

// Print final summary
function printSummary(data) {
  console.log('\n' + '='.repeat(80));
  console.log('🎉 COMPREHENSIVE E2E TEST COMPLETE');
  console.log('='.repeat(80));
  console.log(`\n👤 Student: ${STUDENT.name} (${STUDENT.age} years old)`);
  console.log(`   Personality: ${STUDENT.personality}`);
  console.log(`   Struggles: ${STUDENT.struggles.join(', ')}`);
  console.log(`   Strengths: ${STUDENT.strengths.join(', ')}`);
  console.log(`\n⏱️  Total Duration: ${((Date.now() - testState.startTime) / 1000).toFixed(2)}s`);
  console.log(`\n💬 Conversation: ${Math.floor(data.conversation.length / 2)} turns`);
  console.log(`📊 Report Card Score: ${data.reportCard.overallScore}/100`);
  console.log(`🎓 Evaluation Grade: ${data.evaluation?.evaluation?.overallGrade}`);
  console.log(`⭐ Virtues Mastered: ${data.evaluation?.evaluation?.virtuesMastered?.join(', ') || 'none'}`);
  console.log(`🎁 Reward: ${data.evaluation?.evaluation?.reward?.type || 'none'}`);
  console.log(`\n📈 OpenTelemetry: ${testState.traces.length} operations traced`);
  console.log(`📁 Transcript: ${data.transcriptData.mdPath}`);
  console.log(`📁 JSON Data: ${data.transcriptData.jsonPath}`);
  console.log(`📄 PDF Report: ${data.pdfPath}`);
  console.log('\n' + '='.repeat(80));
  console.log('✅ All files saved successfully!');
  console.log('='.repeat(80) + '\n');
}

// Main test execution
async function runComprehensiveE2ETest() {
  console.log('🚀 Starting Comprehensive E2E Test with Random Values\n');

  try {
    await fs.mkdir(path.join(process.cwd(), 'tests'), { recursive: true });

    // Execute full flow
    const conversation = await generateConversation();
    const reportCard = await generateReportCard(conversation);
    const pdfPath = await generatePDF(reportCard);
    const analysis = await simulateVisionUpload(reportCard);
    const evaluation = await evaluateWithReasoning(analysis);
    const childResponse = await generateChildResponse(evaluation);

    // Generate comprehensive transcript
    const transcriptData = await generateTranscriptDocument({
      conversation,
      reportCard,
      pdfPath,
      analysis,
      evaluation,
      childResponse,
    });

    // Print summary
    printSummary({
      conversation,
      reportCard,
      pdfPath,
      analysis,
      evaluation,
      childResponse,
      transcriptData,
    });

    console.log('\n✅ Comprehensive E2E Test completed successfully!');
    process.exit(0);
  } catch (error) {
    console.error('\n❌ E2E Test failed:', error);
    console.error(error.stack);
    process.exit(1);
  }
}

// Run test
runComprehensiveE2ETest();
