# E2E Tests for Vision Upload Feature

This directory contains end-to-end tests for the vision upload feature using Playwright.

## Overview

The vision upload feature allows users to:
1. Upload a report card image
2. Receive AI-powered analysis of the report card
3. Get personalized feedback from Optimus Prime

## Test Structure

```
tests/e2e/
├── vision-upload.spec.ts    # Main test suite
├── utils/
│   └── test-helpers.ts       # Test utilities and helper functions
├── fixtures/
│   ├── mock-data.ts          # Mock API responses
│   ├── sample-report-cards.ts # Sample report card data
│   └── images/               # Test image files
│       ├── good-report-card.png
│       ├── excellent-report-card.png
│       ├── needs-improvement-report-card.png
│       ├── minimal-report-card.png
│       └── invalid-file.txt
├── reports/                  # Test reports (generated)
└── test-results/             # Test artifacts (generated)
```

## Running Tests

### Run all E2E tests
```bash
npm run test:e2e
```

### Run tests in headed mode (see browser)
```bash
npm run test:e2e:headed
```

### Run tests in UI mode (interactive)
```bash
npm run test:e2e:ui
```

### Run specific test file
```bash
npx playwright test vision-upload.spec.ts
```

### Run tests with debugging
```bash
npx playwright test --debug
```

### View test report
```bash
npx playwright show-report tests/e2e/reports/html
```

## Test Coverage

### Core Functionality Tests
- ✅ Display upload page with all elements
- ✅ Upload and preview report card image
- ✅ Reject non-image files
- ✅ Analyze report card with good performance
- ✅ Handle excellent performance report card
- ✅ Handle needs improvement report card with encouragement
- ✅ Work without student name

### Data Validation Tests
- ✅ Validate analysis data structure from API
- ✅ Detect false positives in grades
- ✅ Verify personalization in Optimus response
- ✅ Stream NDJSON responses progressively

### Error Handling Tests
- ✅ Handle API errors gracefully
- ✅ Handle network timeout
- ✅ Allow reset and re-upload

### UI/UX Tests
- ✅ Display all Optimus response sections
- ✅ Run in headless mode
- ✅ Complete analysis within acceptable time

## Key Features

### 1. False Positive Detection
Tests include validation to ensure:
- Grades are in valid format (A+, A, B+, etc. or percentages)
- Subject names are reasonable length
- No XSS/SQL injection attempts in data
- Data structure matches expected schema

### 2. Personalization Verification
Tests verify that Optimus Prime's response:
- Mentions the student by name or uses "you/your"
- Provides specific, personalized feedback
- Adapts tone to performance level

### 3. Stream Validation
Tests verify NDJSON streaming:
- Analysis data arrives first
- Optimus response arrives second
- Both are displayed progressively

### 4. Error Resilience
Tests verify graceful error handling:
- API errors show user-friendly messages
- Network timeouts are handled
- Form can be reset and retried

## Test Utilities

### Upload Helpers
- `uploadReportCardImage()` - Upload image and fill in student name
- `clickAnalyzeButton()` - Click analyze button and wait for processing
- `waitForAnalysisComplete()` - Wait for analysis to complete

### Data Parsers
- `parseNDJSON()` - Parse NDJSON stream response
- `extractAnalysisFromPage()` - Extract analysis data from rendered page
- `extractOptimusResponseFromPage()` - Extract Optimus response from rendered page

### Validators
- `validateReportCardAnalysis()` - Validate analysis data structure
- `validateOptimusResponse()` - Validate Optimus response structure
- `verifyNoFalsePositiveGrades()` - Ensure grades are valid and realistic
- `verifyPersonalization()` - Verify response contains personalization

## Mock Data

### Scenarios
- `good` - Good performance (As and Bs)
- `excellent` - Excellent performance (all As)
- `needs-improvement` - Lower performance (Cs and Ds)
- `minimal` - Minimal data
- `error` - Error response

### Generating Mock Data
```typescript
import { generateNDJSONForScenario } from './fixtures/mock-data';

const response = generateNDJSONForScenario('excellent');
```

## Configuration

Playwright is configured in `/playwright.config.ts` with:
- **Headless mode**: Enabled by default
- **Browser**: Chromium
- **Base URL**: http://localhost:3000
- **Timeout**: 60 seconds per test
- **Retries**: 2 on CI, 0 locally
- **Reports**: HTML, JSON, and list format

## Best Practices

1. **Always mock API responses** - Don't rely on external services in E2E tests
2. **Use data-testid attributes** - Makes selectors more reliable
3. **Test error scenarios** - Ensure graceful degradation
4. **Validate data structures** - Catch schema changes early
5. **Check for false positives** - Verify data quality
6. **Test personalization** - Ensure responses are tailored to users
7. **Keep tests independent** - Each test should run in isolation

## Debugging Tips

### View browser during test
```bash
npx playwright test --headed --debug
```

### Slow down test execution
```bash
npx playwright test --headed --slow-mo=1000
```

### Take screenshots
```typescript
await page.screenshot({ path: 'debug.png' });
```

### Print page content
```typescript
console.log(await page.content());
```

### Use Playwright Inspector
```bash
npx playwright test --debug
```

## Continuous Integration

Tests are configured to run automatically on CI with:
- Headless mode
- Single worker (no parallel execution)
- 2 retries per test
- Full trace on first retry

## Troubleshooting

### Tests fail with "page not found"
- Ensure dev server is running: `npm run dev`
- Check BASE_URL in config

### Tests fail with timeout
- Increase timeout in playwright.config.ts
- Check if API route is responding

### Image upload fails
- Verify images exist in fixtures/images/
- Run: `node tests/e2e/fixtures/images/generate-test-images.js`

### Tests pass locally but fail on CI
- Check for race conditions
- Verify all dependencies are installed
- Ensure Playwright browsers are installed

## Future Enhancements

- [ ] Add visual regression testing
- [ ] Test with real report card images
- [ ] Add accessibility testing
- [ ] Test mobile viewports
- [ ] Add performance benchmarks
- [ ] Test with multiple languages
- [ ] Add API contract testing
- [ ] Test offline behavior

## Resources

- [Playwright Documentation](https://playwright.dev)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Playwright API Reference](https://playwright.dev/docs/api/class-playwright)
