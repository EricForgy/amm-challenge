# AMM Challenge Web Frontend

This is a standalone Next.js frontend intended for Vercel deployment.

It supports two modes:

- Local mode: browser worker simulation for fast, unverified iteration.
- Verified mode: Next.js API (`app/api/*`) re-runs submissions and records leaderboard entries with rate limiting.

All of this lives under `web/`.

TEVM-backed strategy profile resolution + compile path is implemented in `web/lib/sim/runtime.server.js`.

- Server/API runs request `useTevm: true`
- Local worker runs use `useTevm: false` (browser-safe fast mode)
- If TEVM/solc fails, runtime falls back to deterministic parser mode so the app remains usable

## Local run

```bash
cd web
npm install
npm run dev
```

This app uses built-in Next.js API routes (`/api/*`) by default and does not require an external API server.

## Deploy to Vercel

1. Import this repo in Vercel.
2. Set the project Root Directory to `web`.
3. Deploy.

## Built-In API Endpoints

- `POST /api/validate`
  - body: `{ "source_code": "..." }`
- `POST /api/run`
  - body: `{ "source_code": "...", "simulations": 100, "steps": 1000 }`
- `POST /api/run-v2`
  - body: `{ "source_code": "...", "config": { ... } }`
- `POST /api/submissions`
  - body: `{ "source_code": "...", "strategy_label": "...", "run_type": "v1|v2", ... }`
- `GET /api/submissions/:id`
- `GET /api/leaderboard`

All responses should be JSON.
