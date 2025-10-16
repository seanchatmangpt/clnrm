import { Page, expect } from '@playwright/test';
import type { ReportCardAnalysis, OptimusResponse } from '@/lib/vision-schema';

/**
 * Test helper utilities for vision upload E2E tests
 */

/**
 * Upload an image file to the report card upload form
 * @param page - Playwright page instance
 * @param filePath - Path to the image file (relative to fixtures directory)
 * @param studentName - Optional student name to fill in
 */
export async function uploadReportCardImage(
  page: Page,
  filePath: string,
  studentName?: string
): Promise<void> {
  // Navigate to upload page
  await page.goto('/upload-report');
  await expect(page.locator('h1:has-text("Upload Your Report Card")')).toBeVisible();

  // Fill in student name if provided
  if (studentName) {
    await page.fill('input[type="text"][placeholder*="Enter your name"]', studentName);
  }

  // Upload file
  const fileInput = page.locator('input[type="file"]');
  await fileInput.setInputFiles(filePath);

  // Verify preview is shown
  await expect(page.locator('img[alt="Report card preview"]')).toBeVisible();
}

/**
 * Click analyze button and wait for processing
 * @param page - Playwright page instance
 */
export async function clickAnalyzeButton(page: Page): Promise<void> {
  const analyzeButton = page.locator('button:has-text("Analyze with Vision AI")');
  await expect(analyzeButton).toBeEnabled();
  await analyzeButton.click();

  // Wait for analyzing state
  await expect(page.locator('button:has-text("Analyzing")')).toBeVisible();
}

/**
 * Wait for analysis to complete
 * @param page - Playwright page instance
 * @param timeout - Maximum time to wait in milliseconds (default: 30000)
 */
export async function waitForAnalysisComplete(
  page: Page,
  timeout: number = 30000
): Promise<void> {
  // Wait for analyze button to be re-enabled (analysis complete)
  await expect(
    page.locator('button:has-text("Analyze with Vision AI")')
  ).toBeVisible({ timeout });

  // Verify analysis results are displayed
  await expect(
    page.locator('h2:has-text("Report Card Analysis")')
  ).toBeVisible({ timeout });
}

/**
 * Parse NDJSON stream response from API
 * @param ndjsonText - NDJSON formatted text
 * @returns Array of parsed JSON objects
 */
export function parseNDJSON(ndjsonText: string): any[] {
  const lines = ndjsonText
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0);

  return lines.map(line => {
    try {
      return JSON.parse(line);
    } catch (e) {
      console.error('Failed to parse NDJSON line:', line, e);
      return null;
    }
  }).filter(obj => obj !== null);
}

/**
 * Validate ReportCardAnalysis structure
 * @param analysis - Analysis object to validate
 * @returns true if valid, throws error otherwise
 */
export function validateReportCardAnalysis(analysis: any): analysis is ReportCardAnalysis {
  if (!analysis || typeof analysis !== 'object') {
    throw new Error('Analysis is not an object');
  }

  const required = ['studentName', 'overallPerformance', 'grades', 'strengths', 'weaknesses', 'virtuesDetected'];
  for (const field of required) {
    if (!(field in analysis)) {
      throw new Error(`Missing required field: ${field}`);
    }
  }

  if (typeof analysis.studentName !== 'string') {
    throw new Error('studentName must be a string');
  }

  if (!['excellent', 'good', 'average', 'needs improvement'].includes(analysis.overallPerformance)) {
    throw new Error(`Invalid overallPerformance: ${analysis.overallPerformance}`);
  }

  if (!Array.isArray(analysis.grades)) {
    throw new Error('grades must be an array');
  }

  for (const grade of analysis.grades) {
    if (!grade.subject || typeof grade.subject !== 'string') {
      throw new Error('Grade must have subject string');
    }
    if (!grade.grade || typeof grade.grade !== 'string') {
      throw new Error('Grade must have grade string');
    }
  }

  if (!Array.isArray(analysis.strengths)) {
    throw new Error('strengths must be an array');
  }

  if (!Array.isArray(analysis.weaknesses)) {
    throw new Error('weaknesses must be an array');
  }

  if (!Array.isArray(analysis.virtuesDetected)) {
    throw new Error('virtuesDetected must be an array');
  }

  return true;
}

/**
 * Validate OptimusResponse structure
 * @param response - Optimus response object to validate
 * @returns true if valid, throws error otherwise
 */
export function validateOptimusResponse(response: any): response is OptimusResponse {
  if (!response || typeof response !== 'object') {
    throw new Error('Response is not an object');
  }

  const required = [
    'greeting',
    'strengthsRecognition',
    'encouragementForWeaknesses',
    'virtueConnection',
    'actionableAdvice',
    'inspirationalMessage',
    'celebrationMessage'
  ];

  for (const field of required) {
    if (!(field in response)) {
      throw new Error(`Missing required field: ${field}`);
    }
  }

  const stringFields = [
    'greeting',
    'strengthsRecognition',
    'encouragementForWeaknesses',
    'virtueConnection',
    'inspirationalMessage',
    'celebrationMessage'
  ];

  for (const field of stringFields) {
    if (typeof response[field] !== 'string') {
      throw new Error(`${field} must be a string`);
    }
    if (response[field].trim().length === 0) {
      throw new Error(`${field} must not be empty`);
    }
  }

  if (!Array.isArray(response.actionableAdvice)) {
    throw new Error('actionableAdvice must be an array');
  }

  if (response.actionableAdvice.length === 0) {
    throw new Error('actionableAdvice must have at least one item');
  }

  for (const advice of response.actionableAdvice) {
    if (typeof advice !== 'string' || advice.trim().length === 0) {
      throw new Error('Each actionableAdvice item must be a non-empty string');
    }
  }

  return true;
}

/**
 * Extract analysis data from page
 * @param page - Playwright page instance
 * @returns Extracted analysis data
 */
export async function extractAnalysisFromPage(page: Page): Promise<Partial<ReportCardAnalysis>> {
  const studentName = await page.locator('[class*="bg-blue-50"] p').first().textContent();
  const overallPerformance = await page.locator('[class*="bg-green-50"] p').first().textContent();

  const grades = await page.locator('[class*="bg-gray-50"]').evaluateAll(elements => {
    return elements.map(el => {
      const subject = el.querySelector('.font-medium')?.textContent || '';
      const grade = el.querySelector('.text-2xl')?.textContent || '';
      return { subject: subject.trim(), grade: grade.trim() };
    }).filter(g => g.subject && g.grade);
  });

  const strengths = await page.locator('h3:has-text("Strengths") + ul li span:last-child').allTextContents();
  const weaknesses = await page.locator('h3:has-text("Growth Areas") + ul li span:last-child').allTextContents();
  const virtues = await page.locator('h3:has-text("Character Virtues") + div span').allTextContents();

  return {
    studentName: studentName?.trim() || '',
    overallPerformance: overallPerformance?.toLowerCase() as any,
    grades,
    strengths: strengths.map(s => s.trim()),
    weaknesses: weaknesses.map(w => w.trim()),
    virtuesDetected: virtues.map(v => v.trim()),
  };
}

/**
 * Extract Optimus response from page
 * @param page - Playwright page instance
 * @returns Extracted Optimus response data
 */
export async function extractOptimusResponseFromPage(page: Page): Promise<Partial<OptimusResponse>> {
  // Wait for Optimus section
  await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible();

  const greeting = await page.locator('[class*="bg-white/10"] p').first().textContent();
  const strengthsRecognition = await page.locator('h3:has-text("Your Strengths") + p').textContent();
  const encouragement = await page.locator('h3:has-text("Room to Grow") + p').textContent();
  const virtueConnection = await page.locator('h3:has-text("Character Connection") + p').textContent();
  const inspirationalMessage = await page.locator('[class*="border-2 border-blue-400"] p').textContent();
  const celebrationMessage = await page.locator('.text-center p.text-2xl').textContent();

  const actionableAdvice = await page.locator('h3:has-text("Advice from Optimus") + ul li span:last-child').allTextContents();

  return {
    greeting: greeting?.trim() || '',
    strengthsRecognition: strengthsRecognition?.trim() || '',
    encouragementForWeaknesses: encouragement?.trim() || '',
    virtueConnection: virtueConnection?.trim() || '',
    actionableAdvice: actionableAdvice.map(a => a.trim()),
    inspirationalMessage: inspirationalMessage?.trim() || '',
    celebrationMessage: celebrationMessage?.trim() || '',
  };
}

/**
 * Verify no false positives in grades
 * Ensures grades are valid and realistic
 * @param grades - Array of grades to validate
 */
export function verifyNoFalsePositiveGrades(grades: Array<{ subject: string; grade: string }>): void {
  const validGrades = ['A+', 'A', 'A-', 'B+', 'B', 'B-', 'C+', 'C', 'C-', 'D+', 'D', 'D-', 'F', 'P', 'N/A'];
  const validSubjects = [
    'Math', 'Mathematics', 'English', 'Science', 'Reading', 'Writing',
    'Social Studies', 'History', 'Art', 'Music', 'Physical Education', 'PE',
    'Computer Science', 'Spanish', 'French', 'Language Arts'
  ];

  for (const grade of grades) {
    // Check if grade format is valid
    const gradeUpper = grade.grade.toUpperCase();
    const isValidGrade = validGrades.some(vg => gradeUpper.includes(vg)) || /^\d+%?$/.test(grade.grade);

    if (!isValidGrade) {
      throw new Error(`Invalid grade format: ${grade.grade}`);
    }

    // Check if subject is reasonable (not empty, not too long)
    if (grade.subject.length < 2 || grade.subject.length > 50) {
      throw new Error(`Suspicious subject length: ${grade.subject}`);
    }

    // Verify no SQL injection or XSS attempts
    if (/<|>|script|SELECT|DROP|INSERT/.test(grade.subject) || /<|>|script/.test(grade.grade)) {
      throw new Error(`Potential XSS/SQL injection in grades: ${grade.subject} / ${grade.grade}`);
    }
  }
}

/**
 * Verify response contains expected personalization
 * @param response - Optimus response to check
 * @param studentName - Expected student name
 */
export function verifyPersonalization(response: Partial<OptimusResponse>, studentName: string): void {
  const fullText = [
    response.greeting,
    response.strengthsRecognition,
    response.encouragementForWeaknesses,
    response.virtueConnection,
    response.inspirationalMessage,
    response.celebrationMessage
  ].join(' ');

  // Should mention the student by name or use "you/your"
  const hasPersonalization =
    fullText.toLowerCase().includes(studentName.toLowerCase()) ||
    fullText.toLowerCase().includes('you') ||
    fullText.toLowerCase().includes('your');

  if (!hasPersonalization) {
    throw new Error('Response lacks personalization for the student');
  }
}
