# Feature Reference: Optimus Prime Character Platform

Complete technical and functional reference for all platform features, organized by user journey and JTBD (Jobs To Be Done).

---

## Table of Contents

- [Feature Overview](#feature-overview)
- [Child Mode Features](#child-mode-features)
- [Executive Mode Features](#executive-mode-features)
- [Admin Dashboard Features](#admin-dashboard-features)
- [API Reference](#api-reference)
- [Configuration Options](#configuration-options)
- [Integration Points](#integration-points)

---

## Feature Overview

### Platform Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Next.js 14 App Router                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Child Mode   │  │ Executive    │  │ Admin        │    │
│  │ /child       │  │ /executive   │  │ /admin       │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    API Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ /api/chat    │  │ /api/metrics │  │ /api/        │    │
│  │              │  │              │  │ telemetry    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                 AI & Data Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Ollama AI    │  │ In-Memory    │  │ A/B Testing  │    │
│  │ qwen3-coder  │  │ Telemetry    │  │ Engine       │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Jobs To Be Done (JTBD) Mapping

| JTBD ID | User Type | Job | Features Used |
|---------|-----------|-----|---------------|
| JTBD-001 | Child | Get recognized for achievements | Child Chat, Virtue Detection, Reward System |
| JTBD-002 | Child | Learn leadership values | Virtue Badges, Optimus Prime Responses |
| JTBD-003 | Parent | Monitor child's learning progress | Session Tracking, Virtue History |
| JTBD-004 | Parent | Evaluate premium content | Premium CTA, A/B Testing |
| JTBD-005 | Executive | Query KPIs in natural language | Executive Chat, Metrics API |
| JTBD-006 | Executive | Monitor real-time dashboards | Admin Dashboard, Auto-refresh |
| JTBD-007 | Educator | Integrate into curriculum | All modes, Telemetry, Reporting |

---

## Child Mode Features

Access: `/child` page

### 1. Optimus Prime Chat Interface

**Feature ID:** CM-001
**JTBD:** JTBD-001, JTBD-002

**Description:**
Real-time conversational interface where children share achievements and receive guidance from Optimus Prime AI character.

**Technical Specs:**
- Component: `src/components/child-chat.tsx`
- API Endpoint: `POST /api/chat`
- Runtime: Edge (for low latency)
- Response Time: ≤2.5s P95

**User Flow:**
1. Child types achievement/message
2. Press "Send" button or Enter key
3. Message displayed in chat with user avatar
4. AI processing (streaming response)
5. Optimus Prime response displayed with character avatar
6. Virtue detection triggers (if applicable)
7. Reward and premium CTAs appear (if applicable)

**Configuration:**
```typescript
// Message structure
interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

// Request format
{
  mode: "child",
  messages: Message[]
}
```

**Styling:**
- User messages: Autobot Red theme, left-aligned
- AI messages: Cyber Blue theme, right-aligned
- Responsive layout with mobile support
- Accessible focus states and ARIA labels

**Error Handling:**
- Network failure: Retry with exponential backoff
- Timeout: Display friendly error message
- Invalid input: Client-side validation

---

### 2. Virtue Detection System

**Feature ID:** CM-002
**JTBD:** JTBD-001, JTBD-002

**Description:**
Automated system that analyzes child's message to identify demonstrated leadership virtues.

**Virtues Detected:**
- **Teamwork**: Keywords include team, group, together, help, support, cooperate, collaboration, united
- **Wisdom**: Keywords include learn, study, school, knowledge, education, understand, smart, clever
- **Compassion**: Keywords include help, care, kind, friend, support, empathy, understanding, caring
- **Courage**: Keywords include brave, challenge, difficult, try, overcome, face, confront, strength

**Technical Implementation:**
```typescript
// src/lib/types.ts
export const VIRTUE_KEYWORDS = {
  teamwork: ["team", "group", "together", "help", "support", "cooperate", "collaboration", "united"],
  wisdom: ["learn", "study", "school", "knowledge", "education", "understand", "smart", "clever"],
  compassion: ["help", "care", "kind", "friend", "support", "empathy", "understanding", "caring"],
  courage: ["brave", "challenge", "difficult", "try", "overcome", "face", "confront", "strength"]
};

export function detectVirtue(text: string): string {
  const lowerText = text.toLowerCase();
  for (const [virtue, keywords] of Object.entries(VIRTUE_KEYWORDS)) {
    if (keywords.some(keyword => lowerText.includes(keyword))) {
      return virtue;
    }
  }
  return "courage"; // Default virtue
}
```

**Detection Logic:**
1. Convert message to lowercase
2. Check each virtue's keywords sequentially
3. Return first matching virtue
4. Default to "courage" if no matches

**Response Headers:**
```
X-Virtue: teamwork|wisdom|compassion|courage
X-Reward-Url: https://example.com/rewards/{virtue}-badge.mp4
X-Premium-Title: [Variant A or B title]
X-Premium-Link: [Variant A or B link]
```

**Telemetry:**
- Event: `virtue_detected`
- Payload: `{ virtue: string, message_length: number, variant: "A"|"B" }`

---

### 3. Virtue Badge Display

**Feature ID:** CM-003
**JTBD:** JTBD-002

**Description:**
Visual recognition of detected virtue with color-coded badge and congratulatory message.

**Badge Specifications:**

| Virtue | Color | Icon | Message |
|--------|-------|------|---------|
| Teamwork | Energon Teal (#3EDDD7) | 🤝 | "Recognized for demonstrating leadership!" |
| Wisdom | Cyber Blue (#1A3D8F) | 📚 | "Recognized for demonstrating leadership!" |
| Compassion | Autobot Red (#D42727) | ❤️ | "Recognized for demonstrating leadership!" |
| Courage | Energon Teal (#3EDDD7) | 🛡️ | "Recognized for demonstrating leadership!" |

**UI Component:**
```tsx
<Card className="bg-gradient-to-r from-[hsl(var(--energon))]/20 to-[hsl(var(--autobot-red))]/20 border-[hsl(var(--energon))]">
  <CardContent className="p-4">
    <Badge className="bg-[hsl(var(--energon))] text-[hsl(var(--gunmetal))] font-semibold">
      {virtue.charAt(0).toUpperCase() + virtue.slice(1)}
    </Badge>
    <span>Recognized for demonstrating leadership!</span>
  </CardContent>
</Card>
```

**Animation:**
- Fade-in effect when badge appears
- Gradient background with virtue color
- Prominent placement above reward section

---

### 4. Reward System

**Feature ID:** CM-004
**JTBD:** JTBD-001

**Description:**
After virtue detection, child receives reward link to educational video content.

**Reward URLs by Virtue:**
```typescript
export const REWARD_URLS = {
  teamwork: "https://example.com/rewards/teamwork-badge.mp4",
  wisdom: "https://example.com/rewards/wisdom-certificate.mp4",
  compassion: "https://example.com/rewards/compassion-heart.mp4",
  courage: "https://example.com/rewards/courage-shield.mp4"
};
```

**UI Component:**
- Title: "🎉 Achievement Unlocked!"
- Description: "You've earned a special reward for your {virtue}!"
- Button: "Claim Reward" (opens in new tab)
- Styling: Autobot Red theme with card layout

**Tracking:**
- Event: `reward_click`
- Payload: `{ virtue: string, variant: "A"|"B" }`
- Metric: Reward CTR (target ≥25%)

**Production Implementation:**
- Links would point to actual video CDN
- Videos would be 30-60 seconds
- Content: Optimus Prime explaining the virtue with examples
- Format: MP4, 720p, child-appropriate

---

### 5. Premium CTA (A/B Tested)

**Feature ID:** CM-005
**JTBD:** JTBD-004

**Description:**
Call-to-action promoting premium subscription, with two variants tested for optimization.

**Variants:**

**Variant A:**
- Title: "🚀 Unlock Premium Adventures"
- Link: https://store.autobot.com/premium
- Theme: Cyber Blue gradient

**Variant B:**
- Title: "🚀 Join the Elite Autobots"
- Link: https://store.autobot.com/elite
- Theme: Cyber Blue gradient

**Assignment Logic:**
```typescript
export function getABVariant(): "A" | "B" {
  // Simple client-side assignment based on timestamp
  return Date.now() % 2 === 0 ? "A" : "B";
}
```

**UI Component:**
```tsx
<Card className="bg-gradient-to-r from-[hsl(var(--cyber-blue))]/20 to-[hsl(var(--energon))]/20">
  <CardContent className="p-4">
    <h3>{premiumTitle}</h3>
    <p>Unlock exclusive adventures and premium features!</p>
    <Button onClick={handlePremiumClick} className="cyber-button">
      <a href={premiumLink}>Upgrade Now</a>
    </Button>
  </CardContent>
</Card>
```

**Tracking:**
- View Event: `premium_view` (fires when CTA appears)
- Click Event: `premium_click` (fires when button clicked)
- Metrics: Views, Clicks, CTR per variant
- Target: ≥8% CTR overall

**Optimization:**
- Dashboard shows which variant performs better
- Recommendation to use higher-performing variant as default
- Statistical significance not implemented (demo)

---

### 6. Session Tracking

**Feature ID:** CM-006
**JTBD:** JTBD-003, JTBD-007

**Description:**
Telemetry system tracks child's interactions for analytics and progress monitoring.

**Events Tracked:**

| Event | Trigger | Payload |
|-------|---------|---------|
| `session_start` | Page load | `{ mode: "child", variant: "A"\|"B" }` |
| `message_sent` | User sends message | `{ mode: "child", message_length: number }` |
| `virtue_detected` | AI detects virtue | `{ virtue: string }` |
| `reward_click` | Reward button clicked | `{ virtue: string, variant: "A"\|"B" }` |
| `premium_view` | Premium CTA shown | `{ variant: "A"\|"B" }` |
| `premium_click` | Premium button clicked | `{ variant: "A"\|"B" }` |

**Implementation:**
```typescript
// Client-side tracking
useEffect(() => {
  const variant = getABVariant();
  setAbVariant(variant);
  trackEvent("session_start", { mode: "child", variant });
}, []);

// API endpoint
POST /api/telemetry
{
  event: EventType,
  payload: Record<string, unknown>
}
```

**Storage:**
- In-memory array (demo version)
- Production: Would use database (PostgreSQL, MongoDB)
- Privacy: No PII stored, session-based only

---

## Executive Mode Features

Access: `/executive` page

### 7. KPI Query Interface

**Feature ID:** EM-001
**JTBD:** JTBD-005

**Description:**
Natural language interface for executives to query business metrics and KPIs.

**Supported Query Types:**

**Revenue Queries:**
- "What's our 7-day revenue?"
- "Show me total revenue"
- "Revenue for last week"

**A/B Testing Queries:**
- "Compare premium CTR by variant"
- "Which variant is performing better?"
- "Show A/B test results"

**Conversion Queries:**
- "What's the conversion rate?"
- "Session to premium click conversion"
- "Funnel conversion rates"

**Engagement Queries:**
- "How many events today?"
- "Total engagement metrics"
- "Virtue detection count"

**Technical Implementation:**
- Component: `src/components/executive-chat.tsx`
- API Endpoint: `POST /api/chat` with `mode: "executive"`
- Runtime: Node.js (for data aggregations)
- Response Time: ≤3.0s P95

**AI Prompt Engineering:**
```typescript
const executivePrompt = `You are an analytics AI for the Optimus Prime platform.
Answer executive queries with specific numeric data. Use the following metrics:
- Total Revenue (7 days): $${totalRevenue}
- Premium CTR Variant A: ${ctrA}%
- Premium CTR Variant B: ${ctrB}%
- Total Events: ${totalEvents}
- Conversion Rate: ${conversionRate}%

Provide concise, data-driven answers.`;
```

**Response Format:**
- Numeric answers with context
- Comparisons when requested
- Time-based breakdowns if applicable
- Actionable insights

**Example Responses:**
- Query: "What's our 7-day revenue?"
  - Response: "Total revenue for the last 7 days is $12,450. This represents a 15% increase compared to the previous period."
- Query: "Compare A/B variants"
  - Response: "Variant A has an 8.2% CTR with 125 views and 10 clicks. Variant B has a 6.1% CTR with 110 views and 7 clicks. Variant A is outperforming by 34%."

---

### 8. Real-Time Metrics API

**Feature ID:** EM-002
**JTBD:** JTBD-005, JTBD-006

**Description:**
RESTful API endpoint providing real-time analytics data for executive queries and dashboard.

**Endpoint:** `GET /api/metrics`

**Response Structure:**
```typescript
interface MetricsData {
  ab: {
    A: { variant: "A", views: number, clicks: number },
    B: { variant: "B", views: number, clicks: number }
  },
  funnel: Array<{ label: string, value: number }>,
  revenue7: {
    labels: string[], // ISO dates
    data: number[]     // USD amounts
  },
  totals: {
    revenue: number,   // Total 7-day revenue
    events: number     // Total events tracked
  }
}
```

**Data Sources:**
- In-memory telemetry events
- Real-time calculations
- No caching (always fresh)

**Calculations:**

**A/B Metrics:**
```typescript
const ctrA = (ab.A.views > 0) ? (ab.A.clicks / ab.A.views) * 100 : 0;
const ctrB = (ab.B.views > 0) ? (ab.B.clicks / ab.B.views) * 100 : 0;
```

**Funnel Stages:**
1. Sessions (session_start events)
2. Messages (message_sent events)
3. Virtues (virtue_detected events)
4. Rewards (reward_click events)
5. Premium Views (premium_view events)
6. Premium Clicks (premium_click events)

**Revenue Calculation:**
- Mock data in demo (1000-3000 per day)
- Production: Would aggregate from payment system
- 7-day rolling window

**Performance:**
- Response time: <100ms
- No rate limiting (demo)
- Auto-refresh friendly

---

## Admin Dashboard Features

Access: `/admin/dashboard` page

### 9. Executive Dashboard Overview

**Feature ID:** AD-001
**JTBD:** JTBD-006

**Description:**
Comprehensive real-time analytics dashboard with visualizations, A/B test results, revenue tracking, and funnel analysis.

**Layout Structure:**

```
┌─────────────────────────────────────────────────────────┐
│  Executive Dashboard Header                              │
├─────────────────────────────────────────────────────────┤
│  [Revenue]  [Events]  [Avg CTR]  <- Summary Cards       │
├─────────────────────────────────────────────────────────┤
│  A/B Test Results                                        │
│  [Variant A]        [Variant B]                         │
├─────────────────────────────────────────────────────────┤
│  Revenue Trend (7 Days)                                  │
│  Bar chart with daily revenue                           │
├─────────────────────────────────────────────────────────┤
│  User Journey Funnel                                     │
│  Sessions → Messages → Virtues → Rewards → Premium      │
├─────────────────────────────────────────────────────────┤
│  Key Insights                                            │
│  Conversion rates, best day, engagement totals          │
└─────────────────────────────────────────────────────────┘
```

**Auto-Refresh:**
- Interval: 3 seconds
- Smooth transitions
- No page reload
- Maintains scroll position

**Component:** `src/components/dashboard.tsx`

---

### 10. Summary Cards

**Feature ID:** AD-002
**JTBD:** JTBD-006

**Three Key Metrics:**

**Total Revenue (7d):**
- Format: `$12,450` (comma-separated)
- Source: Sum of revenue7.data array
- Update: Every refresh
- Color: Cyber Blue

**Total Events:**
- Format: `2,847` (count)
- Source: Total telemetry events
- Update: Real-time
- Color: Cyber Blue

**Avg Premium CTR:**
- Format: `7.2%` (one decimal)
- Source: Average of Variant A and B CTRs
- Update: Real-time
- Color: Cyber Blue

**UI Styling:**
- Large bold numbers (2xl font)
- Descriptive labels
- Executive panel theme
- Grid layout (responsive)

---

### 11. A/B Testing Results Panel

**Feature ID:** AD-003
**JTBD:** JTBD-004, JTBD-006

**Description:**
Side-by-side comparison of premium CTA variants with performance metrics and recommendations.

**Displayed Metrics:**

**Variant A:**
- CTR: Calculated percentage (one decimal)
- Views: Total premium_view events for variant A
- Clicks: Total premium_click events for variant A
- Color: Autobot Red accent

**Variant B:**
- CTR: Calculated percentage (one decimal)
- Views: Total premium_view events for variant B
- Clicks: Total premium_click events for variant B
- Color: Energon Teal accent

**Performance Indicator:**
```typescript
{ctrA > ctrB && (
  <div className="success-banner">
    🎯 Variant A is performing better! Consider making it the default.
  </div>
)}

{ctrB > ctrA && (
  <div className="success-banner">
    🎯 Variant B is performing better! Consider making it the default.
  </div>
)}
```

**Layout:**
- Two-column grid
- Large CTR percentage display
- Small view/click counts
- Winner recommendation banner

**Business Value:**
- Data-driven CTA optimization
- Quick decision-making
- Revenue optimization insights
- Testing culture enablement

---

### 12. Revenue Trend Visualization

**Feature ID:** AD-004
**JTBD:** JTBD-006

**Description:**
7-day revenue bar chart showing daily trends and patterns.

**Data Structure:**
```typescript
revenue7: {
  labels: ["2025-10-10", "2025-10-11", ..., "2025-10-16"],
  data: [1250, 2100, 1850, 2400, 1975, 2200, 2675]
}
```

**Visualization:**
- Horizontal bars (progress bar style)
- Proportional widths (scaled to max value)
- Daily dates on left
- Dollar amounts on right
- Energon Teal color for bars
- Steel gray background bars

**Calculation:**
```typescript
const barWidth = (value / Math.max(...revenue7.data)) * 100;
```

**Insights:**
- Best performing day highlighted
- Trend identification (up/down/flat)
- Quick visual comparison
- Supports executive queries

---

### 13. User Journey Funnel

**Feature ID:** AD-005
**JTBD:** JTBD-006, JTBD-007

**Description:**
Conversion funnel showing drop-off at each stage of user journey.

**Funnel Stages:**

1. **Sessions** (100% baseline)
   - Event: session_start
   - Represents: Total users entering platform

2. **Messages**
   - Event: message_sent
   - Represents: Users who engaged with chat

3. **Virtues**
   - Event: virtue_detected
   - Represents: Users who demonstrated virtues

4. **Rewards**
   - Event: reward_click
   - Represents: Users who claimed rewards

5. **Premium Views**
   - Event: premium_view
   - Represents: Users shown premium CTA

6. **Premium Clicks**
   - Event: premium_click
   - Represents: Users who clicked premium CTA (conversion goal)

**Visualization:**
- Horizontal bars showing relative volume
- Counts on right side
- Autobot Red color bars
- Proportional widths

**Key Metrics:**
- Overall conversion rate: (Premium Clicks / Sessions) × 100
- Stage-specific drop-off rates
- Bottleneck identification
- Optimization opportunities

**Business Value:**
- Identify where users drop off
- Optimize weakest conversion points
- Measure end-to-end effectiveness
- Guide product improvements

---

### 14. Key Insights Panel

**Feature ID:** AD-006
**JTBD:** JTBD-006

**Description:**
Automatically generated insights from analytics data.

**Insight Types:**

**Conversion Rate:**
```typescript
const conversionRate = ((premiumClicks / sessions) * 100).toFixed(1);
// Display: "💡 Conversion Rate: 5.2% from session to premium click"
```

**Best Day:**
```typescript
const bestDay = revenue7.labels[revenue7.data.indexOf(Math.max(...revenue7.data))];
const bestRevenue = Math.max(...revenue7.data);
// Display: "📈 Best Day: 2025-10-16 ($2,675)"
```

**Total Engagement:**
```typescript
// Display: "🎯 Total Engagement: 2,847 events tracked across all sessions"
```

**UI Styling:**
- Color-coded cards (blue, teal, gray)
- Icon indicators
- Descriptive text
- Actionable language

---

## API Reference

### POST /api/chat

**Description:** AI chat endpoint for both child and executive modes.

**Runtime:**
- Child mode: Edge runtime
- Executive mode: Node.js runtime

**Request:**
```typescript
{
  mode: "child" | "executive",
  messages: Array<{
    id: string,
    role: "user" | "assistant",
    content: string,
    timestamp: number
  }>
}
```

**Response:** Streaming

**Child Mode Headers:**
```
X-Virtue: teamwork|wisdom|compassion|courage
X-Reward-Url: https://example.com/rewards/{virtue}.mp4
X-Premium-Title: Unlock Premium Adventures | Join the Elite Autobots
X-Premium-Link: https://store.autobot.com/premium|elite
```

**Streaming Format:**
```json
{"response": "First chunk"}
{"response": " of AI"}
{"response": " response"}
{"done": true}
```

**Error Responses:**
- 400: Invalid request format
- 500: AI service unavailable
- 503: Model not loaded

---

### GET /api/metrics

**Description:** Real-time analytics data endpoint.

**Runtime:** Node.js

**Request:** None (GET)

**Response:**
```json
{
  "ab": {
    "A": { "variant": "A", "views": 125, "clicks": 10 },
    "B": { "variant": "B", "views": 110, "clicks": 7 }
  },
  "funnel": [
    { "label": "Sessions", "value": 450 },
    { "label": "Messages", "value": 380 },
    { "label": "Virtues", "value": 320 },
    { "label": "Rewards", "value": 285 },
    { "label": "Premium Views", "value": 235 },
    { "label": "Premium Clicks", "value": 17 }
  ],
  "revenue7": {
    "labels": ["2025-10-10", "2025-10-11", "2025-10-12", "2025-10-13", "2025-10-14", "2025-10-15", "2025-10-16"],
    "data": [1250, 2100, 1850, 2400, 1975, 2200, 2675]
  },
  "totals": {
    "revenue": 14450,
    "events": 2847
  }
}
```

**Performance:**
- Response time: <100ms
- No caching
- Real-time calculations

---

### POST /api/telemetry

**Description:** Event tracking endpoint for analytics.

**Runtime:** Node.js

**Request:**
```json
{
  "event": "session_start" | "message_sent" | "virtue_detected" | "reward_click" | "premium_view" | "premium_click" | "purchase",
  "payload": {
    // Event-specific data
    "mode": "child" | "executive",
    "variant": "A" | "B",
    "virtue": "teamwork" | "wisdom" | "compassion" | "courage",
    // ... other properties
  }
}
```

**Response:**
```json
{
  "success": true,
  "eventId": "evt_abc123xyz"
}
```

**Storage:**
- In-memory array (demo)
- Event ID: Generated with crypto.randomUUID()
- Timestamp: Automatic (Date.now())

---

## Configuration Options

### Environment Variables

**AI Configuration:**
```env
# Ollama API endpoint (default: http://localhost:11434)
OLLAMA_API_URL=http://localhost:11434

# AI Model (default: qwen3-coder:30b)
OLLAMA_MODEL=qwen3-coder:30b

# Optional: OpenAI API key for comparison
OPENAI_API_KEY=sk-...
```

**Application Configuration:**
```env
# Next.js configuration
NODE_ENV=development | production
PORT=3000

# Feature flags (future)
ENABLE_AUTHENTICATION=false
ENABLE_PERSISTENCE=false
ENABLE_PAYMENT=false
```

### Design Tokens

**Colors** (`app/globals.css`):
```css
--autobot-red: 0 76% 50%;      /* #D42727 */
--cyber-blue: 217 68% 33%;     /* #1A3D8F */
--energon: 177 68% 55%;        /* #3EDDD7 */
--gunmetal: 0 0% 29%;          /* #4A4A4A */
--steel: 0 0% 69%;             /* #B0B0B0 */
```

**Component Styles:**
- Border radius: 4px (sharp, tech-inspired)
- Panel blur: backdrop-blur-xl
- Hover transitions: 300ms ease
- Focus rings: 2px solid color

### A/B Testing Configuration

**Variants** (`src/lib/types.ts`):
```typescript
export const PREMIUM_CTA_VARIANTS = {
  A: {
    title: "Unlock Premium Adventures",
    link: "https://store.autobot.com/premium"
  },
  B: {
    title: "Join the Elite Autobots",
    link: "https://store.autobot.com/elite"
  }
};
```

**Assignment:**
- Method: Client-side timestamp modulo
- Persistence: Session-based (not persistent across reloads)
- Production: Would use consistent hashing with user ID

### Performance Targets

**Response Times:**
- Child AI response: ≤2.5s P95
- Executive AI response: ≤3.0s P95
- Metrics API: <100ms P95
- Dashboard refresh: <50ms P95

**Conversion Targets:**
- Reward CTR: ≥25%
- Premium CTR: ≥8%
- Session to premium: ≥5%
- D7 Retention: ≥95%

---

## Integration Points

### AI Provider Integration

**Current: Ollama Local AI**
```typescript
const response = await fetch('http://localhost:11434/api/generate', {
  method: 'POST',
  body: JSON.stringify({
    model: 'qwen3-coder:30b',
    prompt: systemPrompt + userMessage,
    stream: true
  })
});
```

**Future: Cloud AI (OpenAI, Anthropic, etc.)**
```typescript
import OpenAI from 'openai';

const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY
});

const stream = await openai.chat.completions.create({
  model: 'gpt-4',
  messages: [{role: 'system', content: systemPrompt}, ...messages],
  stream: true
});
```

### Database Integration

**Current: In-Memory**
```typescript
let events: TelemetryEvent[] = [];
```

**Future: PostgreSQL**
```sql
CREATE TABLE telemetry_events (
  id UUID PRIMARY KEY,
  created_at TIMESTAMP DEFAULT NOW(),
  event_type VARCHAR(50),
  payload JSONB,
  user_id UUID REFERENCES users(id),
  session_id UUID
);

CREATE INDEX idx_events_type ON telemetry_events(event_type);
CREATE INDEX idx_events_time ON telemetry_events(created_at);
CREATE INDEX idx_events_session ON telemetry_events(session_id);
```

### Payment Integration

**Future: Stripe Integration**
```typescript
import Stripe from 'stripe';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY);

const session = await stripe.checkout.sessions.create({
  mode: 'subscription',
  line_items: [{
    price: 'price_premium_monthly',
    quantity: 1
  }],
  success_url: 'https://app.autobot.com/success',
  cancel_url: 'https://app.autobot.com/child'
});
```

### Analytics Integration

**Future: Google Analytics / Mixpanel**
```typescript
import mixpanel from 'mixpanel-browser';

mixpanel.init('YOUR_TOKEN');

// Track events
mixpanel.track('Premium CTA Click', {
  variant: 'A',
  virtue: 'teamwork',
  session_id: sessionId
});
```

### Authentication Integration

**Future: NextAuth.js**
```typescript
import NextAuth from 'next-auth';
import GoogleProvider from 'next-auth/providers/google';

export default NextAuth({
  providers: [
    GoogleProvider({
      clientId: process.env.GOOGLE_CLIENT_ID,
      clientSecret: process.env.GOOGLE_CLIENT_SECRET
    })
  ],
  callbacks: {
    async session({ session, token }) {
      session.user.id = token.sub;
      session.user.role = 'child'; // or 'parent', 'executive', 'educator'
      return session;
    }
  }
});
```

---

## Production Roadmap

### Phase 1: User Management
- [ ] Authentication system (NextAuth.js)
- [ ] User roles (child, parent, executive, educator)
- [ ] Profile management
- [ ] Parental controls

### Phase 2: Data Persistence
- [ ] PostgreSQL database
- [ ] Session history storage
- [ ] Virtue badge history
- [ ] Progress tracking over time

### Phase 3: Payment & Premium
- [ ] Stripe integration
- [ ] Subscription management
- [ ] Premium content delivery
- [ ] Revenue tracking (real)

### Phase 4: Enhanced Analytics
- [ ] Historical trend analysis
- [ ] Cohort analysis
- [ ] Predictive insights
- [ ] Export capabilities

### Phase 5: Scale & Performance
- [ ] Redis caching
- [ ] CDN for static assets
- [ ] Database optimization
- [ ] Load balancing

### Phase 6: Advanced Features
- [ ] Multi-language support
- [ ] Custom virtue configuration
- [ ] School/class management for educators
- [ ] Parent dashboard with child progress
- [ ] Mobile apps (iOS/Android)

---

**Last Updated:** 2025-10-16
**Version:** 1.0.0 (Demo)
**For Questions:** See repository documentation or submit GitHub issue
