import { NextResponse } from "next/server";
import { getMetrics } from "@/lib/telemetry";

export async function GET() {
  try {
    const metrics = getMetrics();
    return NextResponse.json(metrics);
  } catch (error) {
    console.error("Metrics API error:", error);
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 }
    );
  }
}
