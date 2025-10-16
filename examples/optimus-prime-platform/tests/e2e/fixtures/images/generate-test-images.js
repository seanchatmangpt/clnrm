/**
 * Script to generate test image files
 * Run with: node tests/e2e/fixtures/images/generate-test-images.js
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Minimal valid PNG files (1x1 pixels in different colors)
const images = {
  'good-report-card.png': Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEBgIApD5fRAAAAABJRU5ErkJggg==',
    'base64'
  ), // Green pixel
  'excellent-report-card.png': Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPj/HwADBwIAMCbHYQAAAABJRU5ErkJggg==',
    'base64'
  ), // Blue pixel
  'needs-improvement-report-card.png': Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==',
    'base64'
  ), // Red pixel
  'minimal-report-card.png': Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==',
    'base64'
  ), // Yellow pixel
};

// Generate test images
Object.entries(images).forEach(([filename, buffer]) => {
  const filepath = path.join(__dirname, filename);
  fs.writeFileSync(filepath, buffer);
  console.log(`Generated: ${filename}`);
});

// Generate an invalid file for error testing
fs.writeFileSync(
  path.join(__dirname, 'invalid-file.txt'),
  'This is not an image file'
);
console.log('Generated: invalid-file.txt');

console.log('\nTest images generated successfully!');
