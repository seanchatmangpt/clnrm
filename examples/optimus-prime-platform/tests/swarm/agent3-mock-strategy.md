# London School TDD Mock Strategy
## Vision Upload System - Comprehensive Mocking Strategy

**Agent**: Agent 3 - London TDD Mock Strategist
**Date**: 2025-10-16
**System**: Optimus Prime Platform - Vision Analysis Features

---

## Executive Summary

This document defines a comprehensive mocking strategy following **London School TDD principles** for the vision upload and report card analysis system. Every external dependency will be mocked to achieve complete unit test isolation, with behavior verification over state testing.

---

## 1. External Dependencies Inventory

### 1.1 AI/ML Model Dependencies

#### A. `ollama` Provider (ollama-ai-provider-v2)
**Location**: Used in all vision and chat routes
**Purpose**: Provides AI model access for vision analysis and text generation

**Interfaces to Mock**:
```typescript
// Mock interface for ollama function
interface MockOllamaProvider {
  (modelName: string): MockModelInstance;
}

interface MockModelInstance {
  modelName: string;
  capabilities: string[];
}
```

**Collaborations**:
- Called by `streamObject()` and `streamText()` functions
- Returns model instances for qwen2.5-vl:latest, qwen3-coder:30b
- No direct network calls (handled by AI SDK)

---

#### B. Vercel AI SDK - `streamObject()`
**Location**: `/src/app/api/vision/analyze-report-card/route.ts`, `/src/app/api/vision/evaluate-with-reasoning/route.ts`
**Purpose**: Streams structured objects from vision models

**Mock Interface**:
```typescript
interface MockStreamObjectParams {
  model: any;
  schema: z.ZodSchema;
  messages?: Array<{role: string; content: Array<{type: string; text?: string; image?: string}>}>;
  prompt?: string;
  mode: 'json';
}

interface MockStreamObjectResult<T> {
  object: Promise<T>;
  partialObjectStream: AsyncIterable<Partial<T>>;
}

// Mock function signature
function mockStreamObject<T>(params: MockStreamObjectParams): MockStreamObjectResult<T>
```

**Collaborations**:
- Receives model instance from `ollama()`
- Receives Zod schema for validation
- Processes image data URLs or text prompts
- Returns async stream of partial objects
- Final object matches schema type

**Stub Response Examples**:
```typescript
// Vision analysis stub
const stubReportCardAnalysis: ReportCardAnalysis = {
  documentType: "report card",
  studentName: "Alice Johnson",
  grades: [
    { subject: "Math", grade: "A", score: 95 },
    { subject: "Science", grade: "B+", score: 88 }
  ],
  overallPerformance: "excellent",
  strengths: ["Problem solving", "Critical thinking"],
  weaknesses: ["Time management"],
  teacherComments: "Excellent work ethic",
  achievements: ["Science fair winner"],
  virtuesDetected: ["wisdom", "courage"]
};

// Evaluation stub
const stubEvaluation = {
  reasoning: {
    academicAnalysis: "Strong performance across all subjects...",
    characterAssessment: "Demonstrates wisdom and courage...",
    growthOpportunities: "Could improve time management...",
    strengthsRecognition: "Exceptional problem-solving abilities..."
  },
  evaluation: {
    overallGrade: "excellent" as const,
    virtuesMastered: ["wisdom", "courage"],
    areasToFocus: ["Time management", "Collaboration skills"],
    encouragement: "Your dedication is inspiring...",
    actionableAdvice: [
      "Create a daily study schedule",
      "Join study groups for collaboration"
    ],
    reward: {
      type: "achievement_badge",
      description: "Master of Wisdom",
      unlockMessage: "You've unlocked the Wisdom Master badge!"
    }
  }
};
```

---

#### C. Vercel AI SDK - `streamText()`
**Location**: `/src/app/api/chat/route.ts`
**Purpose**: Streams text responses for chat functionality

**Mock Interface**:
```typescript
interface MockStreamTextParams {
  model: any;
  system?: string;
  messages?: Array<{role: 'user' | 'assistant'; content: string}>;
  prompt?: string;
}

interface MockStreamTextResult {
  textStream: AsyncIterable<string>;
}

function mockStreamText(params: MockStreamTextParams): MockStreamTextResult
```

**Stub Response Examples**:
```typescript
const stubChatChunks = [
  "Greetings, ",
  "young one. ",
  "Your courage ",
  "inspires us all."
];

// Streaming behavior simulation
async function* stubTextStream() {
  for (const chunk of stubChatChunks) {
    yield chunk;
  }
}
```

---

### 1.2 Browser APIs

#### A. File API
**Purpose**: Handle image file uploads from browser

**Interfaces to Mock**:
```typescript
// File object
interface MockFile extends Blob {
  name: string;
  lastModified: number;
  size: number;
  type: string;
  arrayBuffer(): Promise<ArrayBuffer>;
}

// FileReader (if used)
interface MockFileReader {
  readAsDataURL(blob: Blob): void;
  result: string | ArrayBuffer | null;
  onload: ((event: ProgressEvent) => void) | null;
  onerror: ((event: ProgressEvent) => void) | null;
}
```

**Stub Response Examples**:
```typescript
const stubImageFile = {
  name: "report-card.jpg",
  size: 1024000,
  type: "image/jpeg",
  lastModified: Date.now(),
  arrayBuffer: jest.fn().mockResolvedValue(
    new ArrayBuffer(1024000)
  )
};
```

---

#### B. FormData API
**Purpose**: Package file uploads for POST requests

**Mock Interface**:
```typescript
interface MockFormData {
  get(name: string): FormDataEntryValue | null;
  set(name: string, value: string | Blob): void;
  append(name: string, value: string | Blob): void;
  delete(name: string): void;
  has(name: string): boolean;
}
```

**Stub Examples**:
```typescript
const mockFormData = {
  entries: new Map<string, FormDataEntryValue>(),
  get: jest.fn((name: string) => mockFormData.entries.get(name) || null),
  set: jest.fn((name: string, value: FormDataEntryValue) => {
    mockFormData.entries.set(name, value);
  }),
  append: jest.fn((name: string, value: FormDataEntryValue) => {
    mockFormData.entries.set(name, value);
  })
};
```

---

#### C. Fetch API
**Purpose**: HTTP requests to API endpoints

**Mock Interface**:
```typescript
interface MockResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: Headers;
  body: ReadableStream<Uint8Array> | null;
  json(): Promise<any>;
  text(): Promise<string>;
}

interface MockFetch {
  (url: string, init?: RequestInit): Promise<MockResponse>;
}
```

**Stub Examples**:
```typescript
// Success response
const stubSuccessResponse: MockResponse = {
  ok: true,
  status: 200,
  statusText: "OK",
  headers: new Headers({
    'Content-Type': 'application/x-ndjson',
    'X-Student-Name': 'Alice Johnson'
  }),
  body: createMockReadableStream(stubNDJSONChunks),
  json: jest.fn().mockResolvedValue(stubAnalysisData),
  text: jest.fn().mockResolvedValue(stubNDJSONText)
};

// Error response
const stubErrorResponse: MockResponse = {
  ok: false,
  status: 500,
  statusText: "Internal Server Error",
  headers: new Headers(),
  body: null,
  json: jest.fn().mockResolvedValue({ error: "Analysis failed" }),
  text: jest.fn().mockResolvedValue("Internal Server Error")
};
```

---

#### D. ReadableStream API
**Purpose**: Handle streaming responses (NDJSON format)

**Mock Interface**:
```typescript
interface MockReadableStreamReader<T> {
  read(): Promise<ReadableStreamReadResult<T>>;
  releaseLock(): void;
  closed: Promise<void>;
}

interface MockReadableStream<T> {
  getReader(): MockReadableStreamReader<T>;
  cancel(reason?: any): Promise<void>;
}

interface ReadableStreamReadResult<T> {
  done: boolean;
  value?: T;
}
```

**Stub Examples**:
```typescript
// NDJSON chunks for vision analysis
const stubNDJSONChunks = [
  '{"type":"analysis","data":{"documentType":"report card","studentName":"Alice"}}\n',
  '{"type":"response","data":{"greeting":"Greetings, Alice"}}\n',
  '{"type":"response","data":{"greeting":"Greetings, Alice","strengthsRecognition":"You excel in..."}}\n'
];

// Mock reader behavior
function createMockReader(chunks: string[]): MockReadableStreamReader<Uint8Array> {
  let index = 0;
  const encoder = new TextEncoder();

  return {
    read: jest.fn().mockImplementation(async () => {
      if (index >= chunks.length) {
        return { done: true, value: undefined };
      }
      const value = encoder.encode(chunks[index]);
      index++;
      return { done: false, value };
    }),
    releaseLock: jest.fn(),
    closed: Promise.resolve()
  };
}
```

---

### 1.3 Node.js APIs

#### A. Buffer
**Purpose**: Convert image ArrayBuffer to base64 for Ollama

**Mock Interface**:
```typescript
interface MockBuffer {
  from(data: ArrayBuffer | string): MockBufferInstance;
}

interface MockBufferInstance {
  toString(encoding: string): string;
}
```

**Stub Example**:
```typescript
const mockBuffer = {
  from: jest.fn().mockReturnValue({
    toString: jest.fn().mockReturnValue("base64encodedimagedata==")
  })
};
```

---

#### B. TextEncoder / TextDecoder
**Purpose**: Convert strings to/from Uint8Array for streaming

**Mock Interfaces**:
```typescript
interface MockTextEncoder {
  encode(input: string): Uint8Array;
}

interface MockTextDecoder {
  decode(input: Uint8Array): string;
}
```

**Stub Examples**:
```typescript
const mockTextEncoder = {
  encode: jest.fn((str: string) => {
    // Simplified: just track the call
    return new Uint8Array(Buffer.from(str));
  })
};

const mockTextDecoder = {
  decode: jest.fn((bytes: Uint8Array) => {
    return Buffer.from(bytes).toString('utf-8');
  })
};
```

---

### 1.4 OpenTelemetry APIs

#### A. Tracer
**Purpose**: Distributed tracing spans

**Mock Interface**:
```typescript
interface MockSpan {
  setAttributes(attributes: Record<string, any>): void;
  setStatus(status: {code: number; message?: string}): void;
  recordException(error: Error): void;
  end(): void;
}

interface MockTracer {
  startSpan(name: string): MockSpan;
}

// Global trace object
const mockTrace = {
  getTracer: jest.fn((name: string, version: string) => mockTracer)
};
```

**Verification Strategy**:
```typescript
// Verify span lifecycle
expect(mockTracer.startSpan).toHaveBeenCalledWith('POST /api/vision/analyze-report-card');
expect(mockSpan.setAttributes).toHaveBeenCalledWith({
  'vision.student_name': 'Alice Johnson',
  'vision.image_size': 1024000,
  'vision.image_type': 'image/jpeg'
});
expect(mockSpan.setStatus).toHaveBeenCalledWith({ code: SpanStatusCode.OK });
expect(mockSpan.end).toHaveBeenCalled();
```

---

#### B. Telemetry Functions
**Purpose**: Track analytics events

**Mock Interface**:
```typescript
interface MockTelemetry {
  trackEvent(eventName: string, payload: Record<string, any>): void;
  trackVirtue(virtue: string, achievement: string): void;
}
```

**Verification Strategy**:
```typescript
// Verify event tracking calls
expect(mockTrackEvent).toHaveBeenCalledWith("report_card_uploaded", {
  studentName: "Alice Johnson",
  imageSize: 1024000
});

expect(mockTrackEvent).toHaveBeenCalledWith("report_card_analyzed", {
  studentName: "Alice Johnson",
  performance: "excellent"
});
```

---

### 1.5 React State & Hooks

#### A. useState Hook
**Purpose**: Component state management

**Mock Interface**:
```typescript
// Mock useState to return controlled state
const mockUseState = <T>(initialValue: T): [T, jest.Mock] => {
  let state = initialValue;
  const setState = jest.fn((newValue: T | ((prev: T) => T)) => {
    if (typeof newValue === 'function') {
      state = (newValue as Function)(state);
    } else {
      state = newValue;
    }
  });
  return [state, setState];
};
```

**Verification Strategy**:
```typescript
// Verify state updates
const [messages, setMessages] = mockUseState<Message[]>([]);

// After user message
expect(setMessages).toHaveBeenCalledWith(
  expect.arrayContaining([
    expect.objectContaining({
      role: "user",
      content: "Test message"
    })
  ])
);
```

---

#### B. useEffect Hook
**Purpose**: Side effects on component mount/update

**Mock Interface**:
```typescript
const mockUseEffect = jest.fn((effect: () => void | (() => void), deps?: any[]) => {
  // In tests, can immediately invoke or track for verification
  const cleanup = effect();
  return cleanup;
});
```

---

## 2. Mock Interface Designs

### 2.1 Vision Analysis Service Mock

```typescript
/**
 * Complete mock for vision analysis workflow
 * Follows London School: mock ALL collaborators
 */
export interface VisionAnalysisServiceMock {
  // Dependencies (all mocked)
  ollamaProvider: jest.Mock<MockModelInstance, [string]>;
  streamObjectFn: jest.Mock<MockStreamObjectResult<any>, [MockStreamObjectParams]>;
  formDataParser: jest.Mock<FormData, [Request]>;
  bufferConverter: jest.Mock<string, [ArrayBuffer]>;
  tracer: MockTracer;
  telemetry: MockTelemetry;

  // Behavior verification points
  verifyVisionModelCalled(modelName: string, imageData: string): void;
  verifyAnalysisSchemaUsed(schema: z.ZodSchema): void;
  verifyStreamingBehavior(expectedChunks: number): void;
  verifyTelemetryTracked(events: string[]): void;
}

/**
 * Factory function for creating vision analysis mocks
 */
export function createVisionAnalysisMock(
  options: {
    analysisResult?: Partial<ReportCardAnalysis>;
    shouldFail?: boolean;
    streamDelayMs?: number;
  } = {}
): VisionAnalysisServiceMock {
  const {
    analysisResult = stubReportCardAnalysis,
    shouldFail = false,
    streamDelayMs = 0
  } = options;

  // Mock ollama provider
  const ollamaProvider = jest.fn((modelName: string) => ({
    modelName,
    capabilities: ['vision', 'text']
  }));

  // Mock streamObject with streaming behavior
  const streamObjectFn = jest.fn((params: MockStreamObjectParams) => {
    if (shouldFail) {
      throw new Error("Vision model error");
    }

    return {
      object: Promise.resolve(analysisResult as any),
      partialObjectStream: createPartialObjectStream(analysisResult, streamDelayMs)
    };
  });

  // Mock other dependencies
  const formDataParser = jest.fn();
  const bufferConverter = jest.fn().mockReturnValue("base64imagedata==");
  const tracer = createMockTracer();
  const telemetry = createMockTelemetry();

  return {
    ollamaProvider,
    streamObjectFn,
    formDataParser,
    bufferConverter,
    tracer,
    telemetry,

    verifyVisionModelCalled(modelName: string, imageData: string) {
      expect(ollamaProvider).toHaveBeenCalledWith(modelName);
      expect(streamObjectFn).toHaveBeenCalledWith(
        expect.objectContaining({
          model: expect.objectContaining({ modelName }),
          messages: expect.arrayContaining([
            expect.objectContaining({
              content: expect.arrayContaining([
                expect.objectContaining({ type: 'image', image: expect.stringContaining('base64') })
              ])
            })
          ])
        })
      );
    },

    verifyAnalysisSchemaUsed(schema: z.ZodSchema) {
      expect(streamObjectFn).toHaveBeenCalledWith(
        expect.objectContaining({ schema })
      );
    },

    verifyStreamingBehavior(expectedChunks: number) {
      // Verify stream was consumed
      expect(streamObjectFn).toHaveBeenCalled();
      const result = streamObjectFn.mock.results[0].value;
      expect(result.partialObjectStream).toBeDefined();
    },

    verifyTelemetryTracked(events: string[]) {
      events.forEach(eventName => {
        expect(telemetry.trackEvent).toHaveBeenCalledWith(
          eventName,
          expect.any(Object)
        );
      });
    }
  };
}
```

---

### 2.2 Chat Service Mock

```typescript
/**
 * Mock for chat service (child and executive modes)
 */
export interface ChatServiceMock {
  // Dependencies
  ollamaProvider: jest.Mock;
  streamTextFn: jest.Mock<MockStreamTextResult, [MockStreamTextParams]>;
  virtueDetector: jest.Mock<string, [string]>;
  metricsProvider: jest.Mock<MetricsData, []>;
  tracer: MockTracer;
  telemetry: MockTelemetry;

  // Behavior verification
  verifyChildModePrompt(virtue: string, messageHistory: number): void;
  verifyExecutiveModeContext(metrics: MetricsData): void;
  verifyResponseStreaming(expectedFormat: 'ollama-json' | 'ndjson'): void;
}

export function createChatServiceMock(
  options: {
    mode: 'child' | 'executive';
    detectedVirtue?: string;
    streamChunks?: string[];
  } = { mode: 'child' }
): ChatServiceMock {
  const {
    mode,
    detectedVirtue = 'courage',
    streamChunks = stubChatChunks
  } = options;

  const ollamaProvider = jest.fn((modelName: string) => ({ modelName }));

  const streamTextFn = jest.fn((params: MockStreamTextParams) => ({
    textStream: (async function* () {
      for (const chunk of streamChunks) {
        yield chunk;
      }
    })()
  }));

  const virtueDetector = jest.fn().mockReturnValue(detectedVirtue);
  const metricsProvider = jest.fn().mockReturnValue(stubMetricsData);
  const tracer = createMockTracer();
  const telemetry = createMockTelemetry();

  return {
    ollamaProvider,
    streamTextFn,
    virtueDetector,
    metricsProvider,
    tracer,
    telemetry,

    verifyChildModePrompt(virtue: string, messageHistory: number) {
      expect(virtueDetector).toHaveBeenCalled();
      expect(streamTextFn).toHaveBeenCalledWith(
        expect.objectContaining({
          system: expect.stringContaining(virtue),
          messages: expect.arrayOfSize(messageHistory)
        })
      );
    },

    verifyExecutiveModeContext(metrics: MetricsData) {
      expect(metricsProvider).toHaveBeenCalled();
      expect(streamTextFn).toHaveBeenCalledWith(
        expect.objectContaining({
          system: expect.stringContaining(`$${metrics.totals.revenue}`)
        })
      );
    },

    verifyResponseStreaming(expectedFormat: 'ollama-json' | 'ndjson') {
      expect(streamTextFn).toHaveBeenCalled();
      // Verify stream was converted to expected format
      // This would be tested in integration layer
    }
  };
}
```

---

### 2.3 File Upload Mock

```typescript
/**
 * Mock for browser file upload workflow
 */
export interface FileUploadMock {
  file: MockFile;
  formData: MockFormData;
  fileReader: MockFileReader;

  verifyFileValidation(expectedType: string, maxSize: number): void;
  verifyFormDataConstruction(expectedFields: string[]): void;
  verifyBase64Conversion(): void;
}

export function createFileUploadMock(
  options: {
    fileName?: string;
    fileType?: string;
    fileSize?: number;
    base64Data?: string;
  } = {}
): FileUploadMock {
  const {
    fileName = 'test-report-card.jpg',
    fileType = 'image/jpeg',
    fileSize = 1024000,
    base64Data = 'base64encodeddata=='
  } = options;

  const arrayBuffer = new ArrayBuffer(fileSize);

  const file: MockFile = {
    name: fileName,
    type: fileType,
    size: fileSize,
    lastModified: Date.now(),
    arrayBuffer: jest.fn().mockResolvedValue(arrayBuffer),
    slice: jest.fn(),
    stream: jest.fn(),
    text: jest.fn()
  } as any;

  const formData = {
    entries: new Map(),
    get: jest.fn(),
    set: jest.fn(),
    append: jest.fn(),
    delete: jest.fn(),
    has: jest.fn()
  };

  const fileReader = {
    readAsDataURL: jest.fn(),
    result: `data:${fileType};base64,${base64Data}`,
    onload: null,
    onerror: null,
    abort: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn()
  } as any;

  return {
    file,
    formData,
    fileReader,

    verifyFileValidation(expectedType: string, maxSize: number) {
      expect(file.type).toBe(expectedType);
      expect(file.size).toBeLessThanOrEqual(maxSize);
    },

    verifyFormDataConstruction(expectedFields: string[]) {
      expectedFields.forEach(field => {
        expect(formData.set).toHaveBeenCalledWith(field, expect.anything());
      });
    },

    verifyBase64Conversion() {
      expect(file.arrayBuffer).toHaveBeenCalled();
      expect(fileReader.result).toMatch(/^data:image\/[a-z]+;base64,/);
    }
  };
}
```

---

## 3. Stub Response Design

### 3.1 Vision Analysis Stub Responses

```typescript
/**
 * Stub responses for different analysis scenarios
 */
export const VISION_STUBS = {
  // Excellent student
  excellentStudent: {
    documentType: "report card",
    studentName: "Alice Johnson",
    grades: [
      { subject: "Math", grade: "A", score: 95 },
      { subject: "Science", grade: "A", score: 94 },
      { subject: "English", grade: "A-", score: 92 }
    ],
    overallPerformance: "excellent" as const,
    strengths: ["Critical thinking", "Problem solving", "Leadership"],
    weaknesses: ["Time management"],
    teacherComments: "Outstanding student with excellent work ethic",
    achievements: ["Science fair winner", "Math olympiad participant"],
    virtuesDetected: ["wisdom", "courage", "teamwork"]
  },

  // Struggling student
  strugglingStudent: {
    documentType: "report card",
    studentName: "Bob Smith",
    grades: [
      { subject: "Math", grade: "C", score: 72 },
      { subject: "Science", grade: "C-", score: 68 },
      { subject: "English", grade: "B", score: 85 }
    ],
    overallPerformance: "needs improvement" as const,
    strengths: ["Creative writing", "Effort"],
    weaknesses: ["Math skills", "Focus", "Study habits"],
    teacherComments: "Needs to focus more in class and complete homework",
    achievements: ["Improved from last quarter"],
    virtuesDetected: ["courage"]
  },

  // Average student
  averageStudent: {
    documentType: "report card",
    studentName: "Charlie Davis",
    grades: [
      { subject: "Math", grade: "B", score: 85 },
      { subject: "Science", grade: "B-", score: 82 },
      { subject: "English", grade: "B+", score: 88 }
    ],
    overallPerformance: "good" as const,
    strengths: ["Consistent effort", "Good behavior"],
    weaknesses: ["Could participate more in class"],
    teacherComments: "Solid student with room to excel",
    achievements: ["Perfect attendance"],
    virtuesDetected: ["teamwork", "compassion"]
  }
};

/**
 * Stub responses for Optimus evaluation
 */
export const EVALUATION_STUBS = {
  excellent: {
    reasoning: {
      academicAnalysis: "This student demonstrates exceptional mastery across all subjects...",
      characterAssessment: "Showing wisdom beyond years and courage to tackle challenges...",
      growthOpportunities: "Could benefit from time management strategies...",
      strengthsRecognition: "Natural leadership abilities and problem-solving skills..."
    },
    evaluation: {
      overallGrade: "excellent" as const,
      virtuesMastered: ["wisdom", "courage", "teamwork"],
      areasToFocus: ["Time management", "Work-life balance"],
      encouragement: "Your dedication and intelligence inspire us all...",
      actionableAdvice: [
        "Create a weekly study schedule to manage time better",
        "Join advanced programs to challenge yourself",
        "Mentor younger students to develop leadership"
      ],
      reward: {
        type: "elite_achievement",
        description: "Prime Scholar Badge",
        unlockMessage: "You've achieved Prime Scholar status! This badge represents wisdom and excellence."
      }
    }
  },

  needsImprovement: {
    reasoning: {
      academicAnalysis: "There are some challenges in core subjects that need attention...",
      characterAssessment: "Showing courage by continuing to try despite difficulties...",
      growthOpportunities: "Developing stronger study habits and seeking help when needed...",
      strengthsRecognition: "Creative thinking and perseverance are valuable strengths..."
    },
    evaluation: {
      overallGrade: "needs improvement" as const,
      virtuesMastered: ["courage"],
      areasToFocus: ["Math fundamentals", "Study habits", "Classroom focus"],
      encouragement: "Every Autobot started somewhere. Your courage to keep trying is the first step...",
      actionableAdvice: [
        "Ask teachers for extra help after class",
        "Create a quiet study space at home",
        "Break study sessions into 20-minute chunks",
        "Work with a study buddy for accountability"
      ],
      reward: {
        type: "improvement_badge",
        description: "Courage to Grow",
        unlockMessage: "You've earned the Courage to Grow badge for facing your challenges head-on!"
      }
    }
  }
};
```

---

### 3.2 Streaming Response Stubs

```typescript
/**
 * Simulate streaming behavior for NDJSON responses
 */
export const STREAMING_STUBS = {
  // Vision analysis NDJSON stream
  visionAnalysisStream: [
    '{"type":"analysis","data":{"documentType":"report card"}}\n',
    '{"type":"analysis","data":{"documentType":"report card","studentName":"Alice"}}\n',
    '{"type":"analysis","data":{"documentType":"report card","studentName":"Alice","grades":[{"subject":"Math","grade":"A"}]}}\n',
    '{"type":"response","data":{"greeting":"Greetings, Alice"}}\n',
    '{"type":"response","data":{"greeting":"Greetings, Alice","strengthsRecognition":"Your excellence in mathematics..."}}\n'
  ],

  // Evaluation stream
  evaluationStream: [
    '{"reasoning":{"academicAnalysis":"Analyzing grades..."}}\n',
    '{"reasoning":{"academicAnalysis":"Analyzing grades...","characterAssessment":"Assessing virtues..."}}\n',
    '{"evaluation":{"overallGrade":"excellent"}}\n',
    '{"evaluation":{"overallGrade":"excellent","virtuesMastered":["wisdom"]}}\n'
  ],

  // Chat stream (Ollama format)
  chatStream: [
    '{"model":"qwen3-coder:30b","message":{"role":"assistant","content":"Greetings, "},"done":false}\n',
    '{"model":"qwen3-coder:30b","message":{"role":"assistant","content":"young one. "},"done":false}\n',
    '{"model":"qwen3-coder:30b","message":{"role":"assistant","content":"Your courage "},"done":false}\n',
    '{"model":"qwen3-coder:30b","message":{"role":"assistant","content":"inspires us all."},"done":false}\n',
    '{"model":"qwen3-coder:30b","message":{"role":"assistant","content":""},"done":true}\n'
  ]
};

/**
 * Helper to create mock ReadableStream from chunks
 */
export function createMockStreamFromChunks(chunks: string[]): ReadableStream<Uint8Array> {
  let index = 0;
  const encoder = new TextEncoder();

  return new ReadableStream({
    pull(controller) {
      if (index >= chunks.length) {
        controller.close();
        return;
      }
      controller.enqueue(encoder.encode(chunks[index]));
      index++;
    }
  });
}
```

---

### 3.3 Error Scenario Stubs

```typescript
/**
 * Stub responses for error conditions
 */
export const ERROR_STUBS = {
  // Vision model timeout
  visionModelTimeout: new Error("Vision model request timed out after 30s"),

  // Invalid image format
  invalidImageFormat: {
    status: 400,
    error: "Invalid image format. Expected JPEG or PNG"
  },

  // File too large
  fileTooLarge: {
    status: 413,
    error: "File size exceeds maximum of 10MB"
  },

  // Model not available
  modelUnavailable: {
    status: 503,
    error: "Vision model qwen2.5-vl:latest is not available"
  },

  // Network error
  networkError: new Error("Network request failed"),

  // Schema validation error
  schemaValidationError: new z.ZodError([
    {
      code: "invalid_type",
      expected: "string",
      received: "undefined",
      path: ["studentName"],
      message: "Student name is required"
    }
  ]),

  // Stream interrupted
  streamInterrupted: new Error("Stream was interrupted before completion")
};
```

---

## 4. Spy & Verification Strategy

### 4.1 Interaction Verification Pattern

Following London School TDD, we verify **how objects collaborate**, not what state they hold.

```typescript
/**
 * Verification helpers for London School TDD
 */

// Verify call order
export function verifyCallOrder(mocks: jest.Mock[], expectedOrder: string[]) {
  const callOrder = mocks.flatMap(mock =>
    mock.mock.invocationCallOrder.map(order => ({
      order,
      name: mock.getMockName()
    }))
  ).sort((a, b) => a.order - b.order);

  const actualOrder = callOrder.map(call => call.name);
  expect(actualOrder).toEqual(expectedOrder);
}

// Verify collaboration sequence
export function verifyCollaborationSequence(
  primaryMock: jest.Mock,
  dependencyMocks: Record<string, jest.Mock>,
  expectedSequence: Array<{mock: string; args?: any[]}>
) {
  expectedSequence.forEach((step, index) => {
    const mock = dependencyMocks[step.mock];
    expect(mock).toHaveBeenCalledTimes(expect.any(Number));

    if (step.args) {
      expect(mock).toHaveBeenCalledWith(...step.args);
    }
  });
}

// Verify streaming behavior
export async function verifyStreamBehavior(
  streamMock: AsyncIterable<any>,
  expectedChunks: any[],
  verifier: (chunk: any, index: number) => void
) {
  let index = 0;
  for await (const chunk of streamMock) {
    verifier(chunk, index);
    index++;
  }
  expect(index).toBe(expectedChunks.length);
}
```

---

### 4.2 Vision Analysis Verification Examples

```typescript
/**
 * Example: Verify vision analysis workflow
 */
describe('Vision Analysis Workflow', () => {
  it('should coordinate vision model and response generation in correct sequence', async () => {
    // Arrange
    const mocks = createVisionAnalysisMock();
    const visionService = new VisionAnalysisService(
      mocks.ollamaProvider,
      mocks.streamObjectFn,
      mocks.bufferConverter,
      mocks.tracer,
      mocks.telemetry
    );

    // Act
    await visionService.analyzeReportCard(stubImageFile, 'Alice');

    // Assert - Verify collaboration sequence
    verifyCollaborationSequence(
      mocks.streamObjectFn,
      {
        buffer: mocks.bufferConverter,
        ollama: mocks.ollamaProvider,
        tracer: mocks.tracer.startSpan,
        telemetry: mocks.telemetry.trackEvent
      },
      [
        { mock: 'tracer', args: ['POST /api/vision/analyze-report-card'] },
        { mock: 'buffer' }, // Convert image to base64
        { mock: 'ollama', args: ['qwen2.5-vl:latest'] }, // Get vision model
        { mock: 'streamObject' }, // Analyze with vision
        { mock: 'ollama', args: ['qwen3-coder:30b'] }, // Get text model
        { mock: 'streamObject' }, // Generate response
        { mock: 'telemetry', args: ['report_card_analyzed', expect.any(Object)] },
        { mock: 'tracer' } // End span
      ]
    );
  });

  it('should pass image data URL to vision model', async () => {
    // Arrange
    const mocks = createVisionAnalysisMock();
    const visionService = new VisionAnalysisService(/* ... */);

    // Act
    await visionService.analyzeReportCard(stubImageFile, 'Alice');

    // Assert - Verify interaction with vision model
    mocks.verifyVisionModelCalled('qwen2.5-vl:latest', 'base64imagedata==');

    // Verify message structure
    expect(mocks.streamObjectFn).toHaveBeenCalledWith(
      expect.objectContaining({
        messages: expect.arrayContaining([
          expect.objectContaining({
            role: 'user',
            content: expect.arrayContaining([
              { type: 'text', text: expect.stringContaining('report card') },
              { type: 'image', image: expect.stringMatching(/^data:image\/jpeg;base64,/) }
            ])
          })
        ])
      })
    );
  });

  it('should stream partial objects to client', async () => {
    // Arrange
    const mocks = createVisionAnalysisMock({
      streamDelayMs: 10 // Simulate streaming delay
    });
    const visionService = new VisionAnalysisService(/* ... */);

    // Act
    const stream = await visionService.analyzeReportCard(stubImageFile, 'Alice');

    // Assert - Verify streaming behavior
    const chunks: any[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }

    // Verify progressive disclosure
    expect(chunks.length).toBeGreaterThan(1);
    expect(chunks[0]).toHaveProperty('type', 'analysis');
    expect(chunks[chunks.length - 1]).toHaveProperty('type', 'response');

    // Verify partial object structure
    const responseChunks = chunks.filter(c => c.type === 'response');
    expect(responseChunks[0].data).toMatchObject({
      greeting: expect.any(String)
    });
    expect(responseChunks[responseChunks.length - 1].data).toMatchObject({
      greeting: expect.any(String),
      strengthsRecognition: expect.any(String),
      actionableAdvice: expect.any(Array)
    });
  });
});
```

---

### 4.3 Error Handling Verification

```typescript
/**
 * Verify error handling and recovery
 */
describe('Vision Analysis Error Handling', () => {
  it('should record exception in telemetry when vision model fails', async () => {
    // Arrange
    const mocks = createVisionAnalysisMock({ shouldFail: true });
    const visionService = new VisionAnalysisService(/* ... */);

    // Act & Assert
    await expect(
      visionService.analyzeReportCard(stubImageFile, 'Alice')
    ).rejects.toThrow('Vision model error');

    // Verify error was recorded
    expect(mocks.tracer.span.recordException).toHaveBeenCalledWith(
      expect.objectContaining({
        message: 'Vision model error'
      })
    );

    expect(mocks.tracer.span.setStatus).toHaveBeenCalledWith({
      code: SpanStatusCode.ERROR,
      message: 'Vision model error'
    });
  });

  it('should clean up resources when stream is cancelled', async () => {
    // Arrange
    const mocks = createVisionAnalysisMock();
    const visionService = new VisionAnalysisService(/* ... */);

    // Act
    const stream = await visionService.analyzeReportCard(stubImageFile, 'Alice');
    await stream.cancel('Client disconnected');

    // Assert - Verify cleanup
    expect(mocks.tracer.span.end).toHaveBeenCalled();
    expect(mocks.streamObjectFn.mock.results[0].value.partialObjectStream.return).toHaveBeenCalled();
  });
});
```

---

### 4.4 Chat Service Verification

```typescript
/**
 * Verify chat service interactions
 */
describe('Chat Service Child Mode', () => {
  it('should detect virtue and include in system prompt', async () => {
    // Arrange
    const mocks = createChatServiceMock({
      mode: 'child',
      detectedVirtue: 'courage'
    });
    const chatService = new ChatService(/* ... */);

    // Act
    await chatService.handleChildChat([
      { role: 'user', content: 'I climbed the big tree today!' }
    ]);

    // Assert - Verify virtue detection collaboration
    expect(mocks.virtueDetector).toHaveBeenCalledWith('I climbed the big tree today!');

    // Verify virtue in system prompt
    mocks.verifyChildModePrompt('courage', 1);

    // Verify telemetry
    expect(mocks.telemetry.trackVirtue).toHaveBeenCalledWith(
      'courage',
      'I climbed the big tree today!'
    );
  });

  it('should include conversation history in model context', async () => {
    // Arrange
    const mocks = createChatServiceMock({ mode: 'child' });
    const chatService = new ChatService(/* ... */);
    const messageHistory = [
      { role: 'user', content: 'First message' },
      { role: 'assistant', content: 'First response' },
      { role: 'user', content: 'Second message' }
    ];

    // Act
    await chatService.handleChildChat(messageHistory);

    // Assert - Verify history passed to model
    expect(mocks.streamTextFn).toHaveBeenCalledWith(
      expect.objectContaining({
        messages: expect.arrayContaining([
          { role: 'user', content: 'First message' },
          { role: 'assistant', content: 'First response' },
          { role: 'user', content: 'Second message' }
        ])
      })
    );
  });

  it('should convert AI SDK stream to Ollama JSON format', async () => {
    // Arrange
    const mocks = createChatServiceMock({
      mode: 'child',
      streamChunks: ['Hello, ', 'brave one.']
    });
    const chatService = new ChatService(/* ... */);

    // Act
    const response = await chatService.handleChildChat([
      { role: 'user', content: 'Test' }
    ]);
    const reader = response.body!.getReader();
    const decoder = new TextDecoder();

    // Assert - Verify Ollama JSON format
    let chunk = await reader.read();
    let text = decoder.decode(chunk.value);
    let json = JSON.parse(text);

    expect(json).toMatchObject({
      model: 'qwen3-coder:30b',
      message: {
        role: 'assistant',
        content: expect.any(String)
      },
      done: false
    });
  });
});
```

---

## 5. Unit vs Integration Test Separation

### 5.1 Unit Tests (Pure London School - All Mocked)

**Scope**: Individual functions/modules with ALL dependencies mocked

**Examples**:

```typescript
// Unit test - Executive chat route handler
describe('Executive Chat Route Handler (Unit)', () => {
  it('should coordinate metrics retrieval and model streaming', async () => {
    // ALL dependencies mocked
    const mocks = {
      getMetrics: jest.fn().mockReturnValue(stubMetrics),
      getCorpData: jest.fn().mockReturnValue(stubCorpData),
      ollama: jest.fn().mockReturnValue({ modelName: 'qwen3-coder:30b' }),
      streamText: jest.fn().mockReturnValue({
        textStream: createMockTextStream(['Revenue is ', '$50k'])
      }),
      tracer: createMockTracer(),
      trackEvent: jest.fn()
    };

    // Inject all mocks
    const handler = createExecutiveChatHandler(mocks);

    // Test pure business logic
    const request = new Request('http://test.com/api/chat', {
      method: 'POST',
      body: JSON.stringify({
        mode: 'executive',
        messages: [{ role: 'user', content: 'What is revenue?' }]
      })
    });

    const response = await handler(request);

    // Verify interactions only
    expect(mocks.getMetrics).toHaveBeenCalled();
    expect(mocks.streamText).toHaveBeenCalledWith(
      expect.objectContaining({
        system: expect.stringContaining(`$${stubMetrics.totals.revenue}`)
      })
    );
  });
});

// Unit test - Virtue detection
describe('Virtue Detection (Unit)', () => {
  it('should identify courage keywords in text', () => {
    // No mocks needed - pure function
    const result = detectVirtue('I was brave and climbed the tree');
    expect(result).toBe('courage');
  });

  it('should default to courage when no keywords match', () => {
    const result = detectVirtue('Random text with no virtue keywords');
    expect(result).toBe('courage');
  });
});

// Unit test - Vision analysis schema validation
describe('Vision Analysis Schema (Unit)', () => {
  it('should validate complete report card analysis', () => {
    // Mock Zod schema behavior
    const result = reportCardAnalysisSchema.safeParse(stubReportCardAnalysis);

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.studentName).toBe('Alice Johnson');
      expect(result.data.virtuesDetected).toContain('wisdom');
    }
  });

  it('should reject invalid performance values', () => {
    const invalidData = {
      ...stubReportCardAnalysis,
      overallPerformance: 'invalid'
    };

    const result = reportCardAnalysisSchema.safeParse(invalidData);
    expect(result.success).toBe(false);
  });
});
```

---

### 5.2 Integration Tests (Real Browser + Mocked External Services)

**Scope**: End-to-end workflows with real DOM/browser APIs but mocked external services

**Examples**:

```typescript
// Integration test - File upload flow
describe('Report Card Upload Flow (Integration)', () => {
  it('should upload file and display analysis results', async () => {
    // Real browser APIs (in test environment)
    const { getByLabelText, getByRole, findByText } = render(<UploadReportPage />);

    // Mock only external API
    global.fetch = jest.fn().mockImplementation((url) => {
      if (url.includes('/api/vision/analyze-report-card')) {
        return Promise.resolve({
          ok: true,
          status: 200,
          headers: new Headers({ 'Content-Type': 'application/x-ndjson' }),
          body: createMockStreamFromChunks(STREAMING_STUBS.visionAnalysisStream)
        });
      }
    });

    // Real file upload interaction
    const file = new File(['test'], 'report-card.jpg', { type: 'image/jpeg' });
    const input = getByLabelText(/upload report card/i);
    await userEvent.upload(input, file);

    const submitButton = getByRole('button', { name: /analyze/i });
    await userEvent.click(submitButton);

    // Verify real DOM updates
    const analysisResult = await findByText(/Alice Johnson/i);
    expect(analysisResult).toBeInTheDocument();

    const optimusGreeting = await findByText(/Greetings, Alice/i);
    expect(optimusGreeting).toBeInTheDocument();
  });
});

// Integration test - Chat message flow
describe('Child Chat Flow (Integration)', () => {
  it('should send message and stream response', async () => {
    const { getByPlaceholderText, getByRole, findByText } = render(<ChildChatPage />);

    // Mock API endpoint only
    global.fetch = jest.fn().mockResolvedValue({
      ok: true,
      body: createMockStreamFromChunks(STREAMING_STUBS.chatStream),
      headers: new Headers({
        'X-Virtue': 'courage',
        'X-Reward-Url': 'https://example.com/rewards/courage-shield.mp4'
      })
    });

    // Real user interaction
    const input = getByPlaceholderText(/share your achievement/i);
    await userEvent.type(input, 'I climbed the big tree!');

    const sendButton = getByRole('button', { name: /send/i });
    await userEvent.click(sendButton);

    // Verify streamed response appears
    const response = await findByText(/courage inspires us all/i);
    expect(response).toBeInTheDocument();

    // Verify reward badge shown
    const rewardBadge = await findByText(/courage/i);
    expect(rewardBadge).toBeInTheDocument();
  });
});
```

---

### 5.3 Contract Tests (Verify Mock Contracts Match Reality)

**Scope**: Ensure mocks accurately represent real dependencies

```typescript
// Contract test - Ollama provider
describe('Ollama Provider Contract', () => {
  it('should match mock interface with real ollama package', async () => {
    // Import real ollama (in test environment with real Ollama running)
    const { ollama } = await import('ollama-ai-provider-v2');

    // Verify contract matches mock
    const realModel = ollama('qwen3-coder:30b');
    const mockModel = mockOllamaProvider('qwen3-coder:30b');

    // Compare interfaces
    expect(typeof realModel).toBe(typeof mockModel);
    expect(Object.keys(realModel)).toEqual(expect.arrayContaining(Object.keys(mockModel)));
  });
});

// Contract test - Vercel AI SDK streamObject
describe('StreamObject Contract', () => {
  it('should match mock interface with real AI SDK', async () => {
    // This would run against real Ollama instance in test environment
    const { streamObject } = await import('ai');
    const { ollama } = await import('ollama-ai-provider-v2');

    const result = await streamObject({
      model: ollama('qwen3-coder:30b'),
      schema: z.object({ test: z.string() }),
      prompt: 'Test prompt',
      mode: 'json'
    });

    // Verify structure matches mock
    expect(result).toHaveProperty('object');
    expect(result).toHaveProperty('partialObjectStream');
    expect(result.object).toBeInstanceOf(Promise);
    expect(result.partialObjectStream[Symbol.asyncIterator]).toBeDefined();
  });
});
```

---

## 6. Test Organization Structure

```
tests/
├── unit/                          # Pure unit tests (all mocked)
│   ├── vision/
│   │   ├── analyze-report-card.test.ts
│   │   ├── evaluate-with-reasoning.test.ts
│   │   └── vision-schema.test.ts
│   ├── chat/
│   │   ├── child-mode.test.ts
│   │   ├── executive-mode.test.ts
│   │   └── virtue-detection.test.ts
│   ├── lib/
│   │   ├── types.test.ts
│   │   └── telemetry.test.ts
│   └── components/
│       ├── executive-chat.test.tsx
│       └── upload-report.test.tsx
│
├── integration/                   # Browser + mocked APIs
│   ├── vision-upload-flow.test.tsx
│   ├── chat-conversation.test.tsx
│   └── report-card-generation.test.tsx
│
├── contract/                      # Verify mocks match reality
│   ├── ollama-provider.contract.test.ts
│   ├── ai-sdk.contract.test.ts
│   └── browser-apis.contract.test.ts
│
├── mocks/                         # Mock implementations
│   ├── vision-analysis.mock.ts
│   ├── chat-service.mock.ts
│   ├── file-upload.mock.ts
│   ├── streaming.mock.ts
│   └── telemetry.mock.ts
│
├── stubs/                         # Stub data
│   ├── vision-stubs.ts
│   ├── evaluation-stubs.ts
│   ├── streaming-stubs.ts
│   └── error-stubs.ts
│
├── fixtures/                      # Test data files
│   ├── images/
│   │   ├── excellent-report-card.jpg
│   │   ├── average-report-card.jpg
│   │   └── poor-report-card.jpg
│   └── json/
│       ├── sample-analysis.json
│       └── sample-evaluation.json
│
└── swarm/                         # Swarm agent deliverables
    ├── agent1-test-architecture.md
    ├── agent2-vision-specs.md
    ├── agent3-mock-strategy.md     # THIS DOCUMENT
    └── ...
```

---

## 7. Mock Implementation Examples

### 7.1 Complete Vision Analysis Mock

```typescript
// tests/mocks/vision-analysis.mock.ts

import { z } from 'zod';
import { reportCardAnalysisSchema, optimusResponseSchema } from '@/lib/vision-schema';
import { SpanStatusCode } from '@opentelemetry/api';

/**
 * Creates a fully mocked vision analysis service following London School TDD
 */
export function createVisionAnalysisMock(config: {
  analysisResult?: Partial<z.infer<typeof reportCardAnalysisSchema>>;
  optimusResult?: Partial<z.infer<typeof optimusResponseSchema>>;
  shouldFail?: boolean;
  failureReason?: Error;
  streamChunkCount?: number;
} = {}) {
  const {
    analysisResult = {},
    optimusResult = {},
    shouldFail = false,
    failureReason = new Error('Mock failure'),
    streamChunkCount = 5
  } = config;

  // Build complete stub data
  const completeAnalysis: z.infer<typeof reportCardAnalysisSchema> = {
    documentType: analysisResult.documentType || 'report card',
    studentName: analysisResult.studentName || 'Test Student',
    grades: analysisResult.grades || [{ subject: 'Math', grade: 'A', score: 95 }],
    overallPerformance: analysisResult.overallPerformance || 'good',
    strengths: analysisResult.strengths || ['Problem solving'],
    weaknesses: analysisResult.weaknesses || ['Time management'],
    teacherComments: analysisResult.teacherComments,
    achievements: analysisResult.achievements || [],
    virtuesDetected: analysisResult.virtuesDetected || ['wisdom']
  };

  const completeOptimus: z.infer<typeof optimusResponseSchema> = {
    greeting: optimusResult.greeting || 'Greetings, young one',
    strengthsRecognition: optimusResult.strengthsRecognition || 'You excel at...',
    encouragementForWeaknesses: optimusResult.encouragementForWeaknesses || 'Continue to improve...',
    virtueConnection: optimusResult.virtueConnection || 'Your wisdom shines through...',
    inspirationalMessage: optimusResult.inspirationalMessage || 'Keep growing...',
    actionableAdvice: optimusResult.actionableAdvice || ['Study regularly', 'Ask questions'],
    celebrationMessage: optimusResult.celebrationMessage || 'Well done!'
  };

  // Create mock span
  const mockSpan = {
    setAttributes: jest.fn(),
    setStatus: jest.fn(),
    recordException: jest.fn(),
    end: jest.fn()
  };

  // Create mock tracer
  const mockTracer = {
    startSpan: jest.fn().mockReturnValue(mockSpan)
  };

  // Create mock telemetry
  const mockTrackEvent = jest.fn();

  // Create mock ollama provider
  const mockOllama = jest.fn((modelName: string) => ({
    modelName,
    capabilities: modelName.includes('vl') ? ['vision', 'text'] : ['text']
  }));

  // Create mock Buffer
  const mockBuffer = {
    from: jest.fn().mockReturnValue({
      toString: jest.fn().mockReturnValue('base64MockImageData==')
    })
  };

  // Create mock streamObject for analysis
  async function* createPartialAnalysisStream() {
    if (shouldFail) return;

    const chunks = streamChunkCount;
    for (let i = 0; i < chunks; i++) {
      // Progressive disclosure of analysis
      const partial: any = {
        documentType: completeAnalysis.documentType
      };

      if (i >= 1) partial.studentName = completeAnalysis.studentName;
      if (i >= 2) partial.grades = completeAnalysis.grades.slice(0, Math.min(i - 1, completeAnalysis.grades.length));
      if (i >= 3) partial.overallPerformance = completeAnalysis.overallPerformance;
      if (i >= 4) {
        partial.strengths = completeAnalysis.strengths;
        partial.weaknesses = completeAnalysis.weaknesses;
        partial.achievements = completeAnalysis.achievements;
        partial.virtuesDetected = completeAnalysis.virtuesDetected;
      }

      yield partial;
    }
  }

  async function* createPartialOptimusStream() {
    if (shouldFail) return;

    const fields = Object.keys(completeOptimus);
    for (let i = 0; i < fields.length; i++) {
      const partial: any = {};
      for (let j = 0; j <= i; j++) {
        const key = fields[j] as keyof typeof completeOptimus;
        partial[key] = completeOptimus[key];
      }
      yield partial;
    }
  }

  const mockStreamObject = jest.fn((params: any) => {
    if (shouldFail) {
      throw failureReason;
    }

    // Determine which schema is being used
    const isAnalysis = params.schema === reportCardAnalysisSchema;

    return {
      object: Promise.resolve(isAnalysis ? completeAnalysis : completeOptimus),
      partialObjectStream: isAnalysis ? createPartialAnalysisStream() : createPartialOptimusStream()
    };
  });

  return {
    // Mocked dependencies
    ollama: mockOllama,
    streamObject: mockStreamObject,
    Buffer: mockBuffer,
    tracer: mockTracer,
    trackEvent: mockTrackEvent,

    // Internal mocks for verification
    span: mockSpan,

    // Stub data
    stubs: {
      analysis: completeAnalysis,
      optimus: completeOptimus
    },

    // Verification helpers
    verify: {
      visionModelUsed: () => {
        expect(mockOllama).toHaveBeenCalledWith('qwen2.5-vl:latest');
      },

      textModelUsed: () => {
        expect(mockOllama).toHaveBeenCalledWith('qwen3-coder:30b');
      },

      imageDataProvided: (expectedBase64: string) => {
        expect(mockStreamObject).toHaveBeenCalledWith(
          expect.objectContaining({
            messages: expect.arrayContaining([
              expect.objectContaining({
                content: expect.arrayContaining([
                  expect.objectContaining({
                    type: 'image',
                    image: expect.stringContaining(expectedBase64)
                  })
                ])
              })
            ])
          })
        );
      },

      analysisSchemaUsed: () => {
        const calls = mockStreamObject.mock.calls;
        const analysisCall = calls.find(call => call[0].schema === reportCardAnalysisSchema);
        expect(analysisCall).toBeDefined();
      },

      responseSchemaUsed: () => {
        const calls = mockStreamObject.mock.calls;
        const responseCall = calls.find(call => call[0].schema === optimusResponseSchema);
        expect(responseCall).toBeDefined();
      },

      telemetryTracked: (...eventNames: string[]) => {
        eventNames.forEach(eventName => {
          expect(mockTrackEvent).toHaveBeenCalledWith(eventName, expect.any(Object));
        });
      },

      spanCompleted: () => {
        expect(mockSpan.setStatus).toHaveBeenCalledWith({ code: SpanStatusCode.OK });
        expect(mockSpan.end).toHaveBeenCalled();
      },

      errorRecorded: (expectedError: Error) => {
        expect(mockSpan.recordException).toHaveBeenCalledWith(expectedError);
        expect(mockSpan.setStatus).toHaveBeenCalledWith({
          code: SpanStatusCode.ERROR,
          message: expectedError.message
        });
      }
    }
  };
}
```

---

## 8. Summary & Key Principles

### London School TDD Checklist

- ✅ **Mock ALL external dependencies** (Ollama, AI SDK, Browser APIs, Node APIs, Telemetry)
- ✅ **Test behavior, not state** (verify interactions, not internal properties)
- ✅ **Outside-in development** (start from API routes, work down to helpers)
- ✅ **Verify collaborations** (how objects talk to each other, not what they store)
- ✅ **Isolate units completely** (no real network, filesystem, or model calls in unit tests)
- ✅ **Separate unit from integration** (unit = all mocked, integration = real browser + mocked APIs)
- ✅ **Use contract tests** (ensure mocks match real dependencies)
- ✅ **Design through mocks** (mock interfaces define contracts between modules)

### Mock Strategy Decision Tree

```
Need to test?
├─ Pure function (no dependencies)
│  └─ No mocks needed, test directly
│
├─ Component/module with dependencies
│  ├─ Unit test?
│  │  └─ Mock ALL dependencies
│  │
│  └─ Integration test?
│     ├─ Real browser APIs (DOM, File, etc.)
│     └─ Mock external services (API, models)
│
└─ Verify mock accuracy?
   └─ Contract test with real dependency
```

### Coverage Goals

- **Unit tests**: 90%+ coverage of business logic
- **Integration tests**: Cover all critical user workflows
- **Contract tests**: Verify all major dependency interfaces
- **Mock quality**: Each mock should be verified by at least one contract test

---

**End of Mock Strategy Document**
