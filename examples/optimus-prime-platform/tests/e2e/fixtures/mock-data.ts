import type { ReportCardAnalysis, OptimusResponse } from '@/lib/vision-schema';

/**
 * Mock data for vision upload testing
 */

export const mockReportCardAnalysis: ReportCardAnalysis = {
  studentName: 'Alex Johnson',
  overallPerformance: 'good',
  grades: [
    { subject: 'Mathematics', grade: 'A' },
    { subject: 'English', grade: 'B+' },
    { subject: 'Science', grade: 'A-' },
    { subject: 'Social Studies', grade: 'B' },
    { subject: 'Art', grade: 'A+' },
    { subject: 'Physical Education', grade: 'B+' },
  ],
  strengths: [
    'Excellent problem-solving skills in mathematics',
    'Creative thinking in art projects',
    'Strong analytical abilities in science',
    'Good participation in class discussions',
  ],
  weaknesses: [
    'Could improve time management for assignments',
    'Needs to work on detailed reading comprehension',
    'Should practice more writing exercises',
  ],
  virtuesDetected: [
    'Perseverance',
    'Creativity',
    'Curiosity',
    'Teamwork',
  ],
  teacherComments: 'Alex is a bright student who shows great enthusiasm for learning. With continued effort in reading comprehension and writing, Alex will achieve even greater success.',
};

export const mockOptimusResponse: OptimusResponse = {
  greeting: 'Greetings, Alex Johnson! I am Optimus Prime, and I am honored to recognize your achievements today.',
  strengthsRecognition: 'Your excellence in mathematics and creative abilities in art demonstrate the power of a balanced mind. Like an Autobot who must master both strategy and innovation, you show remarkable versatility in your learning.',
  encouragementForWeaknesses: 'Every great warrior must learn discipline and patience. Your growth areas in time management and reading comprehension are not weaknesses - they are opportunities to develop new strengths. Remember, even the mightiest Autobot started as a young learner.',
  virtueConnection: 'I see in you the virtues of Perseverance, Creativity, Curiosity, and Teamwork - the very qualities that make a true leader. These virtues will serve you well throughout your journey, just as they have guided the Autobots through countless challenges.',
  actionableAdvice: [
    'Create a daily schedule to practice time management - plan your study time like a battle strategy',
    'Read for 20 minutes each day to strengthen comprehension skills',
    'Practice writing in a journal to improve your expression',
    'Keep exploring your creativity in art - it strengthens all areas of learning',
  ],
  inspirationalMessage: 'Freedom is the right of all sentient beings, and education is the key to that freedom. Your dedication to learning opens doors to unlimited possibilities. Continue to grow, learn, and inspire others with your journey.',
  celebrationMessage: '🎉 You are on the path to greatness, young warrior! Keep charging forward! 🚀',
};

/**
 * Mock NDJSON response for streaming API
 */
export function generateMockNDJSONResponse(): string {
  const analysisLine = JSON.stringify({ type: 'analysis', data: mockReportCardAnalysis });
  const responseLine = JSON.stringify({ type: 'response', data: mockOptimusResponse });
  return `${analysisLine}\n${responseLine}\n`;
}

/**
 * Mock analysis for error scenarios
 */
export const mockErrorResponse = {
  error: 'Failed to analyze report card',
  message: 'Invalid image format or unreadable content',
};

/**
 * Mock analysis with minimal data (edge case)
 */
export const mockMinimalAnalysis: ReportCardAnalysis = {
  studentName: 'Student',
  overallPerformance: 'average',
  grades: [],
  strengths: ['Shows effort'],
  weaknesses: ['Needs improvement in multiple areas'],
  virtuesDetected: [],
  teacherComments: null,
};

/**
 * Mock analysis with excellent performance
 */
export const mockExcellentAnalysis: ReportCardAnalysis = {
  studentName: 'Emma Chen',
  overallPerformance: 'excellent',
  grades: [
    { subject: 'Mathematics', grade: 'A+' },
    { subject: 'English', grade: 'A+' },
    { subject: 'Science', grade: 'A+' },
    { subject: 'Social Studies', grade: 'A' },
    { subject: 'Art', grade: 'A+' },
    { subject: 'Music', grade: 'A' },
  ],
  strengths: [
    'Outstanding academic performance across all subjects',
    'Exceptional leadership in group projects',
    'Demonstrates advanced critical thinking',
    'Excellent communication skills',
    'Highly motivated and self-directed learner',
  ],
  weaknesses: [],
  virtuesDetected: [
    'Excellence',
    'Leadership',
    'Integrity',
    'Perseverance',
    'Responsibility',
  ],
  teacherComments: 'Emma is an exceptional student who consistently exceeds expectations. Her dedication and positive attitude inspire her peers.',
};

/**
 * Mock analysis with needs improvement
 */
export const mockNeedsImprovementAnalysis: ReportCardAnalysis = {
  studentName: 'Jordan Smith',
  overallPerformance: 'needs improvement',
  grades: [
    { subject: 'Mathematics', grade: 'C-' },
    { subject: 'English', grade: 'D+' },
    { subject: 'Science', grade: 'C' },
    { subject: 'Social Studies', grade: 'C-' },
    { subject: 'Art', grade: 'B' },
    { subject: 'Physical Education', grade: 'B-' },
  ],
  strengths: [
    'Shows creativity in art class',
    'Good athletic abilities',
    'Friendly with classmates',
  ],
  weaknesses: [
    'Struggles with reading comprehension',
    'Needs to focus more during lessons',
    'Frequently late with homework submissions',
    'Could benefit from additional tutoring',
  ],
  virtuesDetected: [
    'Creativity',
    'Friendliness',
  ],
  teacherComments: 'Jordan has potential but needs to apply more consistent effort. Additional support and study time would be beneficial.',
};

/**
 * Generate mock NDJSON for different scenarios
 */
export function generateNDJSONForScenario(scenario: 'good' | 'excellent' | 'needs-improvement' | 'minimal' | 'error'): string {
  let analysis: ReportCardAnalysis | null = null;
  let response: OptimusResponse | null = null;

  switch (scenario) {
    case 'excellent':
      analysis = mockExcellentAnalysis;
      response = {
        ...mockOptimusResponse,
        greeting: `Greetings, ${analysis.studentName}! Your excellence shines like the AllSpark itself!`,
        strengthsRecognition: 'Your outstanding performance across all subjects demonstrates the mark of a true champion. Like the greatest Autobots, you excel through dedication and unwavering commitment.',
        celebrationMessage: '🌟 Magnificent achievement, young hero! You inspire us all! 🏆',
      };
      break;
    case 'needs-improvement':
      analysis = mockNeedsImprovementAnalysis;
      response = {
        ...mockOptimusResponse,
        greeting: `Greetings, ${analysis.studentName}. Every journey begins with a single step, and I see your potential.`,
        strengthsRecognition: 'Your creativity and athletic abilities show that you have unique talents. These strengths are the foundation upon which you can build greater success.',
        encouragementForWeaknesses: 'The challenges you face now are opportunities for growth. Even the greatest warriors must train and develop their skills. With focus and determination, you will overcome these obstacles.',
        celebrationMessage: '💪 Keep pushing forward - every day is a chance to grow stronger! 🌱',
      };
      break;
    case 'minimal':
      analysis = mockMinimalAnalysis;
      response = {
        greeting: 'Greetings, Student. I see your journey has just begun.',
        strengthsRecognition: 'You are showing effort, which is the first step toward success.',
        encouragementForWeaknesses: 'Growth takes time and patience. Focus on one area at a time and you will see progress.',
        virtueConnection: 'The path to greatness requires perseverance and dedication.',
        actionableAdvice: [
          'Set small, achievable goals each day',
          'Ask for help when you need it',
          'Celebrate small victories',
        ],
        inspirationalMessage: 'Every expert was once a beginner. Your journey is just beginning.',
        celebrationMessage: '🌱 Keep growing, one step at a time! 🌟',
      };
      break;
    case 'error':
      return JSON.stringify({ type: 'error', data: mockErrorResponse }) + '\n';
    default:
      analysis = mockReportCardAnalysis;
      response = mockOptimusResponse;
  }

  const analysisLine = JSON.stringify({ type: 'analysis', data: analysis });
  const responseLine = JSON.stringify({ type: 'response', data: response });
  return `${analysisLine}\n${responseLine}\n`;
}
