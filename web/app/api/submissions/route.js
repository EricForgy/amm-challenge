import { NextResponse } from "next/server";
import { runDeterministicV1, runDeterministicV2, validateSource } from "../../../lib/sim/runner.js";
import { resolveStrategyProfileWithTevm } from "../../../lib/sim/runtime.server.js";
import {
  addLeaderboardEntry,
  checkRateLimit,
  createSubmission,
  getClientIp,
  updateSubmission
} from "../../../lib/server/store.js";

export const runtime = "nodejs";

export async function POST(request) {
  try {
    const ip = getClientIp(request);
    const rate = checkRateLimit(ip);
    if (!rate.ok) {
      return NextResponse.json({ error: rate.error }, { status: 429 });
    }

    const body = await request.json();
    const sourceCode = String(body?.source_code || "");
    const runType = body?.run_type === "v2" ? "v2" : "v1";
    const strategyLabel = String(body?.strategy_label || "Anonymous");

    const validation = validateSource(sourceCode);
    if (!validation.valid) {
      return NextResponse.json(validation, { status: 400 });
    }

    const created = createSubmission({
      ip,
      strategyLabel,
      runType,
      status: "running"
    });

    const profile = await resolveStrategyProfileWithTevm(sourceCode);
    const result =
      runType === "v2"
        ? runDeterministicV2({ sourceCode, config: body?.config || {}, profile })
        : runDeterministicV1({
            sourceCode,
            simulations: Math.max(1, Number(body?.simulations ?? 300)),
            steps: Math.max(1, Number(body?.steps ?? 1000)),
            profile
          });

    const finished = updateSubmission(created.id, {
      status: "verified",
      result
    });

    addLeaderboardEntry({
      id: created.id,
      strategy_label: strategyLabel,
      run_type: runType,
      average_edge: result.average_edge,
      simulations: result.simulations,
      steps: result.steps,
      created_at: new Date().toISOString()
    });

    return NextResponse.json(
      {
        id: created.id,
        status: finished?.status || "verified",
        remaining_submissions_today: rate.remaining
      },
      { status: 200 }
    );
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Submission failed" },
      { status: 400 }
    );
  }
}
