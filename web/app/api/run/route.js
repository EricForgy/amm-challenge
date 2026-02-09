import { NextResponse } from "next/server";
import { runDeterministicV1, validateSource } from "../../../lib/sim/runner.js";
import { resolveStrategyProfileWithTevm } from "../../../lib/sim/runtime.server.js";

export const runtime = "nodejs";

export async function POST(request) {
  try {
    const body = await request.json();
    const sourceCode = String(body?.source_code || "");
    const simulations = Math.max(1, Number(body?.simulations ?? 100));
    const steps = Math.max(1, Number(body?.steps ?? 1000));

    const validation = validateSource(sourceCode);
    if (!validation.valid) {
      return NextResponse.json(validation, { status: 400 });
    }

    const profile = await resolveStrategyProfileWithTevm(sourceCode);
    const result = runDeterministicV1({
      sourceCode,
      simulations,
      steps,
      profile
    });
    return NextResponse.json(result, { status: 200 });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Run failed" },
      { status: 400 }
    );
  }
}
