"use client";

import Link from "next/link";

const features = [
  {
    title: "Strategy Validation",
    detail: "Static checks + Solidity compile path, surfaced as actionable feedback."
  },
  {
    title: "Single-Pair Sim Runs",
    detail: "Run the classic challenge mode and inspect edge/PnL quickly."
  },
  {
    title: "Multi-Asset V2 Runs",
    detail: "Submit N-asset / multi-pool JSON and compare against the normalizer."
  }
];

export default function HomePage() {
  return (
    <main className="shell">
      <section className="hero card">
        <p className="kicker">AMM Challenge</p>
        <h1>Dark-mode local simulator with built-in verified API routes.</h1>
        <p>
          Run quick local experiments in-browser, then submit for trusted scoring and
          leaderboard placement through the same Next.js app.
        </p>
        <div className="heroActions">
          <Link href="/submit" className="btn btnPrimary">
            Open Submission Console
          </Link>
          <Link href="/leaderboard" className="btn btnGhost">
            View Leaderboard
          </Link>
          <a href="https://www.ammchallenge.com/" className="btn btnGhost" target="_blank" rel="noreferrer">
            Reference Site
          </a>
        </div>
      </section>

      <section className="grid">
        {features.map((f) => (
          <article key={f.title} className="card tile">
            <h2>{f.title}</h2>
            <p>{f.detail}</p>
          </article>
        ))}
      </section>
    </main>
  );
}
