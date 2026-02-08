"use client";

import { useMemo, useState } from "react";
import { runStrategy, runStrategyV2, validateStrategy } from "../../lib/api";

const defaultSource = `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {AMMStrategyBase} from "./AMMStrategyBase.sol";
import {TradeInfo} from "./IAMMStrategy.sol";

contract Strategy is AMMStrategyBase {
    function afterInitialize(uint256, uint256) external override returns (uint256, uint256) {
        return (bpsToWad(30), bpsToWad(30));
    }
    function afterSwap(TradeInfo calldata) external override returns (uint256, uint256) {
        return (bpsToWad(30), bpsToWad(30));
    }
    function getName() external pure override returns (string memory) {
        return "Starter";
    }
}`;

const defaultV2Config = `{
  "n_simulations": 20,
  "n_steps": 500,
  "initial_prices": [1.0, 100.0, 150.0],
  "numeraire_token": 0,
  "gbm_mu": 0.0,
  "gbm_sigma": 0.001,
  "gbm_dt": 1.0,
  "retail_arrival_rate": 0.8,
  "retail_mean_size": 20.0,
  "retail_size_sigma": 1.2,
  "retail_buy_prob": 0.5,
  "seed": 42,
  "pools": [
    {"token_a": 0, "token_b": 1, "initial_a": 10000.0, "initial_b": 100.0},
    {"token_a": 0, "token_b": 2, "initial_a": 10000.0, "initial_b": 66.6667},
    {"token_a": 1, "token_b": 2, "initial_a": 100.0, "initial_b": 66.6667}
  ]
}`;

const tabs = [
  { id: "validate", label: "Validate" },
  { id: "run", label: "Run V1" },
  { id: "run-v2", label: "Run V2" }
];

export default function SubmitPage() {
  const [activeTab, setActiveTab] = useState("validate");
  const [sourceCode, setSourceCode] = useState(defaultSource);
  const [v2ConfigText, setV2ConfigText] = useState(defaultV2Config);
  const [simulations, setSimulations] = useState(100);
  const [steps, setSteps] = useState(1000);
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState(null);
  const [error, setError] = useState("");

  const apiBase = useMemo(
    () => process.env.NEXT_PUBLIC_API_BASE_URL || "(unset)",
    []
  );

  async function onSubmit() {
    setLoading(true);
    setError("");
    setResponse(null);
    try {
      if (activeTab === "validate") {
        const result = await validateStrategy(sourceCode);
        setResponse(result);
      } else if (activeTab === "run") {
        const result = await runStrategy(sourceCode, {
          simulations,
          steps
        });
        setResponse(result);
      } else {
        const parsed = JSON.parse(v2ConfigText);
        const result = await runStrategyV2(sourceCode, parsed);
        setResponse(result);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Request failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="shell">
      <section className="card">
        <p className="kicker">Submission Console</p>
        <h1>Validate and run strategies from the browser.</h1>
        <p className="apiHint">
          API target: <code>{apiBase}</code>
        </p>

        <div className="tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`tab ${activeTab === tab.id ? "tabActive" : ""}`}
              onClick={() => setActiveTab(tab.id)}
              type="button"
            >
              {tab.label}
            </button>
          ))}
        </div>

        <label className="label">Strategy Solidity</label>
        <textarea
          className="editor"
          value={sourceCode}
          onChange={(e) => setSourceCode(e.target.value)}
          spellCheck={false}
        />

        {activeTab === "run" && (
          <div className="inlineFields">
            <label>
              Simulations
              <input
                type="number"
                value={simulations}
                min={1}
                onChange={(e) => setSimulations(Number(e.target.value || 1))}
              />
            </label>
            <label>
              Steps
              <input
                type="number"
                value={steps}
                min={1}
                onChange={(e) => setSteps(Number(e.target.value || 1))}
              />
            </label>
          </div>
        )}

        {activeTab === "run-v2" && (
          <>
            <label className="label">V2 Config JSON</label>
            <textarea
              className="editor editorSmall"
              value={v2ConfigText}
              onChange={(e) => setV2ConfigText(e.target.value)}
              spellCheck={false}
            />
          </>
        )}

        <button type="button" className="btn btnPrimary" onClick={onSubmit} disabled={loading}>
          {loading ? "Running..." : "Submit"}
        </button>

        {error && (
          <pre className="result resultError">{error}</pre>
        )}
        {response && (
          <pre className="result">{JSON.stringify(response, null, 2)}</pre>
        )}
      </section>
    </main>
  );
}
