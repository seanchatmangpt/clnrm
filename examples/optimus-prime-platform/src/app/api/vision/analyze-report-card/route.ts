import { streamObject } from "ai";
import { ollama } from "ollama-ai-provider-v2";
import { reportCardAnalysisSchema, optimusResponseSchema } from "@/lib/vision-schema";
import { trace, SpanStatusCode } from '@opentelemetry/api';
import { trackEvent } from "@/lib/telemetry";

const tracer = trace.getTracer('vision-api', '0.1.0');

/**
 * POST /api/vision/analyze-report-card
 *
 * Uses qwen2.5vl vision model to analyze uploaded report card images
 * Returns structured analysis and Optimus Prime's response
 */
export async function POST(request: Request) {
  const span = tracer.startSpan('POST /api/vision/analyze-report-card');

  try {
    const formData = await request.formData();
    const imageFile = formData.get('image') as File;
    const studentName = formData.get('studentName') as string;

    if (!imageFile) {
      return new Response(
        JSON.stringify({ error: "No image provided" }),
        { status: 400 }
      );
    }

    span.setAttributes({
      'vision.student_name': studentName || 'unknown',
      'vision.image_size': imageFile.size,
      'vision.image_type': imageFile.type,
    });

    trackEvent("report_card_uploaded", {
      studentName: studentName || 'unknown',
      imageSize: imageFile.size,
    });

    // Convert image to base64 for Ollama
    const arrayBuffer = await imageFile.arrayBuffer();
    const base64Image = Buffer.from(arrayBuffer).toString('base64');
    const imageDataUrl = `data:${imageFile.type};base64,${base64Image}`;

    // Step 1: Analyze the report card with vision model
    const analysisPrompt = `You are analyzing a student's report card or academic document.
Extract all visible information including:
- Student name
- Grades/scores for each subject
- Teacher comments
- Areas of strength and weakness
- Any achievements or awards mentioned
- Character traits or virtues demonstrated

Be thorough and accurate in extracting the information.`;

    const analysisResult = await streamObject({
      model: ollama('qwen2.5-vl:latest'),
      schema: reportCardAnalysisSchema,
      messages: [
        {
          role: 'user',
          content: [
            { type: 'text', text: analysisPrompt },
            { type: 'image', image: imageDataUrl },
          ],
        },
      ],
      mode: 'json',
    });

    // Get the complete analysis
    let analysis = await analysisResult.object;

    span.setAttributes({
      'vision.document_type': analysis.documentType,
      'vision.performance': analysis.overallPerformance,
      'vision.virtues_count': analysis.virtuesDetected.length,
    });

    // Step 2: Generate Optimus Prime's response based on the analysis
    const responsePrompt = `You are Optimus Prime, leader of the Autobots, speaking to ${analysis.studentName || studentName || 'young one'}.

You have reviewed their report card showing:
- Overall Performance: ${analysis.overallPerformance}
- Grades: ${analysis.grades.map(g => `${g.subject}: ${g.grade}`).join(', ')}
- Strengths: ${analysis.strengths.join(', ')}
- Areas to Improve: ${analysis.weaknesses.join(', ')}
- Achievements: ${analysis.achievements.join(', ')}
- Character Virtues: ${analysis.virtuesDetected.join(', ')}

Create a personalized, inspiring response that:
1. Warmly greets them by name
2. Celebrates their strengths and achievements
3. Encourages them about areas to improve (without being negative)
4. Connects their academic performance to character virtues
5. Provides 3-5 actionable pieces of advice
6. Ends with an inspiring message about growth and potential

Speak with wisdom, nobility, and warmth. Make them feel valued and motivated.`;

    const optimusResult = await streamObject({
      model: ollama('qwen3-coder:30b'),
      schema: optimusResponseSchema,
      prompt: responsePrompt,
      mode: 'json',
    });

    span.setStatus({ code: SpanStatusCode.OK });

    trackEvent("report_card_analyzed", {
      studentName: analysis.studentName || studentName || 'unknown',
      performance: analysis.overallPerformance,
    });

    // Stream both the analysis and Optimus response as they're generated
    const encoder = new TextEncoder();
    const stream = new ReadableStream({
      async start(controller) {
        try {
          // Send analysis first
          controller.enqueue(encoder.encode(JSON.stringify({
            type: 'analysis',
            data: analysis,
          }) + '\n'));

          // Stream Optimus response
          for await (const partialOptimus of optimusResult.partialObjectStream) {
            controller.enqueue(encoder.encode(JSON.stringify({
              type: 'response',
              data: partialOptimus,
            }) + '\n'));
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
        'X-Student-Name': analysis.studentName || studentName || 'unknown',
      },
    });

  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    console.error("Vision analysis error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to analyze report card" }),
      { status: 500 }
    );
  } finally {
    span.end();
  }
}
