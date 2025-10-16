import { z } from "zod";

/**
 * Report Card Schema
 *
 * Defines the structure for AI-generated child achievement report cards
 * with virtue assessment and personalized feedback from Optimus Prime
 */

export const reportCardSchema = z.object({
  studentName: z.string().describe("Child's name"),
  period: z.string().describe("Reporting period (e.g., 'Week of Oct 16, 2025')"),
  overallScore: z.number().min(0).max(100).describe("Overall virtue score (0-100)"),

  virtueAssessment: z.object({
    teamwork: z.object({
      score: z.number().min(0).max(100),
      examples: z.array(z.string()).describe("Specific examples demonstrated"),
      feedback: z.string().describe("Personalized feedback from Optimus Prime"),
    }),
    courage: z.object({
      score: z.number().min(0).max(100),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    honesty: z.object({
      score: z.number().min(0).max(100),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    compassion: z.object({
      score: z.number().min(0).max(100),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
    wisdom: z.object({
      score: z.number().min(0).max(100),
      examples: z.array(z.string()),
      feedback: z.string(),
    }),
  }),

  achievements: z.array(z.object({
    title: z.string(),
    description: z.string(),
    virtue: z.enum(["teamwork", "courage", "honesty", "compassion", "wisdom"]),
    date: z.string(),
  })),

  areasOfStrength: z.array(z.string()).describe("Top 3 strengths"),
  areasForGrowth: z.array(z.string()).describe("Areas to develop"),

  optimusPrimeMessage: z.string().describe("Personalized message from Optimus Prime"),

  badges: z.array(z.object({
    name: z.string(),
    virtue: z.string(),
    earnedDate: z.string(),
  })),
});

export type ReportCard = z.infer<typeof reportCardSchema>;

/**
 * Request schema for generating report cards
 */
export const reportCardRequestSchema = z.object({
  studentName: z.string(),
  conversationHistory: z.array(z.object({
    role: z.enum(["user", "assistant"]),
    content: z.string(),
    virtue: z.string().optional(),
    timestamp: z.string(),
  })),
  period: z.string().optional(),
});

export type ReportCardRequest = z.infer<typeof reportCardRequestSchema>;
