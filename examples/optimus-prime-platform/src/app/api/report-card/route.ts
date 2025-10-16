import { streamObject } from "ai";
import { ollama } from "ollama-ai-provider-v2";
import { reportCardSchema, type ReportCardRequest } from "@/lib/report-card-schema";
import { trace, SpanStatusCode } from '@opentelemetry/api';
import { trackEvent } from "@/lib/telemetry";

const tracer = trace.getTracer('report-card-api', '0.1.0');

/**
 * POST /api/report-card
 *
 * Generates an AI-powered report card with structured data using Ollama
 * Returns streaming JSON as the report card is generated
 */
export async function POST(request: Request) {
  const span = tracer.startSpan('POST /api/report-card');

  try {
    const body: ReportCardRequest = await request.json();
    const { studentName, conversationHistory, period } = body;

    span.setAttributes({
      'report.student_name': studentName,
      'report.history_count': conversationHistory.length,
    });

    trackEvent("report_card_requested", {
      studentName,
      conversationCount: conversationHistory.length,
    });

    // Analyze conversation history to extract virtues and achievements
    const virtueMap = new Map<string, number>();
    conversationHistory.forEach(msg => {
      if (msg.virtue) {
        virtueMap.set(msg.virtue, (virtueMap.get(msg.virtue) || 0) + 1);
      }
    });

    // Build context for AI generation
    const conversationSummary = conversationHistory
      .filter(msg => msg.role === 'user')
      .map(msg => `- ${msg.content.substring(0, 200)}`)
      .join('\n');

    const virtueStats = Array.from(virtueMap.entries())
      .map(([virtue, count]) => `${virtue}: ${count} instances`)
      .join(', ');

    const systemPrompt = `You are Optimus Prime, generating a detailed achievement report card for ${studentName}.

Analyze their conversation history and demonstrated virtues to create a comprehensive, personalized report.

Conversation Summary:
${conversationSummary}

Detected Virtues: ${virtueStats}

Create a report card that:
1. Assigns fair scores (0-100) based on demonstrated behaviors
2. Provides specific, encouraging feedback for each virtue
3. Lists concrete examples from their conversations
4. Identifies 3 key strengths and 2-3 growth areas
5. Includes a warm, inspiring message from Optimus Prime
6. Lists badges earned (one per demonstrated virtue)

Be authentic, specific, and encouraging. Reference actual achievements from the conversations.
The overall score should reflect consistency across all virtues.`;

    // Use Ollama with structured output (streamObject)
    const result = await streamObject({
      model: ollama('qwen3-coder:30b'),
      schema: reportCardSchema,
      prompt: systemPrompt,
      mode: 'json',
    });

    span.setStatus({ code: SpanStatusCode.OK });

    // Stream the partial object as it's generated
    const encoder = new TextEncoder();
    const stream = new ReadableStream({
      async start(controller) {
        try {
          for await (const partialObject of result.partialObjectStream) {
            const chunk = JSON.stringify(partialObject) + '\n';
            controller.enqueue(encoder.encode(chunk));
          }

          trackEvent("report_card_generated", {
            studentName,
          });

          controller.close();
        } catch (error) {
          controller.error(error);
        }
      }
    });

    return new Response(stream, {
      headers: {
        'Content-Type': 'application/x-ndjson',
        'X-Student-Name': studentName,
      },
    });

  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    console.error("Report card generation error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to generate report card" }),
      { status: 500 }
    );
  } finally {
    span.end();
  }
}
