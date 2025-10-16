'use client';

import PromptInputUpload from '@/components/prompt-input-upload';

export default function EnhancedUploadPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 via-blue-50 to-indigo-100 p-8">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-indigo-900 mb-2">
            📸 Upload Your Report Card
          </h1>
          <p className="text-gray-600 text-lg">
            Optimus Prime will analyze your achievements with AI vision and chain-of-thought reasoning
          </p>
        </div>

        {/* Enhanced Upload Component */}
        <PromptInputUpload />
      </div>
    </div>
  );
}
