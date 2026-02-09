import { runDeterministicV1, runDeterministicV2, validateSource } from "../lib/sim/runner.js";

self.onmessage = async (event) => {
  const { type, payload } = event.data || {};

  try {
    if (type === "validate") {
      self.postMessage({ ok: true, data: validateSource(payload.sourceCode) });
      return;
    }

    if (type === "run-v1") {
      const result = runDeterministicV1(payload);
      self.postMessage({ ok: true, data: result });
      return;
    }

    if (type === "run-v2") {
      const result = runDeterministicV2(payload);
      self.postMessage({ ok: true, data: result });
      return;
    }

    self.postMessage({ ok: false, error: `Unknown worker message type: ${type}` });
  } catch (error) {
    self.postMessage({
      ok: false,
      error: error instanceof Error ? error.message : "Local simulation failed"
    });
  }
};
