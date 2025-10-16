import { z } from "zod";

/**
 * Schema for analyzing uploaded report card images with vision model
 */
export const reportCardAnalysisSchema = z.object({
  documentType: z.string().describe("Type of document (e.g., 'report card', 'certificate', 'test')"),
  studentName: z.string().describe("Student's name from the document"),

  grades: z.array(z.object({
    subject: z.string(),
    grade: z.string(),
    score: z.number().optional(),
  })).describe("Grades or scores visible in the document"),

  overallPerformance: z.enum(["excellent", "good", "average", "needs improvement"]).describe("Overall assessment"),

  strengths: z.array(z.string()).describe("Areas of strength mentioned"),
  weaknesses: z.array(z.string()).describe("Areas needing improvement mentioned"),

  teacherComments: z.string().optional().describe("Any teacher comments or notes"),

  achievements: z.array(z.string()).describe("Specific achievements or accomplishments mentioned"),

  virtuesDetected: z.array(z.enum(["teamwork", "courage", "honesty", "compassion", "wisdom"])).describe("Character virtues demonstrated"),
});

export type ReportCardAnalysis = z.infer<typeof reportCardAnalysisSchema>;

/**
 * Schema for Optimus Prime's response to the report card
 */
export const optimusResponseSchema = z.object({
  greeting: z.string().describe("Warm greeting addressing the student by name"),

  strengthsRecognition: z.string().describe("Acknowledgment of their strengths and achievements"),

  encouragementForWeaknesses: z.string().describe("Encouraging words about areas to improve"),

  virtueConnection: z.string().describe("How their academic performance connects to character virtues"),

  inspirationalMessage: z.string().describe("Inspiring message from Optimus Prime about continuing to grow"),

  actionableAdvice: z.array(z.string()).describe("3-5 specific, actionable pieces of advice"),

  celebrationMessage: z.string().describe("Celebration of their efforts and dedication"),
});

export type OptimusResponse = z.infer<typeof optimusResponseSchema>;
