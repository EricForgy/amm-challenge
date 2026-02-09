"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { getLeaderboard } from "../../lib/api";

export default function LeaderboardPage() {
  const [rows, setRows] = useState([]);
  const [error, setError] = useState("");

  async function refresh() {
    try {
      setError("");
      const data = await getLeaderboard();
      setRows(data.rows || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load leaderboard");
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  return (
    <main className="shell">
      <section className="card">
        <p className="kicker">Verified Scores</p>
        <h1>Leaderboard</h1>
        <div className="heroActions">
          <button className="btn btnPrimary" type="button" onClick={refresh}>
            Refresh
          </button>
          <Link href="/submit" className="btn btnGhost">
            Back to Submit
          </Link>
        </div>

        {error ? <pre className="result resultError">{error}</pre> : null}

        <div className="tableWrap">
          <table className="scoreTable">
            <thead>
              <tr>
                <th>#</th>
                <th>Strategy</th>
                <th>Run Type</th>
                <th>Avg Edge</th>
                <th>Sims</th>
                <th>Steps</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, idx) => (
                <tr key={row.id}>
                  <td>{idx + 1}</td>
                  <td>{row.strategy_label}</td>
                  <td>{row.run_type.toUpperCase()}</td>
                  <td>{Number(row.average_edge).toFixed(2)}</td>
                  <td>{row.simulations}</td>
                  <td>{row.steps}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  );
}
