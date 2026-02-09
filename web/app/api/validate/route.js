import { NextResponse } from "next/server";
import { validateSource } from "../../../lib/sim/runner.js";

export const runtime = "nodejs";

export async function POST(request) {
  try {
    const body = await request.json();
    const sourceCode = String(body?.source_code || "");
    const result = validateSource(sourceCode);
    return NextResponse.json(result, { status: 200 });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Validation failed" },
      { status: 400 }
    );
  }
}
