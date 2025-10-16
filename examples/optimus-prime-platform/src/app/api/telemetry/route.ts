import { NextRequest, NextResponse } from "next/server";
import { trackEvent } from "@/lib/telemetry";

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { event, payload, ts } = body;

    if (!event || !payload) {
      return NextResponse.json(
        { error: "Missing event or payload" },
        { status: 400 }
      );
    }

    trackEvent(event, payload);

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error("Telemetry API error:", error);
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 }
    );
  }
}
