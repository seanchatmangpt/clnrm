import {
  ChatRequest,
  ChatResponse,
  detectVirtue,
  REWARD_URLS,
  PREMIUM_CTA_VARIANTS,
} from "@/lib/types";
import { trackEvent } from "@/lib/telemetry";

export async function POST(request: Request) {
  try {
    const body: ChatRequest = await request.json();
    const { mode, messages } = body;

    const lastMessage = messages[messages.length - 1];
    if (!lastMessage || lastMessage.role !== "user") {
      throw new Error("Invalid message format");
    }

    trackEvent("message_sent", {
      mode,
      messageLength: lastMessage.content.length,
    });

    if (mode === "child") {
      return await handleChildChat(lastMessage.content, request.headers);
    } else if (mode === "executive") {
      return await handleExecutiveChat(lastMessage.content);
    } else {
      throw new Error("Invalid mode");
    }
  } catch (error) {
    console.error("Chat API error:", error);
    return new Response("Internal Server Error", { status: 500 });
  }
}

async function handleChildChat(
  userInput: string,
  headers: Headers
): Promise<Response> {
  const virtue = detectVirtue(userInput);

  trackEvent("virtue_detected", { virtue });

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

  return new Response(ollamaResponse.body, {
    headers: responseHeaders,
  });
}

async function handleExecutiveChat(userInput: string): Promise<Response> {
  // Import telemetry functions here to avoid circular dependencies
  const { getMetrics, getStaticCorpData } = await import("@/lib/telemetry");

  const metrics = getMetrics();
  const corpData = getStaticCorpData();

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

  return new Response(ollamaResponse.body);
}
