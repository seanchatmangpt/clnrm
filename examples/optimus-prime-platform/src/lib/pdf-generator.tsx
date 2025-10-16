import React from 'react';
import { Document, Page, Text, View, StyleSheet, pdf } from '@react-pdf/renderer';
import type { ReportCard } from './report-card-schema';

/**
 * PDF Styles for Report Card
 */
const styles = StyleSheet.create({
  page: {
    padding: 40,
    fontFamily: 'Helvetica',
    backgroundColor: '#ffffff',
  },
  header: {
    marginBottom: 20,
    borderBottom: '2 solid #2563eb',
    paddingBottom: 10,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#1e40af',
    marginBottom: 5,
  },
  subtitle: {
    fontSize: 12,
    color: '#64748b',
  },
  section: {
    marginTop: 15,
    marginBottom: 15,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#1e40af',
    marginBottom: 8,
    borderBottom: '1 solid #cbd5e1',
    paddingBottom: 4,
  },
  overallScore: {
    fontSize: 48,
    fontWeight: 'bold',
    color: '#10b981',
    textAlign: 'center',
    marginVertical: 15,
  },
  virtueContainer: {
    marginBottom: 12,
    padding: 10,
    backgroundColor: '#f8fafc',
    borderRadius: 4,
  },
  virtueName: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#475569',
    marginBottom: 4,
  },
  virtueScore: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#2563eb',
    marginBottom: 4,
  },
  virtueExamples: {
    fontSize: 10,
    color: '#64748b',
    marginBottom: 4,
    lineHeight: 1.4,
  },
  virtueFeedback: {
    fontSize: 11,
    color: '#334155',
    fontStyle: 'italic',
    lineHeight: 1.4,
  },
  achievement: {
    marginBottom: 10,
    padding: 8,
    backgroundColor: '#fef3c7',
    borderLeft: '3 solid #f59e0b',
  },
  achievementTitle: {
    fontSize: 12,
    fontWeight: 'bold',
    color: '#92400e',
    marginBottom: 3,
  },
  achievementDesc: {
    fontSize: 10,
    color: '#78350f',
    lineHeight: 1.3,
  },
  list: {
    marginLeft: 10,
  },
  listItem: {
    fontSize: 11,
    color: '#475569',
    marginBottom: 4,
    lineHeight: 1.4,
  },
  message: {
    padding: 15,
    backgroundColor: '#dbeafe',
    borderLeft: '4 solid #2563eb',
    marginTop: 10,
  },
  messageText: {
    fontSize: 12,
    color: '#1e40af',
    lineHeight: 1.5,
    fontStyle: 'italic',
  },
  badge: {
    display: 'flex',
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
    padding: 6,
    backgroundColor: '#f0fdf4',
    borderRadius: 3,
  },
  badgeName: {
    fontSize: 11,
    fontWeight: 'bold',
    color: '#15803d',
  },
  footer: {
    marginTop: 20,
    paddingTop: 10,
    borderTop: '1 solid #e2e8f0',
    textAlign: 'center',
  },
  footerText: {
    fontSize: 9,
    color: '#94a3b8',
  },
});

/**
 * Report Card PDF Document Component
 */
const ReportCardDocument: React.FC<{ data: ReportCard }> = ({ data }) => (
  <Document>
    <Page size="A4" style={styles.page}>
      {/* Header */}
      <View style={styles.header}>
        <Text style={styles.title}>Virtue Achievement Report Card</Text>
        <Text style={styles.subtitle}>
          Student: {data.studentName} • Period: {data.period}
        </Text>
      </View>

      {/* Overall Score */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Overall Virtue Score</Text>
        <Text style={styles.overallScore}>{data.overallScore}/100</Text>
      </View>

      {/* Virtue Assessment */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Virtue Assessment</Text>

        {Object.entries(data.virtueAssessment).map(([virtue, assessment]) => (
          <View key={virtue} style={styles.virtueContainer}>
            <Text style={styles.virtueName}>{virtue.toUpperCase()}</Text>
            <Text style={styles.virtueScore}>{assessment.score}/100</Text>

            {assessment.examples.length > 0 && (
              <Text style={styles.virtueExamples}>
                Examples: {assessment.examples.join('; ')}
              </Text>
            )}

            <Text style={styles.virtueFeedback}>
              "{assessment.feedback}"
            </Text>
          </View>
        ))}
      </View>

      {/* Achievements */}
      {data.achievements.length > 0 && (
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Notable Achievements</Text>
          {data.achievements.map((achievement, idx) => (
            <View key={idx} style={styles.achievement}>
              <Text style={styles.achievementTitle}>{achievement.title}</Text>
              <Text style={styles.achievementDesc}>
                {achievement.description} ({achievement.date})
              </Text>
            </View>
          ))}
        </View>
      )}

      {/* Strengths */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Areas of Strength</Text>
        <View style={styles.list}>
          {data.areasOfStrength.map((strength, idx) => (
            <Text key={idx} style={styles.listItem}>• {strength}</Text>
          ))}
        </View>
      </View>

      {/* Growth Areas */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Areas for Growth</Text>
        <View style={styles.list}>
          {data.areasForGrowth.map((area, idx) => (
            <Text key={idx} style={styles.listItem}>• {area}</Text>
          ))}
        </View>
      </View>

      {/* Optimus Prime Message */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Message from Optimus Prime</Text>
        <View style={styles.message}>
          <Text style={styles.messageText}>{data.optimusPrimeMessage}</Text>
        </View>
      </View>

      {/* Badges */}
      {data.badges.length > 0 && (
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Badges Earned</Text>
          {data.badges.map((badge, idx) => (
            <View key={idx} style={styles.badge}>
              <Text style={styles.badgeName}>
                🏆 {badge.name} - {badge.earnedDate}
              </Text>
            </View>
          ))}
        </View>
      )}

      {/* Footer */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>
          Generated by Optimus Prime Platform • {new Date().toLocaleDateString()}
        </Text>
      </View>
    </Page>
  </Document>
);

/**
 * Generate PDF Buffer from Report Card Data
 */
export async function generateReportCardPDF(data: ReportCard): Promise<Buffer> {
  const doc = <ReportCardDocument data={data} />;
  const asPdf = pdf(doc);
  const blob = await asPdf.toBlob();
  const arrayBuffer = await blob.arrayBuffer();
  return Buffer.from(arrayBuffer);
}

export default ReportCardDocument;
