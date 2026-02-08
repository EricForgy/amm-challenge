# AMM Challenge Web Frontend

This is a standalone Next.js frontend intended for Vercel deployment.

It does not run simulations locally inside Vercel. Instead, it calls an external API backend that executes validation/simulation using the Python/Rust engine.

## Local run

```bash
cd web
cp .env.example .env.local
npm install
npm run dev
```

Set `NEXT_PUBLIC_API_BASE_URL` in `.env.local` to your backend URL.

## Deploy to Vercel

1. Import this repo in Vercel.
2. Set the project Root Directory to `web`.
3. Add env var `NEXT_PUBLIC_API_BASE_URL` in Vercel Project Settings.
4. Deploy.

## Expected backend endpoints

- `POST /api/validate`
  - body: `{ "source_code": "..." }`
- `POST /api/run`
  - body: `{ "source_code": "...", "simulations": 100, "steps": 1000 }`
- `POST /api/run-v2`
  - body: `{ "source_code": "...", "config": { ... } }`

All responses should be JSON.
