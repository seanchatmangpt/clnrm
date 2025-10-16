# CLNRM Case Study - Hasbro AI Studio Integration

An interactive Next.js application showcasing how the Cleanroom Testing Framework (CLNRM) eliminates false positives in AI development, specifically addressing the requirements from [Hasbro AI Studio's Principal Engineer role](https://aistudio.digital.hasbro.com/).

## Features

- **Interactive CLI Demo**: See the difference between false positives and real functionality
- **Real AI Testing**: Live testing of Ollama AI provider with qwen3-coder:30b model
- **Hasbro Integration**: Detailed mapping of CLNRM solutions to Hasbro's requirements
- **Implementation Roadmap**: Step-by-step integration guide
- **Business Impact**: Measurable results and ROI analysis

## Tech Stack

- **Next.js 14** with App Router
- **TypeScript** for type safety
- **Tailwind CSS** for styling
- **shadcn/ui** for UI components
- **Vercel AI SDK** for AI integration
- **Lucide React** for icons

## Getting Started

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## Project Structure

```
src/
├── app/
│   ├── globals.css          # Global styles and custom CSS
│   ├── layout.tsx           # Root layout
│   └── page.tsx             # Main case study page
├── components/
│   ├── ui/                  # shadcn/ui components
│   ├── clnrm-demo.tsx       # Interactive CLI demo
│   └── hasbro-integration.tsx # Hasbro-specific integration details
└── lib/
    └── utils.ts             # Utility functions
```

## Key Components

### Interactive CLI Demo
- Simulates CLNRM commands with before/after comparisons
- Shows real vs false positive outputs
- Demonstrates command execution timing and results

### Hasbro Integration
- Maps CLNRM solutions to Hasbro's 4 core requirements
- Provides implementation roadmap with 3 phases
- Shows business impact metrics

### Case Study Content
- Executive summary of the false positive problem
- Before/after command success rate comparison
- Technical implementation details
- Measurable business results

## Deployment

The application is optimized for deployment on Vercel:

```bash
# Deploy to Vercel
vercel --prod
```

## Customization

### Adding New Commands
Edit `src/components/clnrm-demo.tsx` to add new CLI commands to the demo.

### Updating Hasbro Requirements
Modify `src/components/hasbro-integration.tsx` to reflect changes in Hasbro's requirements.

### Styling
The application uses a dark theme with purple/slate gradients. Customize colors in `src/app/globals.css`.

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Related Links

- [Hasbro AI Studio Job Posting](https://aistudio.digital.hasbro.com/)
- [CLNRM Framework Documentation](../README.md)
- [Vercel AI SDK Documentation](https://sdk.vercel.ai/)
- [shadcn/ui Components](https://ui.shadcn.com/)