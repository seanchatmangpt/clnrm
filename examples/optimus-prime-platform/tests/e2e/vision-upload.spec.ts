import { test, expect, Page } from '@playwright/test';
import { generateNDJSONForScenario, mockReportCardAnalysis, mockOptimusResponse } from './fixtures/mock-data';
import {
  uploadReportCardImage,
  clickAnalyzeButton,
  waitForAnalysisComplete,
  parseNDJSON,
  validateReportCardAnalysis,
  validateOptimusResponse,
  extractAnalysisFromPage,
  extractOptimusResponseFromPage,
  verifyNoFalsePositiveGrades,
  verifyPersonalization,
} from './utils/test-helpers';
import path from 'path';

/**
 * E2E Tests for Vision Upload Feature
 * Tests the complete flow of uploading a report card image and receiving AI analysis
 */

test.describe('Vision Upload - Report Card Analysis', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the upload page
    await page.goto('/upload-report');
    await expect(page.locator('h1:has-text("Upload Your Report Card")')).toBeVisible();
  });

  test('should display upload page with all elements', async ({ page }) => {
    // Verify page title
    await expect(page.locator('h1:has-text("Upload Your Report Card")')).toBeVisible();
    await expect(
      page.locator('text=Let Optimus Prime analyze your achievements with AI vision')
    ).toBeVisible();

    // Verify form elements
    await expect(page.locator('input[type="text"][placeholder*="Enter your name"]')).toBeVisible();
    await expect(page.locator('input[type="file"]')).toBeVisible();
    await expect(page.locator('button:has-text("Analyze with Vision AI")')).toBeDisabled();

    // Verify upload area text
    await expect(page.locator('text=Click to upload report card image')).toBeVisible();
  });

  test('should upload and preview report card image', async ({ page }) => {
    const studentName = 'Alex Johnson';
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Fill in student name
    await page.fill('input[type="text"][placeholder*="Enter your name"]', studentName);

    // Upload image
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);

    // Verify preview is displayed
    await expect(page.locator('img[alt="Report card preview"]')).toBeVisible();

    // Verify filename is shown
    await expect(page.locator('text=good-report-card.png')).toBeVisible();

    // Verify analyze button is enabled
    await expect(page.locator('button:has-text("Analyze with Vision AI")')).toBeEnabled();
  });

  test('should reject non-image files', async ({ page }) => {
    const invalidFilePath = path.join(
      process.cwd(),
      'tests/e2e/fixtures/images/invalid-file.txt'
    );

    // Try to upload invalid file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(invalidFilePath);

    // Should show error message
    await expect(page.locator('text=Please select an image file')).toBeVisible();

    // Analyze button should remain disabled
    await expect(page.locator('button:has-text("Analyze with Vision AI")')).toBeDisabled();
  });

  test('should analyze report card with good performance', async ({ page, context }) => {
    const studentName = 'Alex Johnson';
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Mock the API response
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload and analyze
    await page.fill('input[type="text"][placeholder*="Enter your name"]', studentName);
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);

    // Click analyze button
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for analyzing state
    await expect(page.locator('button:has-text("Analyzing")')).toBeVisible();

    // Wait for results
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Verify analysis data is displayed
    await expect(page.locator('text=Alex Johnson')).toBeVisible();
    await expect(page.locator('text=good')).toBeVisible();

    // Verify grades are shown
    await expect(page.locator('h3:has-text("Grades")')).toBeVisible();
    await expect(page.locator('text=Mathematics')).toBeVisible();

    // Verify strengths and weaknesses
    await expect(page.locator('h3:has-text("Strengths")')).toBeVisible();
    await expect(page.locator('h3:has-text("Growth Areas")')).toBeVisible();

    // Verify virtues
    await expect(page.locator('h3:has-text("Character Virtues")')).toBeVisible();

    // Verify Optimus Prime response
    await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible();
    await expect(page.locator('text=Greetings')).toBeVisible();
  });

  test('should handle excellent performance report card', async ({ page }) => {
    const imagePath = path.join(
      process.cwd(),
      'tests/e2e/fixtures/images/excellent-report-card.png'
    );

    // Mock the API response
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('excellent');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload and analyze
    await page.fill('input[type="text"][placeholder*="Enter your name"]', 'Emma Chen');
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Verify excellent performance
    await expect(page.locator('text=excellent')).toBeVisible();
    await expect(page.locator('text=A+')).toBeVisible();

    // Verify Optimus response is appropriately enthusiastic
    await expect(page.locator('text=excellence')).toBeVisible();
    await expect(page.locator('text=Magnificent')).toBeVisible();
  });

  test('should handle needs improvement report card with encouragement', async ({ page }) => {
    const imagePath = path.join(
      process.cwd(),
      'tests/e2e/fixtures/images/needs-improvement-report-card.png'
    );

    // Mock the API response
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('needs-improvement');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload and analyze
    await page.fill('input[type="text"][placeholder*="Enter your name"]', 'Jordan Smith');
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Verify needs improvement performance
    await expect(page.locator('text=needs improvement')).toBeVisible();

    // Verify encouraging message from Optimus
    await expect(page.locator('text=potential')).toBeVisible();
    await expect(page.locator('text=grow')).toBeVisible();

    // Should still show actionable advice
    await expect(page.locator('h3:has-text("Advice from Optimus")')).toBeVisible();
  });

  test('should validate analysis data structure from API', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Intercept API response and validate structure
    let responseData: any = null;

    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      responseData = parseNDJSON(ndjsonResponse);
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Validate response structure
    expect(responseData).toBeTruthy();
    expect(responseData.length).toBe(2); // analysis and response

    const analysisObj = responseData.find((r: any) => r.type === 'analysis');
    const responseObj = responseData.find((r: any) => r.type === 'response');

    expect(analysisObj).toBeTruthy();
    expect(responseObj).toBeTruthy();

    // Validate analysis structure
    expect(() => validateReportCardAnalysis(analysisObj.data)).not.toThrow();

    // Validate Optimus response structure
    expect(() => validateOptimusResponse(responseObj.data)).not.toThrow();
  });

  test('should detect false positives in grades', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Mock response with suspicious data
    const suspiciousResponse = generateNDJSONForScenario('good');
    const parsed = parseNDJSON(suspiciousResponse);
    const analysis = parsed.find((r: any) => r.type === 'analysis').data;

    await page.route('**/api/vision/analyze-report-card', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: suspiciousResponse,
      });
    });

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Extract and validate grades
    const extractedAnalysis = await extractAnalysisFromPage(page);

    // Verify no false positive grades
    expect(() => verifyNoFalsePositiveGrades(extractedAnalysis.grades || [])).not.toThrow();
  });

  test('should verify personalization in Optimus response', async ({ page }) => {
    const studentName = 'Taylor Swift';
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload with student name
    await page.fill('input[type="text"][placeholder*="Enter your name"]', studentName);
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible({ timeout: 30000 });

    // Extract Optimus response
    const optimusResponse = await extractOptimusResponseFromPage(page);

    // Verify personalization (should mention student or use "you/your")
    expect(() => verifyPersonalization(optimusResponse, studentName)).not.toThrow();
  });

  test('should handle API errors gracefully', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Mock API error
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal server error' }),
      });
    });

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Should show error message
    await expect(page.locator('text=Failed to analyze report card')).toBeVisible({ timeout: 10000 });

    // Analyze button should be re-enabled
    await expect(page.locator('button:has-text("Analyze with Vision AI")')).toBeEnabled();
  });

  test('should handle network timeout', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Mock slow/hanging API
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      await new Promise(resolve => setTimeout(resolve, 5000));
      await route.abort('timedout');
    });

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Should show error message
    await expect(page.locator('text=Failed to analyze report card')).toBeVisible({ timeout: 10000 });
  });

  test('should allow reset and re-upload', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Mock API response
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // First upload
    await page.fill('input[type="text"][placeholder*="Enter your name"]', 'First Student');
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Click reset
    await page.locator('button:has-text("Reset")').click();

    // Verify form is reset
    await expect(page.locator('img[alt="Report card preview"]')).not.toBeVisible();
    await expect(page.locator('button:has-text("Analyze with Vision AI")')).toBeDisabled();
    await expect(page.locator('h2:has-text("Report Card Analysis")')).not.toBeVisible();

    // Can upload again
    await page.fill('input[type="text"][placeholder*="Enter your name"]', 'Second Student');
    await fileInput.setInputFiles(imagePath);
    await expect(page.locator('button:has-text("Analyze with Vision AI")')).toBeEnabled();
  });

  test('should work without student name', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload without student name
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Should still work
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });
    await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible();
  });

  test('should stream NDJSON responses progressively', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    // Mock streaming response
    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const analysisLine = generateNDJSONForScenario('good').split('\n')[0];
      const responseLine = generateNDJSONForScenario('good').split('\n')[1];

      // Simulate streaming by sending chunks
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: `${analysisLine}\n${responseLine}\n`,
      });
    });

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Analysis should appear first
    await expect(page.locator('h2:has-text("Report Card Analysis")')).toBeVisible({ timeout: 30000 });

    // Then Optimus response
    await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible({ timeout: 5000 });
  });

  test('should display all Optimus response sections', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for Optimus section
    await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible({ timeout: 30000 });

    // Verify all sections are present
    await expect(page.locator('h3:has-text("Your Strengths")')).toBeVisible();
    await expect(page.locator('h3:has-text("Room to Grow")')).toBeVisible();
    await expect(page.locator('h3:has-text("Character Connection")')).toBeVisible();
    await expect(page.locator('h3:has-text("Advice from Optimus")')).toBeVisible();

    // Verify greeting and inspirational message
    await expect(page.locator('[class*="bg-white/10"] p').first()).toBeVisible();
    await expect(page.locator('[class*="border-2 border-blue-400"] p')).toBeVisible();

    // Verify celebration message
    await expect(page.locator('.text-center p.text-2xl')).toBeVisible();
  });
});

test.describe('Vision Upload - Headless Mode', () => {
  test('should run in headless mode', async ({ page, browserName }) => {
    // This test verifies that tests run properly in headless mode
    await page.goto('/upload-report');
    await expect(page.locator('h1:has-text("Upload Your Report Card")')).toBeVisible();

    // Verify browser is chromium (as configured)
    expect(browserName).toBe('chromium');

    // Take screenshot to verify rendering works in headless
    const screenshot = await page.screenshot();
    expect(screenshot).toBeTruthy();
    expect(screenshot.length).toBeGreaterThan(0);
  });
});

test.describe('Vision Upload - Performance', () => {
  test('should complete analysis within acceptable time', async ({ page }) => {
    const imagePath = path.join(process.cwd(), 'tests/e2e/fixtures/images/good-report-card.png');

    await page.route('**/api/vision/analyze-report-card', async (route) => {
      const ndjsonResponse = generateNDJSONForScenario('good');
      await route.fulfill({
        status: 200,
        contentType: 'application/x-ndjson',
        body: ndjsonResponse,
      });
    });

    const startTime = Date.now();

    // Upload and analyze
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(imagePath);
    await page.locator('button:has-text("Analyze with Vision AI")').click();

    // Wait for results
    await expect(page.locator('h2:has-text("Optimus Prime")')).toBeVisible({ timeout: 30000 });

    const endTime = Date.now();
    const duration = endTime - startTime;

    // Should complete in under 5 seconds (with mocked API)
    expect(duration).toBeLessThan(5000);
  });
});
