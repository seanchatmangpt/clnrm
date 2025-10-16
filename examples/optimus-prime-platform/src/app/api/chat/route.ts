import {
  ChatRequest,
  detectVirtue,
  REWARD_URLS,
  PREMIUM_CTA_VARIANTS,
} from "@/lib/types";
import { trackEvent, trackVirtue } from "@/lib/telemetry";
import { trace, SpanStatusCode } from '@opentelemetry/api';

const tracer = trace.getTracer('optimus-prime-platform-api', '0.1.0');

export async function POST(request: Request) {
  const span = tracer.startSpan('POST /api/chat');

  try {
    const body: ChatRequest = await request.json();
    const { mode, messages } = body;

    span.setAttributes({
      'chat.mode': mode,
      'chat.messages.count': messages.length,
    });

    const lastMessage = messages[messages.length - 1];
    if (!lastMessage || lastMessage.role !== "user") {
      throw new Error("Invalid message format");
    }

    trackEvent("message_sent", {
      mode,
      messageLength: lastMessage.content.length,
    });

    let response: Response;
    if (mode === "child") {
      response = await handleChildChat(lastMessage.content, request.headers);
    } else if (mode === "executive") {
      response = await handleExecutiveChat(lastMessage.content);
    } else {
      throw new Error("Invalid mode");
    }

    span.setStatus({ code: SpanStatusCode.OK });
    return response;
  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    console.error("Chat API error:", error);
    return new Response("Internal Server Error", { status: 500 });
  } finally {
    span.end();
  }
}

async function handleChildChat(
  userInput: string,
  _headers: Headers
): Promise<Response> {
  const span = tracer.startSpan('handleChildChat');

  try {
    const virtue = detectVirtue(userInput);

    span.setAttributes({
      'chat.child.virtue': virtue,
      'chat.child.input_length': userInput.length,
    });

    // Track virtue with achievement text for history
    trackVirtue(virtue, userInput);

    const systemPrompt = `You are Optimus Prime, leader of the Autobots. Speak with noble leadership. Do not mirror the user's words. Recognize the virtue: ${virtue}. Provide one to two sentences. Encourage forward action.`;

    // Use Ollama API directly
    const ollamaResponse = await fetch("http://localhost:11434/api/generate", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model: "qwen3-coder:30b",
        prompt: systemPrompt + "\n\nUser: " + userInput,
        stream: true,
      }),
    });

    if (!ollamaResponse.ok) {
      throw new Error(`Ollama API error: ${ollamaResponse.status}`);
    }

    // Set response headers for virtue, reward, and premium CTA
    const responseHeaders = new Headers();
    responseHeaders.set("X-Virtue", virtue);
    responseHeaders.set(
      "X-Reward-Url",
      REWARD_URLS[virtue as keyof typeof REWARD_URLS] || REWARD_URLS.courage
    );

    // Add premium CTA headers (A/B testing)
    const variant = Math.random() > 0.5 ? "A" : "B";
    const premiumCTA = PREMIUM_CTA_VARIANTS[variant];
    responseHeaders.set("X-Premium-Title", premiumCTA.title);
    responseHeaders.set("X-Premium-Link", premiumCTA.link);

    span.setAttributes({
      'chat.child.variant': variant,
    });
    span.setStatus({ code: SpanStatusCode.OK });

    return new Response(ollamaResponse.body, {
      headers: responseHeaders,
    });
  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    throw error;
  } finally {
    span.end();
  }
}

async function handleExecutiveChat(userInput: string): Promise<Response> {
  const span = tracer.startSpan('handleExecutiveChat');

  try {
    // Import telemetry functions here to avoid circular dependencies
    const { getMetrics, getStaticCorpData } = await import("@/lib/telemetry");

    const metrics = getMetrics();
    const corpData = getStaticCorpData();

    span.setAttributes({
      'chat.executive.total_revenue': metrics.totals.revenue,
      'chat.executive.total_events': metrics.totals.events,
      'chat.executive.input_length': userInput.length,
    });

    const context = `
Current Metrics:
- Total Revenue (7d): $${metrics.totals.revenue}
- Total Events: ${metrics.totals.events}
- Premium CTR A: ${
      metrics.ab.A.views > 0
        ? ((metrics.ab.A.clicks / metrics.ab.A.views) * 100).toFixed(1)
        : "0.0"
    }%
- Premium CTR B: ${
      metrics.ab.B.views > 0
        ? ((metrics.ab.B.clicks / metrics.ab.B.views) * 100).toFixed(1)
        : "0.0"
    }%

Company Targets:
- Monthly Revenue Target: $${corpData.targets.monthlyRevenueUSD}
- Retention Target (D7): ${corpData.targets.retentionD7}%
- Premium CTR Target: ${corpData.targets.premiumCTR}%
`;

    const systemPrompt = `You are an executive analyst. Use only provided context. Return concrete numbers with units. If unknown, say 'insufficient data'. Keep answers ≤ 5 lines.

Context:
${context}`;

    // Use Ollama API directly
    const ollamaResponse = await fetch("http://localhost:11434/api/generate", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model: "qwen3-coder:30b",
        prompt: systemPrompt + "\n\nQuestion: " + userInput,
        stream: true,
      }),
    });

    if (!ollamaResponse.ok) {
      throw new Error(`Ollama API error: ${ollamaResponse.status}`);
    }

    span.setStatus({ code: SpanStatusCode.OK });
    return new Response(ollamaResponse.body);
  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    throw error;
  } finally {
    span.end();
  }
}
