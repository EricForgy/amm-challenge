import { NextResponse } from "next/server";
import { getSubmission } from "../../../../lib/server/store.js";

export const runtime = "nodejs";

export async function GET(_request, { params }) {
  const submission = getSubmission(params.id);
  if (!submission) {
    return NextResponse.json({ error: "Submission not found" }, { status: 404 });
  }
  return NextResponse.json(submission, { status: 200 });
}
