export type ChatMode = "child" | "executive";

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

export interface ChatRequest {
  mode: ChatMode;
  messages: Message[];
}

export interface ChatResponse {
  messages: Message[];
  virtue?: string;
  rewardUrl?: string;
  premiumTitle?: string;
  premiumLink?: string;
}

export interface TelemetryEvent {
  id: string;
  ts: number;
  event: EventType;
  payload: Record<string, unknown>;
}

export type EventType =
  | "session_start"
  | "message_sent"
  | "virtue_detected"
  | "reward_click"
  | "reward_view"
  | "premium_view"
  | "premium_click"
  | "purchase"
  | "report_card_requested"
  | "report_card_generated"
  | "report_card_pdf_generated"
  | "report_card_uploaded"
  | "report_card_analyzed"
  | "report_card_evaluation_started"
  | "report_card_evaluation_completed";

export interface VirtueHistory {
  id: string;
  virtue: string;
  timestamp: number;
  achievement: string;
}

export interface ABVariant {
  variant: "A" | "B";
  views: number;
  clicks: number;
}

export interface RevenueData {
  day: string;
  amount_usd: number;
}

export interface FunnelStep {
  label: string;
  value: number;
}

export interface MetricsData {
  ab: Record<"A" | "B", ABVariant>;
  funnel: FunnelStep[];
  revenue7: {
    labels: string[];
    data: number[];
  };
  totals: {
    revenue: number;
    events: number;
  };
}

export interface StaticCorpData {
  company: string;
  period: string;
  okrs: string[];
  targets: {
    monthlyRevenueUSD: number;
    retentionD7: number;
    premiumCTR: number;
  };
}

// Static corporate data as per PRD
export const STATIC_CORP_DATA: StaticCorpData = {
  company: "Autobot Industries",
  period: "Q4 2025",
  okrs: [
    "Increase premium conversion by 15%",
    "Achieve 95% user retention in first week",
    "Expand to 5 new markets",
  ],
  targets: {
    monthlyRevenueUSD: 250000,
    retentionD7: 95,
    premiumCTR: 8,
  },
};

// Virtue mapping as per PRD
export const VIRTUE_KEYWORDS = {
  teamwork: [
    "team",
    "group",
    "together",
    "help",
    "support",
    "cooperate",
    "collaboration",
    "united",
  ],
  wisdom: [
    "learn",
    "study",
    "school",
    "knowledge",
    "education",
    "understand",
    "smart",
    "clever",
  ],
  compassion: [
    "help",
    "care",
    "kind",
    "friend",
    "support",
    "empathy",
    "understanding",
    "caring",
  ],
  courage: [
    "brave",
    "challenge",
    "difficult",
    "try",
    "overcome",
    "face",
    "confront",
    "strength",
  ],
};

export function detectVirtue(text: string): string {
  const lowerText = text.toLowerCase();

  for (const [virtue, keywords] of Object.entries(VIRTUE_KEYWORDS)) {
    if (keywords.some((keyword) => lowerText.includes(keyword))) {
      return virtue;
    }
  }

  return "courage"; // Default virtue
}

// Reward URLs by virtue
export const REWARD_URLS = {
  teamwork: "https://example.com/rewards/teamwork-badge.mp4",
  wisdom: "https://example.com/rewards/wisdom-certificate.mp4",
  compassion: "https://example.com/rewards/compassion-heart.mp4",
  courage: "https://example.com/rewards/courage-shield.mp4",
};

// Premium CTA variants for A/B testing
export const PREMIUM_CTA_VARIANTS = {
  A: {
    title: "Unlock Premium Adventures",
    link: "https://store.autobot.com/premium",
  },
  B: {
    title: "Join the Elite Autobots",
    link: "https://store.autobot.com/elite",
  },
};
