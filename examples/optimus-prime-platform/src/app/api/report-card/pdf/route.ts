import { NextResponse } from 'next/server';
import { generateReportCardPDF } from '@/lib/pdf-generator';
import type { ReportCard } from '@/lib/report-card-schema';
import { trace, SpanStatusCode } from '@opentelemetry/api';
import { trackEvent } from '@/lib/telemetry';

const tracer = trace.getTracer('report-card-pdf-api', '0.1.0');

/**
 * POST /api/report-card/pdf
 *
 * Generates a PDF from completed report card data
 * Returns PDF file as downloadable attachment
 */
export async function POST(request: Request) {
  const span = tracer.startSpan('POST /api/report-card/pdf');

  try {
    const reportData: ReportCard = await request.json();

    span.setAttributes({
      'report.student_name': reportData.studentName,
      'report.overall_score': reportData.overallScore,
    });

    trackEvent("report_card_pdf_generated", {
      studentName: reportData.studentName,
      overallScore: reportData.overallScore,
    });

    // Generate PDF buffer
    const pdfBuffer = await generateReportCardPDF(reportData);

    span.setStatus({ code: SpanStatusCode.OK });

    // Return PDF as downloadable file
    const filename = `report-card-${reportData.studentName.replace(/\s+/g, '-')}-${new Date().toISOString().split('T')[0]}.pdf`;

    return new NextResponse(new Uint8Array(pdfBuffer), {
      headers: {
        'Content-Type': 'application/pdf',
        'Content-Disposition': `attachment; filename="${filename}"`,
        'Content-Length': pdfBuffer.length.toString(),
      },
    });

  } catch (error) {
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error instanceof Error ? error.message : 'Unknown error'
    });
    span.recordException(error as Error);
    console.error("PDF generation error:", error);
    return NextResponse.json(
      { error: "Failed to generate PDF" },
      { status: 500 }
    );
  } finally {
    span.end();
  }
}
