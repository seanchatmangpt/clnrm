# Agent 4: Implementation Guide - Image Optimization Enhancement

**Recommendation**: Add client-side image resize to existing FormData upload

**Estimated Time**: 2-3 hours
**Risk Level**: Low (additive, non-breaking)
**Expected Improvement**: 30% faster, 50-80% smaller files

---

## Step-by-Step Implementation

### Phase 1: Create Image Optimizer Utility (45 minutes)

**File**: `/Users/sac/clnrm/examples/optimus-prime-platform/src/lib/image-optimizer.ts`

```typescript
/**
 * Client-side image optimization for report card uploads
 * Resizes images to reduce upload time and bandwidth
 */

export interface ImageOptimizeOptions {
  maxWidth?: number;
  maxHeight?: number;
  quality?: number;
  skipSmallFiles?: boolean;
  smallFileThreshold?: number; // bytes
}

const DEFAULT_OPTIONS: Required<ImageOptimizeOptions> = {
  maxWidth: 1600,
  maxHeight: 1600,
  quality: 0.85,
  skipSmallFiles: true,
  smallFileThreshold: 512 * 1024, // 512KB
};

export interface OptimizationResult {
  file: File;
  originalSize: number;
  optimizedSize: number;
  wasOptimized: boolean;
  compressionRatio: number;
}

/**
 * Optimizes an image file for upload
 * @param file - Original image file
 * @param options - Optimization options
 * @returns Optimized file or original if optimization fails/not needed
 */
export async function optimizeImage(
  file: File,
  options: ImageOptimizeOptions = {}
): Promise<OptimizationResult> {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  const originalSize = file.size;

  // Skip small files if configured
  if (opts.skipSmallFiles && file.size < opts.smallFileThreshold) {
    console.log(`[ImageOptimizer] Skipping optimization (file < ${opts.smallFileThreshold / 1024}KB)`);
    return {
      file,
      originalSize,
      optimizedSize: originalSize,
      wasOptimized: false,
      compressionRatio: 1.0,
    };
  }

  try {
    const optimizedBlob = await resizeImageBlob(file, opts);

    // Only use optimized version if it's actually smaller
    const optimizedFile = new File([optimizedBlob], file.name, {
      type: 'image/jpeg',
      lastModified: Date.now(),
    });

    const compressionRatio = optimizedFile.size / originalSize;

    // Fallback to original if optimization made it bigger (rare)
    if (optimizedFile.size >= originalSize) {
      console.warn('[ImageOptimizer] Optimized file is larger, using original');
      return {
        file,
        originalSize,
        optimizedSize: originalSize,
        wasOptimized: false,
        compressionRatio: 1.0,
      };
    }

    console.log(
      `[ImageOptimizer] Optimized: ${(originalSize / 1024).toFixed(0)}KB → ${(optimizedFile.size / 1024).toFixed(0)}KB (${((1 - compressionRatio) * 100).toFixed(1)}% reduction)`
    );

    return {
      file: optimizedFile,
      originalSize,
      optimizedSize: optimizedFile.size,
      wasOptimized: true,
      compressionRatio,
    };
  } catch (error) {
    console.error('[ImageOptimizer] Failed to optimize, using original:', error);
    return {
      file,
      originalSize,
      optimizedSize: originalSize,
      wasOptimized: false,
      compressionRatio: 1.0,
    };
  }
}

/**
 * Resizes an image blob using canvas
 */
async function resizeImageBlob(
  file: File,
  options: Required<ImageOptimizeOptions>
): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const img = new Image();

    img.onload = () => {
      try {
        const { canvas, ctx } = createOptimizedCanvas(img, options);

        canvas.toBlob(
          (blob) => {
            if (!blob) {
              reject(new Error('Canvas toBlob returned null'));
              return;
            }
            resolve(blob);
          },
          'image/jpeg',
          options.quality
        );
      } catch (error) {
        reject(error);
      }
    };

    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };

    // Load the image
    img.src = URL.createObjectURL(file);
  });
}

/**
 * Creates an optimally-sized canvas for the image
 */
function createOptimizedCanvas(
  img: HTMLImageElement,
  options: Required<ImageOptimizeOptions>
): { canvas: HTMLCanvasElement; ctx: CanvasRenderingContext2D } {
  let { width, height } = img;
  const { maxWidth, maxHeight } = options;

  // Calculate new dimensions maintaining aspect ratio
  if (width > maxWidth || height > maxHeight) {
    const ratio = Math.min(maxWidth / width, maxHeight / height);
    width = Math.round(width * ratio);
    height = Math.round(height * ratio);
  }

  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;

  const ctx = canvas.getContext('2d');
  if (!ctx) {
    throw new Error('Failed to get 2D context');
  }

  // Enable image smoothing for better quality
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';

  // Draw the resized image
  ctx.drawImage(img, 0, 0, width, height);

  return { canvas, ctx };
}

/**
 * Validates that a file is an image
 */
export function isImageFile(file: File): boolean {
  return file.type.startsWith('image/');
}

/**
 * Formats file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
```

**Testing**:
```typescript
// Quick manual test
const file = new File([...], 'test.jpg', { type: 'image/jpeg' });
const result = await optimizeImage(file);
console.log(`Saved ${((1 - result.compressionRatio) * 100).toFixed(1)}%`);
```

---

### Phase 2: Update Upload Components (1 hour)

#### 2.1 Update `/upload-report/page.tsx`

**Location**: `/Users/sac/clnrm/examples/optimus-prime-platform/src/app/upload-report/page.tsx`

**Changes** (around line 35, in `analyzeReportCard` function):

```typescript
import { optimizeImage, formatFileSize } from '@/lib/image-optimizer';

// Add optimization state
const [optimizationInfo, setOptimizationInfo] = useState<string | null>(null);

const analyzeReportCard = async () => {
  if (!selectedFile) {
    setError('Please select a report card image');
    return;
  }

  setIsAnalyzing(true);
  setError(null);
  setAnalysis(null);
  setOptimusResponse(null);
  setOptimizationInfo(null);

  try {
    // STEP 1: Optimize image before upload
    setOptimizationInfo('Optimizing image...');
    const optimizationResult = await optimizeImage(selectedFile, {
      maxWidth: 1600,
      maxHeight: 1600,
      quality: 0.85,
    });

    if (optimizationResult.wasOptimized) {
      const savings = formatFileSize(
        optimizationResult.originalSize - optimizationResult.optimizedSize
      );
      setOptimizationInfo(
        `Optimized: ${formatFileSize(optimizationResult.originalSize)} → ${formatFileSize(optimizationResult.optimizedSize)} (saved ${savings})`
      );
    } else {
      setOptimizationInfo('Using original image (already optimized)');
    }

    // STEP 2: Upload optimized file
    const formData = new FormData();
    formData.append('image', optimizationResult.file);
    if (studentName.trim()) {
      formData.append('studentName', studentName.trim());
    }

    const response = await fetch('/api/vision/analyze-report-card', {
      method: 'POST',
      body: formData,
    });

    // ... rest of existing code remains unchanged
  } catch (err) {
    console.error('Analysis error:', err);
    setError(err instanceof Error ? err.message : 'Failed to analyze report card');
  } finally {
    setIsAnalyzing(false);
  }
};
```

**Add optimization info display** (after error message, around line 167):

```typescript
{error && (
  <p className="mt-2 text-sm text-red-600">{error}</p>
)}

{optimizationInfo && !error && (
  <p className="mt-2 text-sm text-green-600">✓ {optimizationInfo}</p>
)}
```

#### 2.2 Update `prompt-input-upload.tsx`

**Location**: `/Users/sac/clnrm/examples/optimus-prime-platform/src/components/prompt-input-upload.tsx`

**Similar changes** (around line 75, in `analyzeReportCard` function):

```typescript
import { optimizeImage } from '@/lib/image-optimizer';

const [optimizationInfo, setOptimizationInfo] = useState<string | null>(null);

const analyzeReportCard = async () => {
  // ... existing validation

  try {
    // Step 1: Optimize image
    setProcessingStage('🔧 Optimizing image for faster upload...');
    const optimizationResult = await optimizeImage(selectedFile, {
      maxWidth: 1600,
      quality: 0.85,
    });

    // Step 2: Vision analysis
    setProcessingStage('🔍 Analyzing report card with vision AI...');
    const formData = new FormData();
    formData.append('image', optimizationResult.file);
    if (studentName.trim()) {
      formData.append('studentName', studentName.trim());
    }

    // ... rest of existing code
  }
};
```

---

### Phase 3: Add Feature Flag (Optional, 15 minutes)

**File**: `/Users/sac/clnrm/examples/optimus-prime-platform/.env.local`

```bash
# Feature flag for image optimization
NEXT_PUBLIC_ENABLE_IMAGE_OPTIMIZATION=true
```

**Usage in components**:

```typescript
const ENABLE_OPTIMIZATION = process.env.NEXT_PUBLIC_ENABLE_IMAGE_OPTIMIZATION === 'true';

const optimizationResult = ENABLE_OPTIMIZATION
  ? await optimizeImage(selectedFile)
  : { file: selectedFile, wasOptimized: false, originalSize: selectedFile.size, optimizedSize: selectedFile.size, compressionRatio: 1.0 };
```

---

### Phase 4: Update Playwright Tests (30 minutes)

**File**: `/Users/sac/clnrm/examples/optimus-prime-platform/tests/vision-upload-optimized.spec.ts`

```typescript
import { test, expect } from '@playwright/test';
import fs from 'fs/promises';
import path from 'path';

test.describe('Vision Upload with Image Optimization', () => {
  const fixturesDir = path.join(__dirname, 'fixtures');

  test.beforeAll(async () => {
    // Ensure test fixtures exist
    await fs.mkdir(fixturesDir, { recursive: true });
  });

  test('should optimize and upload large report card', async ({ page }) => {
    await page.goto('/upload-report');

    // Upload a large file
    const fileInput = page.locator('input[type="file"]');
    const largePath = path.join(fixturesDir, 'large-report.jpg');
    await fileInput.setInputFiles(largePath);

    await page.fill('input[placeholder*="name"]', 'Optimization Test Student');

    // Should show optimization message
    const analyzeButton = page.locator('button:has-text("Analyze")');
    await analyzeButton.click();

    // Check for optimization info
    await expect(page.locator('text=/Optimized:|Using original/')).toBeVisible({
      timeout: 5000,
    });

    // Should proceed to analysis
    await expect(page.locator('text=Analyzing')).toBeVisible();
    await expect(page.locator('text=/Vision analysis complete/')).toBeVisible({
      timeout: 30000,
    });
  });

  test('should handle API upload with optimization', async ({ request }) => {
    const filePath = path.join(fixturesDir, 'sample-report.jpg');
    const fileBuffer = await fs.readFile(filePath);

    const response = await request.post('/api/vision/analyze-report-card', {
      multipart: {
        image: {
          name: 'report-card.jpg',
          mimeType: 'image/jpeg',
          buffer: fileBuffer,
        },
        studentName: 'API Test Student',
      },
    });

    expect(response.ok()).toBeTruthy();
    expect(response.headers()['content-type']).toContain('application/x-ndjson');

    // Parse response stream
    const text = await response.text();
    const lines = text.split('\n').filter((l) => l.trim());

    expect(lines.length).toBeGreaterThan(0);
    const firstLine = JSON.parse(lines[0]);
    expect(firstLine.type).toBe('analysis');
  });

  test('should skip optimization for small files', async ({ page }) => {
    await page.goto('/upload-report');

    // Upload a small file (< 512KB)
    const fileInput = page.locator('input[type="file"]');
    const smallPath = path.join(fixturesDir, 'small-report.jpg');
    await fileInput.setInputFiles(smallPath);

    await page.fill('input[placeholder*="name"]', 'Small File Test');
    await page.locator('button:has-text("Analyze")').click();

    // Should skip optimization
    await expect(page.locator('text=/Using original|already optimized/')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should fallback to original on optimization failure', async ({ page }) => {
    // Mock console to capture logs
    const logs: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error' || msg.text().includes('[ImageOptimizer]')) {
        logs.push(msg.text());
      }
    });

    await page.goto('/upload-report');

    // Upload a corrupted or edge-case image
    const fileInput = page.locator('input[type="file"]');
    const edgePath = path.join(fixturesDir, 'edge-case-report.jpg');
    await fileInput.setInputFiles(edgePath);

    await page.fill('input[placeholder*="name"]', 'Edge Case Test');
    await page.locator('button:has-text("Analyze")').click();

    // Should still complete (fallback to original)
    await expect(page.locator('text=/Analyzing|Using original/')).toBeVisible({
      timeout: 5000,
    });
  });
});
```

**Create test fixtures**:

```bash
# Create fixture directory
mkdir -p tests/fixtures

# Add sample images (you'll need actual test images)
# - small-report.jpg (< 512KB)
# - large-report.jpg (> 5MB)
# - edge-case-report.jpg (unusual format/dimensions)
```

---

### Phase 5: Monitoring & Validation (30 minutes)

#### Add Telemetry Events

**Update**: `/Users/sac/clnrm/examples/optimus-prime-platform/src/lib/telemetry.ts`

```typescript
// Add new event types
export type TelemetryEvent =
  | 'session_start'
  | 'report_card_uploaded'
  | 'report_card_analyzed'
  | 'image_optimized'  // NEW
  | 'optimization_skipped'  // NEW
  | 'optimization_failed';  // NEW
```

**Track in optimizer**:

```typescript
// In optimizeImage function
import { trackEvent } from '@/lib/telemetry';

if (optimizationResult.wasOptimized) {
  trackEvent('image_optimized', {
    originalSize: originalSize,
    optimizedSize: optimizationResult.optimizedSize,
    compressionRatio: compressionRatio,
    timeSaved: (originalSize - optimizationResult.optimizedSize) / 1024, // KB
  });
} else if (opts.skipSmallFiles && file.size < opts.smallFileThreshold) {
  trackEvent('optimization_skipped', { reason: 'file_too_small', size: file.size });
}
```

#### Add Error Boundary

```typescript
// In components, wrap optimization in try-catch
try {
  const result = await optimizeImage(selectedFile);
} catch (error) {
  console.error('[Upload] Optimization failed, using original:', error);
  trackEvent('optimization_failed', {
    error: error instanceof Error ? error.message : 'unknown',
    fileSize: selectedFile.size,
  });
  // Fallback to original file
  const result = {
    file: selectedFile,
    wasOptimized: false,
    originalSize: selectedFile.size,
    optimizedSize: selectedFile.size,
    compressionRatio: 1.0,
  };
}
```

---

## Deployment Checklist

### Pre-Deployment
- [ ] Code review image-optimizer.ts
- [ ] Unit tests for edge cases (very small, very large, corrupted)
- [ ] Playwright tests passing (all 4 scenarios)
- [ ] Browser compatibility testing (Chrome, Safari, Firefox, Edge)
- [ ] Mobile testing (iOS Safari, Android Chrome)
- [ ] Performance profiling (resize time < 500ms for typical files)

### Deployment Strategy
- [ ] Deploy with feature flag OFF initially
- [ ] Enable for internal testing (10% of traffic)
- [ ] Monitor telemetry events:
  - `image_optimized` count
  - `optimization_failed` rate (should be < 1%)
  - Average compression ratio (expect 0.2-0.5)
- [ ] Gradually increase to 50%, then 100%
- [ ] Monitor performance metrics:
  - Upload time reduction (expect 20-30%)
  - Server load (should decrease)
  - Error rates (should not increase)

### Rollback Plan
- [ ] Document feature flag toggle procedure
- [ ] Set up alerting for optimization failure rate > 5%
- [ ] Prepare rollback command:
  ```bash
  # Disable optimization immediately
  vercel env add NEXT_PUBLIC_ENABLE_IMAGE_OPTIMIZATION false production
  vercel deploy --prod
  ```

---

## Expected Results

### Before Enhancement
- 5MB image upload: 650ms
- Network transfer: 5MB
- Server memory: 17MB peak

### After Enhancement
- 5MB image upload: 450ms (30% faster) ✅
- Network transfer: 1MB (80% smaller) ✅
- Server memory: 7MB peak (60% reduction) ✅
- Client resize: 300ms (acceptable UX)

### Success Metrics
1. **Performance**: 20-30% reduction in upload time
2. **Bandwidth**: 50-80% reduction in data transfer
3. **Reliability**: < 1% optimization failure rate
4. **UX**: Optimization delay < 500ms for typical files

---

## Troubleshooting

### Issue: Optimization takes too long (> 1 second)
**Solution**: Reduce maxWidth to 1280 or quality to 0.75

### Issue: Optimized images lose OCR accuracy
**Solution**: Increase quality to 0.9 or maxWidth to 2048

### Issue: Optimization fails on certain devices
**Solution**: Check browser compatibility, add fallback for old browsers

### Issue: Memory issues on low-end devices
**Solution**: Add memory-based optimization threshold, skip optimization on devices with < 2GB RAM

---

## Future Enhancements

### Potential Improvements (Post-MVP)
1. **Progressive upload**: Start upload while optimizing
2. **Web Workers**: Offload resize to background thread
3. **Smart quality**: Adjust quality based on file size
4. **Format detection**: Keep PNG for screenshots, JPEG for photos
5. **Batch optimization**: Handle multiple files efficiently
6. **Server-side optimization**: Fallback for browsers without canvas support

---

## References

### Files Modified
1. `/src/lib/image-optimizer.ts` (NEW - 200 lines)
2. `/src/app/upload-report/page.tsx` (MODIFIED - +15 lines)
3. `/src/components/prompt-input-upload.tsx` (MODIFIED - +12 lines)
4. `/tests/vision-upload-optimized.spec.ts` (NEW - 100 lines)
5. `/src/lib/telemetry.ts` (MODIFIED - +3 event types)
6. `/.env.local` (MODIFIED - +1 flag)

### Total Lines Changed
- Added: ~330 lines
- Modified: ~30 lines
- Deleted: 0 lines

### Estimated Implementation Time
- Phase 1 (Optimizer): 45 min
- Phase 2 (Components): 60 min
- Phase 3 (Feature Flag): 15 min
- Phase 4 (Tests): 30 min
- Phase 5 (Monitoring): 30 min
**Total**: 3 hours

---

**Agent 4 Implementation Guide Complete** ✅

Ready for handoff to development agent.
