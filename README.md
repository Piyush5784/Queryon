# Queryon

A pnpm monorepo:

- **`apps/desktop/`** — the Queryon desktop database client (Tauri + React + Rust). See `apps/desktop/README.md`.
- **`apps/web/`** — the marketing site and install docs (Next.js).

## Setup

```bash
pnpm install
```

## Working on the desktop app

```bash
pnpm --filter queryon tauri dev
```

or `cd apps/desktop && pnpm tauri dev`. See `apps/desktop/README.md` for database setup and testing.

## Working on the website

```bash
pnpm --filter queryon-web dev
```

or `cd apps/web && pnpm dev`.
