/**
 * Sample report card images for testing
 * These are base64-encoded 1x1 pixel images for testing purposes
 * In production, you would use actual report card images
 */

/**
 * Generate a test image file
 * Creates a simple colored rectangle with text indicating the image type
 */
export function generateTestImage(
  width: number = 800,
  height: number = 600,
  text: string = 'Test Report Card'
): string {
  // For testing, we'll use a minimal PNG data URL
  // In real tests, you can use actual image files or generate canvas images

  // This is a minimal 1x1 red pixel PNG
  const redPixel = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==';

  // This is a minimal 1x1 green pixel PNG
  const greenPixel = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEBgIApD5fRAAAAABJRU5ErkJggg==';

  // This is a minimal 1x1 blue pixel PNG
  const bluePixel = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPj/HwADBwIAMCbHYQAAAABJRU5ErkJggg==';

  return text.includes('good') ? greenPixel : text.includes('excellent') ? bluePixel : redPixel;
}

/**
 * Sample report card scenarios
 */
export const sampleReportCards = {
  good: {
    name: 'good-report-card.png',
    dataUrl: generateTestImage(800, 600, 'good report card'),
    description: 'A report card with good grades (mostly As and Bs)',
  },
  excellent: {
    name: 'excellent-report-card.png',
    dataUrl: generateTestImage(800, 600, 'excellent report card'),
    description: 'A report card with excellent grades (all As)',
  },
  needsImprovement: {
    name: 'needs-improvement-report-card.png',
    dataUrl: generateTestImage(800, 600, 'needs improvement report card'),
    description: 'A report card with lower grades (Cs and Ds)',
  },
  minimal: {
    name: 'minimal-report-card.png',
    dataUrl: generateTestImage(800, 600, 'minimal report card'),
    description: 'A report card with minimal information',
  },
  invalid: {
    name: 'invalid-file.txt',
    dataUrl: 'data:text/plain;base64,VGhpcyBpcyBub3QgYW4gaW1hZ2U=',
    description: 'An invalid file (not an image)',
  },
};

/**
 * Convert data URL to Blob for file upload
 */
export function dataUrlToBlob(dataUrl: string): Blob {
  const arr = dataUrl.split(',');
  const mime = arr[0].match(/:(.*?);/)?.[1] || 'image/png';
  const bstr = atob(arr[1]);
  let n = bstr.length;
  const u8arr = new Uint8Array(n);
  while (n--) {
    u8arr[n] = bstr.charCodeAt(n);
  }
  return new Blob([u8arr], { type: mime });
}

/**
 * Create a File object from data URL for testing
 */
export function createTestFile(
  dataUrl: string,
  filename: string = 'test-report-card.png'
): File {
  const blob = dataUrlToBlob(dataUrl);
  return new File([blob], filename, { type: blob.type });
}

/**
 * Get file path for Playwright file upload
 * @param scenario - The report card scenario
 * @returns Path to the test fixture file
 */
export function getReportCardFixturePath(scenario: keyof typeof sampleReportCards): string {
  return `tests/e2e/fixtures/images/${sampleReportCards[scenario].name}`;
}

/**
 * Save test images to disk for Playwright tests
 * This should be run during test setup
 */
export async function saveTestImagesToFixtures(): Promise<void> {
  // Note: This function is for documentation purposes
  // In practice, you would save actual PNG files to the fixtures/images directory
  // For this implementation, we'll use the data URLs directly in tests
  console.log('Test images would be saved to fixtures/images directory');
}
