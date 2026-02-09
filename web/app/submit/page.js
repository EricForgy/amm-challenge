"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import CodeEditor from "../../components/CodeEditor";
import PriceSeriesChart from "../../components/PriceSeriesChart";
import CollapsibleSection from "../../components/CollapsibleSection";
import {
  getSubmission,
  submitLeaderboard,
  validateStrategy
} from "../../lib/api";

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
  "n_steps": 1000,
  "initial_prices": [1.0, 100.0, 150.0],
  "numeraire_token": 0,
  "gbm_mu": 0.0,
  "gbm_sigma": 0.0025,
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
  { id: "local-v1", label: "Local V1" },
  { id: "local-v2", label: "Local V2" },
  { id: "submit-v1", label: "Submit V1" },
  { id: "submit-v2", label: "Submit V2" }
];

const SAVED_SETTINGS_KEY = "amm_submit_saved_settings_v1";

export default function SubmitPage() {
  const [activeTab, setActiveTab] = useState("validate");
  const [sourceCode, setSourceCode] = useState(defaultSource);
  const [v2ConfigText, setV2ConfigText] = useState(defaultV2Config);
  const [simulations, setSimulations] = useState(250);
  const [steps, setSteps] = useState(1000);
  const [strategyLabel, setStrategyLabel] = useState("My Strategy");
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [submissionId, setSubmissionId] = useState("");
  const [savedSettings, setSavedSettings] = useState([]);
  const [selectedSettingId, setSelectedSettingId] = useState("");
  const [settingName, setSettingName] = useState("Default");
  const workerRef = useRef(null);

  const apiBase = "built-in /api";
  const sourceLineCount = sourceCode.split("\n").length;
  const configLineCount = v2ConfigText.split("\n").length;
  const sourceEditorHeight = `${Math.max(440, sourceLineCount * 22 + 40)}px`;
  const configEditorHeight = `${Math.max(260, configLineCount * 22 + 36)}px`;
  const chartSeries = response?.price_bands || response?.result?.price_bands || null;

  useEffect(() => {
    const worker = new Worker(new URL("../../workers/localSim.worker.js", import.meta.url));
    workerRef.current = worker;
    return () => worker.terminate();
  }, []);

  useEffect(() => {
    try {
      const raw = localStorage.getItem(SAVED_SETTINGS_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed)) {
        setSavedSettings(parsed);
      }
    } catch {
      // ignore malformed local storage
    }
  }, []);

  function persistSettings(next) {
    setSavedSettings(next);
    localStorage.setItem(SAVED_SETTINGS_KEY, JSON.stringify(next));
  }

  function buildSettingsPayload() {
    return {
      activeTab,
      strategyLabel,
      simulations,
      steps,
      v2ConfigText
    };
  }

  function saveCurrentSettings() {
    const name = settingName.trim();
    if (!name) {
      setError("Provide a preset name before saving settings.");
      return;
    }
    setError("");
    const payload = buildSettingsPayload();
    const existing = savedSettings.find((s) => s.name.toLowerCase() === name.toLowerCase());
    const next = existing
      ? savedSettings.map((s) =>
          s.id === existing.id
            ? { ...s, name, settings: payload, updatedAt: new Date().toISOString() }
            : s
        )
      : [
          {
            id: `preset_${Date.now()}_${Math.floor(Math.random() * 1000)}`,
            name,
            createdAt: new Date().toISOString(),
            settings: payload
          },
          ...savedSettings
        ];
    persistSettings(next);
    const chosen = existing ? existing.id : next[0].id;
    setSelectedSettingId(chosen);
    setNotice(`Saved preset "${name}".`);
  }

  function applySelectedSettings() {
    const selected = savedSettings.find((s) => s.id === selectedSettingId);
    if (!selected) {
      setError("Select a saved preset to apply.");
      return;
    }
    setError("");
    const s = selected.settings || {};
    if (typeof s.activeTab === "string") setActiveTab(s.activeTab);
    if (typeof s.strategyLabel === "string") setStrategyLabel(s.strategyLabel);
    if (typeof s.simulations === "number") setSimulations(Math.max(1, s.simulations));
    if (typeof s.steps === "number") setSteps(Math.max(1, s.steps));
    if (typeof s.v2ConfigText === "string") setV2ConfigText(s.v2ConfigText);
    setNotice(`Applied preset "${selected.name}".`);
  }

  function deleteSelectedSettings() {
    const selected = savedSettings.find((s) => s.id === selectedSettingId);
    if (!selected) {
      setError("Select a saved preset to delete.");
      return;
    }
    setError("");
    const next = savedSettings.filter((s) => s.id !== selectedSettingId);
    persistSettings(next);
    setSelectedSettingId("");
    setNotice(`Deleted preset "${selected.name}".`);
  }

  function workerRequest(type, payload) {
    return new Promise((resolve, reject) => {
      if (!workerRef.current) {
        reject(new Error("Local worker not initialized"));
        return;
      }
      const onMessage = (event) => {
        const msg = event.data || {};
        workerRef.current.removeEventListener("message", onMessage);
        if (msg.ok) {
          resolve(msg.data);
        } else {
          reject(new Error(msg.error || "Worker failed"));
        }
      };
      workerRef.current.addEventListener("message", onMessage);
      workerRef.current.postMessage({ type, payload });
    });
  }

  async function pollSubmission(id) {
    for (let i = 0; i < 30; i += 1) {
      const status = await getSubmission(id);
      if (status.status === "verified" || status.status === "failed") {
        return status;
      }
      await new Promise((resolve) => {
        setTimeout(resolve, 800);
      });
    }
    throw new Error("Timed out waiting for submission result");
  }

  async function onSubmit() {
    setLoading(true);
    setError("");
    setNotice("");
    setResponse(null);
    setSubmissionId("");
    try {
      if (activeTab === "validate") {
        const result = await validateStrategy(sourceCode);
        setResponse(result);
      } else if (activeTab === "local-v1") {
        const result = await workerRequest("run-v1", {
          sourceCode,
          simulations,
          steps
        });
        setResponse({ trust: "local-unverified", ...result });
      } else if (activeTab === "local-v2") {
        const parsed = JSON.parse(v2ConfigText);
        const result = await workerRequest("run-v2", {
          sourceCode,
          config: parsed
        });
        setResponse({ trust: "local-unverified", ...result });
      } else if (activeTab === "submit-v1") {
        const submission = await submitLeaderboard(sourceCode, {
          strategy_label: strategyLabel,
          run_type: "v1",
          simulations,
          steps
        });
        setSubmissionId(submission.id);
        const status = await pollSubmission(submission.id);
        setResponse(status);
      } else if (activeTab === "submit-v2") {
        const parsed = JSON.parse(v2ConfigText);
        const submission = await submitLeaderboard(sourceCode, {
          strategy_label: strategyLabel,
          run_type: "v2",
          config: parsed
        });
        setSubmissionId(submission.id);
        const status = await pollSubmission(submission.id);
        setResponse(status);
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
        <h1>Local mode for iteration, verified mode for leaderboard.</h1>
        <p className="apiHint">
          API target: <code>{apiBase}</code>
        </p>
        <p className="apiHint">
          Use local tabs for untrusted preview, submit tabs for trusted leaderboard score.
        </p>
        <div className="heroActions">
          <Link href="/leaderboard" className="btn btnGhost">
            View Leaderboard
          </Link>
        </div>

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

        <CollapsibleSection
          title="Strategy Editor"
          subtitle={`${sourceLineCount} lines`}
          defaultOpen={true}
        >
          <div className="ideShell">
            <div className="ideTop">
              <span>
                <span className="ideDots">
                  <span />
                  <span />
                  <span />
                </span>{" "}
                Strategy.sol
              </span>
              <span>Solidity 0.8.x</span>
            </div>
            <CodeEditor
              value={sourceCode}
              onChange={setSourceCode}
              language="solidity"
              height={sourceEditorHeight}
            />
          </div>
          <div className="editorMeta">
            <span>{sourceLineCount} lines</span>
            <span>UTF-8</span>
            <span>LF</span>
          </div>
        </CollapsibleSection>

        {activeTab !== "validate" && (
          <CollapsibleSection title="Run Settings" defaultOpen={true}>
            <div className="inlineFields">
              <label>
                Strategy Label
                <input
                  type="text"
                  value={strategyLabel}
                  onChange={(e) => setStrategyLabel(e.target.value)}
                />
              </label>
              <label>
                Simulations (V1)
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
            <div className="settingsBar">
              <input
                className="settingsInput"
                type="text"
                value={settingName}
                placeholder="Preset name"
                onChange={(e) => setSettingName(e.target.value)}
              />
              <button type="button" className="btn btnGhost" onClick={saveCurrentSettings}>
                Save Settings
              </button>
            </div>
            <div className="settingsBar">
              <select
                className="settingsSelect"
                value={selectedSettingId}
                onChange={(e) => setSelectedSettingId(e.target.value)}
              >
                <option value="">Select saved preset</option>
                {savedSettings.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
              <button type="button" className="btn btnGhost" onClick={applySelectedSettings}>
                Apply
              </button>
              <button type="button" className="btn btnGhost btnDanger" onClick={deleteSelectedSettings}>
                Delete
              </button>
            </div>
            <p className="hintTiny">
              Saved presets are stored in your browser local storage.
            </p>
          </CollapsibleSection>
        )}

        {(activeTab === "local-v2" || activeTab === "submit-v2") && (
          <CollapsibleSection
            title="V2 Config JSON"
            subtitle={`${configLineCount} lines`}
            defaultOpen={false}
          >
            <div className="ideShell">
              <div className="ideTop">
                <span>
                  <span className="ideDots">
                    <span />
                    <span />
                    <span />
                  </span>{" "}
                  v2_config.json
                </span>
                <span>JSON</span>
              </div>
              <CodeEditor
                value={v2ConfigText}
                onChange={setV2ConfigText}
                language="json"
                height={configEditorHeight}
              />
            </div>
          </CollapsibleSection>
        )}

        <button type="button" className="btn btnPrimary" onClick={onSubmit} disabled={loading}>
          {loading ? "Running..." : "Submit"}
        </button>

        {submissionId && (
          <p className="apiHint">
            Submission ID: <code>{submissionId}</code>
          </p>
        )}

        {(error || notice || response) ? (
          <CollapsibleSection
            title="Run Output"
            subtitle={error ? "error" : notice ? "message" : "json"}
            defaultOpen={true}
          >
            {notice && !error ? (
              <pre className="result">{notice}</pre>
            ) : null}
            {error && (
              <pre className="result resultError">{error}</pre>
            )}
            {response && (
              <pre className="result">{JSON.stringify(response, null, 2)}</pre>
            )}
          </CollapsibleSection>
        ) : null}
      </section>
      {chartSeries ? (
        <CollapsibleSection title="Simulated Token Prices" defaultOpen={true}>
          <PriceSeriesChart series={chartSeries} />
        </CollapsibleSection>
      ) : null}
    </main>
  );
}
