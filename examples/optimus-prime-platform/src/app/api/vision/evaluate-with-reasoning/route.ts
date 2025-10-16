import { streamObject } from "ai";
import { ollama } from "ollama-ai-provider-v2";
import { z } from "zod";
import { trace, SpanStatusCode } from '@opentelemetry/api';
import { trackEvent } from "@/lib/telemetry";

const tracer = trace.getTracer('vision-evaluation-api', '0.1.0');

/**
 * Chain-of-thought evaluation schema
 * Optimus Prime reasons through the evaluation before providing final assessment
 */
const evaluationSchema = z.object({
  reasoning: z.object({
    academicAnalysis: z.string().describe("Detailed analysis of academic performance, examining grades, effort, and learning patterns"),
    characterAssessment: z.string().describe("Deep assessment of character virtues demonstrated through academic work and behavior"),
    growthOpportunities: z.string().describe("Thoughtful identification of areas where the student can grow and improve"),
    strengthsRecognition: z.string().describe("Recognition and celebration of key strengths, talents, and positive qualities"),
  }).describe("Optimus Prime's chain-of-thought reasoning process"),

  evaluation: z.object({
    overallGrade: z.enum(["excellent", "good", "average", "needs improvement"]).describe("Final overall grade after reasoning"),
    virtuesMastered: z.array(z.string()).describe("Character virtues the student has demonstrated mastery in"),
    areasToFocus: z.array(z.string()).describe("Specific areas the student should focus on for growth"),
    encouragement: z.string().describe("Warm, encouraging message that motivates continued effort"),
    actionableAdvice: z.array(z.string()).describe("3-5 specific, actionable pieces of advice for improvement"),
    reward: z.object({
      type: z.string().describe("Type of reward earned (badge, achievement, unlock, etc.)"),
      description: z.string().describe("Description of what the reward represents"),
      unlockMessage: z.string().describe("Exciting message about what they've unlocked"),
    }).describe("Special reward based on their performance"),
  }).describe("Final evaluation after chain-of-thought reasoning"),
});

/**
 * POST /api/vision/evaluate-with-reasoning
 *
 * Uses chain-of-thought reasoning to evaluate report card analysis
 * Optimus Prime thinks through the evaluation before providing final assessment
 */
export async function POST(request: Request) {
  const span = tracer.startSpan('POST /api/vision/evaluate-with-reasoning');

  try {
    const { analysis } = await request.json();

    if (!analysis) {
      return new Response(
        JSON.stringify({ error: "No analysis provided" }),
        { status: 400 }
      );
    }

    span.setAttributes({
      'evaluation.student_name': analysis.studentName || 'unknown',
      'evaluation.performance': analysis.overallPerformance,
      'evaluation.virtues_detected': analysis.virtuesDetected?.length || 0,
    });

    trackEvent("report_card_evaluation_started", {
      studentName: analysis.studentName || 'unknown',
      performance: analysis.overallPerformance,
    });

    // Construct detailed prompt for chain-of-thought reasoning
    const prompt = `You are Optimus Prime, leader of the Autobots, conducting a thorough evaluation of ${analysis.studentName}'s report card.

REPORT CARD DATA:
- Document Type: ${analysis.documentType}
- Overall Performance: ${analysis.overallPerformance}
- Grades: ${analysis.grades.map((g: any) => `${g.subject}: ${g.grade}${g.score ? ` (${g.score})` : ''}`).join(', ')}
- Strengths: ${analysis.strengths.join(', ')}
- Weaknesses/Growth Areas: ${analysis.weaknesses.join(', ')}
- Achievements: ${analysis.achievements.join(', ')}
- Character Virtues Detected: ${analysis.virtuesDetected.join(', ')}
${analysis.teacherComments ? `- Teacher Comments: ${analysis.teacherComments}` : ''}

YOUR TASK:
Use chain-of-thought reasoning to deeply evaluate this student's performance.

REASONING PROCESS (think through each step):
1. **Academic Analysis**: Examine their grades, effort level, and learning patterns. What do the grades tell you? Are they improving? What subjects need attention?

2. **Character Assessment**: Look beyond grades. What virtues have they demonstrated? How do their achievements reflect their character? What kind of person are they becoming?

3. **Growth Opportunities**: Where can they improve? Be specific and constructive. What skills or habits would help them most?

4. **Strengths Recognition**: What are they doing exceptionally well? What talents should be celebrated and encouraged?

FINAL EVALUATION:
After your reasoning, provide:
- An overall grade (excellent/good/average/needs improvement)
- Virtues they've mastered
- Specific areas to focus on (3-5 items)
- Warm encouragement that motivates them
- Actionable advice (3-5 concrete steps)
- A special reward that matches their achievements

Remember: You are Optimus Prime. Be wise, noble, encouraging, and inspiring. See the potential in every young one. Make them feel valued while guiding them toward growth.`;

    const result = await streamObject({
      model: ollama('qwen3-coder:30b'),
      schema: evaluationSchema,
      prompt: prompt,
      mode: 'json',
    });

    span.setStatus({ code: SpanStatusCode.OK });

    // Stream the evaluation as it's generated
    const encoder = new TextEncoder();
    const stream = new ReadableStream({
      async start(controller) {
        try {
          for await (const partialEvaluation of result.partialObjectStream) {
            controller.enqueue(encoder.encode(JSON.stringify(partialEvaluation) + '\n'));

            // Track completion
            if (partialEvaluation.evaluation?.overallGrade) {
              trackEvent("report_card_evaluation_completed", {
                studentName: analysis.studentName || 'unknown',
                grade: partialEvaluation.evaluation.overallGrade,
                virtuesMastered: partialEvaluation.evaluation.virtuesMastered?.length || 0,
              });
            }
          }
          controller.close();
        } catch (error) {
          controller.error(error);
        }
      }
    });

    return new Response(stream, {
      headers: {
        'Content-Type': 'application/x-ndjson',
        'X-Student-Name': analysis.studentName || 'unknown',
      },
    });

  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    console.error("Evaluation error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to evaluate report card" }),
      { status: 500 }
    );
  } finally {
    span.end();
  }
}
