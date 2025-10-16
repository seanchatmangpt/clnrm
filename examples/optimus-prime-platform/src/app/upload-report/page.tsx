'use client';

import { useState, useRef } from 'react';
import type { ReportCardAnalysis, OptimusResponse } from '@/lib/vision-schema';

export default function UploadReportPage() {
  const [studentName, setStudentName] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysis, setAnalysis] = useState<ReportCardAnalysis | null>(null);
  const [optimusResponse, setOptimusResponse] = useState<OptimusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      if (!file.type.startsWith('image/')) {
        setError('Please select an image file');
        return;
      }
      setSelectedFile(file);
      setError(null);

      // Create preview
      const reader = new FileReader();
      reader.onload = (e) => {
        setPreviewUrl(e.target?.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  const analyzeReportCard = async () => {
    if (!selectedFile) {
      setError('Please select a report card image');
      return;
    }

    setIsAnalyzing(true);
    setError(null);
    setAnalysis(null);
    setOptimusResponse(null);

    try {
      const formData = new FormData();
      formData.append('image', selectedFile);
      if (studentName.trim()) {
        formData.append('studentName', studentName.trim());
      }

      const response = await fetch('/api/vision/analyze-report-card', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Failed to analyze report card');
      }

      // Read NDJSON stream
      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      if (reader) {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split('\n').filter(line => line.trim());

          for (const line of lines) {
            try {
              const parsed = JSON.parse(line);

              if (parsed.type === 'analysis') {
                setAnalysis(parsed.data as ReportCardAnalysis);
              } else if (parsed.type === 'response') {
                setOptimusResponse(parsed.data as OptimusResponse);
              }
            } catch (e) {
              console.error('Failed to parse chunk:', e);
            }
          }
        }
      }

    } catch (err) {
      console.error('Analysis error:', err);
      setError(err instanceof Error ? err.message : 'Failed to analyze report card');
    } finally {
      setIsAnalyzing(false);
    }
  };

  const reset = () => {
    setSelectedFile(null);
    setPreviewUrl(null);
    setAnalysis(null);
    setOptimusResponse(null);
    setError(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 via-blue-50 to-indigo-100 p-8">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-indigo-900 mb-2">
            📸 Upload Your Report Card
          </h1>
          <p className="text-gray-600 text-lg">
            Let Optimus Prime analyze your achievements with AI vision
          </p>
        </div>

        {/* Upload Section */}
        <div className="bg-white rounded-lg shadow-lg p-8 mb-8">
          <div className="grid md:grid-cols-2 gap-8">
            {/* Left: Upload Area */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Student Name (Optional)
              </label>
              <input
                type="text"
                value={studentName}
                onChange={(e) => setStudentName(e.target.value)}
                placeholder="Enter your name..."
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent mb-4"
                disabled={isAnalyzing}
              />

              <label className="block text-sm font-medium text-gray-700 mb-2">
                Report Card Image
              </label>
              <div
                onClick={() => fileInputRef.current?.click()}
                className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center cursor-pointer hover:border-indigo-500 hover:bg-indigo-50 transition-colors"
              >
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  onChange={handleFileSelect}
                  className="hidden"
                  disabled={isAnalyzing}
                />
                <svg className="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
                  <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
                <p className="mt-2 text-sm text-gray-600">
                  {selectedFile ? selectedFile.name : 'Click to upload report card image'}
                </p>
                <p className="text-xs text-gray-500 mt-1">
                  PNG, JPG, JPEG up to 10MB
                </p>
              </div>

              {error && (
                <p className="mt-2 text-sm text-red-600">{error}</p>
              )}

              <div className="flex gap-4 mt-6">
                <button
                  onClick={analyzeReportCard}
                  disabled={!selectedFile || isAnalyzing}
                  className="flex-1 px-6 py-3 bg-indigo-600 text-white font-semibold rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isAnalyzing ? '🔍 Analyzing...' : '🚀 Analyze with Vision AI'}
                </button>
                {(selectedFile || analysis) && (
                  <button
                    onClick={reset}
                    disabled={isAnalyzing}
                    className="px-6 py-3 bg-gray-200 text-gray-700 font-semibold rounded-lg hover:bg-gray-300 disabled:opacity-50 transition-colors"
                  >
                    Reset
                  </button>
                )}
              </div>
            </div>

            {/* Right: Preview */}
            <div>
              {previewUrl && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Preview
                  </label>
                  <div className="border border-gray-300 rounded-lg overflow-hidden">
                    <img
                      src={previewUrl}
                      alt="Report card preview"
                      className="w-full h-auto"
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Analysis Results */}
        {analysis && (
          <div className="bg-white rounded-lg shadow-lg p-8 mb-8">
            <h2 className="text-2xl font-bold text-indigo-900 mb-6">
              📊 Report Card Analysis
            </h2>

            <div className="grid md:grid-cols-2 gap-6 mb-6">
              <div className="bg-blue-50 p-4 rounded-lg">
                <h3 className="font-semibold text-blue-900 mb-2">Student</h3>
                <p className="text-lg">{analysis.studentName}</p>
              </div>
              <div className="bg-green-50 p-4 rounded-lg">
                <h3 className="font-semibold text-green-900 mb-2">Performance</h3>
                <p className="text-lg capitalize">{analysis.overallPerformance}</p>
              </div>
            </div>

            {/* Grades */}
            {analysis.grades.length > 0 && (
              <div className="mb-6">
                <h3 className="font-semibold text-gray-800 mb-3">📚 Grades</h3>
                <div className="grid md:grid-cols-3 gap-4">
                  {analysis.grades.map((grade, idx) => (
                    <div key={idx} className="bg-gray-50 p-3 rounded-lg">
                      <div className="font-medium text-gray-700">{grade.subject}</div>
                      <div className="text-2xl font-bold text-indigo-600">{grade.grade}</div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Strengths & Weaknesses */}
            <div className="grid md:grid-cols-2 gap-6 mb-6">
              {analysis.strengths.length > 0 && (
                <div>
                  <h3 className="font-semibold text-green-700 mb-3">💪 Strengths</h3>
                  <ul className="space-y-2">
                    {analysis.strengths.map((strength, idx) => (
                      <li key={idx} className="flex items-start">
                        <span className="text-green-500 mr-2">✓</span>
                        <span className="text-gray-700">{strength}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              {analysis.weaknesses.length > 0 && (
                <div>
                  <h3 className="font-semibold text-blue-700 mb-3">📈 Growth Areas</h3>
                  <ul className="space-y-2">
                    {analysis.weaknesses.map((weakness, idx) => (
                      <li key={idx} className="flex items-start">
                        <span className="text-blue-500 mr-2">→</span>
                        <span className="text-gray-700">{weakness}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>

            {/* Virtues */}
            {analysis.virtuesDetected.length > 0 && (
              <div className="mb-6">
                <h3 className="font-semibold text-purple-700 mb-3">⭐ Character Virtues</h3>
                <div className="flex flex-wrap gap-2">
                  {analysis.virtuesDetected.map((virtue, idx) => (
                    <span
                      key={idx}
                      className="px-4 py-2 bg-purple-100 text-purple-800 rounded-full font-medium"
                    >
                      {virtue}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Teacher Comments */}
            {analysis.teacherComments && (
              <div className="bg-yellow-50 p-4 rounded-lg border-l-4 border-yellow-400">
                <h3 className="font-semibold text-yellow-900 mb-2">📝 Teacher Comments</h3>
                <p className="text-gray-700 italic">{analysis.teacherComments}</p>
              </div>
            )}
          </div>
        )}

        {/* Optimus Prime Response */}
        {optimusResponse && (
          <div className="bg-gradient-to-br from-blue-900 to-indigo-900 rounded-lg shadow-2xl p-8 text-white">
            <div className="flex items-center mb-6">
              <div className="w-16 h-16 bg-blue-500 rounded-full flex items-center justify-center text-3xl mr-4">
                🤖
              </div>
              <div>
                <h2 className="text-2xl font-bold">Optimus Prime</h2>
                <p className="text-blue-200">Leader of the Autobots</p>
              </div>
            </div>

            <div className="space-y-6">
              {/* Greeting */}
              <div className="bg-white/10 p-4 rounded-lg backdrop-blur">
                <p className="text-lg leading-relaxed">{optimusResponse.greeting}</p>
              </div>

              {/* Strengths Recognition */}
              <div>
                <h3 className="font-semibold text-blue-300 mb-2 flex items-center">
                  <span className="mr-2">💪</span> Your Strengths
                </h3>
                <p className="text-blue-100 leading-relaxed">{optimusResponse.strengthsRecognition}</p>
              </div>

              {/* Encouragement */}
              <div>
                <h3 className="font-semibold text-green-300 mb-2 flex items-center">
                  <span className="mr-2">🌱</span> Room to Grow
                </h3>
                <p className="text-green-100 leading-relaxed">{optimusResponse.encouragementForWeaknesses}</p>
              </div>

              {/* Virtue Connection */}
              <div>
                <h3 className="font-semibold text-purple-300 mb-2 flex items-center">
                  <span className="mr-2">⭐</span> Character Connection
                </h3>
                <p className="text-purple-100 leading-relaxed">{optimusResponse.virtueConnection}</p>
              </div>

              {/* Actionable Advice */}
              {optimusResponse.actionableAdvice.length > 0 && (
                <div>
                  <h3 className="font-semibold text-yellow-300 mb-3 flex items-center">
                    <span className="mr-2">💡</span> Advice from Optimus
                  </h3>
                  <ul className="space-y-2">
                    {optimusResponse.actionableAdvice.map((advice, idx) => (
                      <li key={idx} className="flex items-start">
                        <span className="text-yellow-400 mr-2">{idx + 1}.</span>
                        <span className="text-yellow-100">{advice}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {/* Inspirational Message */}
              <div className="bg-white/10 p-6 rounded-lg backdrop-blur border-2 border-blue-400">
                <p className="text-xl leading-relaxed italic">{optimusResponse.inspirationalMessage}</p>
              </div>

              {/* Celebration */}
              <div className="text-center py-4">
                <p className="text-2xl leading-relaxed">{optimusResponse.celebrationMessage}</p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
