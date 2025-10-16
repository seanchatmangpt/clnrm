import { NextResponse } from "next/server";
import { getVirtueHistory, getVirtueCount } from "@/lib/telemetry";

export async function GET() {
  try {
    const history = getVirtueHistory();
    const count = getVirtueCount();

    return NextResponse.json({
      history,
      count,
      total: history.length,
    });
  } catch (error) {
    console.error("Virtue History API error:", error);
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 }
    );
  }
}
