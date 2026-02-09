import { NextResponse } from "next/server";
import { getLeaderboard } from "../../../lib/server/store.js";

export const runtime = "nodejs";

export async function GET() {
  const rows = getLeaderboard();
  return NextResponse.json({ rows }, { status: 200 });
}
