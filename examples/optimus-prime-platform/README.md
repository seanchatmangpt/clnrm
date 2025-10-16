# Optimus Prime Character Platform

A production-ready AI character engine that reinforces child virtues through Optimus Prime while providing executives with real-time analytics and revenue optimization.

## 🎯 Overview

This is a comprehensive implementation of the Optimus Prime Character Platform as specified in the PRD. The platform features:

- **Child Mode**: Leadership development through Optimus Prime character interactions
- **Executive Mode**: Real-time KPI analytics and performance metrics
- **Admin Dashboard**: Comprehensive analytics with Chart.js visualizations
- **A/B Testing**: Premium CTA optimization
- **Real-time Telemetry**: In-memory event tracking and analytics

## 🚀 Tech Stack

- **Next.js 14** with App Router and TypeScript
- **ShadCN UI** for professional UI components
- **Tailwind CSS** with custom design tokens
- **Vercel AI SDK** for AI chat functionality
- **Chart.js** for analytics visualizations
- **In-memory telemetry** for event tracking

## 📁 Project Structure

```
src/
├── app/
│   ├── api/
│   │   ├── chat/route.ts        # AI chat endpoint
│   │   ├── metrics/route.ts     # Analytics endpoint
│   │   └── telemetry/route.ts   # Event tracking endpoint
│   ├── child/page.tsx           # Child mode interface
│   ├── executive/page.tsx       # Executive mode interface
│   ├── admin/dashboard/page.tsx # Admin dashboard
│   └── page.tsx                 # Landing page
├── components/
│   ├── child-chat.tsx           # Child chat interface
│   ├── executive-chat.tsx       # Executive chat interface
│   ├── dashboard.tsx            # Analytics dashboard
│   └── ui/                      # ShadCN UI components
└── lib/
    ├── types.ts                 # TypeScript types and interfaces
    └── telemetry.ts             # Event tracking and analytics
```

## 🛠️ Setup & Installation

### Prerequisites

1. **Node.js 18+** installed
2. **OpenAI API Key** for AI functionality
3. **Ollama** (optional) for local AI model testing

### Installation

```bash
# Clone and install
npm install

# Set up environment variables
cp .env.example .env.local
# Edit .env.local and add your OpenAI API key

# Start development server
npm run dev
```

### Environment Variables

Create a `.env.local` file with:

```env
OPENAI_API_KEY=your_openai_api_key_here
```

## 🎮 Usage

### Development Server

```bash
npm run dev
```

The application will be available at `http://localhost:3000`

### Build for Production

```bash
npm run build
npm start
```

## 🎯 Features Implemented

### ✅ Child Mode (`/child`)
- Real-time chat with Optimus Prime character
- Virtue detection and recognition
- Reward system with static video links
- Premium CTA with A/B testing variants
- Leadership-focused responses

### ✅ Executive Mode (`/executive`)
- KPI query interface with natural language
- Real-time analytics responses
- Data-driven insights from telemetry
- Executive-focused UI design

### ✅ Admin Dashboard (`/admin/dashboard`)
- Real-time metrics visualization
- A/B testing results comparison
- Revenue and funnel analytics
- Event tracking overview

### ✅ Technical Features
- **TypeScript**: Full type safety throughout
- **Real-time Updates**: Live analytics refresh
- **A/B Testing**: Client-side variant assignment
- **Responsive Design**: Mobile-friendly interface
- **Accessibility**: Readable contrast and focus states

## 📊 Analytics & Metrics

The platform tracks:

- **Session Events**: Start, message sent, interactions
- **Virtue Detection**: Leadership quality recognition
- **Conversion Funnel**: From session to premium engagement
- **A/B Test Results**: Premium CTA performance comparison
- **Revenue Metrics**: 7-day revenue tracking

## 🎨 Design System

### Color Palette (Autobot Theme)
- **Autobot Red**: `#D42727` - Primary brand color
- **Cyber Blue**: `#1A3D8F` - Executive accent
- **Energon Teal**: `#3EDDD7` - Premium accent
- **Gunmetal**: `#4A4A4A` - Text and borders
- **Steel**: `#B0B0B0` - Secondary elements

### Component Styling
- **4px Border Radius**: Consistent button and card styling
- **Glass Effects**: Translucent panels with backdrop blur
- **Hover Animations**: Smooth transitions and glow effects
- **Beveled Borders**: Distinctive panel styling

## 🔧 API Endpoints

### POST `/api/chat`
AI chat endpoint supporting both child and executive modes.

### POST `/api/telemetry`
Event tracking for analytics and metrics.

### GET `/api/metrics`
Real-time metrics and analytics data.

## 🎯 Demo Scripts

### Child Interaction
```
User: "I helped my team at school."
Optimus: "Excellent leadership! That's the spirit of teamwork."
→ Teamwork virtue detected
→ Reward link provided
→ Premium CTA shown
```

### Executive Query
```
User: "Compare premium CTR by variant and total revenue last 7 days."
Response: "Variant A: 8.2% CTR, Variant B: 6.1% CTR. Total revenue: $12,450."
```

## 🚀 Production Considerations

- **Edge Runtime**: Child mode uses edge runtime for low latency
- **Node.js Runtime**: Executive mode uses Node.js for aggregations
- **In-Memory Storage**: Events stored in memory (session-based)
- **Static Assets**: Optimized for production deployment
- **Type Safety**: Full TypeScript coverage

## 📈 Performance Metrics

- **Time to First Answer**: ≤ 2.5s P95 (local AI)
- **Executive Response Time**: ≤ 3.0s P95
- **Reward CTR**: ≥ 25% target
- **Premium CTA CTR**: ≥ 8% (A/B optimized)

## 🔒 Safety & Compliance

- **Leadership Reframing**: No punitive language
- **Child Safety**: Age-appropriate responses only
- **AI Disclosure**: Clear indication of AI assistance
- **No PII Storage**: In-memory only, no persistence

## 🎉 Success Metrics

The platform successfully implements all PRD requirements:

- ✅ **Child Loop**: Achievement → Optimus response → reward → premium CTA
- ✅ **Executive Loop**: KPI queries → numeric analytics answers
- ✅ **One Codebase**: App Router, ShadCN, Vercel AI SDK
- ✅ **A/B Testing**: Premium copy optimization
- ✅ **Real-time Dashboard**: Chart.js analytics with auto-refresh

This is a production-credible demo of an AI character engine with real business value and technical sophistication.