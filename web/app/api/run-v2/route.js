import { NextResponse } from "next/server";
import { runDeterministicV2, validateSource } from "../../../lib/sim/runner.js";
import { resolveStrategyProfileWithTevm } from "../../../lib/sim/runtime.server.js";

export const runtime = "nodejs";

export async function POST(request) {
  try {
    const body = await request.json();
    const sourceCode = String(body?.source_code || "");
    const config = body?.config || {};

    const validation = validateSource(sourceCode);
    if (!validation.valid) {
      return NextResponse.json(validation, { status: 400 });
    }

    const profile = await resolveStrategyProfileWithTevm(sourceCode);
    const result = runDeterministicV2({ sourceCode, config, profile });
    return NextResponse.json(result, { status: 200 });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Run V2 failed" },
      { status: 400 }
    );
  }
}
