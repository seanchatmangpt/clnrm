'use client';

import { useState, useRef } from 'react';
import type { ReportCardAnalysis, OptimusResponse } from '@/lib/vision-schema';

interface ChainOfThoughtEvaluation {
  reasoning: {
    academicAnalysis: string;
    characterAssessment: string;
    growthOpportunities: string;
    strengthsRecognition: string;
  };
  evaluation: {
    overallGrade: string;
    virtuesMastered: string[];
    areasToFocus: string[];
    encouragement: string;
    actionableAdvice: string[];
    reward: {
      type: string;
      description: string;
      unlockMessage: string;
    };
  };
}

interface PromptInputUploadProps {
  onAnalysisComplete?: (analysis: ReportCardAnalysis, response: OptimusResponse) => void;
}

export default function PromptInputUpload({ onAnalysisComplete }: PromptInputUploadProps) {
  const [studentName, setStudentName] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysis, setAnalysis] = useState<ReportCardAnalysis | null>(null);
  const [optimusResponse, setOptimusResponse] = useState<OptimusResponse | null>(null);
  const [evaluation, setEvaluation] = useState<ChainOfThoughtEvaluation | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [processingStage, setProcessingStage] = useState<string>('');
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

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    if (file && file.type.startsWith('image/')) {
      setSelectedFile(file);
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
    setEvaluation(null);

    try {
      // Step 1: Vision analysis
      setProcessingStage('🔍 Analyzing report card with vision AI...');
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
                setProcessingStage('📊 Vision analysis complete! Generating Optimus response...');
              } else if (parsed.type === 'response') {
                setOptimusResponse(parsed.data as OptimusResponse);
                setProcessingStage('🤖 Optimus Prime response ready!');
              }
            } catch (e) {
              console.error('Failed to parse chunk:', e);
            }
          }
        }
      }

      // Step 2: Chain-of-thought evaluation
      if (analysis) {
        setProcessingStage('🧠 Optimus Prime is thinking deeply about your progress...');
        await generateEvaluation(analysis);
      }

      if (onAnalysisComplete && analysis && optimusResponse) {
        onAnalysisComplete(analysis, optimusResponse);
      }

    } catch (err) {
      console.error('Analysis error:', err);
      setError(err instanceof Error ? err.message : 'Failed to analyze report card');
    } finally {
      setIsAnalyzing(false);
      setProcessingStage('');
    }
  };

  const generateEvaluation = async (analysisData: ReportCardAnalysis) => {
    try {
      const response = await fetch('/api/vision/evaluate-with-reasoning', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ analysis: analysisData }),
      });

      if (!response.ok) {
        throw new Error('Failed to generate evaluation');
      }

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
              setEvaluation(parsed);
              setProcessingStage('⭐ Evaluation complete with detailed reasoning!');
            } catch (e) {
              console.error('Failed to parse evaluation:', e);
            }
          }
        }
      }
    } catch (err) {
      console.error('Evaluation error:', err);
    }
  };

  const reset = () => {
    setSelectedFile(null);
    setPreviewUrl(null);
    setAnalysis(null);
    setOptimusResponse(null);
    setEvaluation(null);
    setError(null);
    setProcessingStage('');
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  return (
    <div className="space-y-6">
      {/* Prompt Input Style Upload */}
      <div className="bg-white rounded-lg shadow-lg p-6">
        <div className="flex items-start gap-4">
          <div className="flex-1">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Student Name
            </label>
            <input
              type="text"
              value={studentName}
              onChange={(e) => setStudentName(e.target.value)}
              placeholder="Enter your name..."
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent mb-4"
              disabled={isAnalyzing}
            />

            <div
              onDrop={handleDrop}
              onDragOver={(e) => e.preventDefault()}
              onClick={() => fileInputRef.current?.click()}
              className="border-2 border-dashed border-indigo-300 rounded-lg p-8 text-center cursor-pointer hover:border-indigo-500 hover:bg-indigo-50 transition-all duration-200 bg-gradient-to-br from-indigo-50 to-blue-50"
            >
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                onChange={handleFileSelect}
                className="hidden"
                disabled={isAnalyzing}
              />
              <div className="flex flex-col items-center">
                <svg className="h-16 w-16 text-indigo-400 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                </svg>
                <p className="text-lg font-medium text-gray-700 mb-1">
                  {selectedFile ? selectedFile.name : 'Drop report card here or click to browse'}
                </p>
                <p className="text-sm text-gray-500">
                  PNG, JPG, JPEG up to 10MB
                </p>
              </div>
            </div>

            {error && (
              <p className="mt-2 text-sm text-red-600">{error}</p>
            )}

            {processingStage && (
              <div className="mt-4 p-4 bg-blue-50 rounded-lg border border-blue-200">
                <p className="text-blue-800 font-medium animate-pulse">{processingStage}</p>
              </div>
            )}
          </div>

          {previewUrl && (
            <div className="w-64">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Preview
              </label>
              <div className="border-2 border-gray-300 rounded-lg overflow-hidden">
                <img
                  src={previewUrl}
                  alt="Report card preview"
                  className="w-full h-auto"
                />
              </div>
            </div>
          )}
        </div>

        <div className="flex gap-4 mt-6">
          <button
            onClick={analyzeReportCard}
            disabled={!selectedFile || isAnalyzing}
            className="flex-1 px-6 py-3 bg-gradient-to-r from-indigo-600 to-blue-600 text-white font-semibold rounded-lg hover:from-indigo-700 hover:to-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 shadow-lg"
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

      {/* Chain of Thought Reasoning Display */}
      {evaluation?.reasoning && (
        <div className="bg-gradient-to-br from-purple-50 to-indigo-50 rounded-lg shadow-lg p-6 border-2 border-purple-200">
          <h2 className="text-2xl font-bold text-purple-900 mb-4 flex items-center">
            <span className="mr-2">🧠</span> Optimus Prime's Reasoning Process
          </h2>

          <div className="space-y-4">
            <div className="bg-white p-4 rounded-lg">
              <h3 className="font-semibold text-indigo-900 mb-2 flex items-center">
                <span className="mr-2">📚</span> Academic Analysis
              </h3>
              <p className="text-gray-700">{evaluation.reasoning.academicAnalysis}</p>
            </div>

            <div className="bg-white p-4 rounded-lg">
              <h3 className="font-semibold text-blue-900 mb-2 flex items-center">
                <span className="mr-2">⭐</span> Character Assessment
              </h3>
              <p className="text-gray-700">{evaluation.reasoning.characterAssessment}</p>
            </div>

            <div className="bg-white p-4 rounded-lg">
              <h3 className="font-semibold text-green-900 mb-2 flex items-center">
                <span className="mr-2">🌱</span> Growth Opportunities
              </h3>
              <p className="text-gray-700">{evaluation.reasoning.growthOpportunities}</p>
            </div>

            <div className="bg-white p-4 rounded-lg">
              <h3 className="font-semibold text-yellow-900 mb-2 flex items-center">
                <span className="mr-2">💪</span> Strengths Recognition
              </h3>
              <p className="text-gray-700">{evaluation.reasoning.strengthsRecognition}</p>
            </div>
          </div>
        </div>
      )}

      {/* Final Evaluation */}
      {evaluation?.evaluation && (
        <div className="bg-gradient-to-br from-blue-900 to-indigo-900 rounded-lg shadow-2xl p-8 text-white">
          <div className="flex items-center mb-6">
            <div className="w-20 h-20 bg-blue-500 rounded-full flex items-center justify-center text-4xl mr-4">
              🤖
            </div>
            <div>
              <h2 className="text-3xl font-bold">Optimus Prime's Evaluation</h2>
              <p className="text-blue-200">Leader of the Autobots</p>
            </div>
          </div>

          <div className="grid md:grid-cols-2 gap-4 mb-6">
            <div className="bg-white/10 p-4 rounded-lg backdrop-blur">
              <h3 className="font-semibold text-blue-300 mb-2">Overall Grade</h3>
              <p className="text-2xl font-bold capitalize">{evaluation.evaluation.overallGrade}</p>
            </div>
            <div className="bg-white/10 p-4 rounded-lg backdrop-blur">
              <h3 className="font-semibold text-green-300 mb-2">Virtues Mastered</h3>
              <div className="flex flex-wrap gap-2">
                {evaluation.evaluation.virtuesMastered.map((virtue, idx) => (
                  <span key={idx} className="px-3 py-1 bg-green-500 rounded-full text-sm">
                    {virtue}
                  </span>
                ))}
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <div className="bg-white/10 p-4 rounded-lg backdrop-blur">
              <h3 className="font-semibold text-yellow-300 mb-2">Encouragement</h3>
              <p className="text-lg">{evaluation.evaluation.encouragement}</p>
            </div>

            <div className="bg-white/10 p-4 rounded-lg backdrop-blur">
              <h3 className="font-semibold text-purple-300 mb-3">Actionable Advice</h3>
              <ul className="space-y-2">
                {evaluation.evaluation.actionableAdvice.map((advice, idx) => (
                  <li key={idx} className="flex items-start">
                    <span className="text-yellow-400 mr-2">{idx + 1}.</span>
                    <span>{advice}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="bg-gradient-to-r from-yellow-500 to-orange-500 p-6 rounded-lg">
              <h3 className="font-bold text-white text-xl mb-2">🎁 Special Reward Unlocked!</h3>
              <p className="font-semibold text-yellow-100 mb-2">{evaluation.evaluation.reward.type}</p>
              <p className="text-white">{evaluation.evaluation.reward.unlockMessage}</p>
            </div>
          </div>
        </div>
      )}

      {/* Original Analysis Display */}
      {analysis && (
        <div className="bg-white rounded-lg shadow-lg p-8">
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
        </div>
      )}
    </div>
  );
}
